include!(concat!(env!("OUT_DIR"), "/protos.session_data.rs"));

use super::ToProtoMessage;
use crate::dtos::PacketSessionData as BPacketSessionData;

impl ToProtoMessage for BPacketSessionData {
    type ProtoType = PacketSessionData;

    fn to_proto(&self) -> Option<Self::ProtoType> {
        Some(PacketSessionData {
            weather: self.weather as u32,
            track_temperature: self.track_temperature as i32,
            air_temperature: self.air_temperature as i32,
            total_laps: self.total_laps as u32,
            track_length: self.track_length as u32,
            session_type: self.session_type as u32,
            track_id: self.track_id as i32,
            formula: self.formula as u32,
            session_time_left: self.session_time_left as u32,
            session_duration: self.session_duration as u32,
            pit_speed_limit: self.pit_speed_limit as u32,
            game_paused: self.game_paused as u32,
            is_spectating: self.is_spectating as u32,
            spectator_car_index: self.spectator_car_index as u32,
            sli_pro_native_support: self.sli_pro_native_support as u32,
            num_marshal_zones: self.num_marshal_zones as u32,
            safety_car_status: self.safety_car_status as u32,
            network_game: self.network_game as u32,
            num_weather_forecast_samples: self.num_weather_forecast_samples as u32,
            forecast_accuracy: self.forecast_accuracy as u32,
            ai_difficulty: self.ai_difficulty as u32,
            season_link_identifier: self.season_link_identifier,
            weekend_link_identifier: self.weekend_link_identifier,
            session_link_identifier: self.session_link_identifier,
            pit_stop_window_ideal_lap: self.pit_stop_window_ideal_lap as u32,
            pit_stop_window_latest_lap: self.pit_stop_window_latest_lap as u32,
            pit_stop_rejoin_position: self.pit_stop_rejoin_position as u32,
            steering_assist: self.steering_assist as u32,
            braking_assist: self.braking_assist as u32,
            gearbox_assist: self.gearbox_assist as u32,
            pit_assist: self.pit_assist as u32,
            pit_release_assist: self.pit_release_assist as u32,
            ers_assist: self.ers_assist as u32,
            drs_assist: self.drs_assist as u32,
            dynamic_racing_line: self.dynamic_racing_line as u32,
            dynamic_racing_line_type: self.dynamic_racing_line_type as u32,
            game_mode: self.game_mode as u32,
            rule_set: self.rule_set as u32,
            time_of_day: self.time_of_day,
            session_length: self.session_length as u32,
            speed_units_lead_player: self.speed_units_lead_player as u32,
            temperature_units_lead_player: self.temperature_units_lead_player as u32,
            speed_units_secondary_player: self.speed_units_secondary_player as u32,
            temperature_units_secondary_player: self.temperature_units_secondary_player as u32,
            num_safety_car_periods: self.num_safety_car_periods as u32,
            num_virtual_safety_car_periods: self.num_virtual_safety_car_periods as u32,
            num_red_flag_periods: self.num_red_flag_periods as u32,

            marshal_zones: self
                .marshal_zones
                .into_iter()
                .map(|marshal_zone| MarshalZone {
                    zone_start: marshal_zone.zone_start,
                    zone_flag: marshal_zone.zone_flag as i32,
                })
                .collect(),

            weather_forecast_samples: self
                .weather_forecast_samples
                .into_iter()
                .map(|sample| WeatherForecastSample {
                    session_type: sample.session_type as u32,
                    time_offset: sample.time_offset as u32,
                    weather: sample.weather as u32,
                    track_temperature: sample.track_temperature as i32,
                    track_temperature_change: sample.track_temperature_change as i32,
                    air_temperature: sample.air_temperature as i32,
                    air_temperature_change: sample.air_temperature_change as i32,
                    rain_percentage: sample.rain_percentage as u32,
                })
                .collect(),
        })
    }
}
