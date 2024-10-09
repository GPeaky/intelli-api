use error::{AppResult, F1ServiceError};

use crate::utils::{cast, header_cast};

use super::game::*;

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

impl F1PacketData<'_> {
    pub fn parse_and_identify(data: &[u8]) -> AppResult<(&PacketHeader, F1PacketData)> {
        let header = header_cast(data)?;
        let packet_id = PacketIds::try_from(header.packet_id).unwrap();

        let packet = match packet_id {
            PacketIds::Event => cast::<PacketEventData>(data).map(F1PacketData::Event),
            PacketIds::Motion => cast::<PacketMotionData>(data).map(F1PacketData::Motion),
            PacketIds::Session => cast::<PacketSessionData>(data).map(F1PacketData::Session),
            PacketIds::CarDamage => cast::<PacketCarDamageData>(data).map(F1PacketData::CarDamage),
            PacketIds::CarStatus => cast::<PacketCarStatusData>(data).map(F1PacketData::CarStatus),
            PacketIds::CarTelemetry => {
                cast::<PacketCarTelemetryData>(data).map(F1PacketData::CarTelemetry)
            }
            PacketIds::Participants => {
                cast::<PacketParticipantsData>(data).map(F1PacketData::Participants)
            }
            PacketIds::SessionHistory => {
                cast::<PacketSessionHistoryData>(data).map(F1PacketData::SessionHistory)
            }
            PacketIds::FinalClassification => {
                cast::<PacketFinalClassificationData>(data).map(F1PacketData::FinalClassification)
            }

            _ => Err(F1ServiceError::InvalidPacketType)?,
        }?;

        Ok((header, packet))
    }
}
