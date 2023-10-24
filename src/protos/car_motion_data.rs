include!(concat!(env!("OUT_DIR"), "/protos.car_motion_data.rs"));

use crate::dtos::{CarMotionData as BCarMotionData, PacketMotionData as BPacketMotionData};

impl From<BCarMotionData> for CarMotionData {
    fn from(value: BCarMotionData) -> Self {
        Self {
            m_world_position_x: value.m_worldPositionX,
            m_world_position_y: value.m_worldPositionY,
            m_world_position_z: value.m_worldPositionZ,
            m_world_velocity_x: value.m_worldVelocityX,
            m_world_velocity_y: value.m_worldVelocityY,
            m_world_velocity_z: value.m_worldVelocityZ,
            m_world_forward_dir_x: value.m_worldForwardDirX as i32,
            m_world_forward_dir_y: value.m_worldForwardDirY as i32,
            m_world_forward_dir_z: value.m_worldForwardDirZ as i32,
            m_world_right_dir_x: value.m_worldRightDirX as i32,
            m_world_right_dir_y: value.m_worldRightDirY as i32,
            m_world_right_dir_z: value.m_worldRightDirZ as i32,
            m_g_force_lateral: value.m_gForceLateral,
            m_g_force_longitudinal: value.m_gForceLongitudinal,
            m_yaw: value.m_yaw,
            m_pitch: value.m_pitch,
            m_roll: value.m_roll,
            m_g_force_vertical: value.m_gForceVertical,
        }
    }
}

impl From<Box<BPacketMotionData>> for PacketMotionData {
    fn from(packet: Box<BPacketMotionData>) -> Self {
        Self {
            m_car_motion_data: packet
                .m_carMotionData
                .into_iter()
                .map(|car_motion_data| car_motion_data.into())
                .collect(),
        }
    }
}
