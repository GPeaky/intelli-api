use capnp::message::{Builder, HeapAllocator};
use packet_header::PacketType;

include!(concat!(env!("OUT_DIR"), "/packet_header_capnp.rs"));

#[inline(always)]
pub fn convert(packet_type: PacketType, payload: Vec<u8>) -> Builder<HeapAllocator> {
    let mut message = Builder::new_default();
    let mut packet_header = message.init_root::<packet_header::Builder>();

    packet_header.set_packet_type(packet_type);
    packet_header.set_payload(payload.as_slice());

    message
}
