use crate::{
    cache::F123InsiderCache,
    config::{constants::*, Database},
    error::{AppResult, F123Error, SocketError},
    protos::{packet_header::PacketType, ToProtoMessage},
    services::f123::packet_batching::PacketBatching,
    structs::{F123Data, PacketIds, SectorsLaps, SessionType},
    FirewallService,
};
use ahash::AHashMap;
use bytes::Bytes;
use parking_lot::RwLock;
use std::{cell::RefCell, sync::Arc, time::Instant};
use tokio::{
    net::UdpSocket,
    sync::broadcast::{channel, Receiver, Sender},
    task::JoinHandle,
    time::timeout,
};
use tracing::{error, info};

type ChanelData = Bytes;
type F123Channel = Arc<Sender<ChanelData>>;
type Channels = Arc<RwLock<AHashMap<i32, F123Channel>>>;
type Sockets = Arc<RwLock<AHashMap<i32, JoinHandle<AppResult<()>>>>>;

#[derive(Clone)]
pub struct F123Service {
    db_conn: Database,
    sockets: Sockets,
    channels: Channels,
    firewall: FirewallService,
}

impl F123Service {
    pub fn new(db_conn: &Database, firewall_service: FirewallService) -> Self {
        Self {
            db_conn: db_conn.clone(),
            firewall: firewall_service,
            channels: Arc::new(RwLock::new(AHashMap::default())),
            sockets: Arc::new(RwLock::new(AHashMap::default())),
        }
    }

    pub async fn get_active_socket_ids(&self) -> Vec<i32> {
        let sockets = self.sockets.read();
        sockets.keys().copied().collect()
    }

    pub async fn is_championship_socket_active(&self, id: &i32) -> bool {
        let sockets = self.sockets.read();
        sockets.contains_key(id)
    }

    #[allow(unused)]
    pub async fn subscribe_to_championship_events(
        &self,
        championship_id: &i32,
    ) -> Option<Receiver<ChanelData>> {
        let channels = self.channels.read();
        let Some(channel) = channels.get(championship_id) else {
            return None;
        };

        Some(channel.subscribe())
    }

