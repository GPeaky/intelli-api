use prost::{Message, Oneof};
use std::{collections::HashMap, ptr::addr_of};

use super::{
    CarDamageData as F1CarDamageData, CarMotionData as F1CarMotionData,
    CarStatusData as F1CarStatusData, CarTelemetryData as F1CarTelemetryData,
    LapHistoryData as F1LapHistoryData, PacketSessionData, PacketSessionHistoryData,
    ParticipantData as F1ParticipantData, TyreStintHistoryData as F1TyreStintHistoryData,
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
    #[prost(uint32, optional, tag = "1")]
    pub speed: Option<u32>,
    #[prost(float, optional, tag = "2")]
    pub throttle: Option<f32>,
    #[prost(float, optional, tag = "3")]
    pub steer: Option<f32>,
    #[prost(float, optional, tag = "4")]
    pub brake: Option<f32>,
    #[prost(sint32, optional, tag = "6")]
    pub gear: Option<i32>,
    #[prost(uint32, optional, tag = "7")]
    pub engine_rpm: Option<u32>,
    #[prost(bool, optional, tag = "8")]
    pub drs: Option<bool>,
    #[prost(uint32, repeated, packed = "true", tag = "9")]
    pub brakes_temperature: Vec<u32>,
    #[prost(uint32, repeated, tag = "10")]
    pub tyres_surface_temperature: Vec<u32>,
    #[prost(uint32, repeated, tag = "11")]
    pub tyres_inner_temperature: Vec<u32>,
    #[prost(uint32, optional, tag = "12")]
    pub engine_temperature: Option<u32>,
    #[prost(float, repeated, packed = "true", tag = "13")]
    pub tyres_pressure: Vec<f32>,
}

#[derive(Clone, PartialEq, Message)]
pub struct CarStatusData {
    #[prost(uint32, optional, tag = "1")]
    pub fuel_mix: Option<u32>,
    #[prost(uint32, optional, tag = "2")]
    pub front_brake_bias: Option<u32>,
    #[prost(float, optional, tag = "3")]
    pub fuel_in_tank: Option<f32>,
    #[prost(float, optional, tag = "4")]
    pub fuel_capacity: Option<f32>,
    #[prost(float, optional, tag = "5")]
    pub fuel_remaining_laps: Option<f32>,
    #[prost(bool, optional, tag = "6")]
    pub drs_allowed: Option<bool>,
    #[prost(uint32, optional, tag = "7")]
    pub drs_activation_distance: Option<u32>,
    #[prost(uint32, optional, tag = "8")]
    pub actual_tyre_compound: Option<u32>,
    #[prost(uint32, optional, tag = "9")]
    pub visual_tyre_compound: Option<u32>,
    #[prost(uint32, optional, tag = "10")]
    pub tyres_age_laps: Option<u32>,
    #[prost(sint32, optional, tag = "11")]
    pub vehicle_fia_flags: Option<i32>,
    #[prost(float, optional, tag = "12")]
    pub engine_power_ice: Option<f32>,
    #[prost(float, optional, tag = "13")]
    pub engine_power_mguk: Option<f32>,
    #[prost(float, optional, tag = "14")]
    pub ers_store_energy: Option<f32>,
    #[prost(uint32, optional, tag = "15")]
    pub ers_deploy_mode: Option<u32>,
    #[prost(float, optional, tag = "16")]
    pub ers_harvested_this_lap_mguk: Option<f32>,
    #[prost(float, optional, tag = "17")]
    pub ers_harvested_this_lap_mguh: Option<f32>,
    #[prost(float, optional, tag = "18")]
    pub ers_deployed_this_lap: Option<f32>,
}

