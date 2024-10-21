use std::{ops::Deref, sync::Arc, time::Duration};

use ahash::{AHashMap, AHashSet};
use ntex::{
    time::interval,
    util::{Bytes, BytesMut},
};
use parking_lot::{Mutex, RwLock};
use prost::Message;
use tokio::sync::{
    broadcast::{Receiver, Sender},
    oneshot,
};
use tracing::{error, warn};

use crate::types::{
    CarDamageData as F1CarDamageData, CarMotionData as F1CarMotionData,
    CarStatusData as F1CarStatusData, CarTelemetryData as F1CarTelemetryData, EventCode,
    EventDataDetails as F1EventDataDetails, FinalClassificationData as F1FinalClassificationData,
    LapHistoryData as F1LapHistoryData, PacketCarDamageData, PacketCarStatusData,
    PacketCarTelemetryData, PacketEventData, PacketEventData as F1PacketEventData,
    PacketFinalClassificationData, PacketMotionData, PacketParticipantsData, PacketSessionData,
    PacketSessionHistoryData, ParticipantData as F1ParticipantData,
    TyreStintHistoryData as F1TyreStintHistoryData,
};

use event_data_details::Details;

// Constants
const GENERAL_INTERVAL: Duration = Duration::from_millis(700);
const TELEMETRY_INTERVAL: Duration = Duration::from_millis(100);
const NOT_SEND_EVENTS: [EventCode; 9] = [
    EventCode::ButtonStatus,
    EventCode::TeamMateInPits,
    EventCode::Flashback,
    EventCode::SessionEnded,
    EventCode::DRSEnabled,
    EventCode::DRSDisabled,
    EventCode::ChequeredFlag,
    EventCode::RedFlag,
    EventCode::LightsOut,
];

include!(concat!(env!("OUT_DIR"), "/f1telemetry.rs"));

// Structs
#[derive(Debug)]
pub struct DriverInfo {
    pub name: Box<str>,
    pub team_id: u8,
}

#[derive(Clone)]
pub struct F1TelemetryPacketHandler {
    inner: Arc<F1TelemetryPacketHandlerInner>,
}

#[derive(Debug)]
pub struct F1TelemetryPacketHandlerInner {
    driver_info: RwLock<AHashMap<usize, DriverInfo>>,
    general: RwLock<F1GeneralInfo>,
    telemetry: RwLock<F1TelemetryInfo>,
    last_general: RwLock<F1GeneralInfo>,
    last_general_encoded: RwLock<Option<Bytes>>,
    last_telemetry: RwLock<F1TelemetryInfo>,
    team_senders: RwLock<AHashMap<u8, Sender<Bytes>>>,
    stop_sender: Mutex<Option<oneshot::Sender<()>>>,
}

