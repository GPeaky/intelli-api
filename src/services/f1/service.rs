use std::sync::Arc;

use ahash::AHashMap;
use dashmap::DashMap;
use ntex::util::Bytes;
use parking_lot::RwLock;
use tokio::{
    net::UdpSocket,
    sync::{
        broadcast::{Receiver, Sender},
        oneshot,
    },
    time::{timeout, Instant},
};
use tracing::{error, info, info_span};

use crate::{
    config::constants::{
        BUFFER_SIZE, HISTORY_INTERVAL, MOTION_INTERVAL, SESSION_INTERVAL, SOCKET_HOST,
        SOCKET_TIMEOUT,
    },
    error::{AppResult, CommonError, F1ServiceError},
    protos::{packet_header::PacketType, ToProtoMessage},
    structs::{
        F1Data, OptionalMessage, PacketCarDamageData, PacketCarStatusData, PacketCarTelemetryData,
        PacketEventData, PacketFinalClassificationData, PacketMotionData, PacketParticipantsData,
        PacketSessionData, PacketSessionHistoryData, SectorsLaps, SessionType,
    },
};

use super::{batching::PacketBatching, firewall::FirewallService, PacketCaching};

pub struct F1Service {
    port: i32,
    championship_id: i32,
    port_partially_opened: bool,
    last_updates: LastUpdates,
    socket: UdpSocket,
    shutdown: oneshot::Receiver<()>,
    session_type: Option<SessionType>,
    car_lap_sector: AHashMap<u8, SectorsLaps>,
    packet_batching: PacketBatching,
    firewall: &'static FirewallService,
    services: &'static DashMap<i32, F1ServiceData>,
}

pub struct F1ServiceData {
    pub cache: Arc<RwLock<PacketCaching>>,
    pub channel: Arc<Receiver<Bytes>>,
    pub shutdown: Option<oneshot::Sender<()>>,
}

struct LastUpdates {
    session: Instant,
    car_motion: Instant,
    participants: Instant,
    car_lap: AHashMap<u8, Instant>,
}

impl F1Service {
    pub async fn new(
        tx: Sender<Bytes>,
        shutdown: oneshot::Receiver<()>,
        cache: Arc<RwLock<PacketCaching>>,
        firewall: &'static FirewallService,
        services: &'static DashMap<i32, F1ServiceData>,
    ) -> Self {
        F1Service {
            port: 0,
            championship_id: 0,
            port_partially_opened: false,
            last_updates: LastUpdates::new(),
            shutdown,
            socket: UdpSocket::bind("0.0.0.0:0").await.unwrap(),
            session_type: None,
            car_lap_sector: AHashMap::with_capacity(20),
            packet_batching: PacketBatching::new(tx, cache),
            firewall,
            services,
        }
    }

    pub async fn initialize(&mut self, port: i32, championship_id: i32) -> AppResult<()> {
        self.port = port;
        self.championship_id = championship_id;

        let Ok(socket) = UdpSocket::bind(format!("{SOCKET_HOST}:{port}")).await else {
            error!("There was an error binding to the socket");
            return Err(CommonError::InternalServerError)?;
        };

        self.socket = socket;

        self.firewall
            .open(self.championship_id, self.port as u16)
            .await?;

        Ok(())
    }

    pub async fn run(&mut self) {
        let span = info_span!("F1 Service", championship_id = self.championship_id);
        let _guard = span.enter();

        info!("Listening for F1 data on port: {}", self.port);

        let mut buf = [0u8; BUFFER_SIZE];

        loop {
            tokio::select! {
                _ = &mut self.shutdown => {
                    info!("Shutting down service");
                    self.close().await;
                    break;
                }

                result = timeout(SOCKET_TIMEOUT, self.socket.recv_from(&mut buf)) => {
                    match result {
                        Ok(Ok((size, address))) => {
                            let buf = &buf[..size];
                            let now = Instant::now();

                            if !self.port_partially_opened {
                                if self
                                    .firewall
                                    .restrict_to_ip(self.championship_id, address.ip().to_string())
                                    .await
                                    .is_err()
                                {
                                    error!("Error restricting port to ip");
                                    self.close().await;
                                    break;
                                }

                                self.port_partially_opened = true;
                            }

                            let _ = self.process_packet(buf, now).await;
                        }

                        Ok(Err(e)) => {
                            error!("Error receiving data from udp socket: {}", e);
                            info!("Stopping socket");
                            self.close().await;
                            break;
                        }

                        Err(_) => {
                            info!("Service Timeout");
                            self.close().await;
                            break;
                        }
                    }
                }
            }
        }
    }

    #[inline]
    async fn process_packet(&mut self, buf: &[u8], now: Instant) -> AppResult<()> {
        let (header, packet) = F1Data::try_cast(buf)?;

        if header.packet_format != 2023 {
            return Err(F1ServiceError::UnsupportedFormat)?;
        }

        if header.session_uid == 0 {
            return Ok(());
        }

        match packet {
            F1Data::Motion(motion_data) => self.handle_motion_packet(motion_data, now),
            F1Data::Session(session_data) => self.handle_session_packet(session_data, now).await,
            F1Data::Participants(participants_data) => {
                self.handle_participants_packet(participants_data, now)
            }
            F1Data::Event(event_data) => self.handle_event_packet(event_data),
            F1Data::SessionHistory(session_history_data) => {
                self.handle_session_history_packet(session_history_data, now)
            }
            F1Data::FinalClassification(final_classification) => {
                self.handle_final_classification_packet(final_classification)
            }
            F1Data::CarDamage(car_damage) => self.handle_car_damage_packet(car_damage),
            F1Data::CarStatus(car_status) => self.handle_car_status_packet(car_status),
            F1Data::CarTelemetry(car_telemetry) => self.handle_car_telemetry_packet(car_telemetry),
        }

        Ok(())
    }

