use std::sync::atomic::{AtomicUsize, Ordering};

use super::{ChunkPacketHeader, PacketHeader};
use log::info;
use ntex::util::{Bytes, BytesMut};
use prost::Message;

static MAX_SIZE: AtomicUsize = AtomicUsize::new(0);

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
        data.encode(&mut buf).unwrap();

        Some(buf.freeze())
    }
}
