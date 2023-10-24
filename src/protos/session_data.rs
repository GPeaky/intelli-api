include!(concat!(env!("OUT_DIR"), "/protos.session_data.rs"));

use super::ToProtoMessage;
use crate::dtos::PacketSessionData as BPacketSessionData;

impl ToProtoMessage for BPacketSessionData {
    type ProtoType = PacketSessionData;

    fn to_proto(self) -> Self::ProtoType {
        PacketSessionData {
            m_weather: self.m_weather as u32,
            m_track_temperature: self.m_trackTemperature as i32,
            m_air_temperature: self.m_airTemperature as i32,
            m_total_laps: self.m_totalLaps as u32,
            m_track_length: self.m_trackLength as u32,
            m_session_type: self.m_sessionType as u32,
            m_track_id: self.m_trackId as i32,
            m_formula: self.m_formula as u32,
            m_session_time_left: self.m_sessionTimeLeft as u32,
            m_session_duration: self.m_sessionDuration as u32,
            m_pit_speed_limit: self.m_pitSpeedLimit as u32,
            m_game_paused: self.m_gamePaused as u32,
            m_is_spectating: self.m_isSpectating as u32,
            m_spectator_car_index: self.m_spectatorCarIndex as u32,
            m_sli_pro_native_support: self.m_sliProNativeSupport as u32,
            m_num_marshal_zones: self.m_numMarshalZones as u32,
            m_safety_car_status: self.m_safetyCarStatus as u32,
            m_network_game: self.m_networkGame as u32,
            m_num_weather_forecast_samples: self.m_numWeatherForecastSamples as u32,
            m_forecast_accuracy: self.m_forecastAccuracy as u32,
            m_ai_difficulty: self.m_aiDifficulty as u32,
            m_season_link_identifier: self.m_seasonLinkIdentifier,
            m_weekend_link_identifier: self.m_weekendLinkIdentifier,
            m_session_link_identifier: self.m_sessionLinkIdentifier,
            m_pit_stop_window_ideal_lap: self.m_pitStopWindowIdealLap as u32,
            m_pit_stop_window_latest_lap: self.m_pitStopWindowLatestLap as u32,
            m_pit_stop_rejoin_position: self.m_pitStopRejoinPosition as u32,
            m_steering_assist: self.m_steeringAssist as u32,
            m_braking_assist: self.m_brakingAssist as u32,
            m_gearbox_assist: self.m_gearboxAssist as u32,
            m_pit_assist: self.m_pitAssist as u32,
            m_pit_release_assist: self.m_pitReleaseAssist as u32,
            m_ers_assist: self.m_ERSAssist as u32,
            m_drs_assist: self.m_DRSAssist as u32,
            m_dynamic_racing_line: self.m_dynamicRacingLine as u32,
            m_dynamic_racing_line_type: self.m_dynamicRacingLineType as u32,
            m_game_mode: self.m_gameMode as u32,
            m_rule_set: self.m_ruleSet as u32,
            m_time_of_day: self.m_timeOfDay,
            m_session_length: self.m_sessionLength as u32,
            m_speed_units_lead_player: self.m_speedUnitsLeadPlayer as u32,
            m_temperature_units_lead_player: self.m_temperatureUnitsLeadPlayer as u32,
            m_speed_units_secondary_player: self.m_speedUnitsSecondaryPlayer as u32,
            m_temperature_units_secondary_player: self.m_temperatureUnitsSecondaryPlayer as u32,
            m_num_safety_car_periods: self.m_numSafetyCarPeriods as u32,
            m_num_virtual_safety_car_periods: self.m_numVirtualSafetyCarPeriods as u32,
            m_num_red_flag_periods: self.m_numRedFlagPeriods as u32,

            m_marshal_zones: self
                .m_marshalZones
                .into_iter()
                .map(|marshal_zone| MarshalZone {
                    m_zone_start: marshal_zone.m_zoneStart,
                    m_zone_flag: marshal_zone.m_zoneFlag as i32,
                })
                .collect(),

            m_weather_forecast_samples: self
                .m_weatherForecastSamples
                .into_iter()
                .map(|sample| WeatherForecastSample {
                    m_session_type: sample.m_sessionType as u32,
                    m_time_offset: sample.m_timeOffset as u32,
                    m_weather: sample.m_weather as u32,
                    m_track_temperature: sample.m_trackTemperature as i32,
                    m_track_temperature_change: sample.m_trackTemperatureChange as i32,
                    m_air_temperature: sample.m_airTemperature as i32,
                    m_air_temperature_change: sample.m_airTemperatureChange as i32,
                    m_rain_percentage: sample.m_rainPercentage as u32,
                })
                .collect(),
        }
    }
}
