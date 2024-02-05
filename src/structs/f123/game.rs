use zerocopy_derive::{FromBytes, FromZeros, KnownLayout, NoCell};

#[repr(C, packed)]
#[derive(Debug, Clone, Copy, FromBytes, FromZeros, NoCell, KnownLayout)]
pub struct PacketHeader {
    pub packet_format: u16,             // 2023
    pub game_year: u8,                  // Game year - last two digits e.g. 23
    pub game_major_version: u8,         // Game major version - "X.00"
    pub game_minor_version: u8,         // Game minor version - "1.XX"
    pub packet_version: u8,             // Version of this packet type, all start from 1
    pub packet_id: u8,                  // Identifier for the packet type, see below
    pub session_uid: u64,               // Unique identifier for the session
    pub session_time: f32,              // Session timestamp
    pub frame_identifier: u32,          // Identifier for the frame the data was retrieved on
    pub overall_frame_identifier: u32, // Overall identifier for the frame the data was retrieved  // on, doesn't go back after flashbacks
    pub player_car_index: u8,          // Index of player's car in the array
    pub secondary_player_car_index: u8, // Index of secondary player's car in the array (splitscreen) // 255 if no second player
}

#[repr(C, packed)]
#[derive(Debug, FromBytes, FromZeros, NoCell, KnownLayout)]
pub struct PacketMotionData {
    pub header: PacketHeader,                 // Header
    pub car_motion_data: [CarMotionData; 22], // Data for all cars on track
}

#[repr(C, packed)]
#[derive(FromBytes, FromZeros, NoCell, KnownLayout)]
pub struct PacketEventData {
    pub header: PacketHeader,            // Header
    pub event_string_code: [u8; 4],      // Event string code, see below
    pub event_details: EventDataDetails, // Event details - should be interpreted differently for each type
}

#[repr(C, packed)]
#[derive(Debug, FromBytes, FromZeros, NoCell, KnownLayout)]
pub struct PacketFinalClassificationData {
    pub header: PacketHeader, // Header
    pub num_cars: u8,         // Number of cars in the final classification
    pub classification_data: [FinalClassificationData; 22],
}

#[repr(C, packed)]
#[derive(Debug, FromBytes, FromZeros, NoCell, KnownLayout)]
pub struct PacketParticipantsData {
    pub header: PacketHeader, // Header
    pub num_active_cars: u8, // Number of active cars in the data – should match number of cars on HUD
    pub participants: [ParticipantData; 22],
}

#[repr(C, packed)]
#[derive(Debug, Clone, Copy, FromBytes, FromZeros, NoCell, KnownLayout)]
pub struct TyreStintHistoryData {
    pub end_lap: u8,              // Lap the tyre usage ends on (255 of current tyre)
    pub tyre_actual_compound: u8, // Actual tyres used by this driver
    pub tyre_visual_compound: u8, // Visual tyres used by this driver
}

#[repr(C, packed)]
#[derive(Debug, Clone, Copy, FromBytes, FromZeros, NoCell, KnownLayout)]
pub struct PacketCarStatusData {
    pub header: PacketHeader,                 // Header
    pub car_status_data: [CarStatusData; 22], // 22
}

#[repr(C, packed)]
#[derive(Debug, Clone, Copy, FromBytes, FromZeros, NoCell, KnownLayout)]
pub struct PacketCarDamageData {
    pub header: PacketHeader,                 // Header
    pub car_damage_data: [CarDamageData; 22], // 22
}

#[repr(C, packed)]
#[derive(Debug, Clone, Copy, FromBytes, FromZeros, NoCell, KnownLayout)]
pub struct PacketCarTelemetryData {
    pub header: PacketHeader,                       // Header
    pub car_telemetry_data: [CarTelemetryData; 22], // 22
    pub mfd_panel_index: u8,                        // Index of MFD panel open - 255 = MFD closed
    pub mfd_panel_index_secondary_player: u8, // Index of MFD panel open for secondary player - 255 = MFD closed
    pub suggested_gear: i8, // Suggested gear for the player (1-8) 0 if no gear suggested
}

