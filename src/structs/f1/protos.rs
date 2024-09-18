use prost::{Message, Oneof};
use std::collections::HashMap;

use super::{
    CarMotionData as F1CarMotionData, LapHistoryData as F1LapHistoryData, PacketSessionData,
    PacketSessionHistoryData, ParticipantData as F1ParticipantData,
    TyreStintHistoryData as F1TyreStintHistoryData,
};

// TODO: Implement manual PartialEq

#[derive(Clone, PartialEq, Message)]
pub struct F1GeneralInfo {
    #[prost(map = "string, message", tag = "1")]
    pub players: HashMap<String, PlayerInfo>,
    #[prost(message, optional, tag = "2")]
    pub session: Option<SessionData>,
    #[prost(message, optional, tag = "3")]
    pub events: Option<PacketsEventsData>,
}

#[derive(Clone, PartialEq, Message)]
pub struct PlayerInfo {
    #[prost(message, optional, tag = "1")]
    pub participant: Option<ParticipantData>,
    #[prost(message, optional, tag = "2")]
    pub car_motion: Option<CarMotionData>,
    #[prost(message, optional, tag = "3")]
    pub lap_history: Option<HistoryData>,
    #[prost(message, optional, tag = "4")]
    pub final_classification: Option<FinalClassificationData>,
}

#[derive(Clone, Copy, PartialEq, Message)]
pub struct ParticipantData {
    #[prost(uint32, optional, tag = "1")]
    pub team_id: Option<u32>,
    #[prost(uint32, optional, tag = "2")]
    pub race_number: Option<u32>,
    #[prost(uint32, optional, tag = "3")]
    pub nationality: Option<u32>,
    #[prost(uint32, optional, tag = "4")]
    pub platform: Option<u32>,
}

#[derive(Clone, Copy, PartialEq, Message)]
pub struct CarMotionData {
    #[prost(float, optional, tag = "1")]
    pub x: Option<f32>,
    #[prost(float, optional, tag = "2")]
    pub y: Option<f32>,
    #[prost(float, optional, tag = "3")]
    pub yaw: Option<f32>,
}

#[derive(Clone, PartialEq, Message)]
pub struct HistoryData {
    #[prost(uint32, optional, tag = "1")]
    pub num_laps: Option<u32>,
    #[prost(uint32, optional, tag = "2")]
    pub num_tyre_stints: Option<u32>,
    #[prost(uint32, optional, tag = "3")]
    pub best_lap_time_lap_num: Option<u32>,
    #[prost(uint32, optional, tag = "4")]
    pub best_s1_lap_num: Option<u32>,
    #[prost(uint32, optional, tag = "5")]
    pub best_s2_lap_num: Option<u32>,
    #[prost(uint32, optional, tag = "6")]
    pub best_s3_lap_num: Option<u32>,
    #[prost(message, repeated, tag = "7")]
    pub lap_history_data: Vec<LapHistoryData>,
    #[prost(message, repeated, tag = "8")]
    pub tyre_stints_history_data: Vec<TyreStintsHistoryData>,
}

#[derive(Clone, PartialEq, Message)]
pub struct LapHistoryData {
    #[prost(uint32, optional, tag = "1")]
    pub lap_time: Option<u32>,
    #[prost(uint32, optional, tag = "2")]
    pub s1_time: Option<u32>,
    #[prost(uint32, optional, tag = "3")]
    pub s2_time: Option<u32>,
    #[prost(uint32, optional, tag = "4")]
    pub s3_time: Option<u32>,
    #[prost(uint32, optional, tag = "5")]
    pub lap_valid_bit_flag: Option<u32>,
}

#[derive(Clone, PartialEq, Message)]
pub struct TyreStintsHistoryData {
    #[prost(uint32, optional, tag = "1")]
    pub end_lap: Option<u32>,
    #[prost(uint32, optional, tag = "2")]
    pub actual_compound: Option<u32>,
    #[prost(uint32, optional, tag = "3")]
    pub visual_compound: Option<u32>,
}

