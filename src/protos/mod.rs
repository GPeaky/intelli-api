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
    fn to_proto(&self) -> Self::ProtoType;

    // TODO: Try to remove packet_type from here
    fn convert_and_encode(&self, packet_type: PacketType) -> Vec<u8>
    where
        Self: Sized,
    {
        let proto_data = self.to_proto();
        let proto_data = proto_data.encode_to_vec();

        PacketHeader {
            r#type: packet_type.into(),
            payload: proto_data,
        }
        .encode_to_vec()
    }
}
