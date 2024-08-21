use crate::{
    error::{AppResult, F1ServiceError},
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
