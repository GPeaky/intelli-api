include!(concat!(env!("OUT_DIR"), "/protos.event_data.rs"));

use crate::dtos::{EventDataDetails as BEventDataDetails, PacketEventData as BPacketEventData};
use event_data_details::Details;

impl From<BEventDataDetails> for Option<EventDataDetails> {
    fn from(value: BEventDataDetails) -> Self {
        let details = match value {
            BEventDataDetails::FastestLap {
                vehicleIdx,
                lapTime,
            } => {
                let fastest_lap = FastestLap {
                    lap_time: lapTime,
                    vehicle_idx: vehicleIdx as u32,
                };

                Details::FastestLap(fastest_lap)
            }
            BEventDataDetails::Retirement { vehicleIdx } => {
                let retirement = Retirement {
                    vehicle_idx: vehicleIdx as u32,
                };

                Details::Retirement(retirement)
            }
            BEventDataDetails::TeamMateInPits { vehicleIdx } => {
                let team_mate_in_pits = TeamMateInPits {
                    vehicle_idx: vehicleIdx as u32,
                };

                Details::TeamMateInPits(team_mate_in_pits)
            }
            BEventDataDetails::RaceWinner { vehicleIdx } => {
                let race_winner = RaceWinner {
                    vehicle_idx: vehicleIdx as u32,
                };

                Details::RaceWinner(race_winner)
            }
            BEventDataDetails::Penalty {
                vehicleIdx,
                penaltyType,
                infringementType,
                otherVehicleIdx,
                time,
                lapNum,
                placesGained,
            } => {
                let penalty = Penalty {
                    penalty_type: penaltyType as u32,
                    infringement_type: infringementType as u32,
                    vehicle_idx: vehicleIdx as u32,
                    other_vehicle_idx: otherVehicleIdx as u32,
                    time: time as u32,
                    lap_num: lapNum as u32,
                    places_gained: placesGained as u32,
                };

                Details::Penalty(penalty)
            }
            BEventDataDetails::SpeedTrap {
                vehicleIdx,
                speed,
                isOverallFastestInSession,
                isDriverFastestInSession,
                fastestVehicleIdxInSession,
                fastestSpeedInSession,
            } => {
                let speed_trap = SpeedTrap {
                    speed,
                    vehicle_idx: vehicleIdx as u32,
                    is_overall_fastest_in_session: isOverallFastestInSession as u32,
                    is_driver_fastest_in_session: isDriverFastestInSession as u32,
                    fastest_vehicle_idx_in_session: fastestVehicleIdxInSession as u32,
                    fastest_speed_in_session: fastestSpeedInSession,
                };

                Details::SpeedTrap(speed_trap)
            }

            BEventDataDetails::StartLights { numLights } => {
                let start_lights = StartLights {
                    num_lights: numLights as u32,
                };

                Details::StartLights(start_lights)
            }

            BEventDataDetails::DriveThroughPenaltyServed { vehicleIdx } => {
                let drive_through_penalty_served = DriveThroughPenaltyServed {
                    vehicle_idx: vehicleIdx as u32,
                };

                Details::DriveThroughPenaltyServed(drive_through_penalty_served)
            }

            BEventDataDetails::StopGoPenaltyServed { vehicleIdx } => {
                let stop_go_penalty_served = StopGoPenaltyServed {
                    vehicle_idx: vehicleIdx as u32,
                };

                Details::StopGoPenaltyServed(stop_go_penalty_served)
            }

            BEventDataDetails::Flashback {
                flashbackFrameIdentifier,
                flashbackSessionTime,
            } => {
                let flashback = Flashback {
                    flashback_frame_identifier: flashbackFrameIdentifier,
                    flashback_session_time: flashbackSessionTime,
                };

                Details::Flashback(flashback)
            }
            BEventDataDetails::Buttons { buttonStatus } => {
                let buttons = Buttons {
                    button_status: buttonStatus,
                };

                Details::Buttons(buttons)
            }
            BEventDataDetails::Overtake {
                overtakingVehicleIdx,
                beingOvertakenVehicleIdx,
            } => {
                let overtake = Overtake {
                    overtaking_vehicle_idx: overtakingVehicleIdx as u32,
                    being_overtaken_vehicle_idx: beingOvertakenVehicleIdx as u32,
                };

                Details::Overtake(overtake)
            }
        };

        Some(EventDataDetails {
            details: Some(details),
        })
    }
}
impl From<BPacketEventData> for PacketEventData {
    fn from(value: BPacketEventData) -> Self {
        Self {
            m_event_string_code: value.m_eventStringCode.to_vec(),
            m_event_details: value.m_eventDetails.into(),
        }
    }
}
