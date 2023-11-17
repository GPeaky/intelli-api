use super::game::*;
use tracing::error;
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

// TODO: Handle Errors
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
