use crate::structs::PacketFinalClassificationData as BFinalClassificationData;

use super::ToProtoMessage;

include!(concat!(env!("OUT_DIR"), "/protos.final_classification.rs"));

impl ToProtoMessage for BFinalClassificationData {
    type ProtoType = PacketFinalClassificationData;

    fn to_proto(&self) -> Option<Self::ProtoType> {
        let mut classification_data = Vec::with_capacity(self.classification_data.len());

        for value in &self.classification_data {
            let mut tyre_stints_actual = Vec::with_capacity(8);
            let mut tyre_stints_visual = Vec::with_capacity(8);
            let mut tyre_stints_end_laps = Vec::with_capacity(8);

            for &x in &value.tyre_stints_actual {
                tyre_stints_actual.push(x as u32);
            }

            for &x in &value.tyre_stints_visual {
                tyre_stints_visual.push(x as u32);
            }

            for &x in &value.tyre_stints_end_laps {
                tyre_stints_end_laps.push(x as u32);
            }

            classification_data.push(FinalClassificationData {
                position: value.position as u32,
                num_laps: value.num_laps as u32,
                grid_position: value.grid_position as u32,
                points: value.points as u32,
                num_pit_stops: value.num_pit_stops as u32,
                result_status: value.result_status as u32,
                best_lap_time_in_ms: value.best_lap_time_in_ms,
                total_race_time: value.total_race_time,
                penalties_time: value.penalties_time as u32,
                num_penalties: value.num_penalties as u32,
                num_tyre_stints: value.num_tyre_stints as u32,
                tyre_stints_actual,
                tyre_stints_visual,
                tyre_stints_end_laps,
            })
        }

        Some(PacketFinalClassificationData {
            num_cars: self.num_cars as u32,
            classification_data,
        })
    }
}