#[repr(C, packed)]
#[derive(Debug, FromBytes, FromZeros, NoCell, KnownLayout)]
pub struct PacketSessionHistoryData {
    pub header: PacketHeader,
    pub car_idx: u8,
    pub num_laps: u8,
    pub num_tyre_stints: u8,
    pub best_lap_time_lap_num: u8,
    pub best_sector1_lap_num: u8,
    pub best_sector2_lap_num: u8,
    pub best_sector3_lap_num: u8,
    pub lap_history_data: [LapHistoryData; 100],
    pub tyre_stints_history_data: [TyreStintHistoryData; 8],
}

#[repr(C, packed)]
#[derive(Debug, FromBytes, FromZeros, NoCell, KnownLayout)]
pub struct PacketSessionData {
    pub header: PacketHeader,
    pub weather: u8, // Weather - 0 = clear, 1 = light cloud, 2 = overcast, 3 = light rain, 4 = heavy rain, 5 = storm
    pub track_temperature: i8, // Track temp. in degrees Celsius
    pub air_temperature: i8, // Air temp. in degrees Celsius
    pub total_laps: u8, // Total number of laps in this race
    pub track_length: u16, // Track length in metres
    pub session_type: u8, // 0 = unknown, 1 = P1, 2 = P2, 3 = P3, 4 = Short P5 = Q1, 6 = Q2, 7 = Q3, 8 = Short Q, 9 = OSQ 10 = R, 11 = R2, 12 = R3, 13 = Time Trial
    pub track_id: i8,     // TrackIds//  -1 for unknown, see appendix
    pub formula: u8, // Formula, 0 = F1 Modern, 1 = F1 Classic, 2 = F2, 3 = F1 Generic, 4 = Beta, 5 = Supercars, 6 = Esports, 7 = F2 2021
    pub session_time_left: u16, // Time left in session in seconds
    pub session_duration: u16, // Session duration in seconds
    pub pit_speed_limit: u8, // Pit speed limit in kilometres per hour
    pub game_paused: u8, // Whether the game is paused – network game only
    pub is_spectating: u8, // Whether the player is spectating
    pub spectator_car_index: u8, // Index of the car being spectated
    pub sli_pro_native_support: u8, // SLI Pro support, 0 = inactive, 1 = active
    pub num_marshal_zones: u8, // Number of marshal zones to follow
    pub marshal_zones: [MarshalZone; 21], // List of marshal zones – max 21
    pub safety_car_status: u8, // 0 = no safety car, 1 = full, 2 = virtual, 3 = formation lap
    pub network_game: u8, // 0 = offline, 1 = online
    pub num_weather_forecast_samples: u8, // Number of weather samples to follow
    pub weather_forecast_samples: [WeatherForecastSample; 56], // Array of weather forecast samples
    pub forecast_accuracy: u8, // 0 = Perfect, 1 = Approximate
    pub ai_difficulty: u8, // AI Difficulty rating – 0-110
    pub season_link_identifier: u32, // Identifier for season - persists across saves
    pub weekend_link_identifier: u32, // Identifier for weekend - persists across saves
    pub session_link_identifier: u32, // Identifier for session - persists across saves
    pub pit_stop_window_ideal_lap: u8, // Ideal lap to pit on for current strategy (player)
    pub pit_stop_window_latest_lap: u8, // Latest lap to pit on for current strategy (player)
    pub pit_stop_rejoin_position: u8, // Predicted position to rejoin at (player)
    pub steering_assist: u8, // 0 = off, 1 = on
    pub braking_assist: u8, // 0 = off, 1 = low, 2 = medium, 3 = high
    pub gearbox_assist: u8, // 1 = manual, 2 = manual & suggested gear, 3 = auto
    pub pit_assist: u8, // 0 = off, 1 = on
    pub pit_release_assist: u8, // 0 = off, 1 = on
    pub ers_assist: u8, // 0 = off, 1 = on
    pub drs_assist: u8, // 0 = off, 1 = on
    pub dynamic_racing_line: u8, // 0 = off, 1 = corners only, 2 = full
    pub dynamic_racing_line_type: u8, // 0 = 2D, 1 = 3D
    pub game_mode: u8, //GameModeIds // Game mode id - see appendix
    pub rule_set: u8, // RuleSetIds // Ruleset - see appendix
    pub time_of_day: u32, // Local time of day - minutes since midnight
    pub session_length: u8, // 0 = None, 2 = Very Short, 3 = Short, 4 = Medium 5 = Medium Long, 6 = Long, 7 = Full
    pub speed_units_lead_player: u8, // 0 = MPH, 1 = KPH
    pub temperature_units_lead_player: u8, // 0 = Celsius, 1 = Fahrenheit
    pub speed_units_secondary_player: u8, // 0 = MPH, 1 = KPH
    pub temperature_units_secondary_player: u8, // 0 = Celsius, 1 = Fahrenheit
    pub num_safety_car_periods: u8, // Number of safety cars called during session
    pub num_virtual_safety_car_periods: u8, // Number of virtual safety cars called
    pub num_red_flag_periods: u8, // Number of red flags called during session
}

