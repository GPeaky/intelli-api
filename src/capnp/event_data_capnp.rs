use crate::dtos::{EventDataDetails, PacketEventData};
use capnp::message::{Builder, HeapAllocator};
use tracing::info;

include!(concat!(env!("OUT_DIR"), "/event_data_capnp.rs"));

#[inline(always)]
pub fn convert(value: PacketEventData) -> Builder<HeapAllocator> {
    let mut message = Builder::new_default();
    let mut packet_event_data = message.init_root::<packet_event_data::Builder>();

    {
        let event_string_code_list = packet_event_data.reborrow().init_event_string_code(4);

        for (i, code) in value.m_eventStringCode.into_iter().enumerate() {
            event_string_code_list[i] = code;
        }
    }

    {
        let event_details = packet_event_data.reborrow().init_event_details();

        match value.m_eventDetails {
            EventDataDetails::Buttons { buttonStatus } => {
                let mut buttons = event_details.init_buttons();
                buttons.set_button_status(buttonStatus);
            }

            EventDataDetails::DriveThroughPenaltyServed { vehicleIdx } => {
                let mut drive_through_penalty_served =
                    event_details.init_drive_through_penalty_served();

                drive_through_penalty_served.set_vehicle_idx(vehicleIdx);
            }

            EventDataDetails::FastestLap {
                vehicleIdx,
                lapTime,
            } => {
                let mut fastest_lap = event_details.init_fastest_lap();
                fastest_lap.set_vehicle_idx(vehicleIdx);
                fastest_lap.set_lap_time(lapTime);
            }

            EventDataDetails::Flashback {
                flashbackFrameIdentifier,
                flashbackSessionTime,
            } => {
                let mut flashback = event_details.init_flashback();
                flashback.set_flashback_frame_identifier(flashbackFrameIdentifier);
                flashback.set_flashback_session_time(flashbackSessionTime);
            }

            EventDataDetails::Overtake {
                overtakingVehicleIdx,
                beingOvertakenVehicleIdx,
            } => {
                let mut overtake = event_details.init_overtake();
                overtake.set_overtaking_vehicle_idx(overtakingVehicleIdx);
                overtake.set_being_overtaken_vehicle_idx(beingOvertakenVehicleIdx);
            }

            EventDataDetails::RaceWinner { vehicleIdx } => {
                let mut race_winner = event_details.init_race_winner();
                race_winner.set_vehicle_idx(vehicleIdx);
            }

            EventDataDetails::Retirement { vehicleIdx } => {
                let mut retirement = event_details.init_retirement();
                retirement.set_vehicle_idx(vehicleIdx);
            }

            EventDataDetails::SpeedTrap {
                vehicleIdx,
                speed,
                isOverallFastestInSession,
                isDriverFastestInSession,
                fastestVehicleIdxInSession,
                fastestSpeedInSession,
            } => {
                let mut speed_trap = event_details.init_speed_trap();
                speed_trap.set_vehicle_idx(vehicleIdx);
                speed_trap.set_speed(speed);
                speed_trap.set_is_overall_fastest_in_session(isOverallFastestInSession);
                speed_trap.set_is_driver_fastest_in_session(isDriverFastestInSession);
                speed_trap.set_fastest_vehicle_idx_in_session(fastestVehicleIdxInSession);
                speed_trap.set_fastest_speed_in_session(fastestSpeedInSession);
            }

            EventDataDetails::TeamMateInPits { vehicleIdx } => {
                let mut team_mate_in_pits = event_details.init_team_mate_in_pits();
                team_mate_in_pits.set_vehicle_idx(vehicleIdx);
            }

            EventDataDetails::Penalty {
                penaltyType,
                infringementType,
                vehicleIdx,
                otherVehicleIdx,
                time,
                lapNum,
                placesGained,
            } => {
                let mut penalty = event_details.init_penalty();
                penalty.set_penalty_type(penaltyType);
                penalty.set_infringement_type(infringementType);
                penalty.set_vehicle_idx(vehicleIdx);
                penalty.set_other_vehicle_idx(otherVehicleIdx);
                penalty.set_time(time);
                penalty.set_lap_num(lapNum);
                penalty.set_places_gained(placesGained);
            }

            EventDataDetails::StartLights { numLights } => {
                let mut start_lights = event_details.init_start_lights();
                start_lights.set_num_lights(numLights);
            }

            EventDataDetails::StopGoPenaltyServed { vehicleIdx } => {
                let mut drive_through_served = event_details.init_stop_go_penalty_served();
                drive_through_served.set_vehicle_idx(vehicleIdx);
            }
        };
    }

    message
}
