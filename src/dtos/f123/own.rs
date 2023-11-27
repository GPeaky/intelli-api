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

#[repr(C, u8)]
#[derive(Debug)]
pub enum GameModes {
    EventMode = 0,
    GrandPrix = 3,
    GrandPrix23 = 4,
    TimeTrial = 5,
    Splitscreen = 6,
    OnlineCustom = 7,
    OnlineLeague = 8,
    CareerInvitational = 11,
    Championship = 13,
    OnlineChampionship = 14,
    OnlineWeeklyEvent = 15,
    StoryMode = 17,
    Career22 = 19,
    Career22Online = 20,
    Career23 = 21,
    Career23Online = 22,
    Benchmark = 127,
}

#[repr(C, i8)]
#[derive(Debug)]
pub enum Tracks {
    Melbourne = 0,
    PaulRicard = 1,
    Shangai = 2,
    Sakhir = 3,
    Catalunya = 4,
    Monaco = 5,
    Montreal = 6,
    Silverstone = 7,
    Hockenheim = 8,
    Hungaroring = 9,
    Spa = 10,
    Monza = 11,
    Singapore = 12,
    Suzuka = 13,
    AbuDhabi = 14,
    Texas = 15,
    Brazil = 16,
    Austria = 17,
    Sochi = 18,
    Mexico = 19,
    Baku = 20,
    SakhirShort = 21,
    SilverstoneShort = 22,
    TexasShort = 23,
    SuzukaShort = 24,
    Hanoi = 25,
    Zandvoort = 26,
    Imola = 27,
    Portimao = 28,
    Jeddah = 29,
    Miami = 30,
    LasVegas = 31,
    Losail = 32,
}

#[repr(C, u8)]
#[derive(Debug)]
pub enum PenaltyTypes {
    DriveThrough = 0,
    StopGo = 1,
    GridPenalty = 2,
    PenaltyReminder = 3,
    TimePenalty = 4,
    Warning = 5,
    Disqualified = 6,
    RemovedFromFormationLap = 7,
    ParkedTooLongTimer = 8,
    TyreRegulations = 9,
    ThisLapInvalidated = 10,
    ThisAndNextLapInvalidated = 11,
    ThisLapInvalidatedWithoutReason = 12,
    ThisAndNextLapInvalidatedWithoutReason = 13,
    ThisAndPreviousLapInvalidated = 14,
    ThisAndPreviousLapInvalidatedWithoutReason = 15,
    Retired = 16,
    BlackFlagTimer = 17,
}

#[repr(C, u8)]
#[derive(Debug)]
pub enum InfringementType {
    BlockingBySlowDriving = 0,
    BlockingByWrongWayDriving = 1,
    ReversingOffTheStartLine = 2,
    BigCollision = 3,
    SmallCollision = 4,
    CollisionFailedToHandBackPositionSingle = 5,
    CollisionFailedToHandBackPositionMultiple = 6,
    CornerCuttingGainedTime = 7,
    CornerCuttingOvertakeSingle = 8,
    CornerCuttingOvertakeMultiple = 9,
    CrossedPitExitLane = 10,
    IgnoringBlueFlags = 11,
    IgnoringYellowFlags = 12,
    IgnoringDriveThrough = 13,
    TooManyDriveThroughs = 14,
    DriveThroughReminderServeWithinNLaps = 15,
    DriveThroughReminderServeThisLap = 16,
    PitLaneSpeeding = 17,
    ParkedForTooLong = 18,
    IgnoringTyreRegulations = 19,
    TooManyPenalties = 20,
    MultipleWarnings = 21,
    ApproachingDisqualification = 22,
    TyreRegulationsSelectSingle = 23,
    TyreRegulationsSelectMultiple = 24,
    LapInvalidatedCornerCutting = 25,
    LapInvalidatedRunningWide = 26,
    CornerCuttingRanWideGainedTimeMinor = 27,
    CornerCuttingRanWideGainedTimeSignificant = 28,
    CornerCuttingRanWideGainedTimeExtreme = 29,
    LapInvalidatedWallRiding = 30,
    LapInvalidatedFlashbackUsed = 31,
    LapInvalidatedResetToTrack = 32,
    BlockingThePitlane = 33,
    JumpStart = 34,
    SafetyCarToCarCollision = 35,
    SafetyCarIllegalOvertake = 36,
    SafetyCarExceedingAllowedPace = 37,
    VirtualSafetyCarExceedingAllowedPace = 38,
    FormationLapBelowAllowedSpeed = 39,
    FormationLapParking = 40,
    RetiredMechanicalFailure = 41,
    RetiredTerminallyDamaged = 42,
    SafetyCarFallingTooFarBack = 43,
    BlackFlagTimer = 44,
    UnservedStopGoPenalty = 45,
    UnservedDriveThroughPenalty = 46,
    EngineComponentChange = 47,
    GearboxChange = 48,
    ParcFermeChange = 49,
    LeagueGridPenalty = 50,
    RetryPenalty = 51,
    IllegalTimeGain = 52,
    MandatoryPitstop = 53,
    AttributeAssigned = 54,
}