#[repr(C, packed)]
#[derive(Debug, Clone, Copy, FromBytes, FromZeros, NoCell, KnownLayout)]
pub struct CarMotionData {
    pub world_position_x: f32,     // World space X position - metres
    pub world_position_y: f32,     // World space Y position
    pub world_position_z: f32,     // World space Z position
    pub world_velocity_x: f32,     // Velocity in world space X – metres/s
    pub world_velocity_y: f32,     // Velocity in world space Y
    pub world_velocity_z: f32,     // Velocity in world space Z
    pub world_forward_dir_x: i16,  // World space forward X direction (normalized)
    pub world_forward_dir_y: i16,  // World space forward Y direction (normalized)
    pub world_forward_dir_z: i16,  // World space forward Z direction (normalized)
    pub world_right_dir_x: i16,    // World space right X direction (normalized)
    pub world_right_dir_y: i16,    // World space right Y direction (normalized)
    pub world_right_dir_z: i16,    // World space right Z direction (normalized)
    pub g_force_lateral: f32,      // Lateral G-Force component
    pub g_force_longitudinal: f32, // Longitudinal G-Force component
    pub g_force_vertical: f32,     // Vertical G-Force component
    pub yaw: f32,                  // Yaw angle in radians
    pub pitch: f32,                // Pitch angle in radians
    pub roll: f32,                 // Roll angle in radians
}

#[repr(C, packed)]
#[derive(Debug, Clone, Copy, FromBytes, FromZeros, NoCell, KnownLayout)]
pub struct MarshalZone {
    pub zone_start: f32, // Fraction (0..1) of way through the lap the marshal zone starts
    pub zone_flag: i8,   // -1 = invalid/unknown, 0 = none, 1 = green, 2 = blue, 3 = yellow
}

#[repr(C, packed)]
#[derive(Debug, Clone, Copy, FromBytes, FromZeros, NoCell, KnownLayout)]
pub struct WeatherForecastSample {
    pub session_type: u8, // 0 = unknown, 1 = P1, 2 = P2, 3 = P3, 4 = Short P, 5 = Q1, 6 = Q2, 7 = Q3, 8 = Short Q, 9 = OSQ, 10 = R, 11 = R2, 12 = R3, 13 = Time Trial
    pub time_offset: u8,  //Time in minutes the forecast is for
    pub weather: u8, // Weather - 0 = clear, 1 = light cloud, 2 = overcast, 3 = light rain, 4 = heavy rain, 5 = storm
    pub track_temperature: i8, // Track temp. in degrees Celsius
    pub track_temperature_change: i8, // Track temp. change – 0 = up, 1 = down, 2 = no change
    pub air_temperature: i8, // Air temp. in degrees Celsius
    pub air_temperature_change: i8, // Air temp. change – 0 = up, 1 = down, 2 = no change
    pub rain_percentage: u8, // Rain percentage (0-100)
}

#[repr(C, packed)]
#[derive(Clone, Copy, FromBytes, FromZeros, NoCell, KnownLayout)]
pub struct FastestLap {
    pub vehicle_idx: u8,
    pub lap_time: f32,
}

