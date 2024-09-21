use std::{
    net::SocketAddr,
    sync::{
        atomic::{AtomicU32, Ordering},
        Arc,
    },
};

use ahash::AHashMap;
use chrono::Utc;
use dashmap::DashMap;
use ntex::util::Bytes;
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
        SOCKET_TIMEOUT, TELEMETRY_INTERVAL,
    },
    error::{AppResult, CommonError, F1ServiceError},
    services::{ChampionshipServiceOperations, DriverServiceOperations},
    states::F1State,
    structs::{
        F1PacketData, PacketCarDamageData, PacketCarStatusData, PacketCarTelemetryData,
        PacketEventData, PacketFinalClassificationData, PacketMotionData, PacketParticipantsData,
        PacketSessionData, PacketSessionHistoryData, SessionType,
    },
};

use super::manager::F1SessionDataManager;

const PARTICIPANTS_TICK_UPDATE: u8 = 6; // 6 * 10 seconds = 600 seconds (1 minutes)

/// Represents an F1 service that processes and manages F1 telemetry data.
pub struct F1Service {
    port: i32,
    race_id: i32,
    tick_counter: u8,
    championship_id: i32,
    port_partially_opened: bool,
    last_updates: LastUpdates,
    socket: UdpSocket,
    shutdown: oneshot::Receiver<()>,
    session_type: Option<SessionType>,
    data_manager: F1SessionDataManager,
    services: &'static DashMap<i32, F1ServiceData>,
    f1_state: &'static F1State,
}

/// Holds data related to an F1 service instance.
pub struct F1ServiceData {
    channel: Arc<Receiver<Bytes>>,
    counter: Arc<AtomicU32>,
    shutdown: Option<oneshot::Sender<()>>,
}

/// Tracks the last update times for various packet types.
struct LastUpdates {
    session: Instant,
    car_motion: Instant,
    car_status: Instant,
    car_damage: Instant,
    car_telemetry: Instant,
    participants: Instant,
    car_lap: AHashMap<u8, Instant>,
}

impl F1Service {
    /// Creates a new F1Service instance.
    ///
    /// # Arguments
    /// - `tx`: Sender for broadcasting data.
    /// - `shutdown`: Receiver for shutdown signal.
    /// - `cache`: Shared cache for packet data.
    /// - `firewall`: Firewall service.
    /// - `services`: Map of active services.
    ///
    /// # Returns
    /// A new F1Service instance.
    pub async fn new(
        _tx: Sender<Bytes>,
        shutdown: oneshot::Receiver<()>,
        services: &'static DashMap<i32, F1ServiceData>,
        f1_state: &'static F1State,
    ) -> Self {
        F1Service {
            port: 0,
            race_id: 0,
            championship_id: 0,
            tick_counter: 10,
            port_partially_opened: false,
            last_updates: LastUpdates::new(),
            shutdown,
            socket: UdpSocket::bind("0.0.0.0:0").await.unwrap(),
            session_type: None,
            data_manager: F1SessionDataManager::new(),
            services,
            f1_state,
        }
    }

    /// Initializes the F1 service with a specific port and championship ID.
    ///
    /// # Arguments
    /// - `port`: Port number to bind the service to.
    /// - `championship_id`: ID of the championship.
    ///
    /// # Returns
    /// Result indicating success or failure.
    pub async fn initialize(
        &mut self,
        port: i32,
        championship_id: i32,
        _race_id: i32,
    ) -> AppResult<()> {
        let Ok(socket) = UdpSocket::bind(SocketAddr::new(SOCKET_HOST, port as u16)).await else {
            error!("There was an error binding to the socket");
            return Err(CommonError::InternalServerError)?;
        };

        // Only for testing this should be send it in the initialize function
        let race_id = self
            .f1_state
            .championship_svc
            .create_race(championship_id, 10, Utc::now())
            .await?;

        self.port = port;
        self.socket = socket;
        self.race_id = race_id;
        self.championship_id = championship_id;

        self.f1_state
            .firewall
            .open(self.championship_id, self.port as u16)
            .await?;

        Ok(())
    }

    /// Runs the main loop of the F1 service, processing incoming packets.
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
                                    .f1_state
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

                            if let Err(e) = self.process_packet(buf, now).await {
                                error!("Error processing packet: {}", e);
                                self.close().await;
                                break;
                            }
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

