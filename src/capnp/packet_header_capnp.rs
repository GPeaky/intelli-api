use super::PacketId;
use capnp::message::{Builder, HeapAllocator};

include!(concat!(env!("OUT_DIR"), "/packet_header_capnp.rs"));

#[inline(always)]
pub fn convert(packet_type: PacketId, payload: Vec<u8>) -> Builder<HeapAllocator> {
    todo!()
}
