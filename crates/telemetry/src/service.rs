use std::{
    mem,
    net::{IpAddr, Ipv4Addr, SocketAddr},
    ops::Deref,
    sync::{
        atomic::{AtomicU32, Ordering},
        Arc,
    },
    time::Duration,
};

use ahash::AHashMap;
use chrono::Utc;
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
use tracing::{error, info, info_span, warn};

use error::{AppResult, CommonError, F1ServiceError};
use intelli_core::services::{ChampionshipServiceOperations, DriverServiceOperations};

use crate::{
    f1::{
        PacketCarDamageData, PacketCarStatusData, PacketCarTelemetryData, PacketEventData,
        PacketFinalClassificationData, PacketHeader, PacketIds, PacketMotionData,
        PacketParticipantsData, PacketSessionData, PacketSessionHistoryData, SessionType,
    },
    F1State,
};

use super::manager::F1SessionDataManager;

// Constants
const BUFFER_SIZE: usize = 1460;
const SOCKET_HOST: IpAddr = IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0));
const SOCKET_TIMEOUT: Duration = Duration::from_secs(15 * 60);
const TELEMETRY_INTERVAL: Duration = Duration::from_millis(100);
const HISTORY_INTERVAL: Duration = Duration::from_secs(1);
const SESSION_INTERVAL: Duration = Duration::from_secs(10);
const MOTION_INTERVAL: Duration = Duration::from_millis(700);
const PARTICIPANTS_TICK_UPDATE: u8 = 6;

