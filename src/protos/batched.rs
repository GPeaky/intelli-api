use super::{ChunkPacketHeader, PacketHeader};
use ntex::util::{Bytes, BytesMut};
use prost::Message;

pub struct ToProtoMessageBatched {}

impl ToProtoMessageBatched {
    #[inline(always)]
    pub fn chunk_packet(packets: Vec<PacketHeader>) -> Option<ChunkPacketHeader> {
        Some(ChunkPacketHeader { packets })
    }

    #[inline(always)]
    pub fn batched_encoded(data: Vec<PacketHeader>) -> Option<Bytes> {
        let data = Self::chunk_packet(data)?;
        // Todo: Check the data.encoded_len() function
        let mut buf = BytesMut::with_capacity(data.encoded_len());

        data.encode_raw(&mut buf);
        Some(buf.freeze())
    }
}