use crate::{
    config::Database,
    dtos::F123Packet,
    error::{AppResult, SocketError},
};
use ahash::AHashMap;
use bincode::serialize;
use redis::Commands;
use std::{
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::{net::UdpSocket, sync::RwLock, task::JoinHandle};
use tracing::error;

#[derive(Clone)]
pub struct F123Service {
    db_conn: Arc<Database>,
    sockets: Arc<RwLock<AHashMap<String, JoinHandle<()>>>>,
}

impl F123Service {
    pub fn new(db_conn: &Arc<Database>) -> Self {
        Self {
            db_conn: db_conn.clone(),
            sockets: Arc::new(RwLock::new(AHashMap::new())),
        }
    }

    pub async fn new_socket(&self, port: i16, championship_id: Arc<String>) -> AppResult<()> {
        {
            let sockets = self.sockets.read().await;

            if sockets.contains_key(&championship_id.to_string()) {
                return Err(SocketError::AlreadyExists.into());
            }
        }

        let db = self.db_conn.clone();
        let championship_clone = championship_id.clone();

        // TODO: Close socket when championship is finished or when the server is idle for a long time
        let socket = tokio::task::spawn(async move {
            let mut buf = [0; 1460];
            let mut last_session_update = Instant::now();
            let mut last_car_motion_update = Instant::now();
            let (session, mut redis) = (db.get_scylla(), db.get_redis());

            // Session History Data
            let mut last_car_lap_update: AHashMap<u8, Instant> = AHashMap::new();
            let mut car_lap_sector_data: AHashMap<u8, (u16, u16, u16)> = AHashMap::new();
            let (ms_interval, secs_interval, sec_interval) = (
                Duration::from_millis(700),
                Duration::from_secs(2),
                Duration::from_secs(30),
            );

            let Ok(socket) = UdpSocket::bind(format!("0.0.0.0:{}", port)).await else {
                error!("There was an error binding to the socket");
                return;
            };

            // TODO: Save all this data in redis and only save it in the database when the session is finished
            loop {
                match socket.recv_from(&mut buf).await {
                    Ok((size, _address)) => {
                        let Ok(header) = F123Packet::parse_header(&buf[..size]) else {
                            continue;
                        };

                        let session_id = header.m_sessionUID as i64;
                        if session_id == 0 {
                            continue;
                        }

                        let Ok(Some(packet)) = F123Packet::parse(header.m_packetId, &buf[..size])
                        else {
                            continue;
                        };

                        match packet {
                            F123Packet::SessionHistory(session_history) => {
                                let now = Instant::now();

                                let Some(last_update) =
                                    last_car_lap_update.get(&session_history.m_carIdx)
                                else {
                                    last_car_lap_update.insert(session_history.m_carIdx, now);
                                    continue;
                                };

                                if now.duration_since(*last_update) >= secs_interval {
                                    let lap = session_history.m_numLaps as usize - 1; // Lap is 0 indexed

                                    let sectors = (
                                        session_history.m_lapHistoryData[lap].m_sector1TimeInMS,
                                        session_history.m_lapHistoryData[lap].m_sector2TimeInMS,
                                        session_history.m_lapHistoryData[lap].m_sector3TimeInMS,
                                    );

                                    let Some(last_sectors) =
                                        car_lap_sector_data.get(&session_history.m_carIdx)
                                    else {
                                        car_lap_sector_data
                                            .insert(session_history.m_carIdx, sectors);
                                        continue;
                                    };

                                    if sectors != *last_sectors {
                                        let Ok(data) = serialize(&session_history) else {
                                            error!("There was an error serializing the session history data");
                                            continue;
                                        };

                                        tracing::info!(
                                            "Saving session history for car: {}",
                                            session_history.m_carIdx
                                        );

                                        redis
                                            .set_ex::<String, Vec<u8>, String>(
                                                format!("f123:championship:{}:session:{session_id}:history:car:{}", championship_id, session_history.m_carIdx),
                                                data,
                                                60 * 60,
                                            )
                                            .unwrap();

                                        last_car_lap_update.insert(session_history.m_carIdx, now);
                                        car_lap_sector_data
                                            .insert(session_history.m_carIdx, sectors);
                                    }
                                }
                            }

                            F123Packet::Motion(motion_data) => {
                                let now = Instant::now();

                                if now.duration_since(last_car_motion_update) >= ms_interval {
                                    let Ok(data) = serialize(&motion_data) else {
                                        error!("There was an error serializing the motion data");
                                        continue;
                                    };

                                    redis
                                        .set_ex::<String, Vec<u8>, String>(
                                            format!(
                                                "f123:championship:{}:session:{session_id}:motion",
                                                championship_id
                                            ),
                                            data,
                                            60 * 60,
                                        )
                                        .unwrap();

                                    last_car_motion_update = now;
                                }
                            }

                            F123Packet::Session(session_data) => {
                                let now = Instant::now();

                                if now.duration_since(last_session_update) >= sec_interval {
                                    let Ok(data) = serialize(&session_data) else {
                                        error!("There was an error serializing the session data");
                                        continue;
                                    };

                                    redis
                                        .set_ex::<String, Vec<u8>, String>(
                                            format!(
                                                "f123:championship:{}:session:{session_id}:session",
                                                championship_id
                                            ),
                                            data,
                                            60 * 60,
                                        )
                                        .unwrap();

                                    last_session_update = now;
                                }
                            }

                            // We don't save events in redis because redis doesn't support lists of lists
                            F123Packet::Event(event_data) => {
                                let select_stmt = db.statements.get("select_event_data").unwrap();
                                let insert_stmt = db.statements.get("insert_event_data").unwrap();
                                let update_stmt = db.statements.get("update_event_data").unwrap();

                                let Ok(event) = serialize(&event_data.m_eventDetails) else {
                                    error!("There was an error serializing the event data");
                                    continue;
                                };

                                let table_exists = session
                                    .execute(
                                        select_stmt,
                                        (session_id, event_data.m_eventStringCode),
                                    )
                                    .await
                                    .unwrap()
                                    .rows_or_empty();

                                if table_exists.is_empty() {
                                    session
                                        .execute(
                                            insert_stmt,
                                            (session_id, event_data.m_eventStringCode, vec![event]),
                                        )
                                        .await
                                        .unwrap();
                                } else {
                                    session
                                        .execute(
                                            update_stmt,
                                            (vec![event], session_id, event_data.m_eventStringCode),
                                        )
                                        .await
                                        .unwrap();
                                }
                            }

                            F123Packet::Participants(participants_data) => {
                                let Ok(participants) = serialize(&participants_data.m_participants)
                                else {
                                    error!("There was an error serializing the participants data");
                                    continue;
                                };

                                redis
                                    .set_ex::<String, Vec<u8>, String>(
                                        format!(
                                        "f123:championship:{}:session:{session_id}:participants",
                                        championship_id
                                    ),
                                        participants.clone(),
                                        60 * 60,
                                    )
                                    .unwrap();
                            }

                            F123Packet::FinalClassification(classification_data) => {
                                let Ok(classifications) =
                                    serialize(&classification_data.m_classificationData)
                                else {
                                    error!("There was an error serializing the final classification data");
                                    continue;
                                };

                                // TODO: Save all laps for each driver in the final classification
                                session
                                    .execute(
                                        db.statements
                                            .get("insert_final_classification_data")
                                            .unwrap(),
                                        (session_id, classifications),
                                    )
                                    .await
                                    .unwrap();
                            }
                        }
                    }

                    Err(e) => {
                        error!("Error receiving packet: {}", e);
                    }
                }
            }
        });

        {
            let mut sockets = self.sockets.write().await;
            sockets.insert(championship_clone.to_string(), socket);
        }

        Ok(())
    }

    pub async fn active_sockets(&self) -> AppResult<Vec<String>> {
        let sockets = self.sockets.read().await;

        Ok(sockets.keys().cloned().collect())
    }

    pub async fn stop_socket(&self, championship_id: String) -> AppResult<()> {
        {
            let mut sockets = self.sockets.write().await;
            let Some(socket) = sockets.remove(&championship_id) else {
                Err(SocketError::NotFound)?
            };

            socket.abort();
        }

        Ok(())
    }

    // pub async fn stop_all_sockets(&self) {
    //     let mut sockets = self.sockets.write().await;

    //     for socket in sockets.iter() {
    //         socket.1.abort();
    //     }

    //     sockets.clear();
    // }
}
