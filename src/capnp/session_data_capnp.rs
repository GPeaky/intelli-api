use crate::dtos::PacketSessionData;
use capnp::message::{Builder, HeapAllocator};

include!(concat!(env!("OUT_DIR"), "/session_data_capnp.rs"));

#[inline(always)]
pub fn convert(value: Box<PacketSessionData>) -> Builder<HeapAllocator> {
    let mut message = Builder::new_default();
    let mut packet_session_data = message.init_root::<packet_session_data::Builder>();

    packet_session_data.set_weather(value.m_weather);
    packet_session_data.set_track_temperature(value.m_trackTemperature);
    packet_session_data.set_air_temperature(value.m_airTemperature);
    packet_session_data.set_total_laps(value.m_totalLaps);
    packet_session_data.set_track_length(value.m_trackLength);
    packet_session_data.set_session_type(value.m_sessionType);
    packet_session_data.set_track_id(value.m_trackId);
    packet_session_data.set_formula(value.m_formula);
    packet_session_data.set_session_time_left(value.m_sessionTimeLeft);
    packet_session_data.set_session_duration(value.m_sessionDuration);
    packet_session_data.set_pit_speed_limit(value.m_pitSpeedLimit);
    packet_session_data.set_game_paused(value.m_gamePaused);
    packet_session_data.set_is_spectating(value.m_isSpectating);
    packet_session_data.set_spectator_car_index(value.m_spectatorCarIndex);
    packet_session_data.set_sli_pro_native_support(value.m_sliProNativeSupport);
    packet_session_data.set_num_marshal_zones(value.m_numMarshalZones);
    packet_session_data.set_safety_car_status(value.m_safetyCarStatus);
    packet_session_data.set_network_game(value.m_networkGame);
    packet_session_data.set_num_weather_forecast_samples(value.m_numWeatherForecastSamples);
    packet_session_data.set_forecast_accuracy(value.m_forecastAccuracy);
    packet_session_data.set_season_link_identifier(value.m_seasonLinkIdentifier);
    packet_session_data.set_weekend_link_identifier(value.m_weekendLinkIdentifier);
    packet_session_data.set_session_link_identifier(value.m_sessionLinkIdentifier);
    packet_session_data.set_pit_stop_window_ideal_lap(value.m_pitStopWindowIdealLap);
    packet_session_data.set_pit_stop_window_latest_lap(value.m_pitStopWindowLatestLap);
    packet_session_data.set_pit_stop_rejoin_position(value.m_pitStopRejoinPosition);
    packet_session_data.set_steering_assist(value.m_steeringAssist);
    packet_session_data.set_braking_assist(value.m_brakingAssist);
    packet_session_data.set_gearbox_assist(value.m_gearboxAssist);
    packet_session_data.set_pit_assist(value.m_pitAssist);
    packet_session_data.set_ers_assist(value.m_ERSAssist);
    packet_session_data.set_drs_assist(value.m_DRSAssist);
    packet_session_data.set_dynamic_racing_line(value.m_dynamicRacingLine);
    packet_session_data.set_dynamic_racing_line_type(value.m_dynamicRacingLineType);
    packet_session_data.set_game_mode(value.m_gameMode);
    packet_session_data.set_time_of_day(value.m_timeOfDay);
    packet_session_data.set_session_length(value.m_sessionLength);
    packet_session_data.set_speed_units_lead_player(value.m_speedUnitsLeadPlayer);
    packet_session_data.set_temperature_units_lead_player(value.m_temperatureUnitsLeadPlayer);
    packet_session_data.set_speed_units_secondary_player(value.m_speedUnitsSecondaryPlayer);
    packet_session_data
        .set_temperature_units_secondary_player(value.m_temperatureUnitsSecondaryPlayer);
    packet_session_data.set_num_safety_car_periods(value.m_numSafetyCarPeriods);
    packet_session_data.set_num_virtual_safety_car_periods(value.m_numVirtualSafetyCarPeriods);
    packet_session_data.set_num_red_flag_periods(value.m_numRedFlagPeriods);

    {
        let mut marshal_zones_list = packet_session_data
            .reborrow()
            .init_marshal_zones(value.m_marshalZones.len() as u32);

        for (i, marshal) in value.m_marshalZones.into_iter().enumerate() {
            let mut marshal_zone = marshal_zones_list.reborrow().get(i as u32);

            marshal_zone.set_zone_start(marshal.m_zoneStart);
            marshal_zone.set_zone_flag(marshal.m_zoneFlag);
        }
    }

    {
        let mut weather_forecast_samples_list = packet_session_data
            .reborrow()
            .init_weather_forecast_samples(value.m_weatherForecastSamples.len() as u32);

        for (i, weather) in value.m_weatherForecastSamples.into_iter().enumerate() {
            let mut weather_forecast_sample =
                weather_forecast_samples_list.reborrow().get(i as u32);

            weather_forecast_sample.set_session_type(weather.m_sessionType);
            weather_forecast_sample.set_time_offset(weather.m_timeOffset);
            weather_forecast_sample.set_weather(weather.m_weather);
            weather_forecast_sample.set_track_temperature(weather.m_trackTemperature);
            weather_forecast_sample.set_air_temperature(weather.m_airTemperature);
        }
    }

    message
}