// Implementations
impl Deref for F1TelemetryPacketHandler {
    type Target = F1TelemetryPacketHandlerInner;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl F1TelemetryPacketHandler {
    /// Creates a new F1TelemetryPacketHandler instance
    pub fn new(tx: Sender<Bytes>) -> Self {
        let inner = Arc::new(F1TelemetryPacketHandlerInner {
            driver_info: RwLock::new(AHashMap::new()),
            general: RwLock::new(F1GeneralInfo::default()),
            telemetry: RwLock::new(F1TelemetryInfo::default()),
            last_general: RwLock::new(F1GeneralInfo::default()),
            last_general_encoded: RwLock::new(None),
            last_telemetry: RwLock::new(F1TelemetryInfo::default()),
            team_senders: RwLock::new(AHashMap::new()),
            stop_sender: Mutex::new(None),
        });

        let mut instance = Self { inner };
        instance.spawn_update_task(tx);
        instance
    }

    /// Returns the cached general data
    #[inline]
    pub fn cache(&self) -> Option<Bytes> {
        self.last_general_encoded.read().clone()
    }

    /// Gets a team-specific receiver for updates
    pub fn get_team_receiver(&self, team_id: u8) -> Option<Receiver<Bytes>> {
        self.team_senders
            .read()
            .get(&team_id)
            .map(|sender| sender.subscribe())
    }

    /// Pushes an event to the general data
    #[inline]
    pub fn push_event(&self, event: &PacketEventData) {
        let driver_info = self.driver_info.read();

        if let Some(event_data) = EventData::from_f1(event, &driver_info) {
            let mut general = self.general.write();
            general
                .events
                .get_or_insert_with(PacketsEventsData::default)
                .events
                .push(event_data);
        }
    }

    /// Saves motion data
    #[inline]
    pub fn save_motion(&self, packet: &PacketMotionData) {
        let driver_info = self.driver_info.read();
        let mut general = self.general.write();

        for (i, motion_data) in packet.car_motion_data.iter().enumerate() {
            if motion_data.world_position_x == 0f32 {
                continue;
            }

            if let Some(driver) = driver_info.get(&i) {
                if let Some(player) = general.players.get_mut(driver.name.as_ref()) {
                    player.update_car_motion(motion_data);
                }
            }
        }
    }

    /// Saves session data
    #[inline]
    pub fn save_session(&self, packet: &PacketSessionData) {
        let mut general = self.general.write();
        general.update_session(packet);
    }

    /// Saves lap history data
    #[inline]
    pub fn save_lap_history(&self, packet: &PacketSessionHistoryData) {
        let driver_info = self.driver_info.read();

        if let Some(driver) = driver_info.get(&(packet.car_idx as usize)) {
            let mut general = self.general.write();
            if let Some(player) = general.players.get_mut(driver.name.as_ref()) {
                player.update_session_history(packet);
            }
        }
    }

    /// Saves participants data
    #[inline]
    pub fn save_participants(&self, packet: &PacketParticipantsData) {
        let mut driver_info = self.driver_info.write();
        let mut general = self.general.write();
        let mut telemetry = self.telemetry.write();
        let mut team_senders = self.team_senders.write();

        for i in 0..packet.num_active_cars as usize {
            let Some(participant) = packet.participants.get(i) else {
                error!(
                    "num_active_cars ({}) exceeds array bound ({})",
                    packet.num_active_cars,
                    packet.participants.len()
                );
                break;
            };

            let steam_name = match participant.steam_name() {
                Some(name) if name != "Player" => name,
                _ => continue,
            };

            driver_info.entry(i).or_insert_with(|| DriverInfo {
                name: steam_name.into(),
                team_id: participant.team_id,
            });

            general
                .players
                .entry(steam_name.to_string())
                .and_modify(|player| player.update_participant_info(participant))
                .or_insert_with(|| {
                    let mut new_player = PlayerInfo::default();
                    new_player.update_participant_info(participant);
                    telemetry
                        .player_telemetry
                        .entry(steam_name.to_string())
                        .or_default();
                    new_player
                });

            team_senders
                .entry(participant.team_id)
                .or_insert_with(|| Sender::new(30));
        }
    }

    /// Saves car damage data
    #[inline]
    pub fn save_car_damage(&self, packet: &PacketCarDamageData) {
        self.process_telemetry_packet(&packet.car_damage_data, |player_telemetry, data| {
            player_telemetry.update_car_damage(data);
        });
    }

    /// Saves car status data
    #[inline]
    pub fn save_car_status(&self, packet: &PacketCarStatusData) {
        self.process_telemetry_packet(&packet.car_status_data, |player_telemetry, data| {
            player_telemetry.update_car_status(data);
        });
    }

    /// Saves car telemetry data
    #[inline]
    pub fn save_car_telemetry(&self, packet: &PacketCarTelemetryData) {
        self.process_telemetry_packet(&packet.car_telemetry_data, |player_telemetry, data| {
            player_telemetry.update_car_telemetry(data);
        });
    }

    /// Saves final classification data
    #[inline]
    pub fn save_final_classification(&self, packet: &PacketFinalClassificationData) {
        let driver_info = self.driver_info.read();
        let mut general = self.general.write();

        for (i, classification_data) in packet.classification_data.iter().enumerate() {
            if let Some(driver) = driver_info.get(&i) {
                if let Some(player) = general.players.get_mut(driver.name.as_ref()) {
                    player.update_classification_data(classification_data);
                }
            }
        }
    }

    /// Processes telemetry packet data
    #[inline]
    fn process_telemetry_packet<T, F>(&self, packet_data: &[T], mut process_fn: F)
    where
        F: FnMut(&mut PlayerTelemetry, &T),
    {
        let driver_info = self.driver_info.read();
        let mut telemetry = self.telemetry.write();

        for (i, data) in packet_data.iter().enumerate() {
            if let Some(driver) = driver_info.get(&i) {
                if let Some(player_telemetry) =
                    telemetry.player_telemetry.get_mut(driver.name.as_ref())
                {
                    process_fn(player_telemetry, data);
                }
            }
        }
    }

    /// Spawns the update task for sending updates
    #[inline]
    fn spawn_update_task(&mut self, tx: Sender<Bytes>) {
        let (stop_sender, mut stop_receiver) = oneshot::channel();
        *self.inner.stop_sender.lock() = Some(stop_sender);

        let inner = self.inner.clone();

        ntex::rt::spawn(async move {
            let general_interval = interval(GENERAL_INTERVAL);
            let telemetry_interval = interval(TELEMETRY_INTERVAL);

            loop {
                tokio::select! {
                    _ = &mut stop_receiver => break,
                    _ = general_interval.tick() => {
                        Self::send_general_updates(&inner, &tx);
                    }
                    _ = telemetry_interval.tick() => {
                        Self::send_telemetry_updates(&inner);
                    }
                }
            }
        });
    }

    /// Sends general updates
    #[inline]
    fn send_general_updates(inner: &Arc<F1TelemetryPacketHandlerInner>, tx: &Sender<Bytes>) {
        if tx.receiver_count() == 0 {
            return;
        }

        let general = inner.general.read();
        let mut last_general = inner.last_general.write();
        let mut last_general_encoded = inner.last_general_encoded.write();

        if let Some(diff) = Self::diff_general(&general, &last_general) {
            if tx.send(diff.clone()).is_err() {
                error!("Failed to send general update");
            }

            *last_general = general.clone();
            *last_general_encoded = Some(diff);
        }
    }

    /// Sends telemetry updates
    #[inline]
    fn send_telemetry_updates(inner: &Arc<F1TelemetryPacketHandlerInner>) {
        let driver_info = inner.driver_info.read();
        let telemetry = inner.telemetry.read();
        let mut last_telemetry = inner.last_telemetry.write();
        let team_senders = inner.team_senders.read();

        let mut team_updates: AHashMap<u8, F1TelemetryInfo> = AHashMap::new();

        let active_teams: AHashSet<u8> = team_senders
            .iter()
            .filter(|(_, sender)| sender.receiver_count() > 0)
            .map(|(&team_id, _)| team_id)
            .collect();

        for (_, driver) in driver_info.iter() {
            if active_teams.contains(&driver.team_id) {
                if let (Some(current_player_telemetry), Some(last_player_telemetry)) = (
                    telemetry.player_telemetry.get(driver.name.as_ref()),
                    last_telemetry.player_telemetry.get(driver.name.as_ref()),
                ) {
                    if let Some(diff_telemetry) =
                        Self::diff_player_telemetry(current_player_telemetry, last_player_telemetry)
                    {
                        team_updates
                            .entry(driver.team_id)
                            .or_default()
                            .player_telemetry
                            .insert(driver.name.to_string(), diff_telemetry);
                    }
                }
            }
        }

        for (team_id, update) in team_updates {
            if let Some(sender) = team_senders.get(&team_id) {
                let mut buf = BytesMut::with_capacity(update.encoded_len());
                update.encode_raw(&mut buf);

                if sender.send(buf.freeze()).is_err() {
                    error!("Failed to send telemetry update for team {}", team_id);
                }
            }
        }

        *last_telemetry = telemetry.clone();
    }

    /// Computes the difference between two F1GeneralInfo instances
    #[inline]
    fn diff_general(current: &F1GeneralInfo, last: &F1GeneralInfo) -> Option<Bytes> {
        let diff = current.diff(last);

        if let Some(diff) = diff {
            let mut buf = BytesMut::with_capacity(diff.encoded_len());
            diff.encode_raw(&mut buf);

            return Some(buf.freeze());
        }

        None
    }

    /// Computes the difference between two PlayerTelemetry instances
    #[inline]
    fn diff_player_telemetry(
        current: &PlayerTelemetry,
        last: &PlayerTelemetry,
    ) -> Option<PlayerTelemetry> {
        current.diff(last)
    }
}

impl Drop for F1TelemetryPacketHandler {
    fn drop(&mut self) {
        if let Some(sender) = self.stop_sender.lock().take() {
            let _ = sender.send(());
        }
    }
}

impl HistoryData {
    /// Updates the history data with new packet information
    #[inline]
    fn update(&mut self, packet: &PacketSessionHistoryData) {
        self.num_laps = Some(packet.num_laps as u32);
        self.num_tyre_stints = Some(packet.num_tyre_stints as u32);
        self.best_lap_time_lap_num = Some(packet.best_lap_time_lap_num as u32);
        self.best_s1_lap_num = Some(packet.best_sector1_lap_num as u32);
        self.best_s2_lap_num = Some(packet.best_sector2_lap_num as u32);
        self.best_s3_lap_num = Some(packet.best_sector3_lap_num as u32);

        let current_lap = packet.num_laps as usize;
        if current_lap > 1 && self.lap_history_data.len() >= current_lap - 1 {
            self.lap_history_data[current_lap - 2]
                .update(&packet.lap_history_data[current_lap - 2]);
        }

        if current_lap > 0 {
            if current_lap > self.lap_history_data.len() {
                self.lap_history_data.push(LapHistoryData::from_f1(
                    &packet.lap_history_data[current_lap - 1],
                ));
            } else {
                self.lap_history_data[current_lap - 1]
                    .update(&packet.lap_history_data[current_lap - 1]);
            }
        }

        let num_stints = packet.num_tyre_stints as usize;
        if num_stints > self.tyre_stints_history_data.len() {
            for i in self.tyre_stints_history_data.len()..num_stints {
                let stint_data = &packet.tyre_stints_history_data[i];
                self.tyre_stints_history_data
                    .push(TyreStintsHistoryData::from_f1(stint_data));
            }
        }
    }
}

impl TyreStintsHistoryData {
    /// Creates a new TyreStintsHistoryData from F1 data
    #[inline]
    fn from_f1(stints_data: &F1TyreStintHistoryData) -> Self {
        Self {
            actual_compound: Some(stints_data.tyre_actual_compound as u32),
            visual_compound: Some(stints_data.tyre_visual_compound as u32),
            end_lap: Some(stints_data.end_lap as u32),
        }
    }
}

impl LapHistoryData {
    /// Creates a new LapHistoryData from F1 data
    #[inline]
    fn from_f1(lap_data: &F1LapHistoryData) -> Self {
        Self {
            lap_time: Some(lap_data.lap_time_in_ms),
            s1_time: Some(lap_data.sector1_time_in_ms as u32),
            s2_time: Some(lap_data.sector2_time_in_ms as u32),
            s3_time: Some(lap_data.sector3_time_in_ms as u32),
            lap_valid_bit_flag: Some(lap_data.lap_valid_bit_flags as u32),
        }
    }

