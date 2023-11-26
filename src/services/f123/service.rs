use crate::cache::F123InsiderCache;
use crate::config::constants::*;
use crate::services::f123::packet_batching::PacketBatching;
use crate::FirewallService;
use crate::{
    config::Database,
    dtos::F123Data,
    error::{AppResult, SocketError},
    protos::{packet_header::PacketType, ToProtoMessage},
};
use flume::{bounded, Receiver};
use log::{error, info};
use ntex::rt;
use parking_lot::RwLock;
use rustc_hash::FxHashMap;
use std::{sync::Arc, time::Instant};
use tokio::{net::UdpSocket, task::JoinHandle, time::timeout};

type ChanelData = Vec<u8>;
type F123Channel = Arc<Receiver<ChanelData>>;
type Channels = Arc<RwLock<FxHashMap<i32, F123Channel>>>;
type Sockets = Arc<RwLock<FxHashMap<i32, Arc<JoinHandle<()>>>>>;

pub struct F123Service {
    db_conn: Arc<Database>,
    sockets: Sockets,
    channels: Channels,
    firewall: Arc<FirewallService>,
}

// TODO: Check implementation of the RwLock
impl F123Service {
    pub fn new(db_conn: &Arc<Database>, firewall_service: Arc<FirewallService>) -> Self {
        Self {
            db_conn: db_conn.clone(),
            firewall: firewall_service,
            channels: Arc::new(RwLock::new(FxHashMap::default())),
            sockets: Arc::new(RwLock::new(FxHashMap::default())),
        }
    }

    pub async fn setup_championship_listening_socket(
        &self,
        port: i32,
        championship_id: Arc<i32>,
    ) -> AppResult<()> {
        let mut sockets = self.sockets.upgradable_read();

        {
            let channels = self.channels.read();

            if sockets.contains_key(&championship_id) || channels.contains_key(&championship_id) {
                error!("Trying to create a new socket or channel for an existing championship: {championship_id:?}");
                return Err(SocketError::AlreadyExists.into());
            }
        }

        let socket = self
            .start_listening_on_socket(port, championship_id.clone())
            .await;

        sockets.with_upgraded(|sockets| {
            sockets.insert(*championship_id, Arc::new(socket));
        });

        Ok(())
    }

    pub async fn get_active_socket_ids(&self) -> Vec<i32> {
        let sockets = self.sockets.read();
        sockets.keys().copied().collect()
    }

    pub async fn stop_socket(&self, championship_id: i32) -> AppResult<()> {
        let mut channels = self.channels.write();
        let mut sockets = self.sockets.write();

        let channel_removed = channels.remove(&championship_id).is_some();

        let socket_removed_and_aborted = if let Some(socket) = sockets.remove(&championship_id) {
            socket.abort();
            true
        } else {
            false
        };

        if !channel_removed && !socket_removed_and_aborted {
            Err(SocketError::NotFound)?;
        }

        info!("Socket stopped for championship: {}", championship_id);
        Ok(())
    }

    pub async fn is_championship_socket_active(&self, id: &i32) -> bool {
        let sockets = self.sockets.read();
        sockets.contains_key(id)
    }

    #[allow(unused)]
    pub async fn subscribe_to_championship_events(
        &self,
        championship_id: &i32,
    ) -> Option<Arc<Receiver<ChanelData>>> {
        let channels = self.channels.read();
        let Some(channel) = channels.get(championship_id) else {
            return None;
        };

        Some(channel.clone())
    }

    async fn external_close_socket(channels: &Channels, sockets: &Sockets, championship_id: &i32) {
        let mut sockets = sockets.write();
        let mut channels = channels.write();

        if let Some(socket) = sockets.remove(championship_id) {
            socket.abort();
        }

        channels.remove(championship_id);
    }

