use zerocopy::{FromBytes, FromZeroes, Unaligned};

//*  --- F1 2023 Packet Data Enums ---

#[repr(C)]
#[derive(Debug)]
pub enum PacketIds {
    Motion,
    Session,
    LapData,
    Event,
    Participants,
    CarSetups,
    CarTelemetry,
    CarStatus,
    FinalClassification,
    LobbyInfo,
    CarDamage,
    SessionHistory,
    TyreSets,
    MotionEx,
}

impl From<u8> for PacketIds {
    fn from(value: u8) -> Self {
        match value {
            0 => PacketIds::Motion,
            1 => PacketIds::Session,
            2 => PacketIds::LapData,
            3 => PacketIds::Event,
            4 => PacketIds::Participants,
            5 => PacketIds::CarSetups,
            6 => PacketIds::CarTelemetry,
            7 => PacketIds::CarStatus,
            8 => PacketIds::FinalClassification,
            9 => PacketIds::LobbyInfo,
            10 => PacketIds::CarDamage,
            11 => PacketIds::SessionHistory,
            12 => PacketIds::TyreSets,
            13 => PacketIds::MotionEx,
            _ => panic!("Unknown packet id {}", value),
        }
    }
}

//*  --- F1 2023 Packet Data Structures ---
#[repr(C, packed)]
#[allow(non_snake_case)]
#[derive(Debug, Clone, Copy, FromBytes, FromZeroes, Unaligned)]
pub struct PacketHeader {
    pub m_packetFormat: u16,           // 2023
    pub m_gameYear: u8,                // Game year - last two digits e.g. 23
    pub m_gameMajorVersion: u8,        // Game major version - "X.00"
    pub m_gameMinorVersion: u8,        // Game minor version - "1.XX"
    pub m_packetVersion: u8,           // Version of this packet type, all start from 1
    pub m_packetId: u8,                // Identifier for the packet type, see below
    pub m_sessionUID: u64,             // Unique identifier for the session
    pub m_sessionTime: f32,            // Session timestamp
    pub m_frameIdentifier: u32,        // Identifier for the frame the data was retrieved on
    pub m_overallFrameIdentifier: u32, // Overall identifier for the frame the data was retrieved  // on, doesn't go back after flashbacks
    pub m_playerCarIndex: u8,          // Index of player's car in the array
    pub m_secondaryPlayerCarIndex: u8, // Index of secondary player's car in the array (splitscreen) // 255 if no second player
}

#[repr(C, packed)]
#[allow(non_snake_case)]
#[derive(Debug, FromBytes, FromZeroes, Unaligned)]
pub struct PacketMotionData {
    pub m_header: PacketHeader,               // Header
    pub m_carMotionData: [CarMotionData; 22], // Data for all cars on track
}

#[repr(C)]
#[allow(non_snake_case)]
#[derive(FromBytes, FromZeroes, Unaligned)]
pub struct PacketEventData {
    pub m_header: PacketHeader,           // Header
    pub m_eventStringCode: [u8; 4],       // Event string code, see below
    pub m_eventDetails: EventDataDetails, // Event details - should be interpreted differently for each type
}

#[repr(C, packed)]
#[allow(non_snake_case)]
#[derive(Debug, FromBytes, FromZeroes, Unaligned)]
pub struct PacketFinalClassificationData {
    pub m_header: PacketHeader, // Header
    pub m_numCars: u8,          // Number of cars in the final classification
    pub m_classificationData: [FinalClassificationData; 22],
}

#[repr(C, packed)]
#[allow(non_snake_case)]
#[derive(Debug, FromBytes, FromZeroes, Unaligned)]
pub struct PacketParticipantsData {
    pub m_header: PacketHeader, // Header
    pub m_numActiveCars: u8, // Number of active cars in the data – should match number of cars on HUD
    pub m_participants: [ParticipantData; 22],
}

