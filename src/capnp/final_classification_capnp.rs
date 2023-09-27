use crate::dtos::PacketFinalClassificationData;
use capnp::message::{Builder, HeapAllocator};

include!(concat!(env!("OUT_DIR"), "/final_classification_capnp.rs"));

#[inline(always)]
pub fn convert(value: Box<PacketFinalClassificationData>) -> Builder<HeapAllocator> {
    let mut message = Builder::new_default();
    let mut packet_final_classification_data =
        message.init_root::<packet_final_classification_data::Builder>();

    packet_final_classification_data.set_num_cars(value.m_numCars);

    let mut final_classification_data_list = packet_final_classification_data
        .init_classification_data(value.m_classificationData.len() as u32);

    for (i, classificationData) in value.m_classificationData.into_iter().enumerate() {
        let mut final_classification_data = final_classification_data_list.reborrow().get(i as u32);

        final_classification_data.set_position(classificationData.m_position);
        final_classification_data.set_num_laps(classificationData.m_numLaps);
        final_classification_data.set_grid_position(classificationData.m_gridPosition);
        final_classification_data.set_points(classificationData.m_points);
        final_classification_data.set_num_pit_stops(classificationData.m_numPitStops);
        final_classification_data.set_result_status(classificationData.m_resultStatus);
        final_classification_data.set_best_lap_time_in_m_s(classificationData.m_bestLapTimeInMS);
        final_classification_data.set_total_race_time(classificationData.m_totalRaceTime);
        final_classification_data.set_penalties_time(classificationData.m_penaltiesTime);
        final_classification_data.set_num_penalties(classificationData.m_numPenalties);
        final_classification_data.set_num_tyre_stints(classificationData.m_numTyreStints);
    }

    message
}