    /// Processes a single packet of F1 telemetry data.
    ///
    /// # Arguments
    /// - `buf`: Buffer containing the packet data.
    /// - `now`: Current timestamp.
    ///
    /// # Returns
    /// Result indicating success or failure.
    #[inline(always)]
    async fn process_packet(&mut self, buf: &[u8], now: Instant) -> AppResult<()> {
        let (header, packet) = match F1PacketData::parse_and_identify(buf) {
            Ok(result) => result,
            Err(_) => return Ok(()),
        };

        if header.packet_format != 2024 {
            return Err(F1ServiceError::UnsupportedFormat)?;
        }

        if header.session_uid == 0 {
            return Ok(());
        }

        match packet {
            F1PacketData::Motion(motion_data) => self.handle_motion_packet(motion_data, now),
            F1PacketData::Session(session_data) => {
                self.handle_session_packet(session_data, now).await
            }
            F1PacketData::Participants(participants_data) => {
                self.handle_participants_packet(participants_data, now)
                    .await?
            }
            F1PacketData::Event(event_data) => self.handle_event_packet(event_data),
            F1PacketData::SessionHistory(session_history_data) => {
                self.handle_session_history_packet(session_history_data, now)
            }
            F1PacketData::FinalClassification(final_classification) => {
                self.handle_final_classification_packet(final_classification)
                    .await?
            }
            F1PacketData::CarDamage(car_damage) => self.handle_car_damage_packet(car_damage, now),
            F1PacketData::CarStatus(car_status) => self.handle_car_status_packet(car_status, now),
            F1PacketData::CarTelemetry(car_telemetry) => {
                self.handle_car_telemetry_packet(car_telemetry, now)
            }
        }

        Ok(())
    }

    #[inline(always)]
    fn handle_motion_packet(&mut self, motion_data: &PacketMotionData, now: Instant) {
        if now.duration_since(self.last_updates.car_motion) < MOTION_INTERVAL {
            return;
        }

        self.data_manager.save_motion(motion_data);
        self.last_updates.car_motion = now;
    }

    #[inline(always)]
    async fn handle_session_packet(&mut self, session_data: &PacketSessionData, now: Instant) {
        if now.duration_since(self.last_updates.session) < SESSION_INTERVAL {
            return;
        }

        #[cfg(not(debug_assertions))]
        if session_data.network_game != 1 {
            error!("Not Online Game, closing service");
            self.close().await;
            return;
        }

        let Ok(session_type) = SessionType::try_from(session_data.session_type) else {
            error!("Error deserializing F1 session type");
            return;
        };

        self.session_type = Some(session_type);
        self.data_manager.save_session(session_data);
        self.last_updates.session = now;
    }

    #[inline(always)]
    async fn handle_participants_packet(
        &mut self,
        participants_data: &PacketParticipantsData,
        now: Instant,
    ) -> AppResult<()> {
        if now.duration_since(self.last_updates.participants) < SESSION_INTERVAL {
            return Ok(());
        }

        self.tick_counter += 1;

        if self.tick_counter >= PARTICIPANTS_TICK_UPDATE {
            self.tick_counter = 0;

            self.ensure_participants_registered(participants_data)
                .await?;
        }

        self.data_manager.save_participants(participants_data);
        self.last_updates.participants = now;
        Ok(())
    }

    #[inline(always)]
    fn handle_event_packet(&mut self, _event_data: &PacketEventData) {
        let Some(_session_type) = &self.session_type else {
            return;
        };

        // if ![SessionType::R, SessionType::R2, SessionType::R3].contains(session_type) {
        //     return;
        // }

        // let Some(_packet) = event_data.to_packet_header() else {
        //     return;
        // };

        // self.packet_batching.push_with_optional_parameter(
        //     packet,
        //     Some(PacketExtraData::EventCode(event_data.event_string_code)),
        // )
    }

