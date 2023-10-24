use crate::protos::{
    car_motion_data::PacketMotionData, event_data::PacketEventData, packet_header::PacketType,
    participants::PacketParticipantsData, session_data::PacketSessionData,
    session_history::PacketSessionHistoryData, PacketHeader,
};
use crate::{
    config::Database,
    dtos::F123Data,
    error::{AppResult, SocketError},
};
use prost::Message;
use redis::AsyncCommands;
use rustc_hash::FxHashMap;
use std::{
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::sync::broadcast::Sender;
use tokio::sync::RwLock;
use tokio::{net::UdpSocket, sync::broadcast::Receiver, task::JoinHandle, time::timeout};
use tracing::{error, info};

const F123_HOST: &str = "0.0.0.0";
const DATA_PERSISTENCE: usize = 15 * 60;
const F123_MAX_PACKET_SIZE: usize = 1460;
const SESSION_INTERVAL: Duration = Duration::from_secs(15);
const MOTION_INTERVAL: Duration = Duration::from_millis(700);
const SESSION_HISTORY_INTERVAL: Duration = Duration::from_secs(2);
const SOCKET_TIMEOUT: Duration = Duration::from_secs(10 * 60);

type F123Channel = Arc<Sender<Vec<u8>>>;
type Sockets = Arc<RwLock<FxHashMap<u32, Arc<JoinHandle<()>>>>>;
type Channels = Arc<RwLock<FxHashMap<u32, F123Channel>>>;

pub struct F123Service {
    db_conn: Arc<Database>,
    sockets: Sockets,
    channels: Channels,
}

impl F123Service {
    pub fn new(db_conn: &Arc<Database>) -> Self {
        Self {
            db_conn: db_conn.clone(),
            channels: Arc::new(RwLock::new(FxHashMap::default())),
            sockets: Arc::new(RwLock::new(FxHashMap::default())),
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

        let socket = self.spawn_socket(championship_id.clone(), port).await;

        {
            let mut sockets = self.sockets.write().await;
            sockets.insert(*championship_id, Arc::new(socket));
        }

        Ok(())
    }

    async fn spawn_socket(&self, championship_id: Arc<u32>, port: u16) -> JoinHandle<()> {
        let db = self.db_conn.clone();
        let channels = self.channels.clone();
        let sockets = self.sockets.clone();

        tokio::spawn(async move {
            let session = db.mysql.clone();
            let mut buf = [0u8; F123_MAX_PACKET_SIZE];
            let mut redis = db.get_redis_async().await;

            let mut last_session_update = Instant::now();
            let mut last_car_motion_update = Instant::now();
            let mut last_participants_update = Instant::now();

            // Session History Data
            let mut last_car_lap_update: FxHashMap<u8, Instant> = FxHashMap::default();
            let mut car_lap_sector_data: FxHashMap<u8, (u16, u16, u16)> = FxHashMap::default();

            // Define channel
            let (tx, _rx) = tokio::sync::broadcast::channel::<Vec<u8>>(100);

            let Ok(socket) = UdpSocket::bind(format!("{F123_HOST}:{port}")).await else {
                error!("There was an error binding to the socket for championship: {championship_id:?}");
                return;
            };

            info!("Listening for F123 data on port: {port} for championship: {championship_id:?}");

            {
                let mut channels = channels.write().await;
                channels.insert(*championship_id, Arc::new(tx.clone()));
            }

            loop {
                match timeout(SOCKET_TIMEOUT, socket.recv_from(&mut buf)).await {
                    Ok(Ok((size, _address))) => {
                        let buf = &buf[..size];

                        let Ok(header) = F123Data::deserialize_header(buf) else {
                            error!("Error deserializing F123 header, for championship: {championship_id:?}");
                            continue;
                        };

                        if header.m_gameYear.ne(&23) || header.m_packetFormat.ne(&2023) {
                            error!(
                                "Not supported client, Game Year : {}, Packet Format: {}",
                                header.m_gameYear, header.m_packetFormat
                            );
                            break;
                        }

                        let session_id = header.m_sessionUID as i64;

                        if session_id.eq(&0) {
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

                                    let packet = PacketHeader {
                                        r#type: PacketType::CarMotion.into(),
                                        payload: data,
                                    }
                                    .encode_to_vec();

                                    tx.send(packet).unwrap();
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
                                        .await
                                        .unwrap();

                                    let data: PacketSessionData = session_data.into();
                                    let data = data.encode_to_vec();

                                    let packet = PacketHeader {
                                        r#type: PacketType::SessionData.into(),
                                        payload: data,
                                    }
                                    .encode_to_vec();

                                    tx.send(packet).unwrap();
                                    last_session_update = now;
                                }
                            }

                            F123Data::Participants(participants_data) => {
                                if now
                                    .duration_since(last_participants_update)
                                    .ge(&SESSION_INTERVAL)
                                {
                                    redis
                                        .set_ex::<String, &[u8], String>(
                                            format!(
                                                "f123:championship:{}:session:{session_id}:participants",
                                                championship_id
                                            ),
                                            &buf[..size],
                                            DATA_PERSISTENCE,
                                        )
                                        .await
                                        .unwrap();

                                    let data: PacketParticipantsData = participants_data.into();
                                    let data = data.encode_to_vec();

                                    let packet = PacketHeader {
                                        r#type: PacketType::Participants.into(),
                                        payload: data,
                                    }
                                    .encode_to_vec();

                                    tx.send(packet).unwrap();
                                    last_participants_update = now;
                                }
                            }

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

                                let data: PacketEventData = event_data.into();
                                let data = data.encode_to_vec();

                                let packet = PacketHeader {
                                    r#type: PacketType::EventData.into(),
                                    payload: data,
                                }
                                .encode_to_vec();

                                tx.send(packet).unwrap();
                            }

                            // TODO: Check if this is overbooking the server
                            F123Data::SessionHistory(session_history) => {
                                let last_update = last_car_lap_update
                                    .entry(session_history.m_carIdx)
                                    .or_insert(now);

                                if now
                                    .duration_since(*last_update)
                                    .lt(&SESSION_HISTORY_INTERVAL)
                                {
                                    continue;
                                }

                                let lap = session_history.m_numLaps as usize - 1; // Lap is 0 indexed

                                let sectors = (
                                    session_history.m_lapHistoryData[lap].m_sector1TimeInMS,
                                    session_history.m_lapHistoryData[lap].m_sector2TimeInMS,
                                    session_history.m_lapHistoryData[lap].m_sector3TimeInMS,
                                );

                                let last_sectors = car_lap_sector_data
                                    .entry(session_history.m_carIdx)
                                    .or_insert(sectors);

                                if sectors == *last_sectors {
                                    continue;
                                }

                                redis.set_ex::<String, &[u8], String>(
                                    format!("f123:championship:{}:session:{session_id}:history:car:{}", championship_id, session_history.m_carIdx),
                                    &buf[..size],
                                    DATA_PERSISTENCE,
                                )
                                    .await
                                    .unwrap();

                                let data: PacketSessionHistoryData = session_history.into();
                                let data = data.encode_to_vec();

                                let packet = PacketHeader {
                                    r#type: PacketType::SessionHistoryData.into(),
                                    payload: data,
                                }
                                .encode_to_vec();

                                tx.send(packet).unwrap();

                                *last_update = now;
                                *last_sectors = sectors;
                            }

                            //TODO Collect All data from redis and save it to the mariadb database
                            F123Data::FinalClassification(_classification_data) => {
                                // tx.send(F123Data::FinalClassification(classification_data))
                                //     .unwrap();

                                // Self::external_close_socket(
                                //     channels.clone(),
                                //     sockets.clone(),
                                //     championship_id.clone(),
                                // )
                                // .await;
                                // return;
                            }
                        }
                    }

                    Ok(Err(e)) => {
                        error!("Error receiving data from F123 socket: {}", e);
                        info!("Stopping socket for championship: {}", championship_id);

                        Self::external_close_socket(&channels, &sockets, &championship_id).await;

                        break;
                    }

                    Err(_e) => {
                        info!("Socket  timeout for championship: {}", championship_id);
                        Self::external_close_socket(&channels, &sockets, &championship_id).await;
                    }
                }
            }
        })
    }

    pub async fn active_sockets(&self) -> Vec<u32> {
        let sockets = self.sockets.read().await;
        sockets.iter().map(|entry| *entry.0).collect()
    }

    pub async fn championship_socket(&self, id: &u32) -> bool {
        let sockets = self.sockets.read().await;
        sockets.contains_key(id)
    }

    pub async fn stop_socket(&self, championship_id: u32) -> AppResult<()> {
        {
            let mut channels = self.channels.write().await;
            let mut sockets = self.sockets.write().await;

            let channel_removed = channels.remove(&championship_id).is_some();

            let socket_removed_and_aborted = if let Some(socket) = sockets.remove(&championship_id)
            {
                socket.abort();
                true
            } else {
                false
            };

            if !channel_removed && !socket_removed_and_aborted {
                Err(SocketError::NotFound)?
            }
        }

        info!("Socket stopped for championship: {}", championship_id);
        Ok(())
    }

    async fn external_close_socket(channels: &Channels, sockets: &Sockets, championship_id: &u32) {
        let mut sockets = sockets.write().await;
        let mut channels = channels.write().await;

        if let Some(socket) = sockets.remove(championship_id) {
            socket.abort();
        }

        channels.remove(championship_id);
    }

    pub async fn get_receiver(&self, championship_id: &u32) -> Option<Receiver<Vec<u8>>> {
        let channels = self.channels.read().await;
        let channel = channels.get(championship_id);

        Some(channel.unwrap().subscribe())
    }
}
