include!(concat!(
    env!("OUT_DIR"),
    "/generated/participants_generated.rs"
));

use super::ToFlatBufferMessage;
use crate::dtos::PacketParticipantsData as BPacketParticipantsData;

impl ToFlatBufferMessage for BPacketParticipantsData {
    fn to_flatbuffer(self) -> Vec<u8> {
        let mut builder = flatbuffers::FlatBufferBuilder::new();

        let participants_vec: Vec<_> = self
            .m_participants
            .into_iter()
            .map(|value| {
                let name_offset = builder.create_vector(&value.m_name);

                protos::participants::ParticipantData::create(
                    &mut builder,
                    &protos::participants::ParticipantDataArgs {
                        m_ai_controlled: value.m_aiControlled,
                        m_driver_id: value.m_driverId,
                        m_network_id: value.m_networkId,
                        m_team_id: value.m_teamId,
                        m_my_team: value.m_myTeam,
                        m_race_number: value.m_raceNumber,
                        m_nationality: value.m_nationality,
                        m_name: Some(name_offset),
                        m_your_telemetry: value.m_yourTelemetry,
                        m_show_online_names: value.m_showOnlineNames,
                        m_platform: value.m_platform,
                    },
                )
            })
            .collect();

        let participants_offset = Some(builder.create_vector(&participants_vec));

        let packet_data = protos::participants::PacketParticipantsData::create(
            &mut builder,
            &protos::participants::PacketParticipantsDataArgs {
                m_num_active_cars: self.m_numActiveCars,
                m_participants: participants_offset,
            },
        );

        builder.finish(packet_data, None);
        builder.finished_data().to_vec()
    }
}
