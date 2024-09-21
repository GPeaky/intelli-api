use crate::structs::{
    protos::*, PacketCarDamageData, PacketCarStatusData, PacketCarTelemetryData, PacketEventData,
    PacketFinalClassificationData, PacketMotionData, PacketParticipantsData, PacketSessionData,
    PacketSessionHistoryData,
};
use ahash::AHashMap;
use tracing_log::log::error;



// TODO: Prune data at the end of the session
pub struct F1SessionDataManager {
    id_to_name: AHashMap<usize, Box<str>>,
    general: F1GeneralInfo,
    telemetry: F1TelemetryInfo,
}

#[allow(unused)]
impl F1SessionDataManager {
    pub fn new() -> Self {
        Self {
            id_to_name: AHashMap::new(),
            general: F1GeneralInfo::default(),
            telemetry: F1TelemetryInfo::default(),
        }
    }

    // TODO: Finish this impl and check if all the data is being saved as it should
    pub fn push_event(&self, _event: &PacketEventData) {}

    // Not using process_general_packet cause a middle check
    pub fn save_motion(&mut self, packet: &PacketMotionData) {
        for i in 0..packet.car_motion_data.len() {
            let motion_data = &packet.car_motion_data[i];

            if motion_data.world_position_x == 0f32 {
                continue;
            }

            if let Some(steam_name) = self.id_to_name.get(&i) {
                if let Some(player) = self.general.players.get_mut(steam_name.as_ref()) {
                    player.update_car_motion(motion_data);
                }
            }
        }
    }

    pub fn save_session(&mut self, packet: &PacketSessionData) {
        self.general.update_session(packet);
    }

    pub fn save_lap_history(&mut self, packet: &PacketSessionHistoryData) {
        if let Some(steam_name) = self.id_to_name.get(&(packet.car_idx as usize)) {
            if let Some(player) = self.general.players.get_mut(steam_name.as_ref()) {
                player.update_session_history(packet);
            }
        }
    }

    pub fn save_participants(&mut self, packet: &PacketParticipantsData) {
        for i in 0..packet.num_active_cars as usize {
            let Some(participant) = packet.participants.get(i) else {
                error!(
                    "num_active_cars ({}) exceeds array bound ({})",
                    packet.num_active_cars,
                    packet.participants.len()
                );

                break;
            };

            let steam_name = self
                .id_to_name
                .entry(i)
                .or_insert_with(|| participant.steam_name().unwrap().into());

            let steam_name = steam_name.as_ref();

            if self
                .telemetry
                .player_telemetry
                .get_mut(steam_name)
                .is_none()
            {
                self.telemetry
                    .player_telemetry
                    .insert(steam_name.to_string(), Default::default());
            }

            match self.general.players.get_mut(steam_name) {
                Some(player) => {
                    player.update_participant_info(participant);
                }

                None => {
                    let mut new_player = PlayerInfo::default();

                    new_player.update_participant_info(participant);

                    self.general
                        .players
                        .insert(steam_name.to_string(), new_player);
                }
            }
        }
    }

    pub fn save_car_damage(&mut self, packet: &PacketCarDamageData) {
        self.process_telemetry_packet(&packet.car_damage_data, |player_telemetry, data| {
            player_telemetry.update_car_damage(data);
        });
    }

    pub fn save_car_status(&mut self, packet: &PacketCarStatusData) {
        self.process_telemetry_packet(&packet.car_status_data, |player_telemetry, data| {
            player_telemetry.update_car_status(data);
        });
    }

    pub fn save_car_telemetry(&mut self, packet: &PacketCarTelemetryData) {
        self.process_telemetry_packet(&packet.car_telemetry_data, |player_telemetry, data| {
            player_telemetry.update_car_telemetry(data);
        });
    }

    pub fn save_final_classification(&mut self, packet: &PacketFinalClassificationData) {
        self.process_general_packet(
            &packet.classification_data,
            |player_info, classification_data| {
                player_info.update_classification_data(classification_data);
            },
        );
    }

    #[inline(always)]
    fn process_general_packet<T, F>(&mut self, packet_data: &[T], mut process_fn: F)
    where
        F: FnMut(&mut PlayerInfo, &T),
    {
        for (i, data) in packet_data.iter().enumerate() {
            if let Some(steam_name) = self.id_to_name.get(&i) {
                if let Some(player) = self.general.players.get_mut(steam_name.as_ref()) {
                    process_fn(player, data);
                }
            }
        }
    }

    #[inline(always)]
    fn process_telemetry_packet<T, F>(&mut self, packet_data: &[T], mut process_fn: F)
    where
        F: FnMut(&mut PlayerTelemetry, &T),
    {
        for (i, data) in packet_data.iter().enumerate() {
            if let Some(steam_name) = self.id_to_name.get(&i) {
                if let Some(player_telemetry) =
                    self.telemetry.player_telemetry.get_mut(steam_name.as_ref())
                {
                    process_fn(player_telemetry, data);
                }
            }
        }
    }
}
