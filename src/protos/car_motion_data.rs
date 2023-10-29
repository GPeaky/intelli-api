include!(concat!(
    env!("OUT_DIR"),
    "/generated/car_motion_generated.rs"
));

use super::ToFlatBufferMessage;
use crate::dtos::PacketMotionData as BPacketMotionData;
use flatbuffers::FlatBufferBuilder;
use protos::car_motion_data::{
    CarMotionData, CarMotionDataArgs, PacketMotionData, PacketMotionDataArgs,
};

impl ToFlatBufferMessage for BPacketMotionData {
    fn to_flatbuffer(self) -> Vec<u8> {
        let mut builder = FlatBufferBuilder::new();

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
                        m_world_forward_dir_x: value.m_worldForwardDirX,
                        m_world_forward_dir_y: value.m_worldForwardDirY,
                        m_world_forward_dir_z: value.m_worldForwardDirZ,
                        m_world_right_dir_x: value.m_worldRightDirX,
                        m_world_right_dir_y: value.m_worldRightDirY,
                        m_world_right_dir_z: value.m_worldRightDirZ,
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

        let car_motion_data_vec = Some(builder.create_vector(&car_motion_data));

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
