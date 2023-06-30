use serde::Deserialize;

#[repr(C)]
#[allow(dead_code, non_snake_case)]
#[derive(Debug, Deserialize)]
pub struct PacketHeader {
    pub m_packetFormat: u16,
    pub m_gameMajorVersion: u8,
    pub m_gameMinorVersion: u8,
    pub m_packetVersion: u8,
    pub m_packetId: u8,
    pub m_sessionUID: u64,
    pub m_sessionTime: f32,
    pub m_frameIdentifier: u32,
    pub m_playerCarIndex: u8,
    pub m_secondaryPlayerCarIndex: u8,
}

#[repr(C)]
#[allow(dead_code, non_snake_case)]
#[derive(Debug, Deserialize)]
pub struct CarMotionData {
    pub m_worldPositionX: f32,     // World space X position
    pub m_worldPositionY: f32,     // World space Y position
    pub m_worldPositionZ: f32,     // World space Z position
    pub m_worldVelocityX: f32,     // Velocity in world space X
    pub m_worldVelocityY: f32,     // Velocity in world space Y
    pub m_worldVelocityZ: f32,     // Velocity in world space Z
    pub m_worldForwardDirX: i16,   // World space forward X direction (normalised)
    pub m_worldForwardDirY: i16,   // World space forward Y direction (normalised)
    pub m_worldForwardDirZ: i16,   // World space forward Z direction (normalised)
    pub m_worldRightDirX: i16,     // World space right X direction (normalised)
    pub m_worldRightDirY: i16,     // World space right Y direction (normalised)
    pub m_worldRightDirZ: i16,     // World space right Z direction (normalised)
    pub m_gForceLateral: f32,      // Lateral G-Force component
    pub m_gForceLongitudinal: f32, // Longitudinal G-Force component
    pub m_gForceVertical: f32,     // Vertical G-Force component
    pub m_yaw: f32,                // Yaw angle in radians
    pub m_pitch: f32,              // Pitch angle in radians
    pub m_roll: f32,               // Roll angle in radians
}

#[repr(C)]
#[allow(dead_code, non_snake_case)]
#[derive(Debug, Deserialize)]
pub struct PacketMotionData {
    pub m_header: PacketHeader, // Header

    pub m_carMotionData: [CarMotionData; 22],

    // Extra player car ONLY data
    pub m_suspensionPosition: [f32; 4], // Note: All wheel arrays have the following order:
    pub m_suspensionVelocity: [f32; 4], // RL, RR, FL, FR
    pub m_suspensionAcceleration: [f32; 4], // RL, RR, FL, FR
    pub m_wheelSpeed: [f32; 4],         // Speed of each wheel
    pub m_wheelSlip: [f32; 4],          // Slip ratio for each wheel
    pub m_localVelocityX: f32,          // Velocity in local space
    pub m_localVelocityY: f32,          // Velocity in local space
    pub m_localVelocityZ: f32,          // Velocity in local space
    pub m_angularVelocityX: f32,        // Angular velocity x-component
    pub m_angularVelocityY: f32,        // Angular velocity y-component
    pub m_angularVelocityZ: f32,        // Angular velocity z-component
    pub m_angularAccelerationX: f32,    // Angular velocity x-component
    pub m_angularAccelerationY: f32,    // Angular velocity y-component
    pub m_angularAccelerationZ: f32,    // Angular velocity z-component
    pub m_frontWheelsAngle: f32,        // Current front wheels angle in radians
}

#[repr(C)]
#[allow(dead_code, non_snake_case)]
#[derive(Debug, Deserialize)]
pub struct MarshalZone {
    pub m_zoneStart: f32, // Fraction (0..1) of way through the lap the marshal zone starts
    pub m_zoneFlag: i8, // -1 = invalid/unknown, 0 = none, 1 = green, 2 = blue, 3 = yellow, 4 = red
}

#[repr(C)]
#[allow(dead_code, non_snake_case)]
#[derive(Debug, Deserialize)]
pub struct WeatherForecastSample {
    pub m_sessionType: u8, // 0 = unknown, 1 = P1, 2 = P2, 3 = P3, 4 = Short P, 5 = Q1, 6 = Q2, 7 = Q3, 8 = Short Q, 9 = OSQ, 10 = R, 11 = R2, 12 = Time Trial
    pub m_timeOffset: u8,  // Time in minutes the forecast is for
    pub m_weather: u8, // Weather - 0 = clear, 1 = light cloud, 2 = overcast 3 = light rain, 4 = heavy rain, 5 = storm
    pub m_trackTemperature: i8, // Track temp. in degrees celsius
    pub m_airTemperature: i8, // Air temp. in degrees celsius
}

#[repr(C)]
#[allow(dead_code, non_snake_case)]
#[derive(Debug, Deserialize)]
pub struct PacketSessionData {
    pub m_header: PacketHeader, // Suponiendo que PacketHeader también se ha definido en Rust

