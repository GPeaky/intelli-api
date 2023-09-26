use crate::dtos::PacketParticipantsData;
use capnp::message::{Builder, HeapAllocator};

include!(concat!(env!("OUT_DIR"), "/participants_capnp.rs"));

#[inline(always)]
pub fn convert(value: Box<PacketParticipantsData>) -> Builder<HeapAllocator> {
    todo!()
}
