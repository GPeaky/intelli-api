use super::{ChunkPacketHeader, PacketHeader};
use ntex::util::{Bytes, BytesMut};
use prost::Message;

pub struct ToProtoMessageBatched {}

impl ToProtoMessageBatched {
    pub fn new() -> Self {
        Self {}
    }

    #[inline(always)]
    pub fn to_proto(data: Vec<PacketHeader>) -> Option<ChunkPacketHeader> {
        Some(ChunkPacketHeader { packets: data })
    }

    #[inline(always)]
    pub fn batched_encoded(data: Vec<PacketHeader>) -> Option<Bytes> {
        let data = Self::to_proto(data)?;

        //TODO: Check if this is enough
        let mut buf = BytesMut::with_capacity(8192);
        data.encode(&mut buf).unwrap();

        Some(buf.freeze())
    }
}
