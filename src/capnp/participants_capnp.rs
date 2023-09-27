use crate::dtos::PacketParticipantsData;
use capnp::message::{Builder, HeapAllocator};

include!(concat!(env!("OUT_DIR"), "/participants_capnp.rs"));

#[inline(always)]
pub fn convert(value: Box<PacketParticipantsData>) -> Builder<HeapAllocator> {
    let mut message = Builder::new_default();
    let mut packet_participants_data = message.init_root::<packet_participants_data::Builder>();

    packet_participants_data.set_num_active_cars(value.m_numActiveCars);

    let mut participants_data_list =
        packet_participants_data.init_participants(value.m_participants.len() as u32);

    for (i, participant_data) in value.m_participants.into_iter().enumerate() {
        let mut participant = participants_data_list.reborrow().get(i as u32);

        participant.set_ai_controlled(participant_data.m_aiControlled);
        participant.set_driver_id(participant_data.m_driverId);
        participant.set_network_id(participant_data.m_networkId);
        participant.set_team_id(participant_data.m_teamId);
        participant.set_race_number(participant_data.m_raceNumber);
        participant.set_nationality(participant_data.m_nationality);
        participant.set_name(capnp::text::Reader(&participant_data.m_name));
        participant.set_your_telemetry(participant_data.m_yourTelemetry);
        participant.set_show_online_names(participant_data.m_showOnlineNames);
        participant.set_platform(participant_data.m_platform);
    }

    message
}