    #[inline(always)]
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
            self.data_manager.save_lap_history(history_data);
            *last_update = now;
        }
    }

    #[inline(always)]
    async fn handle_final_classification_packet(
        &mut self,
        final_classification: &PacketFinalClassificationData,
    ) -> AppResult<()> {
        let Some(_session_type) = self.session_type.take() else {
            error!("Not defined session type when trying to save final_classification_data");
            return Ok(());
        };

        // Only testing, we should save last lastHistoryData with the final_classification as a tuple or something
        // self.f1_state
        //     .championship_svc
        //     .add_race_result(self.race_id, session_type as i16, &[0, 0, 0])
        //     .await?;

        self.data_manager
            .save_final_classification(final_classification);

        Ok(())
    }

    // TODO: Limit updates to 11hz or something
    #[inline(always)]
    fn handle_car_damage_packet(&mut self, car_damage: &PacketCarDamageData, now: Instant) {
        if now.duration_since(self.last_updates.car_damage) > TELEMETRY_INTERVAL {
            self.data_manager.save_car_damage(car_damage);
        }
    }

    #[inline(always)]
    fn handle_car_status_packet(&mut self, car_status: &PacketCarStatusData, now: Instant) {
        if now.duration_since(self.last_updates.car_status) > TELEMETRY_INTERVAL {
            self.data_manager.save_car_status(car_status);
        }
    }

    #[inline(always)]
    fn handle_car_telemetry_packet(
        &mut self,
        car_telemetry: &PacketCarTelemetryData,
        now: Instant,
    ) {
        if now.duration_since(self.last_updates.car_telemetry) > TELEMETRY_INTERVAL {
            self.data_manager.save_car_telemetry(car_telemetry);
        }
    }

    #[inline(always)]
    async fn ensure_participants_registered(
        &self,
        participants_data: &PacketParticipantsData,
    ) -> AppResult<()> {
        let mut drivers = self
            .f1_state
            .championship_repo
            .drivers_linked(self.championship_id)
            .await?;

        drivers.sort_unstable();

        for idx in 0..participants_data.num_active_cars {
            let Some(participant) = participants_data.participants.get(idx as usize) else {
                error!("Participant id out of bounce");
                continue;
            };

            // 255 means an online driver
            if participant.driver_id != 255 {
                continue;
            }

            let Some(steam_name) = participant.steam_name() else {
                error!("Error getting steam name");
                continue;
            };

            if steam_name == "Player" {
                continue;
            }

            if self.f1_state.driver_repo.find(steam_name).await?.is_none() {
                self.f1_state
                    .driver_svc
                    .create(steam_name, participant.nationality as i16, None)
                    .await?;
            }

            if drivers
                .binary_search_by(|probe| probe.as_str().cmp(steam_name))
                .is_err()
            {
                self.f1_state
                    .championship_svc
                    .add_driver(
                        self.championship_id,
                        steam_name,
                        participant.team_id as i16,
                        participant.race_number as i16,
                    )
                    .await?;
            }
        }

        Ok(())
    }

    /// Closes the F1 service, releasing resources and removing it from active services.
    async fn close(&self) {
        if self
            .f1_state
            .firewall
            .close(self.championship_id)
            .await
            .is_err()
        {
            error!("Error closing port in firewall");
        }

        self.services.remove(&self.championship_id);
    }
}

impl F1ServiceData {
    /// Creates a new F1ServiceData instance.
    ///
    /// # Arguments
    /// - `channel`: Receiver for broadcast channel.
    /// - `shutdown`: Sender for shutdown signal.
    ///
    /// # Returns
    /// A new F1ServiceData instance.
    pub fn new(channel: Arc<Receiver<Bytes>>, shutdown: oneshot::Sender<()>) -> Self {
        // let cache = Arc::new(RwLock::new(PacketCaching::new()));

        Self {
            // cache,
            channel,
            shutdown: Some(shutdown),
            counter: Arc::new(AtomicU32::new(0)),
        }
    }

    /// Subscribes to the service's broadcast channel.
    ///
    /// # Returns
    /// A new Receiver for the broadcast channel.
    pub fn subscribe(&self) -> Receiver<Bytes> {
        self.counter.fetch_add(1, Ordering::Relaxed);
        self.channel.resubscribe()
    }

    /// Gets the current number of subscribers.
    ///
    /// # Returns
    /// The number of active subscribers.
    pub fn subscribers_count(&self) -> u32 {
        self.counter.load(Ordering::Relaxed)
    }

    /// Decrements the subscriber count when a client unsubscribes.
    pub fn unsubscribe(&self) {
        self.counter.fetch_sub(1, Ordering::Relaxed);
    }

    /// Initiates the shutdown process for the service.
    ///
    /// # Returns
    /// Result indicating success or failure of sending the shutdown signal.
    pub fn shutdown(&mut self) -> Result<(), ()> {
        self.shutdown.take().unwrap().send(())
    }
}

impl LastUpdates {
    /// Creates a new LastUpdates instance with current time for all fields.
    fn new() -> Self {
        let time = Instant::now();

        Self {
            session: time,
            car_motion: time,
            participants: time,
            car_damage: time,
            car_status: time,
            car_telemetry: time,
            car_lap: AHashMap::with_capacity(20),
        }
    }
}
