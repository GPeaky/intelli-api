include!(concat!(env!("OUT_DIR"), "/event_data_generated.rs"));

use super::ToFlatBufferMessage;
use crate::dtos::{EventDataDetails as BEventDataDetails, PacketEventData as BPacketEventData};
use flatbuffers::FlatBufferBuilder;

impl ToFlatBufferMessage for BPacketEventData {
    fn to_flatbuffer(self) -> Vec<u8> {
        let mut builder = FlatBufferBuilder::new();

        let event_string_code = Some(builder.create_vector(&self.m_eventStringCode));

        let (event_detail_type, event_detail_offset) = match self.m_eventDetails {
            BEventDataDetails::FastestLap {
                vehicleIdx,
                lapTime,
            } => {
                let fastest_lap = protos::event_data::FastestLap::create(
                    &mut builder,
                    &protos::event_data::FastestLapArgs {
                        vehicle_idx: vehicleIdx,
                        lap_time: lapTime,
                    },
                );
                (
                    protos::event_data::EventDataDetailsUnion::FastestLap,
                    fastest_lap.as_union_value(),
                )
            }

            BEventDataDetails::Retirement { vehicleIdx } => {
                let data = protos::event_data::Retirement::create(
                    &mut builder,
                    &protos::event_data::RetirementArgs {
                        vehicle_idx: vehicleIdx,
                        ..Default::default()
                    },
                );

                (
                    protos::event_data::EventDataDetailsUnion::Retirement,
                    data.as_union_value(),
                )
            }

            BEventDataDetails::TeamMateInPits { vehicleIdx } => {
                let data = protos::event_data::TeamMateInPits::create(
                    &mut builder,
                    &protos::event_data::TeamMateInPitsArgs {
                        vehicle_idx: vehicleIdx,
                        ..Default::default()
                    },
                );
                (
                    protos::event_data::EventDataDetailsUnion::TeamMateInPits,
                    data.as_union_value(),
                )
            }

            BEventDataDetails::RaceWinner { vehicleIdx } => {
                let data = protos::event_data::RaceWinner::create(
                    &mut builder,
                    &protos::event_data::RaceWinnerArgs {
                        vehicle_idx: vehicleIdx,
                        ..Default::default()
                    },
                );
                (
                    protos::event_data::EventDataDetailsUnion::RaceWinner,
                    data.as_union_value(),
                )
            }

            BEventDataDetails::Penalty {
                penaltyType,
                infringementType,
                vehicleIdx,
                otherVehicleIdx,
                time,
                lapNum,
                placesGained,
            } => {
                let data = protos::event_data::Penalty::create(
                    &mut builder,
                    &protos::event_data::PenaltyArgs {
                        time,
                        penalty_type: penaltyType,
                        infringement_type: infringementType,
                        vehicle_idx: vehicleIdx,
                        other_vehicle_idx: otherVehicleIdx,
                        lap_num: lapNum,
                        places_gained: placesGained,
                        ..Default::default()
                    },
                );

                (
                    protos::event_data::EventDataDetailsUnion::Penalty,
                    data.as_union_value(),
                )
            }

            BEventDataDetails::SpeedTrap {
                vehicleIdx,
                speed,
                isOverallFastestInSession,
                isDriverFastestInSession,
                fastestVehicleIdxInSession,
                fastestSpeedInSession,
            } => {
                let data = protos::event_data::SpeedTrap::create(
                    &mut builder,
                    &protos::event_data::SpeedTrapArgs {
                        vehicle_idx: vehicleIdx,
                        speed,
                        is_overall_fastest_in_session: isOverallFastestInSession,
                        is_driver_fastest_in_session: isDriverFastestInSession,
                        fastest_vehicle_idx_in_session: fastestVehicleIdxInSession,
                        fastest_speed_in_session: fastestSpeedInSession,
                        ..Default::default()
                    },
                );
                (
                    protos::event_data::EventDataDetailsUnion::SpeedTrap,
                    data.as_union_value(),
                )
            }

            BEventDataDetails::StartLights { numLights } => {
                let data = protos::event_data::StartLights::create(
                    &mut builder,
                    &protos::event_data::StartLightsArgs {
                        num_lights: numLights,
                        ..Default::default()
                    },
                );
                (
                    protos::event_data::EventDataDetailsUnion::StartLights,
                    data.as_union_value(),
                )
            }

            BEventDataDetails::DriveThroughPenaltyServed { vehicleIdx } => {
                let data = protos::event_data::DriveThroughPenaltyServed::create(
                    &mut builder,
                    &protos::event_data::DriveThroughPenaltyServedArgs {
                        vehicle_idx: vehicleIdx,
                        ..Default::default()
                    },
                );
                (
                    protos::event_data::EventDataDetailsUnion::DriveThroughPenaltyServed,
                    data.as_union_value(),
                )
            }

            BEventDataDetails::StopGoPenaltyServed { vehicleIdx } => {
                let data = protos::event_data::StopGoPenaltyServed::create(
                    &mut builder,
                    &protos::event_data::StopGoPenaltyServedArgs {
                        vehicle_idx: vehicleIdx,
                        ..Default::default()
                    },
                );
                (
                    protos::event_data::EventDataDetailsUnion::StopGoPenaltyServed,
                    data.as_union_value(),
                )
            }

            BEventDataDetails::Flashback {
                flashbackFrameIdentifier,
                flashbackSessionTime,
            } => {
                let data = protos::event_data::Flashback::create(
                    &mut builder,
                    &protos::event_data::FlashbackArgs {
                        flashback_frame_identifier: flashbackFrameIdentifier,
                        flashback_session_time: flashbackSessionTime,
                        ..Default::default()
                    },
                );
                (
                    protos::event_data::EventDataDetailsUnion::Flashback,
                    data.as_union_value(),
                )
            }

            BEventDataDetails::Buttons { buttonStatus } => {
                let data = protos::event_data::Buttons::create(
                    &mut builder,
                    &protos::event_data::ButtonsArgs {
                        button_status: buttonStatus,
                        ..Default::default()
                    },
                );
                (
                    protos::event_data::EventDataDetailsUnion::Buttons,
                    data.as_union_value(),
                )
            }

            BEventDataDetails::Overtake {
                overtakingVehicleIdx,
                beingOvertakenVehicleIdx,
            } => {
                let data = protos::event_data::Overtake::create(
                    &mut builder,
                    &protos::event_data::OvertakeArgs {
                        overtaking_vehicle_idx: overtakingVehicleIdx,
                        being_overtaken_vehicle_idx: beingOvertakenVehicleIdx,
                        ..Default::default()
                    },
                );
                (
                    protos::event_data::EventDataDetailsUnion::Overtake,
                    data.as_union_value(),
                )
            }
        };

        let event_details = protos::event_data::event_data_details::create(
            &mut builder,
            &protos::event_data::event_data_detailsArgs {
                details_type: event_detail_type,
                details: Some(event_detail_offset),
            },
        );

        let event_data = protos::event_data::PacketEventData::create(
            &mut builder,
            &protos::event_data::PacketEventDataArgs {
                m_event_string_code: event_string_code,
                m_event_details: Some(event_details),
            },
        );

        builder.finish(event_data, None);
        builder.finished_data().to_vec()
    }
}
