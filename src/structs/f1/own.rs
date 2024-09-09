use crate::{
    error::{AppResult, F1ServiceError},
    protos::packet_header::PacketType,
    utils::cast,
};

use super::game::*;

pub enum PacketExtraData {
    EventCode([u8; 4]),
    CarNumber(u8),
}

pub enum F1PacketData<'a> {
    Motion(&'a PacketMotionData),
    Session(&'a PacketSessionData),
    Event(&'a PacketEventData),
    Participants(&'a PacketParticipantsData),
    FinalClassification(&'a PacketFinalClassificationData),
    SessionHistory(&'a PacketSessionHistoryData),
    CarDamage(&'a PacketCarDamageData),
    CarStatus(&'a PacketCarStatusData),
    CarTelemetry(&'a PacketCarTelemetryData),
}

impl<'a> F1PacketData<'a> {
    pub fn parse_and_identify(raw_data: &[u8]) -> AppResult<(&PacketHeader, F1PacketData)> {
        let header = cast::<PacketHeader>(raw_data)?;
        let packet_id = PacketIds::try_from(header.packet_id).unwrap();

        let packet = match packet_id {
            PacketIds::Event => cast::<PacketEventData>(raw_data).map(F1PacketData::Event),
            PacketIds::Motion => cast::<PacketMotionData>(raw_data).map(F1PacketData::Motion),
            PacketIds::Session => cast::<PacketSessionData>(raw_data).map(F1PacketData::Session),
            PacketIds::CarDamage => {
                cast::<PacketCarDamageData>(raw_data).map(F1PacketData::CarDamage)
            }
            PacketIds::CarStatus => {
                cast::<PacketCarStatusData>(raw_data).map(F1PacketData::CarStatus)
            }
            PacketIds::CarTelemetry => {
                cast::<PacketCarTelemetryData>(raw_data).map(F1PacketData::CarTelemetry)
            }
            PacketIds::Participants => {
                cast::<PacketParticipantsData>(raw_data).map(F1PacketData::Participants)
            }
            PacketIds::SessionHistory => {
                cast::<PacketSessionHistoryData>(raw_data).map(F1PacketData::SessionHistory)
            }
            PacketIds::FinalClassification => cast::<PacketFinalClassificationData>(raw_data)
                .map(F1PacketData::FinalClassification),

            _ => Err(F1ServiceError::InvalidPacketType)?,
        }?;

        Ok((header, packet))
    }
}

// TODO: Search a better way to avoid sending packet type into the function call

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum F1PacketTypeTag {
    Motion,
    Participants,
    Session,
    Event,
    SessionHistory,
    FinalClassification,
}

pub trait F1PacketTypeTagged {
    const PACKET_TYPE_TAG: F1PacketTypeTag;
}

macro_rules! impl_f1_packet_type_tagged {
    ($($t:ty => $tag:expr),* $(,)?) => {
        $(
            impl F1PacketTypeTagged for $t {
                const PACKET_TYPE_TAG: F1PacketTypeTag = $tag;
            }
        )*
    };
}

impl_f1_packet_type_tagged! {
    PacketMotionData => F1PacketTypeTag::Motion,
    PacketParticipantsData => F1PacketTypeTag::Participants,
    PacketSessionData => F1PacketTypeTag::Session,
    PacketEventData => F1PacketTypeTag::Event,
    PacketSessionHistoryData => F1PacketTypeTag::SessionHistory,
    PacketFinalClassificationData => F1PacketTypeTag::FinalClassification,
}

#[inline(always)]
pub const fn get_f1_packet_type<T: F1PacketTypeTagged>() -> Option<PacketType> {
    match T::PACKET_TYPE_TAG {
        F1PacketTypeTag::Motion => Some(PacketType::CarMotion),
        F1PacketTypeTag::Participants => Some(PacketType::Participants),
        F1PacketTypeTag::Session => Some(PacketType::SessionData),
        F1PacketTypeTag::Event => Some(PacketType::EventData),
        F1PacketTypeTag::SessionHistory => Some(PacketType::SessionHistoryData),
        F1PacketTypeTag::FinalClassification => Some(PacketType::FinalClassificationData),
    }
}
