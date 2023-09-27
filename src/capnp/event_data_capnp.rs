use crate::dtos::PacketEventData;
use capnp::message::{Builder, HeapAllocator};

include!(concat!(env!("OUT_DIR"), "/event_data_capnp.rs"));

#[inline(always)]
pub fn convert(value: PacketEventData) -> Builder<HeapAllocator> {
    todo!()
}