    /// Updates the lap history data with new information
    #[inline]
    fn update(&mut self, lap_data: &F1LapHistoryData) {
        *self = Self::from_f1(lap_data);
    }
}

impl PlayerInfo {
    /// Updates car motion data for the player
    #[inline]
    pub fn update_car_motion(&mut self, incoming_motion: &F1CarMotionData) {
        let car_motion = self.car_motion.get_or_insert_with(Default::default);
        car_motion.x = Some(incoming_motion.world_position_x);
        car_motion.y = Some(incoming_motion.world_position_y);
        car_motion.yaw = Some(incoming_motion.yaw);
    }

    /// Updates session history for the player
    #[inline]
    pub fn update_session_history(&mut self, packet: &PacketSessionHistoryData) {
        self.lap_history
            .get_or_insert_with(Default::default)
            .update(packet);
    }

    /// Updates participant information for the player
    #[inline]
    pub fn update_participant_info(&mut self, incoming_participant: &F1ParticipantData) {
        let participant = self.participant.get_or_insert_with(Default::default);
        participant.team_id = Some(incoming_participant.team_id as u32);
        participant.race_number = Some(incoming_participant.race_number as u32);
        participant.nationality = Some(incoming_participant.nationality as u32);
        participant.platform = Some(incoming_participant.platform as u32);
    }

