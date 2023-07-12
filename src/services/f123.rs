use crate::{
    config::Database,
    dtos::F123Packet,
    error::{AppResult, SocketError},
};
use bincode::serialize;
use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::{net::UdpSocket, sync::RwLock, task::JoinHandle};
use tracing::error;

#[derive(Clone)]
pub struct F123Service {
    db_conn: Arc<Database>,
    sockets: Arc<RwLock<HashMap<String, JoinHandle<()>>>>,
}

impl F123Service {
    pub fn new(db_conn: &Arc<Database>) -> Self {
        Self {
            db_conn: db_conn.clone(),
            sockets: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn new_socket(&self, port: i16, championship_id: String) {
        let db = self.db_conn.clone();

        let socket = tokio::spawn(async move {
            let session = db.get_scylla();
            let statements = &db.statements;
            let mut last_session_update: HashMap<i64, Instant> = HashMap::new();
            let mut last_car_motion_update: HashMap<i64, Instant> = HashMap::new();
            let Ok(socket) = UdpSocket::bind(format!("0.0.0.0:{}", port)).await else {
                error!("There was an error binding to the socket");
                return;
            };
            let mut buf = vec![0; 2048];

            loop {
                match socket.recv_from(&mut buf).await {
                    Ok((size, _)) => {
                        let Ok(header) = F123Packet::parse_header(&buf[..size]) else {
                            error!("There was an error parsing the header");
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
                            F123Packet::Motion(motion_data) => {
                                let now = Instant::now();

                                if !last_car_motion_update.contains_key(&session_id)
                                    || now.duration_since(last_car_motion_update[&session_id])
                                        >= Duration::from_millis(500)
                                {
                                    let Ok(data) = serialize(&motion_data) else {
                                        error!("There was an error serializing the motion data");
                                        continue;
                                    };

                                    session
                                        .execute(
                                            statements.get("insert_car_motion").unwrap(),
                                            (session_id, data),
                                        )
                                        .await
                                        .unwrap();

                                    last_car_motion_update.insert(session_id, now);
                                }
                            }

                            F123Packet::Session(session_data) => {
                                let now = Instant::now();

                                if !last_session_update.contains_key(&session_id)
                                    || now.duration_since(last_session_update[&session_id])
                                        >= Duration::from_secs(30)
                                {
                                    let Ok(data) = serialize(&session_data) else {
                                        error!("There was an error serializing the session data");
                                        continue;
                                    };

                                    session
                                        .execute(
                                            statements.get("insert_game_session").unwrap(),
                                            (session_id, data),
                                        )
                                        .await
                                        .unwrap();

                                    last_session_update.insert(session_id, now);
                                }
                            }

                            F123Packet::LapData(lap_data) => {
                                let Ok(lap_info) = serialize(&lap_data.m_lapData) else {
                                    error!("There was an error serializing the lap data");
                                    continue;
                                };

                                // TODO: Save lap data to database
                                session
                                    .execute(
                                        statements.get("insert_lap_data").unwrap(),
                                        (session_id, lap_info),
                                    )
                                    .await
                                    .unwrap();
                            }

                            F123Packet::Event(event_data) => {
                                let Ok(event) = serialize(&event_data.m_eventDetails) else {
                                    error!("There was an error serializing the event data");
                                    continue;
                                };

                                session
                                    .execute(
                                        statements.get("insert_event_data").unwrap(),
                                        (session_id, event_data.m_eventStringCode, event),
                                    )
                                    .await
                                    .unwrap();
                            }

                            F123Packet::Participants(participants_data) => {
                                let Ok(participants) = serialize(&participants_data.m_participants)
                                else {
                                    error!("There was an error serializing the participants data");
                                    continue;
                                };

                                session
                                    .execute(
                                        statements.get("insert_participant_data").unwrap(),
                                        (session_id, participants),
                                    )
                                    .await
                                    .unwrap();
                            }

                            F123Packet::FinalClassification(classification_data) => {
                                let Ok(classifications) =
                                    serialize(&classification_data.m_classificationData)
                                else {
                                    error!("There was an error serializing the final classification data");
                                    continue;
                                };

                                session
                                    .execute(
                                        statements.get("insert_final_classification_data").unwrap(),
                                        (session_id, classifications),
                                    )
                                    .await
                                    .unwrap();
                            }

                            // TODO: use unused packets
                            _ => {}
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
            sockets.insert(championship_id, socket);
        }
    }

    // pub async fn active_sockets(&self) {
    //     let sockets = self.sockets.read().await;

    //     for socket in sockets.iter() {
    //         println!("Socket: {:?}", socket);
    //     }
    // }

    pub async fn stop_socket(&self, championship_id: String) -> AppResult<()> {
        let mut sockets = self.sockets.write().await;

        let Some(socket) = sockets.remove(&championship_id) else {
            Err(SocketError::NotFound)?
        };

        socket.abort();
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
