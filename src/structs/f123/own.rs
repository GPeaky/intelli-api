use std::sync::Arc;

use ntex::util::Bytes;
use parking_lot::RwLock;
use tokio::{sync::broadcast::Receiver, task::JoinHandle};
use tracing::error;
use zerocopy::{FromBytes, KnownLayout, NoCell};

use crate::{error::AppResult, services::PacketCaching};

use super::game::*;

pub struct F123ServiceData {
    pub cache: Arc<RwLock<PacketCaching>>,
    pub channel: Arc<Receiver<Bytes>>,
    pub handler: JoinHandle<AppResult<()>>,
}

pub enum OptionalMessage {
    Code([u8; 4]),
    Number(u8),
}

pub enum F123Data<'a> {
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

impl<'a> F123Data<'a> {
    pub fn try_deserialize(packet_id: PacketIds, data: &[u8]) -> Option<F123Data> {
        match packet_id {
            PacketIds::Motion => {
                Self::try_deserialize_packet::<PacketMotionData>(data).map(F123Data::Motion)
            }

            PacketIds::Session => {
                Self::try_deserialize_packet::<PacketSessionData>(data).map(F123Data::Session)
            }

            PacketIds::Participants => Self::try_deserialize_packet::<PacketParticipantsData>(data)
                .map(F123Data::Participants),

            PacketIds::FinalClassification => {
                Self::try_deserialize_packet::<PacketFinalClassificationData>(data)
                    .map(F123Data::FinalClassification)
            }

            PacketIds::SessionHistory => {
                Self::try_deserialize_packet::<PacketSessionHistoryData>(data)
                    .map(F123Data::SessionHistory)
            }

            PacketIds::Event => {
                Self::try_deserialize_packet::<PacketEventData>(data).map(F123Data::Event)
            }

            PacketIds::CarDamage => {
                Self::try_deserialize_packet::<PacketCarDamageData>(data).map(F123Data::CarDamage)
            }

            PacketIds::CarStatus => {
                Self::try_deserialize_packet::<PacketCarStatusData>(data).map(F123Data::CarStatus)
            }

            PacketIds::CarTelemetry => Self::try_deserialize_packet::<PacketCarTelemetryData>(data)
                .map(F123Data::CarTelemetry),

            _ => None,
        }
    }

    pub fn try_deserialize_header(data: &'a [u8]) -> Option<&'a PacketHeader> {
        Self::try_deserialize_packet::<PacketHeader>(data)
    }

    #[inline(always)]
    fn try_deserialize_packet<T: FromBytes + NoCell + KnownLayout>(bytes: &[u8]) -> Option<&T> {
        match T::ref_from_prefix(bytes) {
            Some(packet) => Some(packet),
            None => {
                error!("Failed to deserialize packet");
                None
            }
        }
    }
}
