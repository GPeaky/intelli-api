include!(concat!(env!("OUT_DIR"), "/protos.packet_header.rs"));

use crate::protos::packet_header::PacketType;
use ntex::util::{Bytes, BytesMut};
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

    fn convert_and_encode(&self, packet_type: PacketType) -> Option<Bytes>
    where
        Self: Sized,
    {
        let Some(proto_data) = self.to_proto() else {
            return None;
        };

        let proto_data: Vec<u8> = proto_data.encode_to_vec();
        //TODO: Check if this is enough
        let mut buf = BytesMut::with_capacity(8192);

        PacketHeader {
            r#type: packet_type.into(),
            payload: proto_data,
        }
        .encode(&mut buf)
        .unwrap();

        Some(buf.freeze())
    }
}

pub trait ToProtoMessageBatched {
    type ProtoType: Message;
    fn to_proto(&self) -> Option<Self::ProtoType>;
    fn batched_encoded(&self) -> Option<Bytes>;
}

// TODO: Implement this for more specific type instead of Vec<Bytes>
impl ToProtoMessageBatched for Vec<Bytes> {
    type ProtoType = ChunkPacketHeader;

    fn to_proto(&self) -> Option<Self::ProtoType> {
        // TODO: See if this can be done without the clone
        Some(ChunkPacketHeader {
            packets: self.iter().map(|b| b.to_vec()).collect(),
        })
    }

    fn batched_encoded(&self) -> Option<Bytes>
    where
        Self: Sized,
    {
        let Some(data) = self.to_proto() else {
            return None;
        };

        //TODO: Check if this is enough
        let mut buf = BytesMut::with_capacity(8192);
        data.encode(&mut buf).unwrap();

        Some(buf.freeze())
    }
}
