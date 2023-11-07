include!(concat!(env!("OUT_DIR"), "/protos.event_data.rs"));

use super::ToProtoMessage;
use crate::{
    dtos::{EventCode, EventDataDetails as BEventDataDetails, PacketEventData as BPacketEventData},
    protos::event_data::event_data_details::Details,
};

static EVENT_NOT_SEND: [EventCode; 2] = [EventCode::ButtonStatus, EventCode::TeamMateInPits];

impl ToProtoMessage for BPacketEventData {
    type ProtoType = PacketEventData;

    // TODO: Not send the whole string code, if it's not important
    fn to_proto(&self) -> Option<Self::ProtoType> {
        let event_code = EventCode::from(&self.event_string_code);

        if EVENT_NOT_SEND.contains(&event_code) {
            return None;
        };

        Some(PacketEventData {
            event_string_code: self.event_string_code.to_vec(),
            event_details: convert_event_data_details(&event_code, &self.event_details),
        })
    }
}

pub fn convert_event_data_details(
    event_code: &EventCode,
    event_data_details: &BEventDataDetails,
) -> Option<EventDataDetails> {
    let details = match event_code {
        EventCode::FastestLap => {
            let fastest_lap = unsafe { &event_data_details.fastest_lap };

            let fastest_lap = FastestLap {
                lap_time: fastest_lap.lap_time,
                vehicle_idx: fastest_lap.vehicle_idx as u32,
            };

            Some(Details::FastestLap(fastest_lap))
        }

        EventCode::Retirement => {
            let retirement = unsafe { &event_data_details.retirement };

            let retirement = Retirement {
                vehicle_idx: retirement.vehicle_idx as u32,
            };

            Some(Details::Retirement(retirement))
        }

        EventCode::TeamMateInPits => {
            let team_mate_in_pits = unsafe { &event_data_details.team_mate_in_pits };

            let team_mate_in_pits = TeamMateInPits {
                vehicle_idx: team_mate_in_pits.vehicle_idx as u32,
            };

            Some(Details::TeamMateInPits(team_mate_in_pits))
        }

        EventCode::RaceWinner => {
            let race_winner = unsafe { &event_data_details.race_winner };

            let race_winner = RaceWinner {
                vehicle_idx: race_winner.vehicle_idx as u32,
            };

            Some(Details::RaceWinner(race_winner))
        }

        EventCode::PenaltyIssued => {
            let penalty = unsafe { &event_data_details.penalty };

            let penalty = Penalty {
                penalty_type: penalty.penalty_type as u32,
                infringement_type: penalty.infringement_type as u32,
                vehicle_idx: penalty.vehicle_idx as u32,
                other_vehicle_idx: penalty.other_vehicle_idx as u32,
                time: penalty.time as u32,
                lap_num: penalty.lap_num as u32,
                places_gained: penalty.places_gained as u32,
            };

            Some(Details::Penalty(penalty))
        }

        EventCode::SpeedTrapTriggered => {
            let speed_trap = unsafe { &event_data_details.speed_trap };

            let speed_trap = SpeedTrap {
                speed: speed_trap.speed,
                vehicle_idx: speed_trap.vehicle_idx as u32,
                is_overall_fastest_in_session: speed_trap.is_overall_fastest_in_session as u32,
                is_driver_fastest_in_session: speed_trap.is_driver_fastest_in_session as u32,
                fastest_vehicle_idx_in_session: speed_trap.fastest_vehicle_idx_in_session as u32,
                fastest_speed_in_session: speed_trap.fastest_speed_in_session,
            };

            Some(Details::SpeedTrap(speed_trap))
        }

        EventCode::StartLights => {
            let start_lights = unsafe { &event_data_details.start_lights };

            let start_lights = StartLights {
                num_lights: start_lights.num_lights as u32,
            };

            Some(Details::StartLights(start_lights))
        }

        EventCode::DriveThroughServed => {
            let drive_through = unsafe { &event_data_details.drive_through_penalty_served };

            let drive_through = DriveThroughPenaltyServed {
                vehicle_idx: drive_through.vehicle_idx as u32,
            };

            Some(Details::DriveThroughPenaltyServed(drive_through))
        }

        EventCode::StopGoServed => {
            let stop_go = unsafe { &event_data_details.stop_go_penalty_served };

            let stop_go = StopGoPenaltyServed {
                vehicle_idx: stop_go.vehicle_idx as u32,
            };

            Some(Details::StopGoPenaltyServed(stop_go))
        }

        EventCode::Flashback => {
            let flashback = unsafe { &event_data_details.flashback };

            let flashback = Flashback {
                flashback_frame_identifier: flashback.flashback_frame_identifier,
                flashback_session_time: flashback.flashback_session_time,
            };

            Some(Details::Flashback(flashback))
        }

        EventCode::ButtonStatus => {
            let buttons = unsafe { &event_data_details.buttons };

            let buttons = Buttons {
                button_status: buttons.button_status,
            };

            Some(Details::Buttons(buttons))
        }

        EventCode::Overtake => {
            let overtake = unsafe { &event_data_details.overtake };

            let overtake = Overtake {
                overtaking_vehicle_idx: overtake.overtaking_vehicle_idx as u32,
                being_overtaken_vehicle_idx: overtake.being_overtaken_vehicle_idx as u32,
            };

            Some(Details::Overtake(overtake))
        }

        _ => None,
    };

    details.map(|details| EventDataDetails {
        details: Some(details),
    })
}
