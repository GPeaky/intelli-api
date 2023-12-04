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

        //TODO: Check if this is enough
        let mut buf = BytesMut::with_capacity(8192);
        data.encode(&mut buf).unwrap();

        {
            let mut last_value = MAX_SIZE.load(Ordering::Relaxed);
            let new_value = buf.len();

            while new_value > last_value {
                let res = MAX_SIZE.compare_exchange_weak(
                    last_value,
                    new_value,
                    Ordering::SeqCst,
                    Ordering::Relaxed,
                );

                match res {
                    Ok(_) => break,
                    Err(value) => last_value = value,
                }
            }

            info!("Max size: {}", MAX_SIZE.load(Ordering::Relaxed));
        }

        Some(buf.freeze())
    }
}
