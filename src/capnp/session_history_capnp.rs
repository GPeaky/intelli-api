use crate::dtos::PacketSessionHistoryData;
use capnp::message::{Builder, HeapAllocator};

include!(concat!(env!("OUT_DIR"), "/session_history_capnp.rs"));

#[inline(always)]
pub fn convert(value: Box<PacketSessionHistoryData>) -> Builder<HeapAllocator> {
    let mut message = Builder::new_default();
    let mut packet_session_history_data =
        message.init_root::<packet_session_history_data::Builder>();

    packet_session_history_data.set_car_idx(value.m_carIdx);
    packet_session_history_data.set_num_laps(value.m_numLaps);
    packet_session_history_data.set_num_tyre_stints(value.m_numTyreStints);
    packet_session_history_data.set_best_lap_time_lap_num(value.m_bestLapTimeLapNum);
    packet_session_history_data.set_best_sector1_lap_num(value.m_bestSector1LapNum);
    packet_session_history_data.set_best_sector2_lap_num(value.m_bestSector2LapNum);
    packet_session_history_data.set_best_sector3_lap_num(value.m_bestSector3LapNum);

    {
        let reborrow = packet_session_history_data.reborrow();

        let mut lap_history_data_list =
            reborrow.init_lap_history_data(value.m_lapHistoryData.len() as u32);

        for (i, lap_history_data) in value.m_lapHistoryData.into_iter().enumerate() {
            let mut lap = lap_history_data_list.reborrow().get(i as u32);

            lap.set_lap_time_in_m_s(lap_history_data.m_lapTimeInMS);
            lap.set_sector1_time_in_m_s(lap_history_data.m_sector1TimeInMS);
            lap.set_sector2_time_in_m_s(lap_history_data.m_sector2TimeInMS);
            lap.set_sector3_time_in_m_s(lap_history_data.m_sector3TimeInMS);
            lap.set_sector1_time_minutes(lap_history_data.m_sector1TimeMinutes);
            lap.set_sector2_time_minutes(lap_history_data.m_sector2TimeMinutes);
            lap.set_sector3_time_minutes(lap_history_data.m_sector3TimeMinutes);
            lap.set_lap_valid_bit_flags(lap_history_data.m_lapValidBitFlags);
        }
    }

    {
        let reborrow = packet_session_history_data.reborrow();

        let mut stint_history_data_list =
            reborrow.init_tyre_stints_history_data(value.m_tyreStintsHistoryData.len() as u32);

        for (i, stint_history_data) in value.m_tyreStintsHistoryData.into_iter().enumerate() {
            let mut stint = stint_history_data_list.reborrow().get(i as u32);

            stint.set_end_lap(stint_history_data.m_endLap);
            stint.set_tyre_actual_compound(stint_history_data.m_tyreActualCompound);
            stint.set_tyre_visual_compound(stint_history_data.m_tyreVisualCompound);
        }
    }

    message
}
