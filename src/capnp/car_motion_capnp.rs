use crate::dtos::PacketMotionData;
use capnp::message::{Builder, HeapAllocator, TypedBuilder, TypedReader};

include!(concat!(env!("OUT_DIR"), "/car_motion_capnp.rs"));

pub fn build_car_motion(value: Box<PacketMotionData>) -> capnp::message::Builder<HeapAllocator> {
    let mut message = capnp::message::Builder::new_default();
    let packet_motion_data = message.init_root::<packet_motion_data::Builder>();

    let mut car_motion_data_list =
        packet_motion_data.init_car_motion_data(value.m_carMotionData.len() as u32);

    for (i, car_motion_data) in value.m_carMotionData.into_iter().enumerate() {
        let mut car = car_motion_data_list.reborrow().get(i as u32);

        car.set_world_position_x(car_motion_data.m_worldPositionX);
        car.set_world_position_y(car_motion_data.m_worldPositionY);
        car.set_world_position_z(car_motion_data.m_worldPositionZ);
        car.set_world_velocity_x(car_motion_data.m_worldVelocityX);
        car.set_world_velocity_y(car_motion_data.m_worldVelocityY);
        car.set_world_velocity_z(car_motion_data.m_worldVelocityZ);
        car.set_world_forward_dir_x(car_motion_data.m_worldForwardDirX);
        car.set_world_forward_dir_y(car_motion_data.m_worldForwardDirY);
        car.set_world_forward_dir_z(car_motion_data.m_worldForwardDirZ);
        car.set_world_right_dir_x(car_motion_data.m_worldRightDirX);
        car.set_world_right_dir_y(car_motion_data.m_worldRightDirY);
        car.set_world_right_dir_z(car_motion_data.m_worldRightDirZ);
        car.set_g_force_lateral(car_motion_data.m_gForceLateral);
        car.set_g_force_longitudinal(car_motion_data.m_gForceLongitudinal);
        car.set_g_force_vertical(car_motion_data.m_gForceVertical);
        car.set_yaw(car_motion_data.m_yaw);
        car.set_pitch(car_motion_data.m_pitch);
        car.set_roll(car_motion_data.m_roll);
    }

    message
}