/// Enum representing different types of F1 packet data
enum F1PacketData<'a> {
    Motion(&'a PacketMotionData),
    Session(&'a PacketSessionData),
    Event(&'a PacketEventData),
    Participants(&'a PacketParticipantsData),
    FinalClassification(&'a PacketFinalClassificationData),
    SessionHistory(&'a PacketSessionHistoryData),
    CarDamage(&'a PacketCarDamageData),
    CarStatus(&'a PacketCarStatusData),
    CarTelemetry(&'a PacketCarTelemetryData),
}

/// Represents an F1 service that processes and manages F1 telemetry data
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

/// Holds data related to an F1 service instance
pub struct F1ServiceData {
    inner: Arc<F1ServiceDataInner>,
    session_manager: F1SessionDataManager,
    shutdown: Option<oneshot::Sender<()>>,
}

/// Internal data structure for F1ServiceData
pub struct F1ServiceDataInner {
    global_channel: Sender<Bytes>,
    global_subscribers: AtomicU32,
    team_subscribers: RwLock<AHashMap<u8, u32>>,
}

/// Tracks the last update times for various packet types
struct LastUpdates {
    session: Instant,
    car_motion: Instant,
    car_status: Instant,
    car_damage: Instant,
    car_telemetry: Instant,
    participants: Instant,
    car_lap: [Instant; 22],
}

impl F1PacketData<'_> {
    /// Attempts to create an F1PacketData from raw bytes and packet ID
    #[inline]
    pub fn try_from_bytes(data: &[u8], packet_id: u8) -> AppResult<F1PacketData> {
        let packet_id = PacketIds::try_from(packet_id).unwrap();

        let packet = match packet_id {
            PacketIds::Event => cast::<PacketEventData>(data).map(F1PacketData::Event),
            PacketIds::Motion => cast::<PacketMotionData>(data).map(F1PacketData::Motion),
            PacketIds::Session => cast::<PacketSessionData>(data).map(F1PacketData::Session),
            PacketIds::CarDamage => cast::<PacketCarDamageData>(data).map(F1PacketData::CarDamage),
            PacketIds::CarStatus => cast::<PacketCarStatusData>(data).map(F1PacketData::CarStatus),
            PacketIds::CarTelemetry => {
                cast::<PacketCarTelemetryData>(data).map(F1PacketData::CarTelemetry)
            }
            PacketIds::Participants => {
                cast::<PacketParticipantsData>(data).map(F1PacketData::Participants)
            }
            PacketIds::SessionHistory => {
                cast::<PacketSessionHistoryData>(data).map(F1PacketData::SessionHistory)
            }
            PacketIds::FinalClassification => {
                cast::<PacketFinalClassificationData>(data).map(F1PacketData::FinalClassification)
            }

            _ => Err(F1ServiceError::InvalidPacketType)?,
        }?;

        Ok(packet)
    }
}

impl F1Service {
    /// Creates a new F1Service instance
    pub async fn new(
        data_manager: F1SessionDataManager,
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
            data_manager,
            services,
            f1_state,
        }
    }

    /// Initializes the F1 service with a specific port and championship ID
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

    /// Runs the main loop of the F1 service, processing incoming packets
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
                                    .restrict_to_ip(self.championship_id, address.ip())
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

    /// Processes a single packet of F1 telemetry data
    #[inline]
    async fn process_packet(&mut self, buf: &[u8], now: Instant) -> AppResult<()> {
        let header = match header_cast(buf) {
            Ok(h) => h,
            Err(_) => return Ok(()),
        };

        let packet = match F1PacketData::try_from_bytes(buf, header.packet_id) {
            Ok(p) => p,
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

    #[inline]
    fn handle_motion_packet(&mut self, motion_data: &PacketMotionData, now: Instant) {
        if now.duration_since(self.last_updates.car_motion) < MOTION_INTERVAL {
            return;
        }

        self.data_manager.save_motion(motion_data);
        self.last_updates.car_motion = now;
    }

    #[inline]
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

    #[inline]
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

    #[inline]
    fn handle_event_packet(&mut self, event_data: &PacketEventData) {
        let Some(session_type) = &self.session_type else {
            return;
        };

        if ![SessionType::R, SessionType::R2, SessionType::R3].contains(session_type) {
            return;
        }

        self.data_manager.push_event(event_data);
    }

    #[inline]
    fn handle_session_history_packet(
        &mut self,
        history_data: &PacketSessionHistoryData,
        now: Instant,
    ) {
        let Some(last_update) = self
            .last_updates
            .car_lap
            .get_mut(history_data.car_idx as usize)
        else {
            warn!("CarIdx out of bounds");
            return;
        };

        if now.duration_since(*last_update) > HISTORY_INTERVAL {
            self.data_manager.save_lap_history(history_data);
            *last_update = now;
        }
    }

    #[inline]
    async fn handle_final_classification_packet(
        &mut self,
        final_classification: &PacketFinalClassificationData,
    ) -> AppResult<()> {
        let Some(_session_type) = self.session_type.take() else {
            error!("Not defined session type when trying to save final_classification_data");
            return Ok(());
        };

        self.data_manager
            .save_final_classification(final_classification);

        Ok(())
    }

    #[inline]
    fn handle_car_damage_packet(&mut self, car_damage: &PacketCarDamageData, now: Instant) {
        if now.duration_since(self.last_updates.car_damage) > TELEMETRY_INTERVAL {
            self.data_manager.save_car_damage(car_damage);
        }
    }

    #[inline]
    fn handle_car_status_packet(&mut self, car_status: &PacketCarStatusData, now: Instant) {
        if now.duration_since(self.last_updates.car_status) > TELEMETRY_INTERVAL {
            self.data_manager.save_car_status(car_status);
        }
    }

    #[inline]
    fn handle_car_telemetry_packet(
        &mut self,
        car_telemetry: &PacketCarTelemetryData,
        now: Instant,
    ) {
        if now.duration_since(self.last_updates.car_telemetry) > TELEMETRY_INTERVAL {
            self.data_manager.save_car_telemetry(car_telemetry);
        }
    }

    /// Ensures all participants are registered in the system
    #[inline]
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
                .binary_search_by(|probe| probe.as_ref().cmp(steam_name))
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

    /// Closes the F1 service, releasing resources and removing it from active services
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
    /// Creates a new F1ServiceData instance
    pub fn new(
        session_manager: F1SessionDataManager,
        global_channel: Sender<Bytes>,
        shutdown: oneshot::Sender<()>,
    ) -> Self {
        let inner = Arc::new(F1ServiceDataInner {
            global_channel,
            global_subscribers: AtomicU32::new(0),
            team_subscribers: RwLock::new(AHashMap::new()),
        });

        Self {
            inner,
            session_manager,
            shutdown: Some(shutdown),
        }
    }

    /// Retrieves the cached data from the session manager
    #[inline]
    pub fn cache(&self) -> Option<Bytes> {
        self.session_manager.cache()
    }

    /// Subscribes to the global broadcast channel
    pub fn global_sub(&self) -> Receiver<Bytes> {
        self.global_subscribers.fetch_add(1, Ordering::Relaxed);
        self.global_channel.subscribe()
    }

    /// Subscribes to a team-specific broadcast channel
    pub fn team_sub(&self, team_id: u8) -> Option<Receiver<Bytes>> {
        let receiver = self.session_manager.get_team_receiver(team_id)?;
        let mut team_subs = self.team_subscribers.write();
        *team_subs.entry(team_id).or_insert(0) += 1;
        Some(receiver)
    }

    /// Gets the current number of global subscribers
    pub fn global_count(&self) -> u32 {
        self.global_subscribers.load(Ordering::Relaxed)
    }

    /// Gets the current number of subscribers for all teams
    pub fn all_team_count(&self) -> u32 {
        self.team_subscribers.read().values().sum()
    }

    /// Gets the current number of subscribers for a specific team
    #[allow(unused)]
    pub fn team_count(&self, team_id: u8) -> u32 {
        *self.team_subscribers.read().get(&team_id).unwrap_or(&0)
    }

    /// Decrements the global subscriber count
    pub fn global_unsub(&self) {
        self.global_subscribers.fetch_sub(1, Ordering::Relaxed);
    }

    /// Decrements the team subscriber count
    pub fn team_unsub(&self, team_id: u8) {
        let mut team_subs = self.team_subscribers.write();
        if let Some(count) = team_subs.get_mut(&team_id) {
            if *count > 0 {
                *count -= 1;
            }
        }
    }

    /// Initiates the shutdown process for the service
    pub fn shutdown(&mut self) -> Result<(), ()> {
        self.shutdown.take().unwrap().send(())
    }
}

impl Deref for F1ServiceData {
    type Target = F1ServiceDataInner;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl LastUpdates {
    /// Creates a new LastUpdates instance with current time for all fields
    fn new() -> Self {
        let time = Instant::now();

        Self {
            session: time,
            car_motion: time,
            participants: time,
            car_damage: time,
            car_status: time,
            car_telemetry: time,
            car_lap: [time; 22],
        }
    }
}

/// Casts raw bytes to a PacketHeader reference
#[inline]
fn header_cast(bytes: &[u8]) -> AppResult<&PacketHeader> {
    if mem::size_of::<PacketHeader>() > bytes.len() {
        Err(F1ServiceError::CastingError)?;
    }

    Ok(unsafe { &*(bytes.as_ptr() as *const PacketHeader) })
}

/// Casts raw bytes to a reference of type T
#[inline]
fn cast<T>(bytes: &[u8]) -> AppResult<&T> {
    if !mem::size_of::<T>() == bytes.len() {
        Err(F1ServiceError::CastingError)?;
    }

    Ok(unsafe { &*(bytes.as_ptr() as *const T) })
}