    pub m_weather: u8,
    pub m_track_temperature: i8,
    pub m_air_temperature: i8,
    pub m_total_laps: u8,
    pub m_track_length: u16,
    pub m_session_type: u8,
    pub m_track_id: i8,
    pub m_formula: u8,
    pub m_session_time_left: u16,
    pub m_session_duration: u16,
    pub m_pit_speed_limit: u8,
    pub m_game_paused: u8,
    pub m_is_spectating: u8,
    pub m_spectator_car_index: u8,
    pub m_sli_pro_native_support: u8,
    pub m_num_marshal_zones: u8,
    pub m_marshal_zones: [MarshalZone; 21], // Suponiendo que MarshalZone también se ha definido en Rust
    pub m_safety_car_status: u8,
    pub m_network_game: u8,
    pub m_num_weather_forecast_samples: u8,
    pub m_weather_forecast_samples: Vec<WeatherForecastSample>, // Suponiendo que WeatherForecastSample también se ha definido en Rust
    pub m_forecast_accuracy: u8,
    pub m_ai_difficulty: u8,
    pub m_season_link_identifier: u32,
    pub m_weekend_link_identifier: u32,
    pub m_session_link_identifier: u32,
    pub m_pit_stop_window_ideal_lap: u8,
    pub m_pit_stop_window_latest_lap: u8,
    pub m_pit_stop_rejoin_position: u8,
    pub m_steering_assist: u8,
    pub m_braking_assist: u8,
    pub m_gearbox_assist: u8,
    pub m_pit_assist: u8,
    pub m_pit_release_assist: u8,
    pub m_ers_assist: u8,
    pub m_drs_assist: u8,
    pub m_dynamic_racing_line: u8,
    pub m_dynamic_racing_line_type: u8,
    pub m_game_mode: u8,
    pub m_rule_set: u8,
    pub m_time_of_day: u32,
    pub m_session_length: u8,
}

#[repr(C)]
#[allow(dead_code, non_snake_case)]
#[derive(Debug, Deserialize)]
pub struct LapData {
    pub m_last_lap_time_in_ms: u32,
    pub m_current_lap_time_in_ms: u32,
    pub m_sector1_time_in_ms: u16,
    pub m_sector2_time_in_ms: u16,
    pub m_lap_distance: f32,
    pub m_total_distance: f32,
    pub m_safety_car_delta: f32,
    pub m_car_position: u8,
    pub m_current_lap_num: u8,
    pub m_pit_status: u8,
    pub m_num_pit_stops: u8,
    pub m_sector: u8,
    pub m_current_lap_invalid: u8,
    pub m_penalties: u8,
    pub m_warnings: u8,
    pub m_num_unserved_drive_through_pens: u8,
    pub m_num_unserved_stop_go_pens: u8,
    pub m_grid_position: u8,
    pub m_driver_status: u8,
    pub m_result_status: u8,
    pub m_pit_lane_timer_active: u8,
    pub m_pit_lane_time_in_lane_in_ms: u16,
    pub m_pit_stop_timer_in_ms: u16,
    pub m_pit_stop_should_serve_pen: u8,
}

#[repr(C)]
#[allow(dead_code, non_snake_case)]
#[derive(Debug, Deserialize)]
pub struct PacketLapData {
    pub m_header: PacketHeader, // Suponiendo que PacketHeader también se ha definido en Rust
    pub m_lap_data: [LapData; 22],
    pub m_time_trial_pb_car_idx: u8,
    pub m_time_trial_rival_car_idx: u8,
}

#[repr(C)]
#[allow(dead_code, non_snake_case)]
#[derive(Debug, Deserialize)]
pub enum EventDataDetails {
    FastestLap {
        vehicle_idx: u8,
        lap_time: f32,
    },
    Retirement {
        vehicle_idx: u8,
    },
    TeamMateInPits {
        vehicle_idx: u8,
    },
    RaceWinner {
        vehicle_idx: u8,
    },
    Penalty {
        penalty_type: u8,
        infringement_type: u8,
        vehicle_idx: u8,
        other_vehicle_idx: u8,
        time: u8,
        lap_num: u8,
        places_gained: u8,
    },
    SpeedTrap {
        vehicle_idx: u8,
        speed: f32,
        is_overall_fastest_in_session: u8,
        is_driver_fastest_in_session: u8,
        fastest_vehicle_idx_in_session: u8,
        fastest_speed_in_session: f32,
    },
    StartLights {
        num_lights: u8,
    },
    DriveThroughPenaltyServed {
        vehicle_idx: u8,
    },
    StopGoPenaltyServed {
        vehicle_idx: u8,
    },
    Flashback {
        flashback_frame_identifier: u32,
        flashback_session_time: f32,
    },
    Buttons {
        m_button_status: u32,
    },
}