#[derive(Clone, PartialEq, Message)]
pub struct FinalClassificationData {
    #[prost(uint32, optional, tag = "1")]
    pub position: Option<u32>,
    #[prost(uint32, optional, tag = "2")]
    pub laps: Option<u32>,
    #[prost(uint32, optional, tag = "3")]
    pub grid_position: Option<u32>,
    #[prost(uint32, optional, tag = "4")]
    pub points: Option<u32>,
    #[prost(uint32, optional, tag = "5")]
    pub pit_stops: Option<u32>,
    #[prost(uint32, optional, tag = "6")]
    pub result_status: Option<u32>,
    #[prost(uint32, optional, tag = "7")]
    pub best_lap_time: Option<u32>,
    #[prost(double, optional, tag = "8")]
    pub race_time: Option<f64>,
    #[prost(uint32, optional, tag = "9")]
    pub penalties_time: Option<u32>,
    #[prost(uint32, optional, tag = "10")]
    pub num_penalties: Option<u32>,
    #[prost(uint32, repeated, tag = "11")]
    pub tyre_stints_actual: Vec<u32>,
    #[prost(uint32, repeated, tag = "12")]
    pub tyre_stints_visual: Vec<u32>,
    #[prost(uint32, repeated, tag = "13")]
    pub tyre_stints_end_laps: Vec<u32>,
}

#[derive(Clone, PartialEq, Message)]
pub struct SessionData {
    #[prost(uint32, optional, tag = "1")]
    pub weather: Option<u32>,
    #[prost(int32, optional, tag = "2")]
    pub track_temperature: Option<i32>,
    #[prost(int32, optional, tag = "3")]
    pub air_temperature: Option<i32>,
    #[prost(uint32, optional, tag = "4")]
    pub total_laps: Option<u32>,
    #[prost(uint32, optional, tag = "5")]
    pub track_length: Option<u32>,
    #[prost(uint32, optional, tag = "6")]
    pub session_type: Option<u32>,
    #[prost(int32, optional, tag = "7")]
    pub track_id: Option<i32>,
    #[prost(uint32, optional, tag = "8")]
    pub session_time_left: Option<u32>,
    #[prost(uint32, optional, tag = "9")]
    pub session_duration: Option<u32>,
    #[prost(uint32, optional, tag = "10")]
    pub safety_car_status: Option<u32>,
    #[prost(uint32, optional, tag = "11")]
    pub session_length: Option<u32>,
    #[prost(uint32, optional, tag = "12")]
    pub num_safety_car: Option<u32>,
    #[prost(uint32, optional, tag = "13")]
    pub num_virtual_safety_car: Option<u32>,
    #[prost(uint32, optional, tag = "14")]
    pub num_red_flags: Option<u32>,
    #[prost(uint32, repeated, tag = "15")]
    pub weekend_structure: Vec<u32>,
    #[prost(float, optional, tag = "16")]
    pub s2_lap_distance_start: Option<f32>,
    #[prost(float, optional, tag = "17")]
    pub s3_lap_distance_start: Option<f32>,
}

#[derive(Clone, PartialEq, Message)]
pub struct PacketsEventsData {
    #[prost(message, repeated, tag = "1")]
    pub events: Vec<EventData>,
}

#[derive(Clone, PartialEq, Message)]
pub struct EventData {
    #[prost(bytes, tag = "1")]
    pub string_code: Vec<u8>,
    #[prost(message, optional, tag = "2")]
    pub event_details: Option<EventDataDetails>,
}

#[derive(Clone, PartialEq, Message)]
pub struct EventDataDetails {
    #[prost(
        oneof = "EventDataDetailsOneof",
        tags = "1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11"
    )]
    pub details: Option<EventDataDetailsOneof>,
}

#[derive(Clone, PartialEq, Oneof)]
pub enum EventDataDetailsOneof {
    #[prost(message, tag = "1")]
    FastestLap(FastestLap),
    #[prost(message, tag = "2")]
    Retirement(Retirement),
    #[prost(message, tag = "3")]
    RaceWinner(RaceWinner),
    #[prost(message, tag = "4")]
    Penalty(Penalty),
    #[prost(message, tag = "5")]
    SpeedTrap(SpeedTrap),
    #[prost(message, tag = "6")]
    StartLights(StartLights),
    #[prost(message, tag = "7")]
    DriveThroughPenaltyServed(DriveThroughPenaltyServed),
    #[prost(message, tag = "8")]
    StopGoPenaltyServed(StopGoPenaltyServed),
    #[prost(message, tag = "9")]
    Overtake(Overtake),
    #[prost(message, tag = "10")]
    SafetyCar(SafetyCar),
    #[prost(message, tag = "11")]
    Collision(Collision),
}

