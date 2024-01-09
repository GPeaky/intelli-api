use crate::dtos::PacketFinalClassificationData as BFinalClassificationData;

use super::ToProtoMessage;

include!(concat!(env!("OUT_DIR"), "/protos.final_classification.rs"));

impl ToProtoMessage for BFinalClassificationData {
    type ProtoType = PacketFinalClassificationData;

    fn to_proto(&self) -> Option<Self::ProtoType> {
        Some(PacketFinalClassificationData {
            num_cars: self.num_cars as u32,
            classification_data: self
                .classification_data
                .iter()
                .map(|data| FinalClassificationData {
                    position: data.position as u32,
                    num_laps: data.num_laps as u32,
                    grid_position: data.grid_position as u32,
                    points: data.points as u32,
                    num_pit_stops: data.num_pit_stops as u32,
                    result_status: data.result_status as u32,
                    best_lap_time_in_ms: data.best_lap_time_in_ms,
                    total_race_time: data.total_race_time,
                    penalties_time: data.penalties_time as u32,
                    num_penalties: data.num_penalties as u32,
                    num_tyre_stints: data.num_tyre_stints as u32,
                    tyre_stints_actual: data.tyre_stints_actual.iter().map(|&x| x as u32).collect(),
                    tyre_stints_visual: data.tyre_stints_visual.iter().map(|&x| x as u32).collect(),
                    tyre_stints_end_laps: data
                        .tyre_stints_end_laps
                        .iter()
                        .map(|&x| x as u32)
                        .collect(),
                })
                .collect(),
        })
    }
}