    pub async fn stop_socket(&self, championship_id: i32) -> AppResult<()> {
        let mut channels = self.channels.write();
        let mut sockets = self.sockets.write();

        if channels.remove(&championship_id).is_none() {
            Err(SocketError::NotFound)?;
        };

        if let Some(socket) = sockets.remove(&championship_id) {
            // todo: Use oneshot channel to stop the socket in the best way possible
            socket.abort();
        } else {
            Err(SocketError::NotFound)?;
        };

        info!("Socket stopped for championship: {}", championship_id);
        Ok(())
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
            sockets.insert(*championship_id, socket);
        });

        Ok(())
    }

    async fn internal_close(
        channels: &Channels,
        sockets: &Sockets,
        championship_id: &i32,
        firewall: &FirewallService,
    ) -> AppResult<()> {
        firewall.close(championship_id).await?;

        let mut sockets = sockets.write();
        let mut channels = channels.write();

        sockets.remove(championship_id);
        channels.remove(championship_id);

        Ok(())
    }

    async fn start_listening_on_socket(
        &self,
        port: i32,
        championship_id: Arc<i32>,
    ) -> JoinHandle<AppResult<()>> {
        let db = self.db_conn.clone();
        let firewall = self.firewall.clone();
        let sockets = self.sockets.clone();
        let channels = self.channels.clone();

        tokio::spawn(async move {
            let mut port_partial_open = false;
            let mut buf = [0u8; BUFFER_SIZE];
            let mut last_session_update = Instant::now();
            let mut last_car_motion_update = Instant::now();
            let mut last_participants_update = Instant::now();
            let session_type = RefCell::new(None);
            let close_socket =
                Self::internal_close(&channels, &sockets, &championship_id, &firewall);

            // Session History Data
            let mut last_car_lap_update: AHashMap<u8, Instant> = AHashMap::default();
            let mut car_lap_sector_data: AHashMap<u8, SectorsLaps> = AHashMap::default();

            // Define channel
            // Todo: Instead of having an external counter use `tx.receiver_count()` to get the active open connections
            let (tx, _) = channel::<ChanelData>(100);

            let cache = F123InsiderCache::new(db.redis.get().await.unwrap(), *championship_id);
            let mut packet_batching = PacketBatching::new(tx.clone(), cache);

            let Ok(socket) = UdpSocket::bind(format!("{SOCKET_HOST}:{port}")).await else {
                error!("There was an error binding to the socket for championship: {championship_id:?}");
                return Err(F123Error::UdpSocket)?;
            };

            info!("Listening for F123 data on port: {port} for championship: {championship_id:?}");

            {
                let mut channels = channels.write();
                channels.insert(*championship_id, Arc::new(tx));
            }

            firewall.open(*championship_id, port).await?;

            loop {
                match timeout(SOCKET_TIMEOUT, socket.recv_from(&mut buf)).await {
                    Ok(Ok((size, address))) => {
                        let buf = &buf[..size];

                        if !port_partial_open {
                            firewall
                                .open_partially(*championship_id, address.ip())
                                .await?;

                            port_partial_open = true;
                        }

                        let Some(header) = F123Data::deserialize_header(buf) else {
                            error!("Error deserializing F123 header, for championship: {championship_id:?}");
                            continue;
                        };

                        if header.packet_format != 2023 {
                            close_socket.await?;
                            return Err(F123Error::UnsupportedPacketFormat)?;
                        };

                        let session_id = header.session_uid;
                        if session_id == 0 {
                            continue;
                        }

                        let now = Instant::now();
                        let Ok(packet_id) = PacketIds::try_from(header.packet_id) else {
                            error!("Error deserializing F123 packet id, for championship: {championship_id:?}");
                            continue;
                        };

                        // TODO: Try to implement this in a more elegant way
                        match packet_id {
                            PacketIds::Motion => {
                                if now.duration_since(last_car_motion_update) < MOTION_INTERVAL {
                                    continue;
                                }
                            }

                            PacketIds::Session => {
                                if now.duration_since(last_session_update) < SESSION_INTERVAL {
                                    continue;
                                }
                            }

                            PacketIds::Participants => {
                                if now.duration_since(last_participants_update) < SESSION_INTERVAL {
                                    continue;
                                }
                            }

                            _ => {}
                        }

                        let Some(packet) = F123Data::deserialize(packet_id, buf) else {
                            continue;
                        };

                        match packet {
                            F123Data::Motion(motion_data) => {
                                let packet = motion_data
                                    .convert(PacketType::CarMotion)
                                    .ok_or(F123Error::Encoding)?;

                                last_car_motion_update = now;
                                packet_batching.push_and_check(packet).await?;
                            }

                            F123Data::Session(session_data) => {
                                #[cfg(not(debug_assertions))]
                                if session_data.network_game != 1 {
                                    error!(
                                        "Not Online Game, closing socket for championship: {}",
                                        championship_id
                                    );

                                    close_socket.await?;
                                    return Err(F123Error::NotOnlineSession)?;
                                }

                                let Ok(converted_session_type) =
                                    SessionType::try_from(session_data.session_type)
                                else {
                                    error!("Error deserializing F123 session type, for championship: {championship_id:?}");
                                    continue;
                                };

                                let _ = session_type.borrow_mut().insert(converted_session_type);

                                let packet = session_data
                                    .convert(PacketType::SessionData)
                                    .ok_or(F123Error::Encoding)?;

                                last_session_update = now;
                                packet_batching.push_and_check(packet).await?;
                            }

                            F123Data::Participants(participants_data) => {
                                let packet = participants_data
                                    .convert(PacketType::Participants)
                                    .ok_or(F123Error::Encoding)?;

                                last_participants_update = now;
                                packet_batching.push_and_check(packet).await?;
                            }

                            F123Data::Event(event_data) => {
                                let Some(packet) = event_data.convert(PacketType::EventData) else {
                                    continue;
                                };

                                packet_batching.push_and_check(packet).await?;
                            }

                            F123Data::SessionHistory(session_history) => {
                                let last_update = last_car_lap_update
                                    .entry(session_history.car_idx)
                                    .or_insert(now);

                                if now.duration_since(*last_update) > HISTORY_INTERVAL {
                                    let lap = (session_history.num_laps as usize) - 1; // Lap is 0 indexed

                                    let sectors = SectorsLaps {
                                        sector1: session_history.lap_history_data[lap]
                                            .sector1_time_in_ms,
                                        sector2: session_history.lap_history_data[lap]
                                            .sector2_time_in_ms,
                                        sector3: session_history.lap_history_data[lap]
                                            .sector3_time_in_ms,
                                    };

                                    let last_sectors = car_lap_sector_data
                                        .entry(session_history.car_idx)
                                        .or_insert(sectors);

                                    if sectors == *last_sectors {
                                        *last_update = now;
                                        continue;
                                    }

                                    let packet = session_history
                                        .convert(PacketType::SessionHistoryData)
                                        .ok_or(F123Error::Encoding)?;

                                    *last_update = now;
                                    *last_sectors = sectors;

                                    packet_batching.push_and_check(packet).await?;
                                }
                            }

                            //TODO Collect All data from redis and save it to the mariadb database
                            F123Data::FinalClassification(classification_data) => {
                                let packet = classification_data
                                    .convert(PacketType::FinalClassificationData)
                                    .ok_or(F123Error::Encoding)?;

                                // Only for testing purposes, in the future this should close the socket when the race is finished
                                {
                                    let session_type = session_type.borrow();

                                    info!("Session type: {:?}", session_type);

                                    if let SessionType::R | SessionType::R2 | SessionType::R3 =
                                        session_type.as_ref().unwrap()
                                    {
                                        info!("Race Finished, saving final classification data");
                                    }
                                }

                                // If session type is race save all session data in the database and close the socket
                                // Todo: this should be called after saving all data in the database
                                packet_batching.final_send(packet).await?;
                            }
                        }
                    }

                    Ok(Err(e)) => {
                        error!("Error receiving data from udp socket: {}", e);
                        info!("Stopping socket for championship: {}", championship_id);
                        close_socket.await?;
                        return Err(F123Error::ReceivingData)?;
                    }

                    Err(_) => {
                        info!("Socket timeout for championship: {}", championship_id);
                        close_socket.await?;
                        return Ok(());
                    }
                }
            }
        })
    }
}