    /// Updates classification data for the player
    #[inline]
    pub fn update_classification_data(&mut self, packet: &F1FinalClassificationData) {
        let final_classification = self
            .final_classification
            .get_or_insert_with(Default::default);

        final_classification.position = Some(packet.position as u32);
        final_classification.laps = Some(packet.num_laps as u32);
        final_classification.grid_position = Some(packet.grid_position as u32);
        final_classification.points = Some(packet.points as u32);
        final_classification.pit_stops = Some(packet.num_pit_stops as u32);
        final_classification.result_status = Some(packet.result_status as u32);
        final_classification.best_lap_time = Some(packet.best_lap_time_in_ms);
        final_classification.race_time = Some(packet.total_race_time);
        final_classification.penalties_time = Some(packet.penalties_time as u32);
        final_classification.num_penalties = Some(packet.num_penalties as u32);

        final_classification.tyre_stints_actual.clear();
        final_classification.tyre_stints_actual.extend(
            packet.tyre_stints_actual[..packet.num_tyre_stints as usize]
                .iter()
                .map(|&x| x as u32),
        );

        final_classification.tyre_stints_visual.clear();
        final_classification.tyre_stints_visual.extend(
            packet.tyre_stints_visual[..packet.num_tyre_stints as usize]
                .iter()
                .map(|&x| x as u32),
        );

        final_classification.tyre_stints_end_laps.clear();
        final_classification.tyre_stints_end_laps.extend(
            packet.tyre_stints_end_laps[..packet.num_tyre_stints as usize]
                .iter()
                .map(|&x| x as u32),
        );
    }
}

impl PlayerTelemetry {
    /// Updates car damage data for the player
    #[inline]
    pub fn update_car_damage(&mut self, data: &F1CarDamageData) {
        let car_damage = self.car_damage.get_or_insert_with(Default::default);

        car_damage.tyres_wear.clear();

        let brakes_temp_ptr = &raw const data.tyres_wear;

        car_damage
            .tyres_wear
            .extend_from_slice(unsafe { &brakes_temp_ptr.read_unaligned() });

        car_damage.tyres_damage.clear();
        car_damage
            .tyres_damage
            .extend(data.tyres_damage.iter().map(|&x| x as u32));

        car_damage.brakes_damage.clear();
        car_damage
            .brakes_damage
            .extend(data.brakes_damage.iter().map(|&x| x as u32));

        car_damage.front_left_wing_damage = Some(data.front_left_wing_damage as u32);
        car_damage.front_right_wing_damage = Some(data.front_right_wing_damage as u32);
        car_damage.rear_wing_damage = Some(data.rear_wing_damage as u32);
        car_damage.floor_damage = Some(data.floor_damage as u32);
        car_damage.diffuser_damage = Some(data.diffuser_damage as u32);
        car_damage.sidepod_damage = Some(data.sidepod_damage as u32);
        car_damage.drs_fault = Some(data.drs_fault != 0);
        car_damage.ers_fault = Some(data.ers_fault != 0);
        car_damage.gear_box_damage = Some(data.gear_box_damage as u32);
        car_damage.engine_damage = Some(data.engine_damage as u32);
        car_damage.engine_mguh_wear = Some(data.engine_mguh_wear as u32);
        car_damage.engine_es_wear = Some(data.engine_es_wear as u32);
        car_damage.engine_ce_wear = Some(data.engine_ce_wear as u32);
        car_damage.engine_ice_wear = Some(data.engine_ice_wear as u32);
        car_damage.engine_mguk_wear = Some(data.engine_mguk_wear as u32);
        car_damage.engine_tc_wear = Some(data.engine_tc_wear as u32);
        car_damage.engine_blown = Some(data.engine_blown != 0);
        car_damage.engine_seized = Some(data.engine_seized != 0);
    }