#[derive(Clone, PartialEq, Message)]
pub struct FastestLap {
    #[prost(string, tag = "1")]
    pub steam_name: String,
    #[prost(float, tag = "2")]
    pub lap_time: f32,
}

#[derive(Clone, PartialEq, Message)]
pub struct Retirement {
    #[prost(string, tag = "1")]
    pub steam_name: String,
}

#[derive(Clone, PartialEq, Message)]
pub struct RaceWinner {
    #[prost(string, tag = "1")]
    pub steam_name: String,
}

#[derive(Clone, PartialEq, Message)]
pub struct Penalty {
    #[prost(uint32, tag = "1")]
    pub penalty_type: u32,
    #[prost(uint32, tag = "2")]
    pub infringement_type: u32,
    #[prost(string, tag = "3")]
    pub steam_name: String,
    #[prost(string, tag = "4")]
    pub other_steam_name: String,
    #[prost(uint32, tag = "5")]
    pub time: u32,
    #[prost(uint32, tag = "6")]
    pub lap_num: u32,
    #[prost(uint32, tag = "7")]
    pub places_gained: u32,
}

#[derive(Clone, PartialEq, Message)]
pub struct SpeedTrap {
    #[prost(string, tag = "1")]
    pub steam_name: String,
    #[prost(float, tag = "2")]
    pub speed: f32,
    #[prost(uint32, tag = "3")]
    pub is_overall_fastest_in_session: u32,
    #[prost(uint32, tag = "4")]
    pub is_driver_fastest_in_session: u32,
    #[prost(string, tag = "5")]
    pub fastest_driver_in_session: String,
    #[prost(float, tag = "6")]
    pub fastest_speed_in_session: f32,
}

#[derive(Clone, PartialEq, Message)]
pub struct StartLights {
    #[prost(uint32, tag = "1")]
    pub num_lights: u32,
}

#[derive(Clone, PartialEq, Message)]
pub struct DriveThroughPenaltyServed {
    #[prost(string, tag = "1")]
    pub steam_name: String,
}

#[derive(Clone, PartialEq, Message)]
pub struct StopGoPenaltyServed {
    #[prost(string, tag = "1")]
    pub steam_name: String,
}

#[derive(Clone, PartialEq, Message)]
pub struct Overtake {
    #[prost(uint32, tag = "1")]
    pub overtaking_vehicle_idx: u32,
    #[prost(uint32, tag = "2")]
    pub being_overtaken_vehicle_idx: u32,
}

#[derive(Clone, PartialEq, Message)]
pub struct SafetyCar {
    #[prost(uint32, tag = "1")]
    pub safety_car_type: u32,
    #[prost(uint32, tag = "2")]
    pub event_type: u32,
}

#[derive(Clone, PartialEq, Message)]
pub struct Collision {
    #[prost(uint32, tag = "1")]
    pub vehicle1_idx: u32,
    #[prost(uint32, tag = "2")]
    pub vehicle2_idx: u32,
}

#[derive(Clone, PartialEq, Message)]
pub struct F1TelemetryInfo {
    #[prost(map = "string, message", tag = "1")]
    pub player_telemetry: HashMap<String, PlayerTelemetry>,
}

#[derive(Clone, PartialEq, Message)]
pub struct PlayerTelemetry {
    #[prost(message, optional, tag = "1")]
    pub car_telemetry: Option<CarTelemetryData>,
    #[prost(message, optional, tag = "2")]
    pub car_status: Option<CarStatusData>,
    #[prost(message, optional, tag = "3")]
    pub car_damage: Option<CarDamageData>,
}

