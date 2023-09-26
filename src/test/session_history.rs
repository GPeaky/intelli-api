include!(concat!(env!("OUT_DIR"), "/session_history_capnp.rs"));

// use crate::dtos::{
//     LapHistoryData as BLapHistoryData, PacketSessionHistoryData as BPacketSessionHistoryData,
//     TyreStintHistoryData as BTyreStintHistoryData,
// };

// impl From<BLapHistoryData> for LapHistoryData {
//     fn from(value: BLapHistoryData) -> Self {
//         Self {
//             m_lap_time_in_ms: value.m_lapTimeInMS,
//             m_sector1_time_in_ms: value.m_sector1TimeInMS as u32,
//             m_sector1_time_minutes: value.m_sector1TimeMinutes as u32,
//             m_sector2_time_in_ms: value.m_sector2TimeInMS as u32,
//             m_sector2_time_minutes: value.m_sector2TimeMinutes as u32,
//             m_sector3_time_in_ms: value.m_sector3TimeInMS as u32,
//             m_sector3_time_minutes: value.m_sector3TimeMinutes as u32,
//             m_lap_valid_bit_flags: value.m_lapValidBitFlags as u32,
//         }
//     }
// }

// impl From<BTyreStintHistoryData> for TyreStintHistoryData {
//     fn from(value: BTyreStintHistoryData) -> Self {
//         Self {
//             m_end_lap: value.m_endLap as u32,
//             m_tyre_actual_compound: value.m_tyreActualCompound as u32,
//             m_tyre_visual_compound: value.m_tyreVisualCompound as u32,
//         }
//     }
// }
// impl From<Box<BPacketSessionHistoryData>> for PacketSessionHistoryData {
//     fn from(value: Box<BPacketSessionHistoryData>) -> Self {
//         Self {
//             m_car_idx: value.m_carIdx as u32,
//             m_num_laps: value.m_numLaps as u32,
//             m_num_tyre_stints: value.m_numTyreStints as u32,
//             m_best_lap_time_lap_num: value.m_bestLapTimeLapNum as u32,
//             m_best_sector1_lap_num: value.m_bestSector1LapNum as u32,
//             m_best_sector2_lap_num: value.m_bestSector2LapNum as u32,
//             m_best_sector3_lap_num: value.m_bestSector3LapNum as u32,
//             m_lap_history_data: value
//                 .m_lapHistoryData
//                 .into_iter()
//                 .map(|lap_history_data| lap_history_data.into())
//                 .collect(),
//             m_tyre_stints_history_data: value
//                 .m_tyreStintsHistoryData
//                 .into_iter()
//                 .map(|tyre_stint_history_data| tyre_stint_history_data.into())
//                 .collect(),
//         }
//     }
// }