#[repr(C, packed)]
#[allow(non_snake_case)]
#[derive(Debug, FromBytes, FromZeroes, Unaligned)]
pub struct PacketSessionHistoryData {
    pub m_header: PacketHeader,
    pub m_carIdx: u8,
    pub m_numLaps: u8,
    pub m_numTyreStints: u8,
    pub m_bestLapTimeLapNum: u8,
    pub m_bestSector1LapNum: u8,
    pub m_bestSector2LapNum: u8,
    pub m_bestSector3LapNum: u8,
    pub m_lapHistoryData: [LapHistoryData; 100],
    pub m_tyreStintsHistoryData: [TyreStintHistoryData; 8],
}

#[repr(C, packed)]
#[allow(non_snake_case)]
#[derive(Debug, FromBytes, FromZeroes, Unaligned)]
pub struct PacketSessionData {
    pub m_header: PacketHeader,            // Header
    pub m_weather: u8, // Weather - 0 = clear, 1 = light cloud, 2 = overcast, 3 = light rain, 4 = heavy rain, 5 = storm
    pub m_trackTemperature: i8, // Track temp. in degrees Celsius
    pub m_airTemperature: i8, // Air temp. in degrees Celsius
    pub m_totalLaps: u8, // Total number of laps in this race
    pub m_trackLength: u16, // Track length in metres
    pub m_sessionType: u8, // 0 = unknown, 1 = P1, 2 = P2, 3 = P3, 4 = Short P5 = Q1, 6 = Q2, 7 = Q3, 8 = Short Q, 9 = OSQ 10 = R, 11 = R2, 12 = R3, 13 = Time Trial
    pub m_trackId: i8,     // TrackIds//  -1 for unknown, see appendix
    pub m_formula: u8, // Formula, 0 = F1 Modern, 1 = F1 Classic, 2 = F2, 3 = F1 Generic, 4 = Beta, 5 = Supercars, 6 = Esports, 7 = F2 2021
    pub m_sessionTimeLeft: u16, // Time left in session in seconds
    pub m_sessionDuration: u16, // Session duration in seconds
    pub m_pitSpeedLimit: u8, // Pit speed limit in kilometres per hour
    pub m_gamePaused: u8, // Whether the game is paused – network game only
    pub m_isSpectating: u8, // Whether the player is spectating
    pub m_spectatorCarIndex: u8, // Index of the car being spectated
    pub m_sliProNativeSupport: u8, // SLI Pro support, 0 = inactive, 1 = active
    pub m_numMarshalZones: u8, // Number of marshal zones to follow
    pub m_marshalZones: [MarshalZone; 21], // List of marshal zones – max 21
    pub m_safetyCarStatus: u8, // 0 = no safety car, 1 = full, 2 = virtual, 3 = formation lap
    pub m_networkGame: u8, // 0 = offline, 1 = online
    pub m_numWeatherForecastSamples: u8, // Number of weather samples to follow
    // #[serde(with = "BigArray")]
    pub m_weatherForecastSamples: [WeatherForecastSample; 56], // Array of weather forecast samples
    pub m_forecastAccuracy: u8,                                // 0 = Perfect, 1 = Approximate
    pub m_aiDifficulty: u8,                                    // AI Difficulty rating – 0-110
    pub m_seasonLinkIdentifier: u32, // Identifier for season - persists across saves
    pub m_weekendLinkIdentifier: u32, // Identifier for weekend - persists across saves
    pub m_sessionLinkIdentifier: u32, // Identifier for session - persists across saves
    pub m_pitStopWindowIdealLap: u8, // Ideal lap to pit on for current strategy (player)
    pub m_pitStopWindowLatestLap: u8, // Latest lap to pit on for current strategy (player)
    pub m_pitStopRejoinPosition: u8, // Predicted position to rejoin at (player)
    pub m_steeringAssist: u8,        // 0 = off, 1 = on
    pub m_brakingAssist: u8,         // 0 = off, 1 = low, 2 = medium, 3 = high
    pub m_gearboxAssist: u8,         // 1 = manual, 2 = manual & suggested gear, 3 = auto
    pub m_pitAssist: u8,             // 0 = off, 1 = on
    pub m_pitReleaseAssist: u8,      // 0 = off, 1 = on
    pub m_ERSAssist: u8,             // 0 = off, 1 = on
    pub m_DRSAssist: u8,             // 0 = off, 1 = on
    pub m_dynamicRacingLine: u8,     // 0 = off, 1 = corners only, 2 = full
    pub m_dynamicRacingLineType: u8, // 0 = 2D, 1 = 3D
    pub m_gameMode: u8,              //GameModeIds // Game mode id - see appendix
    pub m_ruleSet: u8,               // RuleSetIds // Ruleset - see appendix
    pub m_timeOfDay: u32,            // Local time of day - minutes since midnight
    pub m_sessionLength: u8, // 0 = None, 2 = Very Short, 3 = Short, 4 = Medium 5 = Medium Long, 6 = Long, 7 = Full
    pub m_speedUnitsLeadPlayer: u8, // 0 = MPH, 1 = KPH
    pub m_temperatureUnitsLeadPlayer: u8, // 0 = Celsius, 1 = Fahrenheit
    pub m_speedUnitsSecondaryPlayer: u8, // 0 = MPH, 1 = KPH
    pub m_temperatureUnitsSecondaryPlayer: u8, // 0 = Celsius, 1 = Fahrenheit
    pub m_numSafetyCarPeriods: u8, // Number of safety cars called during session
    pub m_numVirtualSafetyCarPeriods: u8, // Number of virtual safety cars called
    pub m_numRedFlagPeriods: u8, // Number of red flags called during session
}