    /// Updates car status data for the player
    #[inline]
    pub fn update_car_status(&mut self, data: &F1CarStatusData) {
        let car_status = self.car_status.get_or_insert_with(Default::default);

        car_status.fuel_mix = Some(data.fuel_mix as u32);
        car_status.front_brake_bias = Some(data.front_brake_bias as u32);
        car_status.fuel_in_tank = Some(data.fuel_in_tank);
        car_status.fuel_capacity = Some(data.fuel_capacity);
        car_status.fuel_remaining_laps = Some(data.fuel_remaining_laps);
        car_status.drs_allowed = Some(data.drs_allowed != 0);
        car_status.drs_activation_distance = Some(data.drs_activation_distance as u32);
        car_status.actual_tyre_compound = Some(data.actual_tyre_compound as u32);
        car_status.visual_tyre_compound = Some(data.visual_tyre_compound as u32);
        car_status.tyres_age_laps = Some(data.tyres_age_laps as u32);
        car_status.vehicle_fia_flags = Some(data.vehicle_fia_flags as i32);
        car_status.engine_power_ice = Some(data.engine_power_ice);
        car_status.engine_power_mguk = Some(data.engine_power_mguk);
        car_status.ers_store_energy = Some(data.ers_store_energy);
        car_status.ers_deploy_mode = Some(data.ers_deploy_mode as u32);
        car_status.ers_harvested_this_lap_mguk = Some(data.ers_harvested_this_lap_mguk);
        car_status.ers_harvested_this_lap_mguh = Some(data.ers_harvested_this_lap_mguh);
        car_status.ers_deployed_this_lap = Some(data.ers_deployed_this_lap);
    }

    /// Updates car telemetry data for the player
    #[inline]
    pub fn update_car_telemetry(&mut self, data: &F1CarTelemetryData) {
        let telemetry = self.car_telemetry.get_or_insert_with(Default::default);

        telemetry.speed = Some(data.speed as u32);
        telemetry.throttle = Some(data.throttle);
        telemetry.steer = Some(data.steer);
        telemetry.brake = Some(data.brake);
        telemetry.gear = Some(data.gear as i32);
        telemetry.engine_rpm = Some(data.engine_rpm as u32);
        telemetry.drs = Some(data.drs != 0);
        telemetry.engine_temperature = Some(data.engine_temperature as u32);

        telemetry.brakes_temperature.clear();
        let brakes_temp_ptr = &raw const data.brakes_temperature;
        let brakes_temp = unsafe { brakes_temp_ptr.read_unaligned() };

        telemetry
            .brakes_temperature
            .extend(brakes_temp.iter().map(|&x| x as u32));

        telemetry.tyres_surface_temperature.clear();
        telemetry
            .tyres_surface_temperature
            .extend(data.tyres_surface_temperature.iter().map(|&x| x as u32));

        telemetry.tyres_inner_temperature.clear();
        telemetry
            .tyres_inner_temperature
            .extend(data.tyres_inner_temperature.iter().map(|&x| x as u32));

        telemetry.tyres_pressure.clear();

        let tyres_pressure_ptr = &raw const data.tyres_pressure;

        telemetry
            .tyres_pressure
            .extend_from_slice(unsafe { &tyres_pressure_ptr.read_unaligned() });
    }