#[repr(C)]
#[allow(dead_code, non_snake_case)]
#[derive(Debug, Deserialize)]
pub struct PacketEventData {
    pub m_header: PacketHeader, // Suponiendo que PacketHeader también se ha definido en Rust
    pub m_event_string_code: [u8; 4],
    pub m_event_details: EventDataDetails,
}

#[repr(C)]
#[allow(dead_code, non_snake_case)]
#[derive(Debug, Deserialize)]
pub struct ParticipantData {
    pub m_ai_controlled: u8,
    pub m_driver_id: u8,
    pub m_network_id: u8,
    pub m_team_id: u8,
    pub m_my_team: u8,
    pub m_race_number: u8,
    pub m_nationality: u8,
    pub m_name: String,
    pub m_your_telemetry: u8,
}

#[repr(C)]
#[allow(dead_code, non_snake_case)]
#[derive(Debug, Deserialize)]
pub struct PacketParticipantsData {
    pub m_header: PacketHeader, // Suponiendo que PacketHeader también se ha definido en Rust
    pub m_num_active_cars: u8,
    pub m_participants: [ParticipantData; 22],
}

#[repr(C)]
#[allow(dead_code, non_snake_case)]
#[derive(Debug, Deserialize)]
pub struct CarSetupData {
    pub m_front_wing: u8,
    pub m_rear_wing: u8,
    pub m_on_throttle: u8,
    pub m_off_throttle: u8,
    pub m_front_camber: f32,
    pub m_rear_camber: f32,
    pub m_front_toe: f32,
    pub m_rear_toe: f32,
    pub m_front_suspension: u8,
    pub m_rear_suspension: u8,
    pub m_front_anti_roll_bar: u8,
    pub m_rear_anti_roll_bar: u8,
    pub m_front_suspension_height: u8,
    pub m_rear_suspension_height: u8,
    pub m_brake_pressure: u8,
    pub m_brake_bias: u8,
    pub m_rear_left_tyre_pressure: f32,
    pub m_rear_right_tyre_pressure: f32,
    pub m_front_left_tyre_pressure: f32,
    pub m_front_right_tyre_pressure: f32,
    pub m_ballast: u8,
    pub m_fuel_load: f32,
}

#[repr(C)]
#[allow(dead_code, non_snake_case)]
#[derive(Debug, Deserialize)]
pub struct PacketCarSetupData {
    pub m_header: PacketHeader,
    pub m_car_setups: [CarSetupData; 22],
}

#[repr(C)]
#[allow(dead_code, non_snake_case)]
#[derive(Debug, Deserialize)]
pub struct CarTelemetryData {
    pub m_speed: u16,
    pub m_throttle: f32,
    pub m_steer: f32,
    pub m_brake: f32,
    pub m_clutch: u8,
    pub m_gear: i8,
    pub m_engine_rpm: u16,
    pub m_drs: u8,
    pub m_rev_lights_percent: u8,
    pub m_rev_lights_bit_value: u16,
    pub m_brakes_temperature: [u16; 4],
    pub m_tyres_surface_temperature: [u8; 4],
    pub m_tyres_inner_temperature: [u8; 4],
    pub m_engine_temperature: u16,
    pub m_tyres_pressure: [f32; 4],
    pub m_surface_type: [u8; 4],
}

#[repr(C)]
#[allow(dead_code, non_snake_case)]
#[derive(Debug, Deserialize)]
pub struct PacketCarTelemetryData {
    pub m_header: PacketHeader,
    pub m_car_telemetry_data: [CarTelemetryData; 22],
    pub m_mfd_panel_index: u8,
    pub m_mfd_panel_index_secondary_player: u8,
    pub m_suggested_gear: i8,
}

#[repr(C)]
#[allow(dead_code, non_snake_case)]
#[derive(Debug, Deserialize)]
pub struct CarStatusData {
    pub m_traction_control: u8,
    pub m_anti_lock_brakes: u8,
    pub m_fuel_mix: u8,
    pub m_front_brake_bias: u8,
    pub m_pit_limiter_status: u8,
    pub m_fuel_in_tank: f32,
    pub m_fuel_capacity: f32,
    pub m_fuel_remaining_laps: f32,
    pub m_max_rpm: u16,
    pub m_idle_rpm: u16,
    pub m_max_gears: u8,
    pub m_drs_allowed: u8,
    pub m_drs_activation_distance: u16,
    pub m_actual_tyre_compound: u8,
    pub m_visual_tyre_compound: u8,
    pub m_tyres_age_laps: u8,
    pub m_vehicle_fia_flags: i8,
    pub m_ers_store_energy: f32,
    pub m_ers_deploy_mode: u8,
    pub m_ers_harvested_this_lap_mguk: f32,
    pub m_ers_harvested_this_lap_mguh: f32,
    pub m_ers_deployed_this_lap: f32,
    pub m_network_paused: u8,
}