//* --- F1 23 Unpacked Data ---

#[repr(C, packed)]
#[allow(non_snake_case)]
#[derive(Debug, Clone, Copy, FromBytes, FromZeroes, Unaligned)]
pub struct CarMotionData {
    pub m_worldPositionX: f32,     // World space X position - metres
    pub m_worldPositionY: f32,     // World space Y position
    pub m_worldPositionZ: f32,     // World space Z position
    pub m_worldVelocityX: f32,     // Velocity in world space X – metres/s
    pub m_worldVelocityY: f32,     // Velocity in world space Y
    pub m_worldVelocityZ: f32,     // Velocity in world space Z
    pub m_worldForwardDirX: i16,   // World space forward X direction (normalized)
    pub m_worldForwardDirY: i16,   // World space forward Y direction (normalized)
    pub m_worldForwardDirZ: i16,   // World space forward Z direction (normalized)
    pub m_worldRightDirX: i16,     // World space right X direction (normalized)
    pub m_worldRightDirY: i16,     // World space right Y direction (normalized)
    pub m_worldRightDirZ: i16,     // World space right Z direction (normalized)
    pub m_gForceLateral: f32,      // Lateral G-Force component
    pub m_gForceLongitudinal: f32, // Longitudinal G-Force component
    pub m_gForceVertical: f32,     // Vertical G-Force component
    pub m_yaw: f32,                // Yaw angle in radians
    pub m_pitch: f32,              // Pitch angle in radians
    pub m_roll: f32,               // Roll angle in radians
}

#[repr(C, packed)]
#[allow(non_snake_case)]
#[derive(Debug, Clone, Copy, FromBytes, FromZeroes, Unaligned)]
pub struct MarshalZone {
    pub m_zoneStart: f32, // Fraction (0..1) of way through the lap the marshal zone starts
    pub m_zoneFlag: i8,   // -1 = invalid/unknown, 0 = none, 1 = green, 2 = blue, 3 = yellow
}

