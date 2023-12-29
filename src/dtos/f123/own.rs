use super::game::*;
use tracing::error;
use zerocopy::FromBytes;

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct SectorsLaps {
    pub sector1: u16,
    pub sector2: u16,
    pub sector3: u16,
}

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

impl TryFrom<u8> for SessionType {
    type Error = &'static str;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Self::P1),
            2 => Ok(Self::P2),
            3 => Ok(Self::P3),
            4 => Ok(Self::ShortP),
            5 => Ok(Self::Q1),
            6 => Ok(Self::Q2),
            7 => Ok(Self::Q3),
            8 => Ok(Self::ShortQ),
            9 => Ok(Self::Osq),
            10 => Ok(Self::R),
            11 => Ok(Self::R2),
            12 => Ok(Self::R3),
            13 => Ok(Self::TimeTrial),
            _ => Err("Unknown session type"),
        }
    }
}

impl TryFrom<u8> for PenaltyTypes {
    type Error = &'static str;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::DriveThrough),
            1 => Ok(Self::StopGo),
            2 => Ok(Self::GridPenalty),
            3 => Ok(Self::PenaltyReminder),
            4 => Ok(Self::TimePenalty),
            5 => Ok(Self::Warning),
            6 => Ok(Self::Disqualified),
            7 => Ok(Self::RemovedFromFormationLap),
            8 => Ok(Self::ParkedTooLongTimer),
            9 => Ok(Self::TyreRegulations),
            10 => Ok(Self::ThisLapInvalidated),
            11 => Ok(Self::ThisAndNextLapInvalidated),
            12 => Ok(Self::ThisLapInvalidatedWithoutReason),
            13 => Ok(Self::ThisAndNextLapInvalidatedWithoutReason),
            14 => Ok(Self::ThisAndPreviousLapInvalidated),
            15 => Ok(Self::ThisAndPreviousLapInvalidatedWithoutReason),
            16 => Ok(Self::Retired),
            17 => Ok(Self::BlackFlagTimer),
            _ => Err("Unknown penalty type"),
        }
    }
}

impl TryFrom<u8> for InfringementType {
    type Error = &'static str;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::BlockingBySlowDriving),
            1 => Ok(Self::BlockingByWrongWayDriving),
            2 => Ok(Self::ReversingOffTheStartLine),
            3 => Ok(Self::BigCollision),
            4 => Ok(Self::SmallCollision),
            5 => Ok(Self::CollisionFailedToHandBackPositionSingle),
            6 => Ok(Self::CollisionFailedToHandBackPositionMultiple),
            7 => Ok(Self::CornerCuttingGainedTime),
            8 => Ok(Self::CornerCuttingOvertakeSingle),
            9 => Ok(Self::CornerCuttingOvertakeMultiple),
            10 => Ok(Self::CrossedPitExitLane),
            11 => Ok(Self::IgnoringBlueFlags),
            12 => Ok(Self::IgnoringYellowFlags),
            13 => Ok(Self::IgnoringDriveThrough),
            14 => Ok(Self::TooManyDriveThroughs),
            15 => Ok(Self::DriveThroughReminderServeWithinNLaps),
            16 => Ok(Self::DriveThroughReminderServeThisLap),
            17 => Ok(Self::PitLaneSpeeding),
            18 => Ok(Self::ParkedForTooLong),
            19 => Ok(Self::IgnoringTyreRegulations),
            20 => Ok(Self::TooManyPenalties),
            21 => Ok(Self::MultipleWarnings),
            22 => Ok(Self::ApproachingDisqualification),
            23 => Ok(Self::TyreRegulationsSelectSingle),
            24 => Ok(Self::TyreRegulationsSelectMultiple),
            25 => Ok(Self::LapInvalidatedCornerCutting),
            26 => Ok(Self::LapInvalidatedRunningWide),
            27 => Ok(Self::CornerCuttingRanWideGainedTimeMinor),
            28 => Ok(Self::CornerCuttingRanWideGainedTimeSignificant),
            29 => Ok(Self::CornerCuttingRanWideGainedTimeExtreme),
            30 => Ok(Self::LapInvalidatedWallRiding),
            31 => Ok(Self::LapInvalidatedFlashbackUsed),
            32 => Ok(Self::LapInvalidatedResetToTrack),
            33 => Ok(Self::BlockingThePitlane),
            34 => Ok(Self::JumpStart),
            35 => Ok(Self::SafetyCarToCarCollision),
            36 => Ok(Self::SafetyCarIllegalOvertake),
            37 => Ok(Self::SafetyCarExceedingAllowedPace),
            38 => Ok(Self::VirtualSafetyCarExceedingAllowedPace),
            39 => Ok(Self::FormationLapBelowAllowedSpeed),
            40 => Ok(Self::FormationLapParking),
            41 => Ok(Self::RetiredMechanicalFailure),
            42 => Ok(Self::RetiredTerminallyDamaged),
            43 => Ok(Self::SafetyCarFallingTooFarBack),
            44 => Ok(Self::BlackFlagTimer),
            45 => Ok(Self::UnservedStopGoPenalty),
            46 => Ok(Self::UnservedDriveThroughPenalty),
            47 => Ok(Self::EngineComponentChange),
            48 => Ok(Self::GearboxChange),
            49 => Ok(Self::ParcFermeChange),
            50 => Ok(Self::LeagueGridPenalty),
            51 => Ok(Self::RetryPenalty),
            52 => Ok(Self::IllegalTimeGain),
            53 => Ok(Self::MandatoryPitstop),
            54 => Ok(Self::AttributeAssigned),
            _ => Err("Unknown infringement type"),
        }
    }
}