#[derive(Clone, PartialEq, Message)]
pub struct CarTelemetryData {
    #[prost(uint32, tag = "1")]
    pub speed: u32,
    #[prost(float, tag = "2")]
    pub throttle: f32,
    #[prost(float, tag = "3")]
    pub steer: f32,
    #[prost(float, tag = "4")]
    pub brake: f32,
    #[prost(sint32, tag = "6")]
    pub gear: i32,
    #[prost(uint32, tag = "7")]
    pub engine_rpm: u32,
    #[prost(bool, tag = "8")]
    pub drs: bool,
    #[prost(uint32, repeated, packed = "true", tag = "9")]
    pub brakes_temperature: Vec<u32>,
    #[prost(uint32, repeated, tag = "10")]
    pub tyres_surface_temperature: Vec<u32>,
    #[prost(uint32, repeated, tag = "11")]
    pub tyres_inner_temperature: Vec<u32>,
    #[prost(uint32, tag = "12")]
    pub engine_temperature: u32,
    #[prost(float, repeated, packed = "true", tag = "13")]
    pub tyres_pressure: Vec<f32>,
}

#[derive(Clone, PartialEq, Message)]
pub struct CarStatusData {
    #[prost(uint32, tag = "1")]
    pub fuel_mix: u32,
    #[prost(uint32, tag = "2")]
    pub front_brake_bias: u32,
    #[prost(float, tag = "3")]
    pub fuel_in_tank: f32,
    #[prost(float, tag = "4")]
    pub fuel_capacity: f32,
    #[prost(float, tag = "5")]
    pub fuel_remaining_laps: f32,
    #[prost(bool, tag = "6")]
    pub drs_allowed: bool,
    #[prost(uint32, tag = "7")]
    pub drs_activation_distance: u32,
    #[prost(uint32, tag = "8")]
    pub actual_tyre_compound: u32,
    #[prost(uint32, tag = "9")]
    pub visual_tyre_compound: u32,
    #[prost(uint32, tag = "10")]
    pub tyres_age_laps: u32,
    #[prost(sint32, tag = "11")]
    pub vehicle_fia_flags: i32,
    #[prost(float, tag = "12")]
    pub engine_power_ice: f32,
    #[prost(float, tag = "13")]
    pub engine_power_mguk: f32,
    #[prost(float, tag = "14")]
    pub ers_store_energy: f32,
    #[prost(uint32, tag = "15")]
    pub ers_deploy_mode: u32,
    #[prost(float, tag = "16")]
    pub ers_harvested_this_lap_mguk: f32,
    #[prost(float, tag = "17")]
    pub ers_harvested_this_lap_mguh: f32,
    #[prost(float, tag = "18")]
    pub ers_deployed_this_lap: f32,
}

#[derive(Clone, PartialEq, Message)]
pub struct CarDamageData {
    #[prost(float, repeated, tag = "1")]
    pub tyres_wear: Vec<f32>,
    #[prost(uint32, repeated, tag = "2")]
    pub tyres_damage: Vec<u32>,
    #[prost(uint32, repeated, tag = "3")]
    pub brakes_damage: Vec<u32>,
    #[prost(uint32, tag = "4")]
    pub front_left_wing_damage: u32,
    #[prost(uint32, tag = "5")]
    pub front_right_wing_damage: u32,
    #[prost(uint32, tag = "6")]
    pub rear_wing_damage: u32,
    #[prost(uint32, tag = "7")]
    pub floor_damage: u32,
    #[prost(uint32, tag = "8")]
    pub diffuser_damage: u32,
    #[prost(uint32, tag = "9")]
    pub sidepod_damage: u32,
    #[prost(bool, tag = "10")]
    pub drs_fault: bool,
    #[prost(bool, tag = "11")]
    pub ers_fault: bool,
    #[prost(uint32, tag = "12")]
    pub gear_box_damage: u32,
    #[prost(uint32, tag = "13")]
    pub engine_damage: u32,
    #[prost(uint32, tag = "14")]
    pub engine_mguh_wear: u32,
    #[prost(uint32, tag = "15")]
    pub engine_es_wear: u32,
    #[prost(uint32, tag = "16")]
    pub engine_ce_wear: u32,
    #[prost(uint32, tag = "17")]
    pub engine_ice_wear: u32,
    #[prost(uint32, tag = "18")]
    pub engine_mguk_wear: u32,
    #[prost(uint32, tag = "19")]
    pub engine_tc_wear: u32,
    #[prost(bool, tag = "20")]
    pub engine_blown: bool,
    #[prost(bool, tag = "21")]
    pub engine_seized: bool,
}

