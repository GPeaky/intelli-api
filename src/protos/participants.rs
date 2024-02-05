use std::ffi::CStr;

use crate::structs::PacketParticipantsData as BPacketParticipantsData;

use super::ToProtoMessage;

include!(concat!(env!("OUT_DIR"), "/protos.participants.rs"));

impl ToProtoMessage for BPacketParticipantsData {
    type ProtoType = PacketParticipantsData;

    fn to_proto(&self) -> Option<Self::ProtoType> {
        Some(PacketParticipantsData {
            num_active_cars: self.num_active_cars as u32,
            participants: self
                .participants
                .iter()
                .map(|value| {
                    let c_str = CStr::from_bytes_until_nul(&value.name).unwrap();

                    ParticipantData {
                        ai_controlled: value.ai_controlled as u32,
                        driver_id: value.driver_id as u32,
                        network_id: value.network_id as u32,
                        team_id: value.team_id as u32,
                        my_team: value.my_team as u32,
                        race_number: value.race_number as u32,
                        nationality: value.nationality as u32,
                        name: c_str.to_str().unwrap().to_string(),
                        your_telemetry: value.your_telemetry as u32,
                        show_online_names: value.show_online_names as u32,
                        platform: value.platform as u32,
                    }
                })
                .collect(),
        })
    }
}