    #[inline]
    fn handle_motion_packet(&mut self, motion_data: &PacketMotionData, now: Instant) {
        if now.duration_since(self.last_updates.car_motion) < MOTION_INTERVAL {
            return;
        }

        let packet = motion_data.convert(PacketType::CarMotion).unwrap();
        self.last_updates.car_motion = now;
        self.packet_batching.push(packet);
    }

    #[inline]
    async fn handle_session_packet(&mut self, session_data: &PacketSessionData, now: Instant) {
        if now.duration_since(self.last_updates.session) < SESSION_INTERVAL {
            return;
        }

        // #[cfg(not(debug_assertions))]
        // if session_data.network_game != 1 {
        //     error!("Not Online Game, closing service");
        //     self.close().await;
        //     return;
        // }

        let Ok(session_type) = SessionType::try_from(session_data.session_type) else {
            error!("Error deserializing F1 session type");
            return;
        };

        self.session_type = Some(session_type);
        let packet = session_data.convert(PacketType::SessionData).unwrap();
        self.last_updates.session = now;
        self.packet_batching.push(packet);
    }

    #[inline]
    fn handle_participants_packet(
        &mut self,
        participants_data: &PacketParticipantsData,
        now: Instant,
    ) {
        if now.duration_since(self.last_updates.participants) < SESSION_INTERVAL {
            return;
        }

        let packet = participants_data.convert(PacketType::Participants).unwrap();

        self.last_updates.participants = now;
        self.packet_batching.push(packet);
    }

    #[inline]
    fn handle_event_packet(&mut self, event_data: &PacketEventData) {
        let Some(session_type) = &self.session_type else {
            return;
        };

        if ![SessionType::R, SessionType::R2, SessionType::R3].contains(session_type) {
            return;
        }

        let Some(packet) = event_data.convert(PacketType::EventData) else {
            return;
        };

        self.packet_batching.push_with_optional_parameter(
            packet,
            Some(OptionalMessage::Code(event_data.event_string_code)),
        )
    }

    #[inline]
    fn handle_session_history_packet(
        &mut self,
        history_data: &PacketSessionHistoryData,
        now: Instant,
    ) {
        let last_update = self
            .last_updates
            .car_lap
            .entry(history_data.car_idx)
            .or_insert(now);

        if now.duration_since(*last_update) > HISTORY_INTERVAL {
            let lap = (history_data.num_laps as usize) - 1;

            let sectors = SectorsLaps {
                s1: history_data.lap_history_data[lap].sector1_time_in_ms,
                s2: history_data.lap_history_data[lap].sector2_time_in_ms,
                s3: history_data.lap_history_data[lap].sector3_time_in_ms,
            };

            let last_sectors = self
                .car_lap_sector
                .entry(history_data.car_idx)
                .or_insert(sectors);

            if sectors == *last_sectors {
                *last_update = now;
                return;
            }

            *last_sectors = sectors;
            let packet = history_data
                .convert(PacketType::SessionHistoryData)
                .unwrap();

            self.packet_batching.push_with_optional_parameter(
                packet,
                Some(OptionalMessage::Number(history_data.car_idx)),
            )
        }
    }

    #[inline]
    fn handle_final_classification_packet(
        &mut self,
        final_classification: &PacketFinalClassificationData,
    ) {
        info!("FinalClassification data received");

        let packet = final_classification
            .convert(PacketType::FinalClassificationData)
            .unwrap();

        {
            info!("Session type: {:?}", self.session_type);

            if let SessionType::R | SessionType::R2 | SessionType::R3 =
                self.session_type.as_ref().unwrap()
            {
                info!("Race Finished, saving final classification data");
            }
        }

        self.packet_batching.push(packet);
    }

    #[inline]
    fn handle_car_damage_packet(&mut self, _car_damage: &PacketCarDamageData) {
        // info!("Car damage: {:?}", car_damage);
    }

    #[inline]
    fn handle_car_status_packet(&mut self, _car_status: &PacketCarStatusData) {
        // info!("Car status: {:?}", car_status);
    }

    #[inline]
    fn handle_car_telemetry_packet(&mut self, _car_telemetry: &PacketCarTelemetryData) {
        // info!("Car telemetry: {:?}", car_telemetry);
    }

    #[inline]
    async fn close(&self) {
        if self.firewall.close(self.championship_id).await.is_err() {
            error!("Error closing port in firewall");
        }

        // Todo: Search a better way to do this without having to save a reference of all active services
        self.services.remove(&self.championship_id);
    }
}

impl F1ServiceData {
    pub fn new(channel: Arc<Receiver<Bytes>>, shutdown: oneshot::Sender<()>) -> Self {
        let cache = Arc::new(RwLock::new(PacketCaching::new()));

        Self {
            cache,
            channel,
            shutdown: Some(shutdown),
        }
    }
}

impl LastUpdates {
    fn new() -> Self {
        let time = Instant::now();

        Self {
            session: time,
            car_motion: time,
            participants: time,
            car_lap: AHashMap::with_capacity(20),
        }
    }
}