#[repr(C, packed)]
#[allow(non_snake_case)]
#[derive(Debug, Clone, Copy, FromBytes, FromZeroes, Unaligned)]
pub struct WeatherForecastSample {
    pub m_sessionType: u8, // 0 = unknown, 1 = P1, 2 = P2, 3 = P3, 4 = Short P, 5 = Q1, 6 = Q2, 7 = Q3, 8 = Short Q, 9 = OSQ, 10 = R, 11 = R2, 12 = R3, 13 = Time Trial
    pub m_timeOffset: u8,  //Time in minutes the forecast is for
    pub m_weather: u8, // Weather - 0 = clear, 1 = light cloud, 2 = overcast, 3 = light rain, 4 = heavy rain, 5 = storm
    pub m_trackTemperature: i8, // Track temp. in degrees Celsius
    pub m_trackTemperatureChange: i8, // Track temp. change – 0 = up, 1 = down, 2 = no change
    pub m_airTemperature: i8, // Air temp. in degrees Celsius
    pub m_airTemperatureChange: i8, // Air temp. change – 0 = up, 1 = down, 2 = no change
    pub m_rainPercentage: u8, // Rain percentage (0-100)
}

#[repr(C, packed)]
#[allow(non_snake_case)]
#[derive(Clone, Copy, FromBytes, FromZeroes, Unaligned)]
pub struct FastestLap {
    pub vehicleIdx: u8,
    pub lapTime: f32,
}

#[repr(C, packed)]
#[allow(non_snake_case)]
#[derive(Clone, Copy, FromBytes, FromZeroes, Unaligned)]
pub struct Retirement {
    pub vehicleIdx: u8,
}

#[repr(C, packed)]
#[allow(non_snake_case)]
#[derive(Clone, Copy, FromBytes, FromZeroes, Unaligned)]
pub struct TeamMateInPits {
    pub vehicleIdx: u8,
}

#[repr(C, packed)]
#[allow(non_snake_case)]
#[derive(Clone, Copy, FromBytes, FromZeroes, Unaligned)]
pub struct RaceWinner {
    pub vehicleIdx: u8,
}

#[repr(C, packed)]
#[allow(non_snake_case)]
#[derive(Clone, Copy, FromBytes, FromZeroes, Unaligned)]
pub struct Penalty {
    pub penaltyType: u8,
    pub infringementType: u8,
    pub vehicleIdx: u8,
    pub otherVehicleIdx: u8,
    pub time: u8,
    pub lapNum: u8,
    pub placesGained: u8,
}

#[repr(C, packed)]
#[allow(non_snake_case)]
#[derive(Clone, Copy, FromBytes, FromZeroes, Unaligned)]
pub struct SpeedTrap {
    pub vehicleIdx: u8,
    pub speed: f32,
    pub isOverallFastestInSession: u8,
    pub isDriverFastestInSession: u8,
    pub fastestVehicleIdxInSession: u8,
    pub fastestSpeedInSession: f32,
}

#[repr(C, packed)]
#[allow(non_snake_case)]
#[derive(Clone, Copy, FromBytes, FromZeroes, Unaligned)]
pub struct StartLights {
    pub numLights: u8,
}

#[repr(C, packed)]
#[allow(non_snake_case)]
#[derive(Clone, Copy, FromBytes, FromZeroes, Unaligned)]
pub struct DriveThroughPenaltyServed {
    pub vehicleIdx: u8,
}

#[repr(C, packed)]
#[allow(non_snake_case)]
#[derive(Clone, Copy, FromBytes, FromZeroes, Unaligned)]
pub struct StopGoPenaltyServed {
    pub vehicleIdx: u8,
}

#[repr(C, packed)]
#[allow(non_snake_case)]
#[derive(Clone, Copy, FromBytes, FromZeroes, Unaligned)]
pub struct Flashback {
    pub flashbackFrameIdentifier: u32, // Frame identifier flashed back to
    pub flashbackSessionTime: f32,     // Session time flashed back to
}

#[repr(C, packed)]
#[allow(non_snake_case)]
#[derive(Clone, Copy, FromBytes, FromZeroes, Unaligned)]
pub struct Buttons {
    pub buttonStatus: u32,
}

#[repr(C, packed)]
#[allow(non_snake_case)]
#[derive(Clone, Copy, FromBytes, FromZeroes, Unaligned)]
pub struct Overtake {
    pub overtakingVehicleIdx: u8,
    pub beingOvertakenVehicleIdx: u8,
}