impl TryFrom<i8> for Tracks {
    type Error = &'static str;

    fn try_from(value: i8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Melbourne),
            1 => Ok(Self::PaulRicard),
            2 => Ok(Self::Shangai),
            3 => Ok(Self::Sakhir),
            4 => Ok(Self::Catalunya),
            5 => Ok(Self::Monaco),
            6 => Ok(Self::Montreal),
            7 => Ok(Self::Silverstone),
            8 => Ok(Self::Hockenheim),
            9 => Ok(Self::Hungaroring),
            10 => Ok(Self::Spa),
            11 => Ok(Self::Monza),
            12 => Ok(Self::Singapore),
            13 => Ok(Self::Suzuka),
            14 => Ok(Self::AbuDhabi),
            15 => Ok(Self::Texas),
            16 => Ok(Self::Brazil),
            17 => Ok(Self::Austria),
            18 => Ok(Self::Sochi),
            19 => Ok(Self::Mexico),
            20 => Ok(Self::Baku),
            21 => Ok(Self::SakhirShort),
            22 => Ok(Self::SilverstoneShort),
            23 => Ok(Self::TexasShort),
            24 => Ok(Self::SuzukaShort),
            25 => Ok(Self::Hanoi),
            26 => Ok(Self::Zandvoort),
            27 => Ok(Self::Imola),
            28 => Ok(Self::Portimao),
            29 => Ok(Self::Jeddah),
            30 => Ok(Self::Miami),
            31 => Ok(Self::LasVegas),
            32 => Ok(Self::Losail),
            _ => Err("Unknown track id"),
        }
    }
}

impl TryFrom<u8> for Ruleset {
    type Error = &'static str;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::PracticeAndQualifying),
            1 => Ok(Self::Race),
            2 => Ok(Self::TimeTrial),
            4 => Ok(Self::TimeAttack),
            6 => Ok(Self::CheckpointChallenge),
            8 => Ok(Self::Autocross),
            9 => Ok(Self::Drift),
            10 => Ok(Self::AverageSpeedZone),
            11 => Ok(Self::RivalDuel),
            _ => Err("Unknown ruleset id"),
        }
    }
}

impl TryFrom<&[u8; 4]> for EventCode {
    type Error = &'static str;

    fn try_from(value: &[u8; 4]) -> Result<Self, Self::Error> {
        match value {
            b"SSTA" => Ok(Self::SessionStarted),
            b"SEND" => Ok(Self::SessionEnded),
            b"FTLP" => Ok(Self::FastestLap),
            b"RTMT" => Ok(Self::Retirement),
            b"DRSE" => Ok(Self::DRSEnabled),
            b"DRSD" => Ok(Self::DRSDisabled),
            b"TMPT" => Ok(Self::TeamMateInPits),
            b"CHQF" => Ok(Self::ChequeredFlag),
            b"RCWN" => Ok(Self::RaceWinner),
            b"PENA" => Ok(Self::PenaltyIssued),
            b"SPTP" => Ok(Self::SpeedTrapTriggered),
            b"STLG" => Ok(Self::StartLights),
            b"LGOT" => Ok(Self::LightsOut),
            b"DTSV" => Ok(Self::DriveThroughServed),
            b"SGSV" => Ok(Self::StopGoServed),
            b"FLBK" => Ok(Self::Flashback),
            b"BUTN" => Ok(Self::ButtonStatus),
            b"RFGO" => Ok(Self::RedFlag),
            b"OVTK" => Ok(Self::Overtake),
            _ => Err("Unknown event code"),
        }
    }
}

impl TryFrom<u8> for PacketIds {
    type Error = &'static str;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(PacketIds::Motion),
            1 => Ok(PacketIds::Session),
            2 => Ok(PacketIds::LapData),
            3 => Ok(PacketIds::Event),
            4 => Ok(PacketIds::Participants),
            5 => Ok(PacketIds::CarSetups),
            6 => Ok(PacketIds::CarTelemetry),
            7 => Ok(PacketIds::CarStatus),
            8 => Ok(PacketIds::FinalClassification),
            9 => Ok(PacketIds::LobbyInfo),
            10 => Ok(PacketIds::CarDamage),
            11 => Ok(PacketIds::SessionHistory),
            12 => Ok(PacketIds::TyreSets),
            13 => Ok(PacketIds::MotionEx),
            _ => Err("Unknown packet id"),
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

    pub fn deserialize_header(data: &'a [u8]) -> Option<&'a PacketHeader> {
        FromBytes::ref_from_prefix(data)
    }
}
