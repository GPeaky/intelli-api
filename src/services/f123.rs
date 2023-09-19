use crate::protos::{car_motion_data::PacketMotionData, session_data::PacketSessionData};
use crate::{
    config::Database,
    dtos::F123Data,
    error::{AppResult, SocketError},
};
use ahash::AHashMap;
use prost::Message;
use redis::Commands;
use std::{
    // net::UdpSocket,
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::sync::broadcast::Sender;
use tokio::{
    net::UdpSocket,
    sync::{broadcast::Receiver, RwLock},
    task::JoinHandle,
};
use tracing::{error, info};

const F123_HOST: &str = "0.0.0.0";
const DATA_PERSISTENCE: usize = 15 * 60;
const F123_MAX_PACKET_SIZE: usize = 1460;
const SESSION_INTERVAL: Duration = Duration::from_secs(15);
const MOTION_INTERVAL: Duration = Duration::from_millis(700);
const SESSION_HISTORY_INTERVAL: Duration = Duration::from_secs(2);

#[derive(Clone)]
pub struct F123Service {
    db_conn: Arc<Database>,
    sockets: Arc<RwLock<AHashMap<u32, JoinHandle<()>>>>,
    channels: Arc<RwLock<AHashMap<u32, Arc<Sender<Vec<u8>>>>>>,
}

impl F123Service {
    pub fn new(db_conn: &Arc<Database>) -> Self {
        Self {
            db_conn: db_conn.clone(),
            channels: Arc::new(RwLock::new(AHashMap::new())),
            sockets: Arc::new(RwLock::new(AHashMap::new())),
        }
    }

    pub async fn new_socket(&self, port: u16, championship_id: Arc<u32>) -> AppResult<()> {
        {
            let sockets = self.sockets.read().await;
            let channels = self.channels.read().await;

            if sockets.contains_key(&championship_id) || channels.contains_key(&championship_id) {
                error!("Trying to create a new socket or channel for an existing championship: {championship_id:?}");
                return Err(SocketError::AlreadyExists.into());
            }
        }

        // TODO: Close socket when championship is finished or when the server is idle for a long time
        let socket = self.spawn_socket(championship_id.clone(), port).await;

        {
            let mut sockets = self.sockets.write().await;
            sockets.insert(*championship_id, socket);
        }

        Ok(())
    }

    async fn spawn_socket(&self, championship_id: Arc<u32>, port: u16) -> JoinHandle<()> {
        let db = self.db_conn.clone();
        let channels = self.channels.clone();
        let sockets = self.sockets.clone();

        tokio::spawn(async move {
            let mut redis = db.get_redis();
            let session = db.mysql.clone();
            let mut buf = [0u8; F123_MAX_PACKET_SIZE];

            let mut last_session_update = Instant::now();
            let mut last_car_motion_update = Instant::now();
            let mut last_participants_update = Instant::now();

            // Session History Data
            let mut last_car_lap_update: AHashMap<u8, Instant> = AHashMap::new();
            let mut car_lap_sector_data: AHashMap<u8, (u16, u16, u16)> = AHashMap::new();

            // Define channel
            let (tx, _rx) = tokio::sync::broadcast::channel::<Vec<u8>>(100);

            let Ok(socket) = UdpSocket::bind(format!("{F123_HOST}:{port}")).await else {
                error!("There was an error binding to the socket for championship: {championship_id:?}");
                return;
            };

            // TODO: Implemente timeout for socket with tokio::time::timeout
            // socket.set_read_timeout(SOCKET_TIMEOUT).unwrap();

            info!("Listening for F123 data on port: {port} for championship: {championship_id:?}");

            {
                let mut channels = channels.write().await;
                channels.insert(*championship_id, Arc::new(tx.clone()));
            }

            // TODO: Save all this data in redis and only save it in the database when the session is finished
            loop {
                match socket.recv_from(&mut buf).await {
                    Ok((size, _address)) => {
                        let buf = &buf[..size];

                        let Ok(header) = F123Data::deserialize_header(buf) else {
                            error!("Error deserializing F123 header, for championship: {championship_id:?}");
                            continue;
                        };

                        let session_id = header.m_sessionUID as i64;

                        if session_id.eq(&0) {
                            tokio::time::sleep(Duration::from_secs(5)).await;
                            continue;
                        }

                        let Ok(packet) = F123Data::deserialize(header.m_packetId.into(), buf)
                        else {
                            error!("Error deserializing F123 packet: {}", header.m_packetId);
                            continue;
                        };

                        let Some(packet) = packet else {
                            continue;
                        };

                        let now = Instant::now();

                        match packet {
                            F123Data::Motion(motion_data) => {
                                if now
                                    .duration_since(last_car_motion_update)
                                    .ge(&MOTION_INTERVAL)
                                {
                                    let data: PacketMotionData = motion_data.into();
                                    let data = data.encode_to_vec();

                                    tx.send(data).unwrap();
                                    last_car_motion_update = now;
                                }
                            }

                            F123Data::Session(session_data) => {
                                if now
                                    .duration_since(last_session_update)
                                    .ge(&SESSION_INTERVAL)
                                {
                                    redis
                                        .set_ex::<String, &[u8], String>(
                                            format!(
                                                "f123:championship:{}:session:{session_id}:session",
                                                championship_id
                                            ),
                                            &buf[..size],
                                            DATA_PERSISTENCE,
                                        )
                                        .unwrap();

                                    let data: PacketSessionData = session_data.into();
                                    let data = data.encode_to_vec();

                                    tx.send(data).unwrap();
                                    last_session_update = now;
                                }
                            }

                            // TODO: Implement to save this in a interval
                            F123Data::Participants(participants_data) => {
                                if now
                                    .duration_since(last_participants_update)
                                    .ge(&SESSION_INTERVAL)
                                {
                                    // tx.send(F123Data::Participants(participants_data)).unwrap();

                                    redis
                                        .set_ex::<String, &[u8], String>(
                                            format!(
                                                "f123:championship:{}:session:{session_id}:participants",
                                                championship_id
                                            ),
                                            &buf[..size],
                                            DATA_PERSISTENCE,
                                        )
                                        .unwrap();

                                    last_participants_update = now;
                                }
                            }

                            // TODO: Save to heidi or redis?
                            F123Data::Event(event_data) => {
                                sqlx::query(
                                    r#"
                                    INSERT INTO event_data (session_id, string_code, event)
                                    VALUES (?, ?, ?)
                                "#,
                                )
                                .bind(session_id)
                                .bind(event_data.m_eventStringCode.to_vec())
                                .bind(&buf[..size])
                                .execute(&session)
                                .await
                                .unwrap();

                                // tx.send(F123Data::Event(event_data)).unwrap();
                            }

                            // TODO: Check if this is overbooking the server
                            F123Data::SessionHistory(session_history) => {
                                let Some(last_update) =
                                    last_car_lap_update.get(&session_history.m_carIdx)
                                else {
                                    last_car_lap_update.insert(session_history.m_carIdx, now);
                                    continue;
                                };

                                if now
                                    .duration_since(*last_update)
                                    .ge(&SESSION_HISTORY_INTERVAL)
                                {
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
                                        redis
                                            .set_ex::<String, &[u8], String>(
                                                format!("f123:championship:{}:session:{session_id}:history:car:{}", championship_id, session_history.m_carIdx),
                                                &buf[..size],
                                                DATA_PERSISTENCE,
                                            )
                                            .unwrap();

                                        last_car_lap_update.insert(session_history.m_carIdx, now);
                                        car_lap_sector_data
                                            .insert(session_history.m_carIdx, sectors);

                                        // tx.send(F123Data::SessionHistory(session_history)).unwrap();
                                    }
                                }
                            }

                            //TODO Collect All data from redis and save it to the maridb database
                            F123Data::FinalClassification(classification_data) => {
                                // tx.send(F123Data::FinalClassification(classification_data))
                                //     .unwrap();

                                return;
                            }
                        }
                    }

                    Err(e) => {
                        error!("Error receiving data from F123 socket: {}", e);
                        info!("Stopping socket for championship: {}", championship_id);

                        {
                            let mut sockets = sockets.write().await;
                            let mut channels = channels.write().await;

                            if let Some(socket) = sockets.remove(&championship_id) {
                                socket.abort();
                            }

                            channels.remove(&championship_id);
                        }

                        break;
                    }
                }
            }
        })
    }

    pub async fn active_sockets(&self) -> Vec<u32> {
        let sockets = self.sockets.read().await;
        sockets.keys().cloned().collect()
    }

    pub async fn championship_socket(&self, id: &u32) -> bool {
        let sockets = self.sockets.read().await;
        sockets.contains_key(id)
    }

    pub async fn stop_socket(&self, championship_id: u32) -> AppResult<()> {
        {
            let mut channels = self.channels.write().await;
            let mut sockets = self.sockets.write().await;

            let Some(_) = channels.remove(&championship_id) else {
                Err(SocketError::NotFound)?
            };

            let Some(socket) = sockets.remove(&championship_id) else {
                Err(SocketError::NotFound)?
            };

            socket.abort();
        }

        info!("Socket stopped for championship: {}", championship_id);

        Ok(())
    }

    pub async fn get_receiver(&self, championship_id: &u32) -> Option<Receiver<Vec<u8>>> {
        let channels = self.channels.read().await;

        if let Some(channel) = channels.get(championship_id) {
            return Some(channel.subscribe());
        }

        None
    }
}
