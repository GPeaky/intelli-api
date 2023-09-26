use crate::dtos::PacketFinalClassificationData;
use capnp::message::{Builder, HeapAllocator};

include!(concat!(env!("OUT_DIR"), "/final_classification_capnp.rs"));

#[inline(always)]
pub fn convert(value: Box<PacketFinalClassificationData>) -> Builder<HeapAllocator> {
    todo!()
}
