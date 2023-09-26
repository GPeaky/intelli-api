use crate::dtos::PacketMotionData;
use capnp::serialize;

pub mod car_motion_capnp;
pub mod event_data_capnp;
pub mod final_classification_capnp;
pub mod packet_header_capnp;
pub mod participants_capnp;
pub mod session_data_capnp;
pub mod session_history_capnp;

pub fn serialize_to_packet(data: Box<PacketMotionData>) -> Vec<u8> {
    let mut buffer = Vec::new();
    let message = car_motion_capnp::build_car_motion(data);

    serialize::write_message(&mut buffer, &message).unwrap();

    buffer
}
