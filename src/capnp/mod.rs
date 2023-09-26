use crate::dtos::F123Data;
use capnp::serialize;

pub mod car_motion_capnp;
pub mod event_data_capnp;
pub mod final_classification_capnp;
pub mod packet_header_capnp;
pub mod participants_capnp;
pub mod session_data_capnp;
pub mod session_history_capnp;

#[repr(u8)]
pub(super) enum PacketId {
    Motion,
    EventData,
    FinalClassification,
    Participants,
    Session,
    SessionHistory,
}

#[inline(always)]
pub fn serialize_to_packet(data: F123Data) -> Vec<u8> {
    let mut payload = Vec::new();

    let packet_type = match data {
        F123Data::Motion(motion) => {
            let message = car_motion_capnp::convert(motion);
            serialize::write_message(&mut payload, &message).unwrap();
            PacketId::Motion
        }

        F123Data::SessionHistory(history_data) => {
            let message = session_history_capnp::convert(history_data);
            serialize::write_message(&mut payload, &message).unwrap();
            PacketId::SessionHistory
        }

        F123Data::Session(session) => {
            let message = session_data_capnp::convert(session);
            serialize::write_message(&mut payload, &message).unwrap();
            PacketId::Session
        }

        F123Data::Participants(participants) => {
            let message = participants_capnp::convert(participants);
            serialize::write_message(&mut payload, &message).unwrap();
            PacketId::Participants
        }

        F123Data::Event(event) => {
            let message = event_data_capnp::convert(event);
            serialize::write_message(&mut payload, &message).unwrap();
            PacketId::EventData
        }

        F123Data::FinalClassification(classification) => {
            let message = final_classification_capnp::convert(classification);
            serialize::write_message(&mut payload, &message).unwrap();
            PacketId::FinalClassification
        }
    };

    let mut buffer = Vec::new();
    let header = packet_header_capnp::convert(packet_type, payload);
    serialize::write_message(&mut buffer, &header).unwrap();

    buffer
}
