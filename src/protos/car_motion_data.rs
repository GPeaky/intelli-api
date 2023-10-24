include!(concat!(env!("OUT_DIR"), "/car_motion_generated.rs"));

use super::ToProtoMessage;
use crate::dtos::PacketMotionData as BPacketMotionData;
use flatbuffers::FlatBufferBuilder;
use protos::car_motion_data::{
    CarMotionData, CarMotionDataArgs, PacketMotionData, PacketMotionDataArgs,
};

impl ToProtoMessage for BPacketMotionData {
    type ProtoType = Vec<u8>; // Vamos a retornar el FlatBuffer como un Vec<u8>

    fn to_proto(self) -> Self::ProtoType {
        let mut builder = FlatBufferBuilder::new();

        // Convertir m_carMotionData en un vector de offsets de CarMotionData
        let car_motion_data: Vec<_> = self
            .m_carMotionData
            .into_iter()
            .map(|value| {
                CarMotionData::create(
                    &mut builder,
                    &CarMotionDataArgs {
                        m_world_position_x: value.m_worldPositionX,
                        m_world_position_y: value.m_worldPositionY,
                        m_world_position_z: value.m_worldPositionZ,
                        m_world_velocity_x: value.m_worldVelocityX,
                        m_world_velocity_y: value.m_worldVelocityY,
                        m_world_velocity_z: value.m_worldVelocityZ,
                        m_world_forward_dir_x: value.m_worldForwardDirX as i16,
                        m_world_forward_dir_y: value.m_worldForwardDirY as i16,
                        m_world_forward_dir_z: value.m_worldForwardDirZ as i16,
                        m_world_right_dir_x: value.m_worldRightDirX as i16,
                        m_world_right_dir_y: value.m_worldRightDirY as i16,
                        m_world_right_dir_z: value.m_worldRightDirZ as i16,
                        m_g_force_lateral: value.m_gForceLateral,
                        m_g_force_longitudinal: value.m_gForceLongitudinal,
                        m_yaw: value.m_yaw,
                        m_pitch: value.m_pitch,
                        m_roll: value.m_roll,
                        m_g_force_vertical: value.m_gForceVertical,
                    },
                )
            })
            .collect();

        // Crear un vector en el FlatBuffer para m_car_motion_data
        let car_motion_data_vec = Some(builder.create_vector(&car_motion_data));

        // Construir el PacketMotionData y finalizar el FlatBuffer
        let packet_motion_data = PacketMotionData::create(
            &mut builder,
            &PacketMotionDataArgs {
                m_car_motion_data: car_motion_data_vec,
            },
        );

        builder.finish(packet_motion_data, None);
        builder.finished_data().to_vec()
    }
}
