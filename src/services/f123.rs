use crate::dtos::f123_dto::{
    PacketCarDamageData, PacketCarSetupData, PacketCarStatusData, PacketCarTelemetryData,
    PacketEventData, PacketFinalClassificationData, PacketHeader, PacketLapData,
    PacketLobbyInfoData, PacketMotionData, PacketMotionExData, PacketParticipantsData,
    PacketSessionData, PacketSessionHistoryData, PacketTyreSetsData,
};
use bincode::deserialize;

pub enum F123Packet {
    Motion(PacketMotionData),
    Session(PacketSessionData),
    LapData(PacketLapData),
    Event(PacketEventData),
    Participants(PacketParticipantsData),
    CarSetups(PacketCarSetupData),
    CarTelemetry(PacketCarTelemetryData),
    CarStatus(PacketCarStatusData),
    FinalClassification(PacketFinalClassificationData),
    LobbyInfo(PacketLobbyInfoData),
    CarDamage(PacketCarDamageData),
    SessionHistory(PacketSessionHistoryData),
    TyresSets(PacketTyreSetsData),
    MotionExData(PacketMotionExData),
}

pub fn deserialize_packet(
    packet_id: u8,
    data: &[u8],
) -> Result<F123Packet, Box<dyn std::error::Error>> {
    // let deserializer = bincode::options()
    //     .with_varint_encoding()
    //     .with_little_endian();

    match packet_id {
        0 => Ok(F123Packet::Motion(deserialize(data)?)),
        1 => Ok(F123Packet::Session(deserialize(data)?)),
        2 => Ok(F123Packet::LapData(deserialize(data)?)),
        3 => Ok(F123Packet::Event(deserialize(data)?)),
        4 => Ok(F123Packet::Participants(deserialize(data)?)),
        5 => Ok(F123Packet::CarSetups(deserialize(data)?)),
        6 => Ok(F123Packet::CarTelemetry(deserialize(data)?)),
        7 => Ok(F123Packet::CarStatus(deserialize(data)?)),
        8 => Ok(F123Packet::FinalClassification(deserialize(data)?)),
        9 => Ok(F123Packet::LobbyInfo(deserialize(data)?)),
        10 => Ok(F123Packet::CarDamage(deserialize(data)?)),
        11 => Ok(F123Packet::SessionHistory(deserialize(data)?)),
        12 => Ok(F123Packet::TyresSets(deserialize(data)?)),
        13 => Ok(F123Packet::MotionExData(deserialize(data)?)),
        _ => Err("Unknown packet type".into()),
    }
}

pub fn deserialize_header(data: &[u8]) -> Result<PacketHeader, Box<dyn std::error::Error>> {
    Ok(deserialize::<PacketHeader>(data)?)
}