#[repr(C)]
#[allow(dead_code, non_snake_case)]
#[derive(Debug, Deserialize)]
pub struct PacketCarStatusData {
    pub m_header: PacketHeader,
    pub m_car_status_data: [CarStatusData; 22],
}

#[repr(C)]
#[allow(dead_code, non_snake_case)]
#[derive(Debug, Deserialize)]
pub struct FinalClassificationData {
    pub m_position: u8,
    pub m_num_laps: u8,
    pub m_grid_position: u8,
    pub m_points: u8,
    pub m_num_pit_stops: u8,
    pub m_result_status: u8,
    pub m_best_lap_time_in_ms: u32,
    pub m_total_race_time: f64,
    pub m_penalties_time: u8,
    pub m_num_penalties: u8,
    pub m_num_tyre_stints: u8,
    pub m_tyre_stints_actual: [u8; 8],
    pub m_tyre_stints_visual: [u8; 8],
    pub m_tyre_stints_end_laps: [u8; 8],
}

#[repr(C)]
#[allow(dead_code, non_snake_case)]
#[derive(Debug, Deserialize)]
pub struct PacketFinalClassificationData {
    pub m_header: PacketHeader,
    pub m_num_cars: u8,
    pub m_classification_data: [FinalClassificationData; 22],
}

#[repr(C)]
#[allow(dead_code, non_snake_case)]
#[derive(Debug, Deserialize)]
pub struct LobbyInfoData {
    pub m_ai_controlled: u8,
    pub m_team_id: u8,
    pub m_nationality: u8,
    pub m_name: String,
    pub m_car_number: u8,
    pub m_ready_status: u8,
}

#[repr(C)]
#[allow(dead_code, non_snake_case)]
#[derive(Debug, Deserialize)]
pub struct PacketLobbyInfoData {
    pub m_header: PacketHeader,
    pub m_num_players: u8,
    pub m_lobby_players: [LobbyInfoData; 22],
}

#[repr(C)]
#[allow(dead_code, non_snake_case)]
#[derive(Debug, Deserialize)]
pub struct CarDamageData {
    pub m_tyres_wear: [f32; 4],
    pub m_tyres_damage: [u8; 4],
    pub m_brakes_damage: [u8; 4],
    pub m_front_left_wing_damage: u8,
    pub m_front_right_wing_damage: u8,
    pub m_rear_wing_damage: u8,
    pub m_floor_damage: u8,
    pub m_diffuser_damage: u8,
    pub m_sidepod_damage: u8,
    pub m_drs_fault: u8,
    pub m_ers_fault: u8,
    pub m_gear_box_damage: u8,
    pub m_engine_damage: u8,
    pub m_engine_mgu_h_wear: u8,
    pub m_engine_es_wear: u8,
    pub m_engine_ce_wear: u8,
    pub m_engine_ice_wear: u8,
    pub m_engine_mgu_k_wear: u8,
    pub m_engine_tc_wear: u8,
    pub m_engine_blown: u8,
    pub m_engine_seized: u8,
}

#[repr(C)]
#[allow(dead_code, non_snake_case)]
#[derive(Debug, Deserialize)]
pub struct PacketCarDamageData {
    pub m_header: PacketHeader,
    pub m_car_damage_data: [CarDamageData; 22],
}

#[repr(C)]
#[allow(dead_code, non_snake_case)]
#[derive(Debug, Deserialize)]
pub struct LapHistoryData {
    pub m_lap_time_in_ms: u32,
    pub m_sector1_time_in_ms: u16,
    pub m_sector2_time_in_ms: u16,
    pub m_sector3_time_in_ms: u16,
    pub m_lap_valid_bit_flags: u8,
}

#[repr(C)]
#[allow(dead_code, non_snake_case)]
#[derive(Debug, Deserialize)]
pub struct TyreStintHistoryData {
    pub m_end_lap: u8,
    pub m_tyre_actual_compound: u8,
    pub m_tyre_visual_compound: u8,
}

#[repr(C)]
#[allow(dead_code, non_snake_case)]
#[derive(Debug, Deserialize)]
pub struct PacketSessionHistoryData {
    pub m_header: PacketHeader,
    pub m_car_idx: u8,
    pub m_num_laps: u8,
    pub m_num_tyre_stints: u8,
    pub m_best_lap_time_lap_num: u8,
    pub m_best_sector1_lap_num: u8,
    pub m_best_sector2_lap_num: u8,
    pub m_best_sector3_lap_num: u8,
    pub m_lap_history_data: Vec<LapHistoryData>,
    pub m_tyre_stints_history_data: [TyreStintHistoryData; 8],
}
