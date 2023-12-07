include!(concat!(env!("OUT_DIR"), "/protos.packet_header.rs"));

pub(crate) mod batched;
pub(crate) mod car_motion_data;
pub(crate) mod event_data;
pub(crate) mod final_classification;
pub(crate) mod participants;
pub(crate) mod session_data;
pub(crate) mod session_history;

use crate::protos::packet_header::PacketType;
use prost::Message;

pub trait ToProtoMessage {
    type ProtoType: Message;
    fn to_proto(&self) -> Option<Self::ProtoType>;

    fn convert(&self, packet_type: PacketType) -> Option<PacketHeader>
    where
        Self: Sized,
    {
        let Some(proto_data) = self.to_proto() else {
            return None;
        };

        let proto_data: Vec<u8> = proto_data.encode_to_vec();

        Some(PacketHeader {
            r#type: packet_type.into(),
            payload: proto_data,
        })
    }
}
