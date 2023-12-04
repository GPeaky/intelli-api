use super::game::*;
use log::error;
use zerocopy::FromBytes;

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

#[derive(Debug, PartialEq, Eq)]
pub enum SessionType {
    P1,
    P2,
    P3,
    ShortP,
    Q1,
    Q2,
    Q3,
    ShortQ,
    Osq,
    R,
    R2,
    R3,
    TimeTrial,
}

#[derive(Debug)]
pub enum Tracks {
    Melbourne,
    PaulRicard,
    Shangai,
    Sakhir,
    Catalunya,
    Monaco,
    Montreal,
    Silverstone,
    Hockenheim,
    Hungaroring,
    Spa,
    Monza,
    Singapore,
    Suzuka,
    AbuDhabi,
    Texas,
    Brazil,
    Austria,
    Sochi,
    Mexico,
    Baku,
    SakhirShort,
    SilverstoneShort,
    TexasShort,
    SuzukaShort,
    Hanoi,
    Zandvoort,
    Imola,
    Portimao,
    Jeddah,
    Miami,
    LasVegas,
    Losail,
}

#[derive(Debug)]
pub enum PenaltyTypes {
    DriveThrough,
    StopGo,
    GridPenalty,
    PenaltyReminder,
    TimePenalty,
    Warning,
    Disqualified,
    RemovedFromFormationLap,
    ParkedTooLongTimer,
    TyreRegulations,
    ThisLapInvalidated,
    ThisAndNextLapInvalidated,
    ThisLapInvalidatedWithoutReason,
    ThisAndNextLapInvalidatedWithoutReason,
    ThisAndPreviousLapInvalidated,
    ThisAndPreviousLapInvalidatedWithoutReason,
    Retired,
    BlackFlagTimer,
}

#[derive(Debug)]
pub enum InfringementType {
    BlockingBySlowDriving,
    BlockingByWrongWayDriving,
    ReversingOffTheStartLine,
    BigCollision,
    SmallCollision,
    CollisionFailedToHandBackPositionSingle,
    CollisionFailedToHandBackPositionMultiple,
    CornerCuttingGainedTime,
    CornerCuttingOvertakeSingle,
    CornerCuttingOvertakeMultiple,
    CrossedPitExitLane,
    IgnoringBlueFlags,
    IgnoringYellowFlags,
    IgnoringDriveThrough,
    TooManyDriveThroughs,
    DriveThroughReminderServeWithinNLaps,
    DriveThroughReminderServeThisLap,
    PitLaneSpeeding,
    ParkedForTooLong,
    IgnoringTyreRegulations,
    TooManyPenalties,
    MultipleWarnings,
    ApproachingDisqualification,
    TyreRegulationsSelectSingle,
    TyreRegulationsSelectMultiple,
    LapInvalidatedCornerCutting,
    LapInvalidatedRunningWide,
    CornerCuttingRanWideGainedTimeMinor,
    CornerCuttingRanWideGainedTimeSignificant,
    CornerCuttingRanWideGainedTimeExtreme,
    LapInvalidatedWallRiding,
    LapInvalidatedFlashbackUsed,
    LapInvalidatedResetToTrack,
    BlockingThePitlane,
    JumpStart,
    SafetyCarToCarCollision,
    SafetyCarIllegalOvertake,
    SafetyCarExceedingAllowedPace,
    VirtualSafetyCarExceedingAllowedPace,
    FormationLapBelowAllowedSpeed,
    FormationLapParking,
    RetiredMechanicalFailure,
    RetiredTerminallyDamaged,
    SafetyCarFallingTooFarBack,
    BlackFlagTimer,
    UnservedStopGoPenalty,
    UnservedDriveThroughPenalty,
    EngineComponentChange,
    GearboxChange,
    ParcFermeChange,
    LeagueGridPenalty,
    RetryPenalty,
    IllegalTimeGain,
    MandatoryPitstop,
    AttributeAssigned,
}

#[derive(Debug)]
pub enum Ruleset {
    PracticeAndQualifying,
    Race,
    TimeTrial,
    TimeAttack,
    CheckpointChallenge,
    Autocross,
    Drift,
    AverageSpeedZone,
    RivalDuel,
}

impl From<u8> for SessionType {
    fn from(value: u8) -> Self {
        match value {
            1 => Self::P1,
            2 => Self::P2,
            3 => Self::P3,
            4 => Self::ShortP,
            5 => Self::Q1,
            6 => Self::Q2,
            7 => Self::Q3,
            8 => Self::ShortQ,
            9 => Self::Osq,
            10 => Self::R,
            11 => Self::R2,
            12 => Self::R3,
            13 => Self::TimeTrial,
            _ => panic!("Unknown session type {}", value),
        }
    }
}