#[repr(C, packed)]
#[derive(Clone, Copy, FromBytes, FromZeros, NoCell, KnownLayout)]
pub struct Retirement {
    pub vehicle_idx: u8,
}

#[repr(C, packed)]
#[derive(Clone, Copy, FromBytes, FromZeros, NoCell, KnownLayout)]
pub struct TeamMateInPits {
    pub vehicle_idx: u8,
}

#[repr(C, packed)]
#[derive(Clone, Copy, FromBytes, FromZeros, NoCell, KnownLayout)]
pub struct RaceWinner {
    pub vehicle_idx: u8,
}

#[repr(C, packed)]
#[derive(Clone, Copy, FromBytes, FromZeros, NoCell, KnownLayout)]
pub struct Penalty {
    pub penalty_type: u8,
    pub infringement_type: u8,
    pub vehicle_idx: u8,
    pub other_vehicle_idx: u8,
    pub time: u8,
    pub lap_num: u8,
    pub places_gained: u8,
}

#[repr(C, packed)]
#[derive(Clone, Copy, FromBytes, FromZeros, NoCell, KnownLayout)]
pub struct SpeedTrap {
    pub vehicle_idx: u8,
    pub speed: f32,
    pub is_overall_fastest_in_session: u8,
    pub is_driver_fastest_in_session: u8,
    pub fastest_vehicle_idx_in_session: u8,
    pub fastest_speed_in_session: f32,
}

#[repr(C, packed)]
#[derive(Clone, Copy, FromBytes, FromZeros, NoCell, KnownLayout)]
pub struct StartLights {
    pub num_lights: u8,
}

#[repr(C, packed)]
#[derive(Clone, Copy, FromBytes, FromZeros, NoCell, KnownLayout)]
pub struct DriveThroughPenaltyServed {
    pub vehicle_idx: u8,
}

#[repr(C, packed)]
#[derive(Clone, Copy, FromBytes, FromZeros, NoCell, KnownLayout)]
pub struct StopGoPenaltyServed {
    pub vehicle_idx: u8,
}

#[repr(C, packed)]
#[derive(Clone, Copy, FromBytes, FromZeros, NoCell, KnownLayout)]
pub struct Flashback {
    pub flashback_frame_identifier: u32, // Frame identifier flashed back to
    pub flashback_session_time: f32,     // Session time flashed back to
}

#[repr(C, packed)]
#[derive(Clone, Copy, FromBytes, FromZeros, NoCell, KnownLayout)]
pub struct Buttons {
    pub button_status: u32,
}

#[repr(C, packed)]
#[derive(Clone, Copy, FromBytes, FromZeros, NoCell, KnownLayout)]
pub struct Overtake {
    pub overtaking_vehicle_idx: u8,
    pub being_overtaken_vehicle_idx: u8,
}

#[repr(C, packed)]
#[derive(FromBytes, FromZeros, NoCell, KnownLayout)]
pub union EventDataDetails {
    pub fastest_lap: FastestLap,
    pub retirement: Retirement,
    pub team_mate_in_pits: TeamMateInPits,
    pub race_winner: RaceWinner,
    pub penalty: Penalty,
    pub speed_trap: SpeedTrap,
    pub start_lights: StartLights,
    pub drive_through_penalty_served: DriveThroughPenaltyServed,
    pub stop_go_penalty_served: StopGoPenaltyServed,
    pub flashback: Flashback,
    pub buttons: Buttons,
    pub overtake: Overtake,
}

#[repr(C, packed)]
#[derive(Debug, Clone, Copy, FromBytes, FromZeros, NoCell, KnownLayout)]
pub struct ParticipantData {
    pub ai_controlled: u8,  // Whether the vehicle is AI (1) or Human (0) controlled
    pub driver_id: u8,      // Driver id - see appendix, 255 if network human
    pub network_id: u8,     // Network id – unique identifier for network players
    pub team_id: u8,        // Team id - see appendix
    pub my_team: u8,        // My team flag – 1 = My Team, 0 = otherwise
    pub race_number: u8,    // Race number of the car
    pub nationality: u8,    // ParticipantNationality // Nationality of the driver
    pub name: [u8; 48], // Name of participant in UTF-8 format – null terminated, Will be truncated with … (U+2026) if too long
    pub your_telemetry: u8, // The player's UDP setting, 0 = restricted, 1 = public
    pub show_online_names: u8, // The player's show online names setting, 0 = off, 1 = on
    pub platform: u8,   // 1 = Steam, 3 = PlayStation, 4 = Xbox, 6 = Origin, 255 = unknown
}

