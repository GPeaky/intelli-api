include!(concat!(
    env!("OUT_DIR"),
    "/generated/session_data_generated.rs"
));

use super::ToFlatBufferMessage;
use crate::dtos::PacketSessionData as BPacketSessionData;
use flatbuffers::FlatBufferBuilder;
use protos::session_data::{
    MarshalZone, MarshalZoneArgs, PacketSessionData, PacketSessionDataArgs, WeatherForecastSample,
    WeatherForecastSampleArgs,
};

impl ToFlatBufferMessage for BPacketSessionData {
    fn to_flatbuffer(self) -> Vec<u8> {
        let mut builder = FlatBufferBuilder::new();

        let weather_forecast_samples_offsets = self
            .m_weatherForecastSamples
            .iter()
            .map(|sample| {
                WeatherForecastSample::create(
                    &mut builder,
                    &WeatherForecastSampleArgs {
                        m_session_type: sample.m_sessionType,
                        m_time_offset: sample.m_timeOffset,
                        m_weather: sample.m_weather,
                        m_track_temperature: sample.m_trackTemperature,
                        m_track_temperature_change: sample.m_trackTemperatureChange,
                        m_air_temperature: sample.m_airTemperature,
                        m_air_temperature_change: sample.m_airTemperatureChange,
                        m_rain_percentage: sample.m_rainPercentage,
                    },
                )
            })
            .collect::<Vec<_>>();

        let weather_forecast_samples_vector =
            builder.create_vector(&weather_forecast_samples_offsets);

        let marshal_zones_offsets: Vec<_> = self
            .m_marshalZones
            .iter()
            .map(|marshal_zone| {
                MarshalZone::create(
                    &mut builder,
                    &MarshalZoneArgs {
                        m_zone_start: marshal_zone.m_zoneStart,
                        m_zone_flag: marshal_zone.m_zoneFlag,
                    },
                )
            })
            .collect();

        let marshal_zones_vector = builder.create_vector(&marshal_zones_offsets);

        let session_data = PacketSessionData::create(
            &mut builder,
            &PacketSessionDataArgs {
                m_weather: self.m_weather,
                m_track_temperature: self.m_trackTemperature,
                m_air_temperature: self.m_airTemperature,
                m_total_laps: self.m_totalLaps,
                m_track_length: self.m_trackLength,
                m_session_type: self.m_sessionType,
                m_track_id: self.m_trackId,
                m_formula: self.m_formula,
                m_session_time_left: self.m_sessionTimeLeft,
                m_session_duration: self.m_sessionDuration,
                m_pit_speed_limit: self.m_pitSpeedLimit,
                m_game_paused: self.m_gamePaused,
                m_is_spectating: self.m_isSpectating,
                m_spectator_car_index: self.m_spectatorCarIndex,
                m_sli_pro_native_support: self.m_sliProNativeSupport,
                m_num_marshal_zones: self.m_numMarshalZones,
                m_marshal_zones: Some(marshal_zones_vector),
                m_safety_car_status: self.m_safetyCarStatus,
                m_network_game: self.m_networkGame,
                m_num_weather_forecast_samples: self.m_numWeatherForecastSamples,
                m_weather_forecast_samples: Some(weather_forecast_samples_vector),
                m_forecast_accuracy: self.m_forecastAccuracy,
                m_ai_difficulty: self.m_aiDifficulty,
                m_season_link_identifier: self.m_seasonLinkIdentifier,
                m_weekend_link_identifier: self.m_weekendLinkIdentifier,
                m_session_link_identifier: self.m_sessionLinkIdentifier,
                m_pit_stop_window_ideal_lap: self.m_pitStopWindowIdealLap,
                m_pit_stop_window_latest_lap: self.m_pitStopWindowLatestLap,
                m_pit_stop_rejoin_position: self.m_pitStopRejoinPosition,
                m_steering_assist: self.m_steeringAssist,
                m_braking_assist: self.m_brakingAssist,
                m_gearbox_assist: self.m_gearboxAssist,
                m_pit_assist: self.m_pitAssist,
                m_pit_release_assist: self.m_pitReleaseAssist,
                m_ers_assist: self.m_ERSAssist,
                m_drs_assist: self.m_DRSAssist,
                m_dynamic_racing_line: self.m_dynamicRacingLine,
                m_dynamic_racing_line_type: self.m_dynamicRacingLineType,
                m_game_mode: self.m_gameMode,
                m_rule_set: self.m_ruleSet,
                m_time_of_day: self.m_timeOfDay,
                m_session_length: self.m_sessionLength,
                m_speed_units_lead_player: self.m_speedUnitsLeadPlayer,
                m_temperature_units_lead_player: self.m_temperatureUnitsLeadPlayer,
                m_speed_units_secondary_player: self.m_speedUnitsSecondaryPlayer,
                m_temperature_inits_secondary_player: self.m_temperatureUnitsSecondaryPlayer,
                m_num_safety_car_periods: self.m_numSafetyCarPeriods,
                m_num_virtual_safety_car_periods: self.m_numVirtualSafetyCarPeriods,
                m_num_red_flag_periods: self.m_numRedFlagPeriods,
            },
        );

        builder.finish(session_data, None);
        builder.finished_data().to_vec()
    }
}