impl HistoryData {
    #[inline]
    fn update(&mut self, packet: &PacketSessionHistoryData) {
        // Update general data
        self.num_laps = Some(packet.num_laps as u32);
        self.num_tyre_stints = Some(packet.num_tyre_stints as u32);
        self.best_lap_time_lap_num = Some(packet.best_lap_time_lap_num as u32);
        self.best_s1_lap_num = Some(packet.best_sector1_lap_num as u32);
        self.best_s2_lap_num = Some(packet.best_sector2_lap_num as u32);
        self.best_s3_lap_num = Some(packet.best_sector3_lap_num as u32);

        // Update lap data
        let current_lap = packet.num_laps as usize;
        if current_lap > 1 && self.lap_history_data.len() >= current_lap - 1 {
            // Update last lap
            let previous_lap_data = &packet.lap_history_data[current_lap - 2];
            self.lap_history_data[current_lap - 2].update(previous_lap_data);
        }
        if current_lap > 0 {
            // Update or add current lap
            let current_lap_data = &packet.lap_history_data[current_lap - 1];
            if current_lap > self.lap_history_data.len() {
                self.lap_history_data
                    .push(LapHistoryData::from_f1(current_lap_data));
            } else {
                self.lap_history_data[current_lap - 1].update(current_lap_data);
            }
        }

        // Update tyre stints
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
    #[inline]
    fn from_f1(stints_data: &F1TyreStintHistoryData) -> Self {
        TyreStintsHistoryData {
            actual_compound: Some(stints_data.tyre_actual_compound as u32),
            visual_compound: Some(stints_data.tyre_visual_compound as u32),
            end_lap: Some(stints_data.end_lap as u32),
        }
    }
}

impl LapHistoryData {
    #[inline]
    fn from_f1(lap_data: &F1LapHistoryData) -> Self {
        LapHistoryData {
            lap_time: Some(lap_data.lap_time_in_ms),
            s1_time: Some(lap_data.sector1_time_in_ms as u32),
            s2_time: Some(lap_data.sector2_time_in_ms as u32),
            s3_time: Some(lap_data.sector3_time_in_ms as u32),
            lap_valid_bit_flag: Some(lap_data.lap_valid_bit_flags as u32),
        }
    }

    #[inline]
    fn update(&mut self, lap_data: &F1LapHistoryData) {
        self.lap_time = Some(lap_data.lap_time_in_ms);
        self.s1_time = Some(lap_data.sector1_time_in_ms as u32);
        self.s2_time = Some(lap_data.sector2_time_in_ms as u32);
        self.s3_time = Some(lap_data.sector3_time_in_ms as u32);
        self.lap_valid_bit_flag = Some(lap_data.lap_valid_bit_flags as u32);
    }
}

impl PlayerInfo {
    #[inline]
    pub fn update_car_motion(&mut self, incoming_motion: &F1CarMotionData) {
        let car_motion = self.car_motion.get_or_insert(CarMotionData::default());
        car_motion.x = Some(incoming_motion.world_position_x);
        car_motion.y = Some(incoming_motion.world_position_y);
        car_motion.yaw = Some(incoming_motion.yaw);
    }

    #[inline]
    pub fn update_session_history(&mut self, packet: &PacketSessionHistoryData) {
        let history = self.lap_history.get_or_insert(HistoryData::default());
        history.update(packet);
    }

    pub fn update_participant_info(&mut self, incoming_participant: &F1ParticipantData) {
        let participant = self.participant.get_or_insert(ParticipantData::default());
        participant.team_id = Some(incoming_participant.team_id as u32);
        participant.race_number = Some(incoming_participant.race_number as u32);
        participant.nationality = Some(incoming_participant.nationality as u32);
        participant.platform = Some(incoming_participant.platform as u32);
    }
}

impl F1GeneralInfo {
    pub fn update_session(&mut self, packet: &PacketSessionData) {
        let session = self.session.get_or_insert(SessionData::default());

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

        // Todo: implement this
        // session.weekend_structure = Some(packet.weekend_structure)
    }
}