impl From<u8> for PenaltyTypes {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::DriveThrough,
            1 => Self::StopGo,
            2 => Self::GridPenalty,
            3 => Self::PenaltyReminder,
            4 => Self::TimePenalty,
            5 => Self::Warning,
            6 => Self::Disqualified,
            7 => Self::RemovedFromFormationLap,
            8 => Self::ParkedTooLongTimer,
            9 => Self::TyreRegulations,
            10 => Self::ThisLapInvalidated,
            11 => Self::ThisAndNextLapInvalidated,
            12 => Self::ThisLapInvalidatedWithoutReason,
            13 => Self::ThisAndNextLapInvalidatedWithoutReason,
            14 => Self::ThisAndPreviousLapInvalidated,
            15 => Self::ThisAndPreviousLapInvalidatedWithoutReason,
            16 => Self::Retired,
            17 => Self::BlackFlagTimer,
            _ => panic!("Unknown penalty type {}", value),
        }
    }
}

impl From<u8> for InfringementType {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::BlockingBySlowDriving,
            1 => Self::BlockingByWrongWayDriving,
            2 => Self::ReversingOffTheStartLine,
            3 => Self::BigCollision,
            4 => Self::SmallCollision,
            5 => Self::CollisionFailedToHandBackPositionSingle,
            6 => Self::CollisionFailedToHandBackPositionMultiple,
            7 => Self::CornerCuttingGainedTime,
            8 => Self::CornerCuttingOvertakeSingle,
            9 => Self::CornerCuttingOvertakeMultiple,
            10 => Self::CrossedPitExitLane,
            11 => Self::IgnoringBlueFlags,
            12 => Self::IgnoringYellowFlags,
            13 => Self::IgnoringDriveThrough,
            14 => Self::TooManyDriveThroughs,
            15 => Self::DriveThroughReminderServeWithinNLaps,
            16 => Self::DriveThroughReminderServeThisLap,
            17 => Self::PitLaneSpeeding,
            18 => Self::ParkedForTooLong,
            19 => Self::IgnoringTyreRegulations,
            20 => Self::TooManyPenalties,
            21 => Self::MultipleWarnings,
            22 => Self::ApproachingDisqualification,
            23 => Self::TyreRegulationsSelectSingle,
            24 => Self::TyreRegulationsSelectMultiple,
            25 => Self::LapInvalidatedCornerCutting,
            26 => Self::LapInvalidatedRunningWide,
            27 => Self::CornerCuttingRanWideGainedTimeMinor,
            28 => Self::CornerCuttingRanWideGainedTimeSignificant,
            29 => Self::CornerCuttingRanWideGainedTimeExtreme,
            30 => Self::LapInvalidatedWallRiding,
            31 => Self::LapInvalidatedFlashbackUsed,
            32 => Self::LapInvalidatedResetToTrack,
            33 => Self::BlockingThePitlane,
            34 => Self::JumpStart,
            35 => Self::SafetyCarToCarCollision,
            36 => Self::SafetyCarIllegalOvertake,
            37 => Self::SafetyCarExceedingAllowedPace,
            38 => Self::VirtualSafetyCarExceedingAllowedPace,
            39 => Self::FormationLapBelowAllowedSpeed,
            40 => Self::FormationLapParking,
            41 => Self::RetiredMechanicalFailure,
            42 => Self::RetiredTerminallyDamaged,
            43 => Self::SafetyCarFallingTooFarBack,
            44 => Self::BlackFlagTimer,
            45 => Self::UnservedStopGoPenalty,
            46 => Self::UnservedDriveThroughPenalty,
            47 => Self::EngineComponentChange,
            48 => Self::GearboxChange,
            49 => Self::ParcFermeChange,
            50 => Self::LeagueGridPenalty,
            51 => Self::RetryPenalty,
            52 => Self::IllegalTimeGain,
            53 => Self::MandatoryPitstop,
            54 => Self::AttributeAssigned,
            _ => panic!("Unknown infringement type {}", value),
        }
    }
}

