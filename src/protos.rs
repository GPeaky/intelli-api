pub mod car_motion_data {
    include!(concat!(env!("OUT_DIR"), "/protos.car_motion_data.rs"));

    impl From<crate::dtos::CarMotionData> for CarMotionData {
        fn from(value: crate::dtos::CarMotionData) -> Self {
            Self {
                m_world_position_x: value.m_worldPositionX,
                m_world_position_y: value.m_worldPositionY,
                m_world_position_z: value.m_worldPositionZ,
                m_world_velocity_x: value.m_worldVelocityX,
                m_world_velocity_y: value.m_worldVelocityY,
                m_world_velocity_z: value.m_worldVelocityZ,
                m_world_forward_dir_x: value.m_worldForwardDirX as i32,
                m_world_forward_dir_y: value.m_worldForwardDirY as i32,
                m_world_forward_dir_z: value.m_worldForwardDirZ as i32,
                m_world_right_dir_x: value.m_worldRightDirX as i32,
                m_world_right_dir_y: value.m_worldRightDirY as i32,
                m_world_right_dir_z: value.m_worldRightDirZ as i32,
                m_g_force_lateral: value.m_gForceLateral,
                m_g_force_longitudinal: value.m_gForceLongitudinal,
                m_yaw: value.m_yaw,
                m_pitch: value.m_pitch,
                m_roll: value.m_roll,
                m_g_force_vertical: value.m_gForceVertical,
            }
        }
    }
    impl From<Box<crate::dtos::PacketMotionData>> for PacketMotionData {
        fn from(packet: Box<crate::dtos::PacketMotionData>) -> Self {
            Self {
                m_car_motion_data: packet
                    .m_carMotionData
                    .into_iter()
                    .map(|car_motion_data| car_motion_data.into())
                    .collect(),
            }
        }
    }
}

pub mod event_data {
    include!(concat!(env!("OUT_DIR"), "/protos.event_data.rs"));

    // impl From<crate::dtos::PacketEventData>for EventData
}

pub mod final_classification {
    include!(concat!(env!("OUT_DIR"), "/protos.final_classification.rs"));
}

pub mod participants {
    include!(concat!(env!("OUT_DIR"), "/protos.participants.rs"));
}

pub mod session_data {
    include!(concat!(env!("OUT_DIR"), "/protos.session_data.rs"));

    impl From<crate::dtos::MarshalZone> for MarshalZone {
        fn from(value: crate::dtos::MarshalZone) -> Self {
            Self {
                m_zone_start: value.m_zoneStart,
                m_zone_flag: value.m_zoneFlag as i32,
            }
        }
    }

    impl From<crate::dtos::WeatherForecastSample> for WeatherForecastSample {
        fn from(value: crate::dtos::WeatherForecastSample) -> Self {
            Self {
                m_session_type: value.m_sessionType as u32,
                m_time_offset: value.m_timeOffset as u32,
                m_weather: value.m_weather as u32,
                m_track_temperature: value.m_trackTemperature as i32,
                m_track_temperature_change: value.m_trackTemperatureChange as i32,
                m_air_temperature: value.m_airTemperature as i32,
                m_air_temperature_change: value.m_airTemperatureChange as i32,
                m_rain_percentage: value.m_rainPercentage as u32,
            }
        }
    }

    impl From<Box<crate::dtos::PacketSessionData>> for PacketSessionData {
        fn from(value: Box<crate::dtos::PacketSessionData>) -> Self {
            Self {
                m_weather: value.m_weather as u32,
                m_track_temperature: value.m_trackTemperature as i32,
                m_air_temperature: value.m_airTemperature as i32,
                m_total_laps: value.m_totalLaps as u32,
                m_track_length: value.m_trackLength as u32,
                m_session_type: value.m_sessionType as u32,
                m_track_id: value.m_trackId as i32,
                m_formula: value.m_formula as u32,
                m_session_time_left: value.m_sessionTimeLeft as u32,
                m_session_duration: value.m_sessionDuration as u32,
                m_pit_speed_limit: value.m_pitSpeedLimit as u32,
                m_game_paused: value.m_gamePaused as u32,
                m_is_spectating: value.m_isSpectating as u32,
                m_spectator_car_index: value.m_spectatorCarIndex as u32,
                m_sli_pro_native_support: value.m_sliProNativeSupport as u32,
                m_num_marshal_zones: value.m_numMarshalZones as u32,
                m_marshal_zones: value
                    .m_marshalZones
                    .into_iter()
                    .map(|marshal_zone| marshal_zone.into())
                    .collect(),
                m_safety_car_status: value.m_safetyCarStatus as u32,
                m_network_game: value.m_networkGame as u32,
                m_num_weather_forecast_samples: value.m_numWeatherForecastSamples as u32,
                m_weather_forecast_samples: value
                    .m_weatherForecastSamples
                    .into_iter()
                    .map(|weather_forecast_sample| weather_forecast_sample.into())
                    .collect(),
                m_forecast_accuracy: value.m_forecastAccuracy as u32,
                m_ai_difficulty: value.m_aiDifficulty as u32,
                m_season_link_identifier: value.m_seasonLinkIdentifier,
                m_weekend_link_identifier: value.m_weekendLinkIdentifier,
                m_session_link_identifier: value.m_sessionLinkIdentifier,
                m_pit_stop_window_ideal_lap: value.m_pitStopWindowIdealLap as u32,
                m_pit_stop_window_latest_lap: value.m_pitStopWindowLatestLap as u32,
                m_pit_stop_rejoin_position: value.m_pitStopRejoinPosition as u32,
                m_steering_assist: value.m_steeringAssist as u32,
                m_braking_assist: value.m_brakingAssist as u32,
                m_gearbox_assist: value.m_gearboxAssist as u32,
                m_pit_assist: value.m_pitAssist as u32,
                m_pit_release_assist: value.m_pitReleaseAssist as u32,
                m_ers_assist: value.m_ERSAssist as u32,
                m_drs_assist: value.m_DRSAssist as u32,
                m_dynamic_racing_line: value.m_dynamicRacingLine as u32,
                m_dynamic_racing_line_type: value.m_dynamicRacingLineType as u32,
                m_game_mode: value.m_gameMode as u32,
                m_rule_set: value.m_ruleSet as u32,
                m_time_of_day: value.m_timeOfDay,
                m_session_length: value.m_sessionLength as u32,
                m_speed_units_lead_player: value.m_speedUnitsLeadPlayer as u32,
                m_temperature_units_lead_player: value.m_temperatureUnitsLeadPlayer as u32,
                m_speed_units_secondary_player: value.m_speedUnitsSecondaryPlayer as u32,
                m_temperature_units_secondary_player: value.m_temperatureUnitsSecondaryPlayer
                    as u32,
                m_num_safety_car_periods: value.m_numSafetyCarPeriods as u32,
                m_num_virtual_safety_car_periods: value.m_numVirtualSafetyCarPeriods as u32,
                m_num_red_flag_periods: value.m_numRedFlagPeriods as u32,
            }
        }
    }
}

pub mod session_history {
    include!(concat!(env!("OUT_DIR"), "/protos.session_history.rs"));
}
