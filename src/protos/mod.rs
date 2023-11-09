include!(concat!(env!("OUT_DIR"), "/protos.packet_header.rs"));

use crate::protos::packet_header::PacketType;
use prost::Message;

pub(crate) mod car_motion_data;
pub(crate) mod event_data;
pub(crate) mod final_classification;
pub(crate) mod participants;
pub(crate) mod session_data;
pub(crate) mod session_history;

pub trait ToProtoMessage {
    type ProtoType: Message;
    fn to_proto(&self) -> Option<Self::ProtoType>;

    // TODO: Try to remove packet_type from here
    fn convert_and_encode(&self, packet_type: PacketType) -> Option<Vec<u8>>
    where
        Self: Sized,
    {
        let Some(proto_data) = self.to_proto() else {
            return None;
        };

        let mut buf = Vec::with_capacity(2048);

        // Encode payload
        proto_data.encode(&mut buf).unwrap();

        // Encode header
        PacketHeader {
            r#type: packet_type.into(),
            payload: buf.clone(),
        }
        .encode(&mut buf)
        .unwrap();

        Some(buf)
    }
}
