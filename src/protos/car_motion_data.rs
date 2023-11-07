include!(concat!(env!("OUT_DIR"), "/protos.car_motion_data.rs"));

use super::ToProtoMessage;
use crate::dtos::PacketMotionData as BPacketMotionData;

impl ToProtoMessage for BPacketMotionData {
    type ProtoType = PacketMotionData;

    fn to_proto(&self) -> Option<Self::ProtoType> {
        Some(PacketMotionData {
            car_motion_data: self
                .car_motion_data
                .into_iter()
                .map(|value| CarMotionData {
                    world_position_x: value.world_position_x,
                    world_position_y: value.world_position_y,
                    world_position_z: value.world_position_z,
                    world_velocity_x: value.world_velocity_x,
                    world_velocity_y: value.world_velocity_y,
                    world_velocity_z: value.world_velocity_z,
                    world_forward_dir_x: value.world_forward_dir_x as i32,
                    world_forward_dir_y: value.world_forward_dir_y as i32,
                    world_forward_dir_z: value.world_forward_dir_z as i32,
                    world_right_dir_x: value.world_right_dir_x as i32,
                    world_right_dir_y: value.world_right_dir_y as i32,
                    world_right_dir_z: value.world_right_dir_z as i32,
                    g_force_lateral: value.g_force_lateral,
                    g_force_longitudinal: value.g_force_longitudinal,
                    yaw: value.yaw,
                    pitch: value.pitch,
                    roll: value.roll,
                    g_force_vertical: value.g_force_vertical,
                })
                .collect(),
        })
    }
}
