use crate::{
    config::Database,
    dtos::{EventDataStatements, F123Data, PreparedStatementsKey},
    error::{AppResult, SocketError},
};
use ahash::AHashMap;
use bincode::serialize;
use redis::Commands;
use std::{
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::{
    net::UdpSocket,
    sync::{mpsc::Receiver, Mutex, RwLock},
    task::JoinHandle,
};
use tracing::{error, info};

type F123Receiver = Receiver<(u8, Vec<u8>)>;

const F123_HOST: &str = "0.0.0.0";
const DATA_PERSISTANCE: usize = 15 * 60;
const F123_MAX_PACKET_SIZE: usize = 1460;
const SESSION_INTERVAL: Duration = Duration::from_secs(30);
const MOTION_INTERVAL: Duration = Duration::from_millis(700);
const SESSION_HISTORY_INTERVAL: Duration = Duration::from_secs(2);

#[derive(Clone)]
pub struct F123Service {
    db_conn: Arc<Database>,
    sockets: Arc<RwLock<AHashMap<i32, JoinHandle<()>>>>,
    channels: Arc<RwLock<AHashMap<i32, Arc<Mutex<F123Receiver>>>>>,
}

impl F123Service {
    pub fn new(db_conn: &Arc<Database>) -> Self {
        Self {
            db_conn: db_conn.clone(),
            channels: Arc::new(RwLock::new(AHashMap::new())),
            sockets: Arc::new(RwLock::new(AHashMap::new())),
        }
    }

    pub async fn new_socket(&self, port: i16, championship_id: Arc<i32>) -> AppResult<()> {
        //  Check if socket already exists
        {
            let sockets = self.sockets.read().await;

            if sockets.contains_key(&championship_id) {
                error!("Trying to create a new socket for an existing championship: {championship_id:?}");
                return Err(SocketError::AlreadyExists.into());
            }
        }

        // Check if channel already exists
        {
            let channels = self.channels.read().await;

            if channels.contains_key(&championship_id) {
                error!("Trying to create a new channel for an existing championship: {championship_id:?}");
                return Err(SocketError::AlreadyExists.into());
            }
        }

        // TODO: Close socket when championship is finished or when the server is idle for a long time
        let socket = self.socket_task(championship_id.clone(), port).await;

        {
            let mut sockets = self.sockets.write().await;
            sockets.insert(*championship_id, socket);
        }

        Ok(())
    }

    async fn socket_task(&self, championship_id: Arc<i32>, port: i16) -> JoinHandle<()> {
        let db = self.db_conn.clone();
        let channels = self.channels.clone();

        tokio::task::spawn(async move {
            let mut buf = [0u8; F123_MAX_PACKET_SIZE];
            let mut redis = db.get_redis();
            let session = db.scylla.clone();
            let mut last_session_update = Instant::now();
            let mut last_car_motion_update = Instant::now();

            // Session History Data
            let mut last_car_lap_update: AHashMap<u8, Instant> = AHashMap::new();
            let mut car_lap_sector_data: AHashMap<u8, (u16, u16, u16)> = AHashMap::new();

            // Define channel
            let (tx, rx) = tokio::sync::mpsc::channel::<(u8, Vec<u8>)>(F123_MAX_PACKET_SIZE);

            {
                let mut channels = channels.write().await;
                channels.insert(*championship_id, Arc::new(Mutex::new(rx)));
            }

            let Ok(socket) = UdpSocket::bind(format!("{F123_HOST}:{port}")).await else {
                error!("There was an error binding to the socket for championship: {championship_id:?}");
                return;
            };

            info!("Listening for F123 data on port: {port} for championship: {championship_id:?}");

            // TODO: Save all this data in redis and only save it in the database when the session is finished
            loop {
                match socket.recv_from(&mut buf).await {
                    Ok((size, _address)) => {
                        let buf = &buf[..size];
                        let now = Instant::now();

                        let Ok(header) = F123Data::deserialize_header(buf) else {
                            error!("Error deserializing F123 header, for championship: {championship_id:?}");
                            continue;
                        };

                        let session_id = header.m_sessionUID as i64;

                        if session_id.eq(&0) {
                            tokio::time::sleep(Duration::from_secs(5)).await;
                            continue;
                        }

                        let Ok(Some(packet)) = F123Data::deserialize(header.m_packetId.into(), buf)
                        else {
                            error!("Error deserializing F123 packet, for championship: {championship_id:?}");
                            continue;
                        };

                        match packet {
                            F123Data::SessionHistory(session_history) => {
                                let Some(last_update) =
                                    last_car_lap_update.get(&session_history.m_carIdx)
                                else {
                                    last_car_lap_update.insert(session_history.m_carIdx, now);
                                    continue;
                                };

                                if now.duration_since(*last_update) >= SESSION_HISTORY_INTERVAL {
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

                                    if sectors.ne(last_sectors) {
                                        let Ok(data) = serialize(&session_history) else {
                                            error!("There was an error serializing the session history data for car: {}", session_history.m_carIdx);
                                            continue;
                                        };

                                        tx.send((header.m_packetId, data.clone())).await.unwrap();

                                        redis
                                            .set_ex::<String, Vec<u8>, String>(
                                                format!("f123:championship:{}:session:{session_id}:history:car:{}", championship_id, session_history.m_carIdx),
                                                data,
                                                DATA_PERSISTANCE,
                                            )
                                            .unwrap();

                                        last_car_lap_update.insert(session_history.m_carIdx, now);
                                        car_lap_sector_data
                                            .insert(session_history.m_carIdx, sectors);
                                    }
                                }
                            }

                            F123Data::Motion(motion_data) => {
                                if now.duration_since(last_car_motion_update) >= MOTION_INTERVAL {
                                    let Ok(data) = serialize(&motion_data) else {
                                        error!("There was an error serializing the motion data for championship: {championship_id:?}");
                                        continue;
                                    };

                                    tx.send((header.m_packetId, data.clone())).await.unwrap();

                                    // redis
                                    //     .set_ex::<String, Vec<u8>, String>(
                                    //         format!(
                                    //             "f123:championship:{}:session:{session_id}:motion",
                                    //             championship_id
                                    //         ),
                                    //         data,
                                    //         60 * 60,
                                    //     )
                                    //     .unwrap();

                                    last_car_motion_update = now;
                                }
                            }

                            F123Data::Session(session_data) => {
                                if now.duration_since(last_session_update) >= SESSION_INTERVAL {
                                    let Ok(data) = serialize(&session_data) else {
                                        error!("There was an error serializing the session data for championship: {championship_id:?}");
                                        continue;
                                    };

                                    tx.send((header.m_packetId, data.clone())).await.unwrap();

                                    redis
                                        .set_ex::<String, Vec<u8>, String>(
                                            format!(
                                                "f123:championship:{}:session:{session_id}:session",
                                                championship_id
                                            ),
                                            data,
                                            DATA_PERSISTANCE,
                                        )
                                        .unwrap();

                                    last_session_update = now;
                                }
                            }

                            // We don't save events in redis because redis doesn't support lists of lists
                            F123Data::Event(event_data) => {
                                let select_stmt = db
                                    .statements
                                    .get(&PreparedStatementsKey::EventData(
                                        EventDataStatements::Select,
                                    ))
                                    .unwrap();
                                let insert_stmt = db
                                    .statements
                                    .get(&PreparedStatementsKey::EventData(
                                        EventDataStatements::Insert,
                                    ))
                                    .unwrap();
                                let update_stmt = db
                                    .statements
                                    .get(&PreparedStatementsKey::EventData(
                                        EventDataStatements::Update,
                                    ))
                                    .unwrap();

                                let Ok(event) = serialize(&event_data.m_eventDetails) else {
                                    error!("There was an error serializing the event data for championship: {championship_id:?}");
                                    continue;
                                };

                                tx.send((header.m_packetId, event.clone())).await.unwrap();

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

                            // TODO: Check why this is never saving to redis
                            F123Data::Participants(participants_data) => {
                                tracing::info!("Saving participants data"); // Test

                                let Ok(participants) = serialize(&participants_data.m_participants)
                                else {
                                    error!("There was an error serializing the participants data for championship: {championship_id:?}");
                                    continue;
                                };

                                tx.send((header.m_packetId, participants.clone()))
                                    .await
                                    .unwrap();

                                redis
                                    .set_ex::<String, Vec<u8>, String>(
                                        format!(
                                        "f123:championship:{}:session:{session_id}:participants",
                                        championship_id
                                    ),
                                        participants.clone(),
                                        DATA_PERSISTANCE,
                                    )
                                    .unwrap();
                            }

                            //TODO Collect All data from redis and save it to the scylla database
                            F123Data::FinalClassification(classification_data) => {
                                let Ok(classifications) =
                                    serialize(&classification_data.m_classificationData)
                                else {
                                    error!("There was an error serializing the final classification data for championship: {championship_id:?}");
                                    continue;
                                };

                                tx.send((header.m_packetId, classifications.clone()))
                                    .await
                                    .unwrap();

                                // TODO Save all laps for each driver in the final classification
                                // TODO: Save the final classification to the database
                                // session
                                //     .execute(
                                //         db.statements.get("final_classification.insert").unwrap(),
                                //         (session_id, classifications),
                                //     )
                                //     .await
                                //     .unwrap();
                            }
                        }
                    }

                    Err(e) => {
                        error!("Error receiving packet: {}", e);
                    }
                }
            }
        })
    }

    pub async fn active_sockets(&self) -> Vec<i32> {
        let sockets = self.sockets.read().await;
        sockets.keys().cloned().collect()
    }

    pub async fn championship_socket(&self, id: &i32) -> bool {
        let sockets = self.sockets.read().await;
        sockets.contains_key(id)
    }

    pub async fn stop_socket(&self, championship_id: i32) -> AppResult<()> {
        {
            let mut sockets = self.sockets.write().await;
            let Some(socket) = sockets.remove(&championship_id) else {
                Err(SocketError::NotFound)?
            };

            socket.abort();
        }

        info!("Socket stopped for championship: {}", championship_id);

        Ok(())
    }

    pub async fn get_receiver(&self, championship_id: &i32) -> Option<(u8, Vec<u8>)> {
        let channels = self.channels.read().await;

        if let Some(channel_mutex) = channels.get(championship_id) {
            let mut channel = channel_mutex.lock().await;
            channel.recv().await
        } else {
            None
        }
    }
}