#[derive(Debug, PartialEq, Eq)]
pub enum EventCode {
    SessionStarted,
    SessionEnded,
    FastestLap,
    Retirement,
    DRSEnabled,
    DRSDisabled,
    TeamMateInPits,
    ChequeredFlag,
    RaceWinner,
    PenaltyIssued,
    SpeedTrapTriggered,
    StartLights,
    LightsOut,
    DriveThroughServed,
    StopGoServed,
    Flashback,
    ButtonStatus,
    RedFlag,
    Overtake,
}

impl From<&[u8; 4]> for EventCode {
    fn from(value: &[u8; 4]) -> Self {
        match value {
            b"SSTA" => EventCode::SessionStarted,
            b"SEND" => EventCode::SessionEnded,
            b"FTLP" => EventCode::FastestLap,
            b"RTMT" => EventCode::Retirement,
            b"DRSE" => EventCode::DRSEnabled,
            b"DRSD" => EventCode::DRSDisabled,
            b"TMPT" => EventCode::TeamMateInPits,
            b"CHQF" => EventCode::ChequeredFlag,
            b"RCWN" => EventCode::RaceWinner,
            b"PENA" => EventCode::PenaltyIssued,
            b"SPTP" => EventCode::SpeedTrapTriggered,
            b"STLG" => EventCode::StartLights,
            b"LGOT" => EventCode::LightsOut,
            b"DTSV" => EventCode::DriveThroughServed,
            b"SGSV" => EventCode::StopGoServed,
            b"FLBK" => EventCode::Flashback,
            b"BUTN" => EventCode::ButtonStatus,
            b"RFGO" => EventCode::RedFlag,
            b"OVTK" => EventCode::Overtake,
            _ => panic!("Unknown event code {:?}", value),
        }
    }
}

#[repr(C, packed)]
#[allow(non_snake_case)]
#[derive(FromBytes, FromZeroes, Unaligned)]
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
#[allow(non_snake_case)]
#[derive(Debug, Clone, Copy, FromBytes, FromZeroes, Unaligned)]
pub struct ParticipantData {
    pub m_aiControlled: u8, // Whether the vehicle is AI (1) or Human (0) controlled
    pub m_driverId: u8,     // Driver id - see appendix, 255 if network human
    pub m_networkId: u8,    // Network id – unique identifier for network players
    pub m_teamId: u8,       // Team id - see appendix
    pub m_myTeam: u8,       // My team flag – 1 = My Team, 0 = otherwise
    pub m_raceNumber: u8,   // Race number of the car
    pub m_nationality: u8,  // ParticipantNationality // Nationality of the driver
    // #[serde(with = "BigArray")]
    pub m_name: [u8; 48], // Name of participant in UTF-8 format – null terminated, Will be truncated with … (U+2026) if too long
    pub m_yourTelemetry: u8, // The player's UDP setting, 0 = restricted, 1 = public
    pub m_showOnlineNames: u8, // The player's show online names setting, 0 = off, 1 = on
    pub m_platform: u8,   // 1 = Steam, 3 = PlayStation, 4 = Xbox, 6 = Origin, 255 = unknown
}

#[repr(C, packed)]
#[allow(non_snake_case)]
#[derive(Debug, Clone, Copy, FromBytes, FromZeroes, Unaligned)]
pub struct FinalClassificationData {
    pub m_position: u8,               // Finishing position
    pub m_numLaps: u8,                // Number of laps completed
    pub m_gridPosition: u8,           // Grid position of the car
    pub m_points: u8,                 // Number of points scored
    pub m_numPitStops: u8,            // Number of pit stops made
    pub m_resultStatus: u8, // Result status - 0 = invalid, 1 = inactive, 2 = active, 3 = finished, 4 = didnotfinish, 5 = disqualified, 6 = not classified, 7 = retired
    pub m_bestLapTimeInMS: u32, // Best lap time of the session in milliseconds
    pub m_totalRaceTime: f64, // Total race time in seconds without penalties
    pub m_penaltiesTime: u8, // Total penalties accumulated in seconds
    pub m_numPenalties: u8, // Number of penalties applied to this driver
    pub m_numTyreStints: u8, // Number of tyres stints up to maximum
    pub m_tyreStintsActual: [u8; 8], // Actual tyres used by this driver
    pub m_tyreStintsVisual: [u8; 8], // Visual tyres used by this driver
    pub m_tyreStintsEndLaps: [u8; 8], // The lap number stints end on
}

