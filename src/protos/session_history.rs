include!(concat!(env!("OUT_DIR"), "/session_history_generated.rs"));

use super::ToFlatBufferMessage;
use crate::dtos::PacketSessionHistoryData as BPacketSessionHistoryData;
use flatbuffers::FlatBufferBuilder;
use protos::session_history::{
    LapHistoryData, LapHistoryDataArgs, PacketSessionHistoryData, PacketSessionHistoryDataArgs,
    TyreStintHistoryData, TyreStintHistoryDataArgs,
};

impl ToFlatBufferMessage for BPacketSessionHistoryData {
    fn to_flatbuffer(self) -> Vec<u8> {
        let mut builder = FlatBufferBuilder::new();

        let lap_history_data: Vec<_> = self
            .m_lapHistoryData
            .iter()
            .take_while(|lap| lap.m_sector1TimeInMS > 0)
            .map(|lap| {
                LapHistoryData::create(
                    &mut builder,
                    &LapHistoryDataArgs {
                        m_lap_time_in_ms: lap.m_lapTimeInMS,
                        m_sector1_time_in_ms: lap.m_sector1TimeInMS,
                        m_sector1_time_minutes: lap.m_sector1TimeMinutes,
                        m_sector2_time_in_ms: lap.m_sector2TimeInMS,
                        m_sector2_time_minutes: lap.m_sector2TimeMinutes,
                        m_sector3_time_in_ms: lap.m_sector3TimeInMS,
                        m_sector3_time_minutes: lap.m_sector3TimeMinutes,
                        m_lap_valid_bit_flags: lap.m_lapValidBitFlags,
                    },
                )
            })
            .collect();

        let tyre_stints_history_data: Vec<_> = self
            .m_tyreStintsHistoryData
            .iter()
            .take_while(|stint| stint.m_tyreActualCompound > 0)
            .map(|stint| {
                TyreStintHistoryData::create(
                    &mut builder,
                    &TyreStintHistoryDataArgs {
                        m_end_lap: stint.m_endLap,
                        m_tyre_actual_compound: stint.m_tyreActualCompound,
                        m_tyre_visual_compound: stint.m_tyreVisualCompound,
                    },
                )
            })
            .collect();

        let lap_history_data = Some(builder.create_vector(&lap_history_data));
        let tyre_stints_history_data = Some(builder.create_vector(&tyre_stints_history_data));

        let session_history_data = PacketSessionHistoryData::create(
            &mut builder,
            &PacketSessionHistoryDataArgs {
                m_car_idx: self.m_carIdx,
                m_num_laps: self.m_numLaps,
                m_num_tyre_stints: self.m_numTyreStints,
                m_best_lap_time_lap_num: self.m_bestLapTimeLapNum,
                m_best_sector1_lap_num: self.m_bestSector1LapNum,
                m_best_sector2_lap_num: self.m_bestSector2LapNum,
                m_best_sector3_lap_num: self.m_bestSector3LapNum,
                m_lap_history_data: lap_history_data,
                m_tyre_stints_history_data: tyre_stints_history_data,
            },
        );

        builder.finish(session_history_data, None);
        builder.finished_data().to_vec()
    }
}
