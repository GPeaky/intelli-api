use crate::dtos::F123Data;
use capnp::serialize;
use packet_header_capnp::packet_header::PacketType;

pub mod car_motion_capnp;
pub mod event_data_capnp;
pub mod final_classification_capnp;
pub mod packet_header_capnp;
pub mod participants_capnp;
pub mod session_data_capnp;
pub mod session_history_capnp;

#[inline(always)]
pub fn serialize_to_packet(data: F123Data) -> Vec<u8> {
    let mut payload = Vec::new();

    let packet_type = match data {
        F123Data::Motion(motion) => {
            let message = car_motion_capnp::convert(motion);
            serialize::write_message(&mut payload, &message).unwrap();
            PacketType::CarMotion
        }

        F123Data::SessionHistory(history_data) => {
            let message = session_history_capnp::convert(history_data);
            serialize::write_message(&mut payload, &message).unwrap();
            PacketType::SessionHistoryData
        }

        F123Data::Session(session) => {
            let message = session_data_capnp::convert(session);
            serialize::write_message(&mut payload, &message).unwrap();
            PacketType::SessionData
        }

        F123Data::Participants(participants) => {
            let message = participants_capnp::convert(participants);
            serialize::write_message(&mut payload, &message).unwrap();
            PacketType::Participants
        }

        F123Data::Event(event) => {
            let message = event_data_capnp::convert(event);
            serialize::write_message(&mut payload, &message).unwrap();
            PacketType::EventData
        }

        F123Data::FinalClassification(classification) => {
            let message = final_classification_capnp::convert(classification);
            serialize::write_message(&mut payload, &message).unwrap();
            PacketType::FinalClassificationData
        }
    };

    let mut buffer = Vec::new();
    let header = packet_header_capnp::convert(packet_type, payload);
    serialize::write_message(&mut buffer, &header).unwrap();

    buffer
}