#[repr(C, u8)]
#[derive(Debug)]
pub enum TeamsIds {
    Mercedes = 0,
    Ferrari = 1,
    RedBullRacing = 2,
    Williams = 3,
    AstonMartin = 4,
    Alpine = 5,
    AlphaTauri = 6,
    Haas = 7,
    McLaren = 8,
    AlfaRomeo = 9,
}

#[repr(C, u8)]
#[derive(Debug)]
pub enum NationalityIds {
    American = 1,
    Argentinean = 2,
    Australian = 3,
    Austrian = 4,
    Azerbaijani = 5,
    Bahraini = 6,
    Belgian = 7,
    Bolivian = 8,
    Brazilian = 9,
    British = 10,
    Bulgarian = 11,
    Cameroonian = 12,
    Canadian = 13,
    Chilean = 14,
    Chinese = 15,
    Colombian = 16,
    CostaRican = 17,
    Croatian = 18,
    Cypriot = 19,
    Czech = 20,
    Danish = 21,
    Dutch = 22,
    Ecuadorian = 23,
    English = 24,
    Emirian = 25,
    Estonian = 26,
    Finnish = 27,
    French = 28,
    German = 29,
    Ghanaian = 30,
    Greek = 31,
    Guatemalan = 32,
    Honduran = 33,
    HongKonger = 34,
    Hungarian = 35,
    Icelander = 36,
    Indian = 37,
    Indonesian = 38,
    Irish = 39,
    Israeli = 40,
    Italian = 41,
    Jamaican = 42,
    Japanese = 43,
    Jordanian = 44,
    Kuwaiti = 45,
    Latvian = 46,
    Lebanese = 47,
    Lithuanian = 48,
    Luxembourger = 49,
    Malaysian = 50,
    Maltese = 51,
    Mexican = 52,
    Monegasque = 53,
    NewZealander = 54,
    Nicaraguan = 55,
    NorthernIrish = 56,
    Norwegian = 57,
    Omani = 58,
    Pakistani = 59,
    Panamanian = 60,
    Paraguayan = 61,
    Peruvian = 62,
    Polish = 63,
    Portuguese = 64,
    Qatari = 65,
    Romanian = 66,
    Russian = 67,
    Salvadoran = 68,
    Saudi = 69,
    Scottish = 70,
    Serbian = 71,
    Singaporean = 72,
    Slovakian = 73,
    Slovenian = 74,
    SouthKorean = 75,
    SouthAfrican = 76,
    Spanish = 77,
    Swedish = 78,
    Swiss = 79,
    Thai = 80,
    Turkish = 81,
    Uruguayan = 82,
    Ukrainian = 83,
    Venezuelan = 84,
    Barbadian = 85,
    Welsh = 86,
    Vietnamese = 87,
}

#[repr(C, u8)]
#[derive(Debug)]
pub enum Ruleset {
    PracticeAndQualifying = 0,
    Race = 1,
    TimeTrial = 2,
    TimeAttack = 4,
    CheckpointChallenge = 6,
    Autocross = 8,
    Drift = 9,
    AverageSpeedZone = 10,
    RivalDuel = 11,
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

    pub fn deserialize_header(data: &[u8]) -> Option<PacketHeader> {
        FromBytes::read_from_prefix(data)
    }
}
