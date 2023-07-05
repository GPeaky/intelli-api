use crate::dtos::f123_dto::{
    PacketEventData, PacketFinalClassificationData, PacketHeader, PacketLapData, PacketMotionData,
    PacketParticipantsData, PacketSessionData, PacketTyreSetsData,
};
use bincode::deserialize;

pub enum F123Packet {
    Motion(PacketMotionData),
    Session(PacketSessionData),
    LapData(PacketLapData),
    Event(PacketEventData),
    Participants(PacketParticipantsData),
    FinalClassification(PacketFinalClassificationData),
    TyresSets(PacketTyreSetsData),
}

pub fn deserialize_packet(
    packet_id: u8,
    data: &[u8],
) -> Result<F123Packet, Box<dyn std::error::Error>> {
    match packet_id {
        0 => Ok(F123Packet::Motion(deserialize(data)?)),
        1 => Ok(F123Packet::Session(deserialize(data)?)),
        2 => Ok(F123Packet::LapData(deserialize(data)?)),
        3 => Ok(F123Packet::Event(deserialize(data)?)),
        4 => Ok(F123Packet::Participants(deserialize(data)?)),
        8 => Ok(F123Packet::FinalClassification(deserialize(data)?)),
        12 => Ok(F123Packet::TyresSets(deserialize(data)?)),
        _ => Err("Unknown packet type".into()),
    }
}

pub fn deserialize_header(data: &[u8]) -> Result<PacketHeader, Box<dyn std::error::Error>> {
    Ok(deserialize::<PacketHeader>(data)?)
}