impl From<i8> for Tracks {
    fn from(value: i8) -> Self {
        match value {
            0 => Self::Melbourne,
            1 => Self::PaulRicard,
            2 => Self::Shangai,
            3 => Self::Sakhir,
            4 => Self::Catalunya,
            5 => Self::Monaco,
            6 => Self::Montreal,
            7 => Self::Silverstone,
            8 => Self::Hockenheim,
            9 => Self::Hungaroring,
            10 => Self::Spa,
            11 => Self::Monza,
            12 => Self::Singapore,
            13 => Self::Suzuka,
            14 => Self::AbuDhabi,
            15 => Self::Texas,
            16 => Self::Brazil,
            17 => Self::Austria,
            18 => Self::Sochi,
            19 => Self::Mexico,
            20 => Self::Baku,
            21 => Self::SakhirShort,
            22 => Self::SilverstoneShort,
            23 => Self::TexasShort,
            24 => Self::SuzukaShort,
            25 => Self::Hanoi,
            26 => Self::Zandvoort,
            27 => Self::Imola,
            28 => Self::Portimao,
            29 => Self::Jeddah,
            30 => Self::Miami,
            31 => Self::LasVegas,
            32 => Self::Losail,
            _ => panic!("Unknown track id {}", value),
        }
    }
}

impl From<u8> for Ruleset {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::PracticeAndQualifying,
            1 => Self::Race,
            2 => Self::TimeTrial,
            4 => Self::TimeAttack,
            6 => Self::CheckpointChallenge,
            8 => Self::Autocross,
            9 => Self::Drift,
            10 => Self::AverageSpeedZone,
            11 => Self::RivalDuel,
            _ => panic!("Unknown ruleset {}", value),
        }
    }
}

impl From<&[u8; 4]> for EventCode {
    fn from(value: &[u8; 4]) -> Self {
        match value {
            b"SSTA" => Self::SessionStarted,
            b"SEND" => Self::SessionEnded,
            b"FTLP" => Self::FastestLap,
            b"RTMT" => Self::Retirement,
            b"DRSE" => Self::DRSEnabled,
            b"DRSD" => Self::DRSDisabled,
            b"TMPT" => Self::TeamMateInPits,
            b"CHQF" => Self::ChequeredFlag,
            b"RCWN" => Self::RaceWinner,
            b"PENA" => Self::PenaltyIssued,
            b"SPTP" => Self::SpeedTrapTriggered,
            b"STLG" => Self::StartLights,
            b"LGOT" => Self::LightsOut,
            b"DTSV" => Self::DriveThroughServed,
            b"SGSV" => Self::StopGoServed,
            b"FLBK" => Self::Flashback,
            b"BUTN" => Self::ButtonStatus,
            b"RFGO" => Self::RedFlag,
            b"OVTK" => Self::Overtake,
            _ => panic!("Unknown event code {:?}", value),
        }
    }
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

pub enum F123Data<'a> {
    Motion(&'a PacketMotionData),
    Session(&'a PacketSessionData),
    Event(&'a PacketEventData),
    Participants(&'a PacketParticipantsData),
    FinalClassification(&'a PacketFinalClassificationData),
    SessionHistory(&'a PacketSessionHistoryData),
}

impl<'a> F123Data<'a> {
    pub fn deserialize(packet_id: PacketIds, data: &[u8]) -> Option<F123Data> {
        match packet_id {
            PacketIds::Motion => {
                let Some(packet): Option<&PacketMotionData> = FromBytes::ref_from_prefix(data)
                else {
                    error!("Failed to deserialize motion packet");
                    return None;
                };

                Some(F123Data::Motion(packet))
            }

            PacketIds::Session => {
                let Some(packet): Option<&PacketSessionData> = FromBytes::ref_from_prefix(data)
                else {
                    error!("Failed to deserialize session");
                    return None;
                };

                Some(F123Data::Session(packet))
            }

            PacketIds::Participants => {
                let Some(packet): Option<&PacketParticipantsData> =
                    FromBytes::ref_from_prefix(data)
                else {
                    error!("Failed to deserialize participants");
                    return None;
                };

                Some(F123Data::Participants(packet))
            }

            PacketIds::FinalClassification => {
                let Some(packet): Option<&PacketFinalClassificationData> =
                    FromBytes::ref_from_prefix(data)
                else {
                    error!("Failed to deserialize final classification");
                    return None;
                };

                Some(F123Data::FinalClassification(packet))
            }

            PacketIds::SessionHistory => {
                let Some(packet): Option<&PacketSessionHistoryData> =
                    FromBytes::ref_from_prefix(data)
                else {
                    error!("Failed to deserialize session history");
                    return None;
                };

                Some(F123Data::SessionHistory(packet))
            }

            PacketIds::Event => {
                let Some(packet): Option<&PacketEventData> = FromBytes::ref_from_prefix(data)
                else {
                    error!("Failed to deserialize event");
                    return None;
                };

                Some(F123Data::Event(packet))
            }

            _ => None,
        }
    }

    pub fn deserialize_header(data: &[u8]) -> Option<&'a PacketHeader> {
        FromBytes::ref_from_prefix(data)
    }
}