    /// Computes the difference between two PlayerTelemetry instances
    #[inline]
    pub fn diff(&self, last: &Self) -> Option<Self> {
        let mut diff = PlayerTelemetry::default();
        let mut has_changes = false;

        // Compare car_telemetry
        if let (Some(cur_telemetry), Some(last_telemetry)) =
            (&self.car_telemetry, &last.car_telemetry)
        {
            let mut diff_telemetry = CarTelemetryData::default();
            let mut telemetry_changed = false;

            macro_rules! diff_telemetry_field {
                ($field:ident) => {
                    if cur_telemetry.$field != last_telemetry.$field {
                        diff_telemetry.$field = cur_telemetry.$field;
                        telemetry_changed = true;
                    }
                };
            }

            diff_telemetry_field!(speed);
            diff_telemetry_field!(throttle);
            diff_telemetry_field!(steer);
            diff_telemetry_field!(brake);
            diff_telemetry_field!(gear);
            diff_telemetry_field!(engine_rpm);
            diff_telemetry_field!(drs);
            diff_telemetry_field!(engine_temperature);

            if cur_telemetry.brakes_temperature != last_telemetry.brakes_temperature {
                diff_telemetry.brakes_temperature = cur_telemetry.brakes_temperature.clone();
                telemetry_changed = true;
            }

            if cur_telemetry.tyres_surface_temperature != last_telemetry.tyres_surface_temperature {
                diff_telemetry.tyres_surface_temperature =
                    cur_telemetry.tyres_surface_temperature.clone();
                telemetry_changed = true;
            }

            if cur_telemetry.tyres_inner_temperature != last_telemetry.tyres_inner_temperature {
                diff_telemetry.tyres_inner_temperature =
                    cur_telemetry.tyres_inner_temperature.clone();
                telemetry_changed = true;
            }

            if cur_telemetry.tyres_pressure != last_telemetry.tyres_pressure {
                diff_telemetry.tyres_pressure = cur_telemetry.tyres_pressure.clone();
                telemetry_changed = true;
            }

            if telemetry_changed {
                diff.car_telemetry = Some(diff_telemetry);
                has_changes = true;
            }
        } else if self.car_telemetry != last.car_telemetry {
            diff.car_telemetry = self.car_telemetry.clone();
            has_changes = true;
        }

        // Compare car_status
        if let (Some(cur_status), Some(last_status)) = (&self.car_status, &last.car_status) {
            let mut diff_status = CarStatusData::default();
            let mut status_changed = false;

            macro_rules! diff_status_field {
                ($field:ident) => {
                    if cur_status.$field != last_status.$field {
                        diff_status.$field = cur_status.$field;
                        status_changed = true;
                    }
                };
            }

            diff_status_field!(fuel_mix);
            diff_status_field!(front_brake_bias);
            diff_status_field!(fuel_in_tank);
            diff_status_field!(fuel_capacity);
            diff_status_field!(fuel_remaining_laps);
            diff_status_field!(drs_allowed);
            diff_status_field!(drs_activation_distance);
            diff_status_field!(actual_tyre_compound);
            diff_status_field!(visual_tyre_compound);
            diff_status_field!(tyres_age_laps);
            diff_status_field!(vehicle_fia_flags);
            diff_status_field!(engine_power_ice);
            diff_status_field!(engine_power_mguk);
            diff_status_field!(ers_store_energy);
            diff_status_field!(ers_deploy_mode);
            diff_status_field!(ers_harvested_this_lap_mguk);
            diff_status_field!(ers_harvested_this_lap_mguh);
            diff_status_field!(ers_deployed_this_lap);

            if status_changed {
                diff.car_status = Some(diff_status);
                has_changes = true;
            }
        } else if self.car_status != last.car_status {
            diff.car_status = self.car_status;
            has_changes = true;
        }

        // Compare car_damage
        if let (Some(cur_damage), Some(last_damage)) = (&self.car_damage, &last.car_damage) {
            let mut diff_damage = CarDamageData::default();
            let mut damage_changed = false;

            macro_rules! diff_damage_field {
                ($field:ident) => {
                    if cur_damage.$field != last_damage.$field {
                        diff_damage.$field = cur_damage.$field.clone();
                        damage_changed = true;
                    }
                };
            }

            diff_damage_field!(tyres_wear);
            diff_damage_field!(tyres_damage);
            diff_damage_field!(brakes_damage);
            diff_damage_field!(front_left_wing_damage);
            diff_damage_field!(front_right_wing_damage);
            diff_damage_field!(rear_wing_damage);
            diff_damage_field!(floor_damage);
            diff_damage_field!(diffuser_damage);
            diff_damage_field!(sidepod_damage);
            diff_damage_field!(drs_fault);
            diff_damage_field!(ers_fault);
            diff_damage_field!(gear_box_damage);
            diff_damage_field!(engine_damage);
            diff_damage_field!(engine_mguh_wear);
            diff_damage_field!(engine_es_wear);
            diff_damage_field!(engine_ce_wear);
            diff_damage_field!(engine_ice_wear);
            diff_damage_field!(engine_mguk_wear);
            diff_damage_field!(engine_tc_wear);
            diff_damage_field!(engine_blown);
            diff_damage_field!(engine_seized);

            if damage_changed {
                diff.car_damage = Some(diff_damage);
                has_changes = true;
            }
        } else if self.car_damage != last.car_damage {
            diff.car_damage = self.car_damage.clone();
            has_changes = true;
        }

        if has_changes {
            Some(diff)
        } else {
            None
        }
    }
}

impl F1GeneralInfo {
    /// Updates session data with new information
    #[inline]
    pub fn update_session(&mut self, packet: &PacketSessionData) {
        let session = self.session.get_or_insert_with(Default::default);
        session.weather = Some(packet.weather as u32);
        session.track_temperature = Some(packet.track_temperature as i32);
        session.air_temperature = Some(packet.air_temperature as i32);
        session.total_laps = Some(packet.total_laps as u32);
        session.track_length = Some(packet.track_length as u32);
        session.session_type = Some(packet.session_type as u32);
        session.track_id = Some(packet.track_id as i32);
        session.session_time_left = Some(packet.session_time_left as u32);
        session.session_duration = Some(packet.session_duration as u32);
        session.safety_car_status = Some(packet.safety_car_status as u32);
        session.session_length = Some(packet.session_length as u32);
        session.num_safety_car = Some(packet.num_safety_car_periods as u32);
        session.num_virtual_safety_car = Some(packet.num_virtual_safety_car_periods as u32);
        session.num_red_flags = Some(packet.num_red_flag_periods as u32);
        session.s2_lap_distance_start = Some(packet.sector2_lap_distance_start);
        session.s3_lap_distance_start = Some(packet.sector3_lap_distance_start);

        session.weekend_structure.clear();
        session.weekend_structure.extend(
            packet.weekend_structure[..packet.num_sessions_in_weekend as usize]
                .iter()
                .map(|&x| x as u32),
        );
    }

