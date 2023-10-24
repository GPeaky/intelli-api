include!(concat!(env!("OUT_DIR"), "/participants_generated.rs"));

use super::ToProtoMessage;
use crate::dtos::PacketParticipantsData as BPacketParticipantsData;
use std::ffi::CString;

impl ToProtoMessage for BPacketParticipantsData {
    type ProtoType = Vec<u8>;

    fn to_proto(self) -> Self::ProtoType {
        let mut builder = flatbuffers::FlatBufferBuilder::new();

        // Convert m_participants from protobuf to FlatBuffers
        let participants_vec: Vec<_> = self
            .m_participants
            .into_iter()
            .map(|value| {
                let c_str = CString::new(value.m_name).unwrap();
                let name_offset = builder.create_vector(c_str.to_bytes_with_nul());

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
                        ..Default::default()
                    },
                )
            })
            .collect();

        let participants_offset = Some(builder.create_vector(&participants_vec));

        // Create PacketParticipantsData in FlatBuffers
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
