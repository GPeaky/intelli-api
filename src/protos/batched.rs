use super::{ChunkPacketHeader, PacketHeader};
use log::error;
use ntex::util::{Bytes, BytesMut};
use prost::Message;

pub struct ToProtoMessageBatched {}

impl ToProtoMessageBatched {
    #[inline(always)]
    pub fn to_proto(data: Vec<PacketHeader>) -> Option<ChunkPacketHeader> {
        Some(ChunkPacketHeader { packets: data })
    }

    #[inline(always)]
    pub fn batched_encoded(data: Vec<PacketHeader>) -> Option<Bytes> {
        let data = Self::to_proto(data)?;

        let mut buf = BytesMut::with_capacity(6144);

        if let Err(e) = data.encode(&mut buf) {
            buf.reserve(e.remaining());

            if let Err(e) = data.encode(&mut buf) {
                error!("Failed to encode protobuf message: {}", e);
                return None;
            }
        };

        Some(buf.freeze())
    }
}
