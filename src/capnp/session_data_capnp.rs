use crate::dtos::PacketSessionData;
use capnp::message::{Builder, HeapAllocator};

include!(concat!(env!("OUT_DIR"), "/session_data_capnp.rs"));

#[inline(always)]
pub fn convert(value: Box<PacketSessionData>) -> Builder<HeapAllocator> {
    todo!()
}