#[repr(C, packed)]
#[allow(non_snake_case)]
#[derive(Debug, Clone, Copy, FromBytes, FromZeroes, Unaligned)]
pub struct LapHistoryData {
    pub m_lapTimeInMS: u32,       // Lap time in milliseconds
    pub m_sector1TimeInMS: u16,   // Sector 1 time in milliseconds
    pub m_sector1TimeMinutes: u8, // Sector 1 whole minute part
    pub m_sector2TimeInMS: u16,   // Sector 2 time in milliseconds
    pub m_sector2TimeMinutes: u8, // Sector 2 whole minute part
    pub m_sector3TimeInMS: u16,   // Sector 3 time in milliseconds
    pub m_sector3TimeMinutes: u8, // Sector 3 whole minute part
    pub m_lapValidBitFlags: u8, // 0x01 bit set - lap valid, 0x02 bit set - sector 1 valid, 0x04 bit set - sector 2 valid, 0x08 bit set - sector 3 valid
}

#[repr(C, packed)]
#[allow(non_snake_case)]
#[derive(Debug, Clone, Copy, FromBytes, FromZeroes, Unaligned)]
pub struct TyreStintHistoryData {
    pub m_endLap: u8,             // Lap the tyre usage ends on (255 of current tyre)
    pub m_tyreActualCompound: u8, // Actual tyres used by this driver
    pub m_tyreVisualCompound: u8, // Visual tyres used by this driver
}

// Cannot use debug in unions
pub enum F123Data<'a> {
    Motion(&'a PacketMotionData),
    Session(&'a PacketSessionData),
    #[allow(dead_code)]
    Event(&'a PacketEventData),
    Participants(&'a PacketParticipantsData),
    FinalClassification(&'a PacketFinalClassificationData),
    SessionHistory(&'a PacketSessionHistoryData),
}

impl<'a> F123Data<'a> {
    // TODO: Try to implement zero-copy deserialization
    pub fn deserialize(packet_id: PacketIds, data: &[u8]) -> Option<F123Data> {
        match packet_id {
            PacketIds::Motion => {
                let packet: Option<&PacketMotionData> = FromBytes::ref_from_prefix(data);
                Some(F123Data::Motion(packet.unwrap()))
            }

            PacketIds::Session => {
                let packet: Option<&PacketSessionData> = FromBytes::ref_from_prefix(data);
                Some(F123Data::Session(packet.unwrap()))
            }

            PacketIds::Participants => {
                let packet: Option<&PacketParticipantsData> = FromBytes::ref_from_prefix(data);
                Some(F123Data::Participants(packet.unwrap()))
            }

            PacketIds::FinalClassification => {
                let packet: Option<&PacketFinalClassificationData> =
                    FromBytes::ref_from_prefix(data);

                Some(F123Data::FinalClassification(packet.unwrap()))
            }

            PacketIds::SessionHistory => {
                let packet: Option<&PacketSessionHistoryData> = FromBytes::ref_from_prefix(data);
                Some(F123Data::SessionHistory(packet.unwrap()))
            }

            PacketIds::Event => {
                let packet: Option<&PacketEventData> = FromBytes::ref_from_prefix(data);
                Some(F123Data::Event(packet.unwrap()))
            }
            _ => None,
        }
    }

    pub fn deserialize_header(data: &[u8]) -> Option<PacketHeader> {
        FromBytes::read_from_prefix(data)
    }
}