    async fn start_listening_on_socket(
        &self,
        port: i32,
        championship_id: Arc<i32>,
    ) -> JoinHandle<()> {
        let db = self.db_conn.clone();
        let firewall = self.firewall.clone();
        let sockets = self.sockets.clone();
        let channels = self.channels.clone();

        rt::spawn(async move {
            let mut port_partial_open = false;
            let mut buf = [0u8; BUFFER_SIZE];
            let mut cache = F123InsiderCache::new(db.redis.get().await.unwrap(), *championship_id);

            let mut last_session_update = Instant::now();
            let mut last_car_motion_update = Instant::now();
            let mut last_participants_update = Instant::now();

            // Session History Data
            let mut last_car_lap_update: FxHashMap<u8, Instant> = FxHashMap::default();
            let mut car_lap_sector_data: FxHashMap<u8, (u16, u16, u16)> = FxHashMap::default();

            // Define channel
            let (tx, rx) = bounded::<ChanelData>(50);
            let mut packet_batching = PacketBatching::new(tx.clone());

            let Ok(socket) = UdpSocket::bind(format!("{SOCKET_HOST}:{port}")).await else {
                error!("There was an error binding to the socket for championship: {championship_id:?}");
                return;
            };

            info!("Listening for F123 data on port: {port} for championship: {championship_id:?}");

            {
                let mut channels = channels.write();
                channels.insert(*championship_id, Arc::new(rx));
            }

            firewall.open(*championship_id, port).await.unwrap();

            loop {
                match timeout(SOCKET_TIMEOUT, socket.recv_from(&mut buf)).await {
                    Ok(Ok((size, address))) => {
                        let buf = &buf[..size];

                        if !port_partial_open {
                            firewall
                                .open_partially(*championship_id, address.ip())
                                .await
                                .unwrap();

                            port_partial_open = true;
                        }

                        let Some(header) = F123Data::deserialize_header(buf) else {
                            error!("Error deserializing F123 header, for championship: {championship_id:?}");
                            continue;
                        };

                        if header.packet_format != 2023 {
                            error!("Not supported client");
                            break;
                        };

                        let session_id = header.session_uid;
                        if session_id.eq(&0) {
                            continue;
                        }

                        let Some(packet) = F123Data::deserialize(header.packet_id.into(), buf)
                        else {
                            continue;
                        };

                        let now = Instant::now();

                        match packet {
                            F123Data::Motion(motion_data) => {
                                if now.duration_since(last_car_motion_update) > MOTION_INTERVAL {
                                    let packet = motion_data
                                        .convert_and_encode(PacketType::CarMotion)
                                        .expect("Error converting motion data to proto message");

                                    if let Err(e) = cache.set_motion_data(&packet).await {
                                        error!("error saving motion_data: {}", e);
                                    };

                                    packet_batching.push_and_check(packet).await;
                                    last_car_motion_update = now;
                                }
                            }

                            F123Data::Session(session_data) => {
                                if now.duration_since(last_session_update) > SESSION_INTERVAL {
                                    let packet = session_data
                                        .convert_and_encode(PacketType::SessionData)
                                        .expect("Error converting session data to proto message");

                                    if let Err(e) = cache.set_session_data(&packet).await {
                                        error!("error saving session_data: {}", e);
                                    };

                                    packet_batching.push_and_check(packet).await;
                                    last_session_update = now;
                                }
                            }

                            F123Data::Participants(participants_data) => {
                                if now.duration_since(last_participants_update) > SESSION_INTERVAL {
                                    let packet = participants_data
                                        .convert_and_encode(PacketType::Participants)
                                        .expect(
                                            "Error converting participants data to proto message",
                                        );

                                    if let Err(e) = cache.set_participants_data(&packet).await {
                                        error!("error saving participants_data: {}", e);
                                    };

                                    packet_batching.push_and_check(packet).await;
                                    last_participants_update = now;
                                }
                            }

                            F123Data::Event(event_data) => {
                                let Some(packet) =
                                    event_data.convert_and_encode(PacketType::EventData)
                                else {
                                    continue;
                                };

                                let string_code =
                                    std::str::from_utf8(&event_data.event_string_code).unwrap();

                                if let Err(e) = cache.push_event_data(&packet, string_code).await {
                                    error!("error pushing event_data: {}", e);
                                };

                                packet_batching.push_and_check(packet).await;
                            }

                            F123Data::SessionHistory(session_history) => {
                                let last_update = last_car_lap_update
                                    .entry(session_history.car_idx)
                                    .or_insert(now);

                                if now.duration_since(*last_update) > HISTORY_INTERVAL {
                                    let lap = (session_history.num_laps as usize) - 1; // Lap is 0 indexed

                                    let sectors = (
                                        session_history.lap_history_data[lap].sector1_time_in_ms,
                                        session_history.lap_history_data[lap].sector2_time_in_ms,
                                        session_history.lap_history_data[lap].sector3_time_in_ms,
                                    );

                                    let last_sectors = car_lap_sector_data
                                        .entry(session_history.car_idx)
                                        .or_insert(sectors);

                                    if sectors == *last_sectors {
                                        *last_update = now;
                                        continue;
                                    }

                                    let car_idx = session_history.car_idx;
                                    let packet = session_history
                                        .convert_and_encode(PacketType::SessionHistoryData)
                                        .expect("Error converting history data to proto message");

                                    if let Err(e) =
                                        cache.set_session_history(&packet, &car_idx).await
                                    {
                                        error!("error saving participants_data: {}", e);
                                    };

                                    packet_batching.push_and_check(packet).await;

                                    *last_update = now;
                                    *last_sectors = sectors;
                                }
                            }

                            //TODO Collect All data from redis and save it to the mariadb database
                            F123Data::FinalClassification(classification_data) => {
                                let packet = classification_data
                                    .convert_and_encode(PacketType::FinalClassificationData)
                                    .expect("Error converting final classification data to proto message");

                                // TODO: If session type is race save all session data in the database and close the socket
                                packet_batching.push_and_check(packet).await;
                            }
                        }
                    }

                    Ok(Err(e)) => {
                        error!("Error receiving data from F123 socket: {}", e);
                        info!("Stopping socket for championship: {}", championship_id);
                        firewall.close(&championship_id).await.unwrap();
                        Self::external_close_socket(&channels, &sockets, &championship_id).await;
                        break;
                    }

                    Err(_) => {
                        info!("Socket  timeout for championship: {}", championship_id);
                        firewall.close(&championship_id).await.unwrap();
                        Self::external_close_socket(&channels, &sockets, &championship_id).await;
                    }
                }
            }
        })
    }
}
