use std::sync::Arc;

use ntex::util::Bytes;
use parking_lot::RwLock;
use tokio::sync::{broadcast::Receiver, oneshot};
use tracing::error;
use zerocopy::{FromBytes, Immutable, KnownLayout};

use crate::{
    error::{AppResult, CommonError, F1ServiceError},
    services::PacketCaching,
};

use super::game::*;

pub struct F1ServiceData {
    pub cache: Arc<RwLock<PacketCaching>>,
    pub channel: Arc<Receiver<Bytes>>,
    pub shutdown_tx: oneshot::Sender<()>,
}

pub enum OptionalMessage {
    Code([u8; 4]),
    Number(u8),
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
    pub fn try_deserialize(packet_id: PacketIds, data: &[u8]) -> AppResult<F1Data> {
        match packet_id {
            PacketIds::Motion => {
                Self::try_deserialize_packet::<PacketMotionData>(data).map(F1Data::Motion)
            }

            PacketIds::Session => {
                Self::try_deserialize_packet::<PacketSessionData>(data).map(F1Data::Session)
            }

            PacketIds::Participants => Self::try_deserialize_packet::<PacketParticipantsData>(data)
                .map(F1Data::Participants),

            PacketIds::FinalClassification => {
                Self::try_deserialize_packet::<PacketFinalClassificationData>(data)
                    .map(F1Data::FinalClassification)
            }

            PacketIds::SessionHistory => {
                Self::try_deserialize_packet::<PacketSessionHistoryData>(data)
                    .map(F1Data::SessionHistory)
            }

            PacketIds::Event => {
                Self::try_deserialize_packet::<PacketEventData>(data).map(F1Data::Event)
            }

            PacketIds::CarDamage => {
                Self::try_deserialize_packet::<PacketCarDamageData>(data).map(F1Data::CarDamage)
            }

            PacketIds::CarStatus => {
                Self::try_deserialize_packet::<PacketCarStatusData>(data).map(F1Data::CarStatus)
            }

            PacketIds::CarTelemetry => Self::try_deserialize_packet::<PacketCarTelemetryData>(data)
                .map(F1Data::CarTelemetry),

            _ => Err(F1ServiceError::InvalidPacketType)?,
        }
    }

    pub fn try_deserialize_header(data: &'a [u8]) -> AppResult<&'a PacketHeader> {
        Self::try_deserialize_packet::<PacketHeader>(data)
    }

    #[inline(always)]
    fn try_deserialize_packet<T: FromBytes + KnownLayout + Immutable>(
        bytes: &[u8],
    ) -> AppResult<&T> {
        match T::ref_from_prefix(bytes) {
            Ok(packet) => Ok(packet.0),
            Err(err) => {
                error!("Error Deserializing Packet: {}", err);
                Err(CommonError::InternalServerError)?
            }
        }
    }
}
