include!(concat!(env!("OUT_DIR"), "/protos.session_history.rs"));

use super::ToProtoMessage;
use crate::dtos::PacketSessionHistoryData as BPacketSessionHistoryData;

impl ToProtoMessage for BPacketSessionHistoryData {
    type ProtoType = PacketSessionHistoryData;

    fn to_proto(&self) -> Self::ProtoType {
        PacketSessionHistoryData {
            m_car_idx: self.m_carIdx as u32,
            m_num_laps: self.m_numLaps as u32,
            m_num_tyre_stints: self.m_numTyreStints as u32,
            m_best_lap_time_lap_num: self.m_bestLapTimeLapNum as u32,
            m_best_sector1_lap_num: self.m_bestSector1LapNum as u32,
            m_best_sector2_lap_num: self.m_bestSector2LapNum as u32,
            m_best_sector3_lap_num: self.m_bestSector3LapNum as u32,
            m_lap_history_data: self
                .m_lapHistoryData
                .into_iter()
                .take_while(|lap| lap.m_sector1TimeInMS > 0)
                .map(|lap| LapHistoryData {
                    m_lap_time_in_ms: lap.m_lapTimeInMS,
                    m_sector1_time_in_ms: lap.m_sector1TimeInMS as u32,
                    m_sector1_time_minutes: lap.m_sector1TimeMinutes as u32,
                    m_sector2_time_in_ms: lap.m_sector2TimeInMS as u32,
                    m_sector2_time_minutes: lap.m_sector2TimeMinutes as u32,
                    m_sector3_time_in_ms: lap.m_sector3TimeInMS as u32,
                    m_sector3_time_minutes: lap.m_sector3TimeMinutes as u32,
                    m_lap_valid_bit_flags: lap.m_lapValidBitFlags as u32,
                })
                .collect(),
            m_tyre_stints_history_data: self
                .m_tyreStintsHistoryData
                .into_iter()
                .take_while(|stint| stint.m_tyreActualCompound > 0)
                .map(|stint| TyreStintHistoryData {
                    m_end_lap: stint.m_endLap as u32,
                    m_tyre_actual_compound: stint.m_tyreActualCompound as u32,
                    m_tyre_visual_compound: stint.m_tyreVisualCompound as u32,
                })
                .collect(),
        }
    }
}