    /// Computes the difference between two F1GeneralInfo instances
    #[inline]
    pub fn diff(&self, last: &Self) -> Option<Self> {
        let mut diff = F1GeneralInfo::default();
        let mut has_changes = false;

        // Optimized session diff
        if let (Some(cur_session), Some(last_session)) = (&self.session, &last.session) {
            let mut diff_session = SessionData::default();
            let mut session_changed = false;

            macro_rules! diff_session_field {
                ($field:ident) => {
                    if cur_session.$field != last_session.$field {
                        diff_session.$field = cur_session.$field;
                        session_changed = true;
                    }
                };
            }

            diff_session_field!(weather);
            diff_session_field!(track_temperature);
            diff_session_field!(air_temperature);
            diff_session_field!(total_laps);
            diff_session_field!(track_length);
            diff_session_field!(session_type);
            diff_session_field!(track_id);
            diff_session_field!(session_time_left);
            diff_session_field!(session_duration);
            diff_session_field!(safety_car_status);
            diff_session_field!(session_length);
            diff_session_field!(num_safety_car);
            diff_session_field!(num_virtual_safety_car);
            diff_session_field!(num_red_flags);
            diff_session_field!(s2_lap_distance_start);
            diff_session_field!(s3_lap_distance_start);

            // Optimized weekend_structure diff
            if cur_session.weekend_structure.len() > last_session.weekend_structure.len() {
                diff_session.weekend_structure =
                    cur_session.weekend_structure[last_session.weekend_structure.len()..].to_vec();
                session_changed = true;
            }

            if session_changed {
                diff.session = Some(diff_session);
                has_changes = true;
            }
        } else if self.session != last.session {
            diff.session = self.session.clone();
            has_changes = true;
        }

        // Optimized events diff
        if let (Some(cur_events), Some(last_events)) = (&self.events, &last.events) {
            let mut diff_events = Vec::new();
            let cur_event_map: AHashMap<_, _> = cur_events
                .events
                .iter()
                .map(|e| (&e.string_code, e))
                .collect();

            let last_event_map: AHashMap<_, _> = last_events
                .events
                .iter()
                .map(|e| (&e.string_code, e))
                .collect();

            for (code, cur_event) in &cur_event_map {
                if !last_event_map.contains_key(code) {
                    diff_events.push((*cur_event).clone());
                }
            }

            if !diff_events.is_empty() {
                diff.events = Some(PacketsEventsData {
                    events: diff_events,
                });
                has_changes = true;
            }
        } else if self.events != last.events {
            diff.events = self.events.clone();
            has_changes = true;
        }

        // Optimized players diff
        for (key, cur_player) in &self.players {
            if let Some(last_player) = last.players.get(key) {
                let mut diff_player = PlayerInfo::default();
                let mut player_changed = false;

                if cur_player.participant != last_player.participant {
                    diff_player.participant = cur_player.participant;
                    player_changed = true;
                }

                if cur_player.car_motion != last_player.car_motion {
                    diff_player.car_motion = cur_player.car_motion;
                    player_changed = true;
                }

                if let (Some(cur_history), Some(last_history)) =
                    (&cur_player.lap_history, &last_player.lap_history)
                {
                    let mut diff_history = HistoryData::default();
                    let mut history_changed = false;

                    macro_rules! diff_history_field {
                        ($field:ident) => {
                            if cur_history.$field != last_history.$field {
                                diff_history.$field = cur_history.$field;
                                history_changed = true;
                            }
                        };
                    }

                    diff_history_field!(num_laps);
                    diff_history_field!(num_tyre_stints);
                    diff_history_field!(best_lap_time_lap_num);
                    diff_history_field!(best_s1_lap_num);
                    diff_history_field!(best_s2_lap_num);
                    diff_history_field!(best_s3_lap_num);

                    // Only add new lap history data
                    if cur_history.lap_history_data.len() > last_history.lap_history_data.len() {
                        diff_history.lap_history_data = cur_history.lap_history_data
                            [last_history.lap_history_data.len()..]
                            .to_vec();
                        history_changed = true;
                    }

                    // Only add new tyre stint history data
                    if cur_history.tyre_stints_history_data.len()
                        > last_history.tyre_stints_history_data.len()
                    {
                        diff_history.tyre_stints_history_data = cur_history
                            .tyre_stints_history_data
                            [last_history.tyre_stints_history_data.len()..]
                            .to_vec();
                        history_changed = true;
                    }

                    if history_changed {
                        diff_player.lap_history = Some(diff_history);
                        player_changed = true;
                    }
                } else if cur_player.lap_history != last_player.lap_history {
                    diff_player.lap_history = cur_player.lap_history.clone();
                    player_changed = true;
                }

                if cur_player.final_classification != last_player.final_classification {
                    diff_player.final_classification = cur_player.final_classification.clone();
                    player_changed = true;
                }

                if player_changed {
                    diff.players.insert(key.clone(), diff_player);
                    has_changes = true;
                }
            } else {
                diff.players.insert(key.clone(), cur_player.clone());
                has_changes = true;
            }
        }

        if has_changes {
            Some(diff)
        } else {
            None
        }
    }
}

impl EventData {
    /// Creates a new EventData from F1 event data
    #[inline]
    pub fn from_f1(
        f1_event: &F1PacketEventData,
        participants: &AHashMap<usize, DriverInfo>,
    ) -> Option<Self> {
        let Ok(event_code) = EventCode::try_from(&f1_event.event_string_code) else {
            warn!("Unknown event code: {:?}", f1_event.event_string_code);
            return None;
        };

        if NOT_SEND_EVENTS.contains(&event_code) {
            return None;
        }

        Some(EventData {
            string_code: f1_event.event_string_code.to_vec(),
            event_details: Some(Self::convert_event_data_details(
                &event_code,
                &f1_event.event_details,
                participants,
            )),
        })
    }

