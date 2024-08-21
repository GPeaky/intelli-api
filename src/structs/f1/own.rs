use crate::{
    error::{AppResult, F1ServiceError},
    utils::cast,
};

use super::game::*;

pub enum PacketExtraData {
    EventCode([u8; 4]),
    CarNumber(u8),
}

pub enum F1Data<'a> {
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

impl<'a> F1Data<'a> {
    pub fn try_cast(data: &[u8]) -> AppResult<(&PacketHeader, F1Data)> {
        let header = cast::<PacketHeader>(data)?;
        let packet_id = PacketIds::try_from(header.packet_id).unwrap();

        let packet = match packet_id {
            PacketIds::Event => cast::<PacketEventData>(data).map(F1Data::Event),
            PacketIds::Motion => cast::<PacketMotionData>(data).map(F1Data::Motion),
            PacketIds::Session => cast::<PacketSessionData>(data).map(F1Data::Session),
            PacketIds::CarDamage => cast::<PacketCarDamageData>(data).map(F1Data::CarDamage),
            PacketIds::CarStatus => cast::<PacketCarStatusData>(data).map(F1Data::CarStatus),
            PacketIds::CarTelemetry => {
                cast::<PacketCarTelemetryData>(data).map(F1Data::CarTelemetry)
            }
            PacketIds::Participants => {
                cast::<PacketParticipantsData>(data).map(F1Data::Participants)
            }
            PacketIds::SessionHistory => {
                cast::<PacketSessionHistoryData>(data).map(F1Data::SessionHistory)
            }
            PacketIds::FinalClassification => {
                cast::<PacketFinalClassificationData>(data).map(F1Data::FinalClassification)
            }

            _ => Err(F1ServiceError::InvalidPacketType)?,
        }?;

        Ok((header, packet))
    }
}