#[derive(Clone, PartialEq, Message)]
pub struct CarDamageData {
    #[prost(float, repeated, tag = "1")]
    pub tyres_wear: Vec<f32>,
    #[prost(uint32, repeated, tag = "2")]
    pub tyres_damage: Vec<u32>,
    #[prost(uint32, repeated, tag = "3")]
    pub brakes_damage: Vec<u32>,
    #[prost(uint32, optional, tag = "4")]
    pub front_left_wing_damage: Option<u32>,
    #[prost(uint32, optional, tag = "5")]
    pub front_right_wing_damage: Option<u32>,
    #[prost(uint32, optional, tag = "6")]
    pub rear_wing_damage: Option<u32>,
    #[prost(uint32, optional, tag = "7")]
    pub floor_damage: Option<u32>,
    #[prost(uint32, optional, tag = "8")]
    pub diffuser_damage: Option<u32>,
    #[prost(uint32, optional, tag = "9")]
    pub sidepod_damage: Option<u32>,
    #[prost(bool, optional, tag = "10")]
    pub drs_fault: Option<bool>,
    #[prost(bool, optional, tag = "11")]
    pub ers_fault: Option<bool>,
    #[prost(uint32, optional, tag = "12")]
    pub gear_box_damage: Option<u32>,
    #[prost(uint32, optional, tag = "13")]
    pub engine_damage: Option<u32>,
    #[prost(uint32, optional, tag = "14")]
    pub engine_mguh_wear: Option<u32>,
    #[prost(uint32, optional, tag = "15")]
    pub engine_es_wear: Option<u32>,
    #[prost(uint32, optional, tag = "16")]
    pub engine_ce_wear: Option<u32>,
    #[prost(uint32, optional, tag = "17")]
    pub engine_ice_wear: Option<u32>,
    #[prost(uint32, optional, tag = "18")]
    pub engine_mguk_wear: Option<u32>,
    #[prost(uint32, optional, tag = "19")]
    pub engine_tc_wear: Option<u32>,
    #[prost(bool, optional, tag = "20")]
    pub engine_blown: Option<bool>,
    #[prost(bool, optional, tag = "21")]
    pub engine_seized: Option<bool>,
}

impl HistoryData {
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

    #[inline]
    fn update(&mut self, lap_data: &F1LapHistoryData) {
        *self = Self::from_f1(lap_data);
    }
}

impl PlayerInfo {
    #[inline]
    pub fn update_car_motion(&mut self, incoming_motion: &F1CarMotionData) {
        let car_motion = self.car_motion.get_or_insert_with(Default::default);
        car_motion.x = Some(incoming_motion.world_position_x);
        car_motion.y = Some(incoming_motion.world_position_y);
        car_motion.yaw = Some(incoming_motion.yaw);
    }

    #[inline]
    pub fn update_session_history(&mut self, packet: &PacketSessionHistoryData) {
        self.lap_history
            .get_or_insert_with(Default::default)
            .update(packet);
    }

    #[inline]
    pub fn update_participant_info(&mut self, incoming_participant: &F1ParticipantData) {
        let participant = self.participant.get_or_insert_with(Default::default);
        participant.team_id = Some(incoming_participant.team_id as u32);
        participant.race_number = Some(incoming_participant.race_number as u32);
        participant.nationality = Some(incoming_participant.nationality as u32);
        participant.platform = Some(incoming_participant.platform as u32);
    }
}

impl PlayerTelemetry {
    #[inline]
    pub fn update_car_damage(&mut self, data: &F1CarDamageData) {
        let car_damage = self.car_damage.get_or_insert_with(Default::default);

        unsafe {
            let tyres_wear_ptr = addr_of!(data.tyres_wear);
            car_damage.tyres_wear.extend_from_slice(&*tyres_wear_ptr);
        }

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
        unsafe {
            let brakes_temp_ptr = addr_of!(data.brakes_temperature);
            telemetry
                .brakes_temperature
                .extend((*brakes_temp_ptr).iter().map(|&x| x as u32));
        }

        telemetry.tyres_surface_temperature.clear();
        telemetry
            .tyres_surface_temperature
            .extend(data.tyres_surface_temperature.iter().map(|&x| x as u32));

        telemetry.tyres_inner_temperature.clear();
        telemetry
            .tyres_inner_temperature
            .extend(data.tyres_inner_temperature.iter().map(|&x| x as u32));

        telemetry.tyres_pressure.clear();
        unsafe {
            let tyres_pressure_ptr = addr_of!(data.tyres_pressure);
            telemetry
                .tyres_pressure
                .extend_from_slice(&*tyres_pressure_ptr);
        }
    }
}

impl F1GeneralInfo {
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
}