#[repr(C, packed)]
#[derive(Debug, Clone, Copy, FromBytes, FromZeros, NoCell, KnownLayout)]
pub struct FinalClassificationData {
    pub position: u8,                  // Finishing position
    pub num_laps: u8,                  // Number of laps completed
    pub grid_position: u8,             // Grid position of the car
    pub points: u8,                    // Number of points scored
    pub num_pit_stops: u8,             // Number of pit stops made
    pub result_status: u8, // Result status - 0 = invalid, 1 = inactive, 2 = active, 3 = finished, 4 = didnotfinish, 5 = disqualified, 6 = not classified, 7 = retired
    pub best_lap_time_in_ms: u32, // Best lap time of the session in milliseconds
    pub total_race_time: f64, // Total race time in seconds without penalties
    pub penalties_time: u8, // Total penalties accumulated in seconds
    pub num_penalties: u8, // Number of penalties applied to this driver
    pub num_tyre_stints: u8, // Number of tyres stints up to maximum
    pub tyre_stints_actual: [u8; 8], // Actual tyres used by this driver
    pub tyre_stints_visual: [u8; 8], // Visual tyres used by this driver
    pub tyre_stints_end_laps: [u8; 8], // The lap number stints end on
}

#[repr(C, packed)]
#[derive(Debug, Clone, Copy, FromBytes, FromZeros, NoCell, KnownLayout)]
pub struct LapHistoryData {
    pub lap_time_in_ms: u32,      // Lap time in milliseconds
    pub sector1_time_in_ms: u16,  // Sector 1 time in milliseconds
    pub sector1_time_minutes: u8, // Sector 1 whole minute part
    pub sector2_time_in_ms: u16,  // Sector 2 time in milliseconds
    pub sector2_time_minutes: u8, // Sector 2 whole minute part
    pub sector3_time_in_ms: u16,  // Sector 3 time in milliseconds
    pub sector3_time_minutes: u8, // Sector 3 whole minute part
    pub lap_valid_bit_flags: u8, // 0x01 bit set - lap valid, 0x02 bit set - sector 1 valid, 0x04 bit set - sector 2 valid, 0x08 bit set - sector 3 valid
}

#[repr(C, packed)]
#[derive(Debug, Clone, Copy, FromBytes, FromZeros, NoCell, KnownLayout)]
pub struct CarTelemetryData {
    pub speed: u16,                         // Speed of car in km/h
    pub throttle: f32,                      // 0.0 - 1.0
    pub steer: f32,                         // -1.0 - 1.0
    pub brake: f32,                         // 0.0 - 1.0
    pub clutch: u8,                         // 0 - 100
    pub gear: i8,                           // (1-8, N=0, R=-1)
    pub engine_rpm: u16,                    // Engine RPM
    pub drs: u8,                            // 0 = off, 1 = on
    pub rev_lights_percent: u8,             // Rev lights indicator (percentage)
    pub rev_lights_bit_value: u16, // Rev lights (bit 0 = lights on the wheel, bit 1 = lights on the dash, bit 2 = lights on the display)
    pub brakes_temperature: [u16; 4], // Brakes temperature (celsius)
    pub tyres_surface_temperature: [u8; 4], // Tyres surface temperature (celsius)
    pub tyres_inner_temperature: [u8; 4], // Tyres inner temperature (celsius)
    pub engine_temperature: u16,   // Engine temperature (celsius)
    pub tyres_pressure: [f32; 4],  // Tyres pressure (PSI)
    pub surface_type: [u8; 4],     // Driving surface, see appendices
}

