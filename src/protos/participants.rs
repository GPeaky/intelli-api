include!(concat!(env!("OUT_DIR"), "/protos.participants.rs"));

use crate::dtos::{
    PacketParticipantsData as BPacketParticipantsData, ParticipantData as BParticipantData,
};
use std::ffi::CStr;

impl From<BParticipantData> for ParticipantData {
    fn from(value: BParticipantData) -> Self {
        let c_str = CStr::from_bytes_until_nul(&value.m_name).unwrap();

        Self {
            m_ai_controlled: value.m_aiControlled as u32,
            m_driver_id: value.m_driverId as u32,
            m_network_id: value.m_networkId as u32,
            m_team_id: value.m_teamId as u32,
            m_my_team: value.m_myTeam as u32,
            m_race_number: value.m_raceNumber as u32,
            m_nationality: value.m_nationality as u32,
            m_name: c_str.to_str().unwrap().to_string(),
            m_your_telemetry: value.m_yourTelemetry as u32,
            m_show_online_names: value.m_showOnlineNames as u32,
            m_platform: value.m_platform as u32,
        }
    }
}

impl From<Box<BPacketParticipantsData>> for PacketParticipantsData {
    fn from(value: Box<BPacketParticipantsData>) -> Self {
        Self {
            m_num_active_cars: value.m_numActiveCars as u32,
            m_participants: value
                .m_participants
                .into_iter()
                .map(|participant| participant.into())
                .collect(),
        }
    }
}