    /// Gets the Steam name for a given vehicle index
    #[inline]
    fn get_steam_name(participants: &AHashMap<usize, DriverInfo>, vehicle_idx: u8) -> String {
        participants
            .get(&(vehicle_idx as usize))
            .map(|participant| participant.name.to_string())
            .unwrap_or_else(|| format!("Unknown Driver {}", vehicle_idx))
    }

    /// Converts F1 event data details to EventDataDetails
    #[inline]
    fn convert_event_data_details(
        event_code: &EventCode,
        event_data_details: &F1EventDataDetails,
        participants: &AHashMap<usize, DriverInfo>,
    ) -> EventDataDetails {
        let details = match event_code {
            EventCode::FastestLap => {
                let fastest_lap = unsafe { &event_data_details.fastest_lap };
                Details::FastestLap(FastestLap {
                    steam_name: Self::get_steam_name(participants, fastest_lap.vehicle_idx),
                    lap_time: fastest_lap.lap_time,
                })
            }
            EventCode::Retirement => {
                let retirement = unsafe { &event_data_details.retirement };
                Details::Retirement(Retirement {
                    steam_name: Self::get_steam_name(participants, retirement.vehicle_idx),
                })
            }
            EventCode::RaceWinner => {
                let race_winner = unsafe { &event_data_details.race_winner };
                Details::RaceWinner(RaceWinner {
                    steam_name: Self::get_steam_name(participants, race_winner.vehicle_idx),
                })
            }
            EventCode::PenaltyIssued => {
                let penalty = unsafe { &event_data_details.penalty };
                Details::Penalty(Penalty {
                    penalty_type: penalty.penalty_type as u32,
                    infringement_type: penalty.infringement_type as u32,
                    steam_name: Self::get_steam_name(participants, penalty.vehicle_idx),
                    other_steam_name: Self::get_steam_name(participants, penalty.other_vehicle_idx),
                    time: penalty.time as u32,
                    lap_num: penalty.lap_num as u32,
                    places_gained: penalty.places_gained as u32,
                })
            }
            EventCode::SpeedTrapTriggered => {
                let speed_trap = unsafe { &event_data_details.speed_trap };
                Details::SpeedTrap(SpeedTrap {
                    steam_name: Self::get_steam_name(participants, speed_trap.vehicle_idx),
                    speed: speed_trap.speed,
                    is_overall_fastest_in_session: speed_trap.is_overall_fastest_in_session as u32,
                    is_driver_fastest_in_session: speed_trap.is_driver_fastest_in_session as u32,
                    fastest_driver_in_session: Self::get_steam_name(
                        participants,
                        speed_trap.fastest_vehicle_idx_in_session,
                    ),
                    fastest_speed_in_session: speed_trap.fastest_speed_in_session,
                })
            }
            EventCode::StartLights => {
                let start_lights = unsafe { &event_data_details.start_lights };
                Details::StartLights(StartLights {
                    num_lights: start_lights.num_lights as u32,
                })
            }
            EventCode::DriveThroughServed => {
                let drive_through = unsafe { &event_data_details.drive_through_penalty_served };
                Details::DriveThroughPenaltyServed(DriveThroughPenaltyServed {
                    steam_name: Self::get_steam_name(participants, drive_through.vehicle_idx),
                })
            }
            EventCode::StopGoServed => {
                let stop_go = unsafe { &event_data_details.stop_go_penalty_served };
                Details::StopGoPenaltyServed(StopGoPenaltyServed {
                    steam_name: Self::get_steam_name(participants, stop_go.vehicle_idx),
                })
            }
            EventCode::Overtake => {
                let overtake = unsafe { &event_data_details.overtake };
                Details::Overtake(Overtake {
                    overtaking_vehicle_idx: overtake.overtaking_vehicle_idx as u32,
                    being_overtaken_vehicle_idx: overtake.being_overtaken_vehicle_idx as u32,
                })
            }
            EventCode::SafetyCar => {
                let safety_car = unsafe { &event_data_details.safety_car };
                Details::SafetyCar(SafetyCar {
                    safety_car_type: safety_car.safety_car_type as u32,
                    event_type: safety_car.event_type as u32,
                })
            }
            EventCode::Collision => {
                let collision = unsafe { &event_data_details.collision };
                Details::Collision(Collision {
                    vehicle1_idx: collision.vehicle1_idx as u32,
                    vehicle2_idx: collision.vehicle2_idx as u32,
                })
            }
            _ => return EventDataDetails { details: None },
        };

        EventDataDetails {
            details: Some(details),
        }
    }
}
