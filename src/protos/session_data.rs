use crate::structs::PacketSessionData as BPacketSessionData;

use super::ToProtoMessage;

include!(concat!(env!("OUT_DIR"), "/protos.session_data.rs"));

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
            num_marshal_zones: self.num_marshal_zones as u32,
            safety_car_status: self.safety_car_status as u32,
            network_game: self.network_game as u32,
            forecast_accuracy: self.forecast_accuracy as u32,
            ai_difficulty: self.ai_difficulty as u32,
            season_link_identifier: self.season_link_identifier,
            weekend_link_identifier: self.weekend_link_identifier,
            session_link_identifier: self.session_link_identifier,
            game_mode: self.game_mode as u32,
            rule_set: self.rule_set as u32,
            time_of_day: self.time_of_day,
            session_length: self.session_length as u32,
            num_safety_car_periods: self.num_safety_car_periods as u32,
            num_virtual_safety_car_periods: self.num_virtual_safety_car_periods as u32,
            num_red_flag_periods: self.num_red_flag_periods as u32,
            rain_percentage: self
                .weather_forecast_samples
                .iter()
                .take(self.num_weather_forecast_samples as usize)
                .map(|weather| weather.rain_percentage as u16)
                .sum::<u16>() as u32
                / self.num_weather_forecast_samples as u32,

            marshall_zones: self
                .marshal_zones
                .iter()
                .take(self.num_marshal_zones as usize)
                .map(|marshal_zone| MarshalZone {
                    zone_start: marshal_zone.zone_start,
                    zone_flag: marshal_zone.zone_flag as i32,
                })
                .collect(),
        })
    }
}