#[repr(C, packed)]
#[derive(Debug, Clone, Copy, FromBytes, FromZeros, NoCell, KnownLayout)]
pub struct CarStatusData {
    pub traction_control: u8, // Traction control - 0 = off, 1 = medium, 2 = full
    pub anti_lock_brakes: u8, // 0 (off) - 1 (on)
    pub fuel_mix: u8,         // Fuel mix - 0 = lean, 1 = standard, 2 = rich, 3 = max
    pub front_brake_bias: u8, // Front brake bias (percentage)
    pub pit_limiter_status: u8, // Pit limiter status - 0 = off, 1 = on
    pub fuel_in_tank: f32,    // Current fuel mass
    pub fuel_capacity: f32,   // Fuel capacity
    pub fuel_remaining_laps: f32, // Fuel remaining in terms of laps (value on MFD)
    pub max_rpm: u16,         // Cars max RPM, point of rev limiter
    pub idle_rpm: u16,        // Cars idle RPM
    pub max_gears: u8,        // Maximum number of gears
    pub drs_allowed: u8,      // 0 = not allowed, 1 = allowed
    pub drs_activation_distance: u16, // 0 = DRS not available, non-zero - DRS will be available in [X] metres
    pub actual_tyre_compound: u8,     // Tyre compound actual
    pub visual_tyre_compound: u8, // Tyre compound visual (can be different from actual compound)
    pub tyres_age_laps: u8,       // Age in laps of the current set of tyres
    pub vehicle_fia_flags: i8,    // -1 = invalid/unknown, 0 = none, 1 = green, 2 = blue, 3 = yellow
    pub engine_power_ice: f32,    // Engine power output of ICE (W)
    pub engine_power_mguk: f32,   // Engine power output of MGU-K (W)
    pub ers_store_energy: f32,    // ERS energy store in Joules
    pub ers_deploy_mode: u8, // ERS deployment mode, 0 = none, 1 = medium, 2 = hotlap, 3 = overtake
    pub ers_harvested_this_lap_mguk: f32, // ERS energy harvested this lap by MGU-K
    pub ers_harvested_this_lap_mguh: f32, // ERS energy harvested this lap by MGU-H
    pub ers_deployed_this_lap: f32, // ERS energy deployed this lap
    pub network_paused: u8,  // Whether the car is paused in a network game
}

#[repr(C, packed)]
#[derive(Debug, Clone, Copy, FromBytes, FromZeros, NoCell, KnownLayout)]
#[repr(C)]
pub struct CarDamageData {
    pub tyres_wear: [f32; 4],        // Tyre wear (percentage)
    pub tyres_damage: [u8; 4],       // Tyre damage (percentage)
    pub brakes_damage: [u8; 4],      // Brakes damage (percentage)
    pub front_left_wing_damage: u8,  // Front left wing damage (percentage)
    pub front_right_wing_damage: u8, // Front right wing damage (percentage)
    pub rear_wing_damage: u8,        // Rear wing damage (percentage)
    pub floor_damage: u8,            // Floor damage (percentage)
    pub diffuser_damage: u8,         // Diffuser damage (percentage)
    pub sidepod_damage: u8,          // Sidepod damage (percentage)
    pub drs_fault: u8,               // Indicator for DRS fault, 0 = OK, 1 = fault
    pub ers_fault: u8,               // Indicator for ERS fault, 0 = OK, 1 = fault
    pub gear_box_damage: u8,         // Gear box damage (percentage)
    pub engine_damage: u8,           // Engine damage (percentage)
    pub engine_mguh_wear: u8,        // Engine wear MGU-H (percentage)
    pub engine_es_wear: u8,          // Engine wear ES (percentage)
    pub engine_ce_wear: u8,          // Engine wear CE (percentage)
    pub engine_ice_wear: u8,         // Engine wear ICE (percentage)
    pub engine_mguk_wear: u8,        // Engine wear MGU-K (percentage)
    pub engine_tc_wear: u8,          // Engine wear TC (percentage)
    pub engine_blown: u8,            // Engine blown, 0 = OK, 1 = fault
    pub engine_seized: u8,           // Engine seized, 0 = OK, 1 = fault
}
