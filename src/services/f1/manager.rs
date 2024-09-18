use ahash::AHashMap;

use crate::structs::{
    protos::*, PacketEventData, PacketMotionData, PacketParticipantsData, PacketSessionData,
    PacketSessionHistoryData,
};

struct F1DataManager {
    id_to_name: AHashMap<usize, Box<str>>,
    general: F1GeneralInfo,
    telemetry: F1TelemetryInfo,
}

impl F1DataManager {
    pub fn new() -> Self {
        F1DataManager {
            id_to_name: AHashMap::new(),
            general: F1GeneralInfo::default(),
            telemetry: F1TelemetryInfo::default(),
        }
    }

    #[allow(unused)]
    pub fn push_event(&self, _event: &PacketEventData) {
        // TODO: Convert from PacketEventData and push it
    }

    #[allow(unused)]
    pub fn save_motion_packet(&mut self, packet: &PacketMotionData) {
        for index in 0..packet.car_motion_data.len() {
            let Some(motion_data) = packet.car_motion_data.get(index) else {
                continue;
            };

            if let Some(steam_name) = self.id_to_name.get(&index) {
                if let Some(player) = self.general.players.get_mut(steam_name.as_ref()) {
                    player.update_car_motion(motion_data);
                }
            }
        }
    }

    #[allow(unused)]
    pub fn save_session_packet(&mut self, packet: &PacketSessionData) {
        self.general.update_session(packet);
    }

    #[allow(unused)]
    pub fn save_lap_history(&mut self, packet: &PacketSessionHistoryData) {
        let player = self
            .id_to_name
            .get(&(packet.car_idx as usize))
            .and_then(|steam_name| self.general.players.get_mut(steam_name.as_ref()));

        if let Some(player) = player {
            player.update_session_history(packet);
        }
    }

    #[allow(unused)]
    pub fn save_participants_packet(&mut self, packet: &PacketParticipantsData) {
        for i in 0..packet.num_active_cars as usize {
            let Some(participant) = packet.participants.get(i) else {
                continue;
            };

            let steam_name = match self.id_to_name.get(&i) {
                Some(steam) => steam.as_ref(),
                None => {
                    let name = participant.steam_name().unwrap();
                    self.id_to_name.insert(i, name.into());

                    name
                }
            };

            let player_info = self
                .general
                .players
                .entry(steam_name.to_owned()) // Avoid creating string
                .or_default();

            player_info.update_participant_info(participant);
        }
    }

    // TODO: add final_classification data an telemetry
}
