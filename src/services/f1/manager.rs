use crate::{
    config::constants::{GENERAL_INTERVAL, TELEMETRY_INTERVAL},
    structs::{
        protos::*, PacketCarDamageData, PacketCarStatusData, PacketCarTelemetryData,
        PacketEventData, PacketFinalClassificationData, PacketMotionData, PacketParticipantsData,
        PacketSessionData, PacketSessionHistoryData,
    },
};
use ahash::{AHashMap, AHashSet};
use ntex::{
    time::interval,
    util::{Bytes, BytesMut},
};
use parking_lot::RwLock;
use prost::Message;
use std::sync::Arc;
use tokio::sync::{
    broadcast::{Receiver, Sender},
    oneshot,
};
use tracing::error;

#[derive(Debug)]
struct DriverInfo {
    name: Box<str>,
    team_id: u8,
}

#[derive(Debug)]
pub struct F1SessionDataManager {
    driver_info: Arc<RwLock<AHashMap<usize, DriverInfo>>>,
    general: Arc<RwLock<F1GeneralInfo>>,
    telemetry: Arc<RwLock<F1TelemetryInfo>>,
    last_general: Arc<RwLock<F1GeneralInfo>>,
    last_telemetry: Arc<RwLock<F1TelemetryInfo>>,
    team_senders: Arc<RwLock<AHashMap<u8, Sender<Bytes>>>>,
    stop_sender: Option<oneshot::Sender<()>>,
}

impl F1SessionDataManager {
    pub fn new(tx: Sender<Bytes>) -> Self {
        let mut instance = Self {
            driver_info: Arc::new(RwLock::new(AHashMap::new())),
            general: Arc::new(RwLock::new(F1GeneralInfo::default())),
            telemetry: Arc::new(RwLock::new(F1TelemetryInfo::default())),
            last_general: Arc::new(RwLock::new(F1GeneralInfo::default())),
            last_telemetry: Arc::new(RwLock::new(F1TelemetryInfo::default())),
            team_senders: Arc::new(RwLock::new(AHashMap::new())),
            stop_sender: None,
        };

        instance.spawn_update_task(tx);
        instance
    }

    #[allow(unused)]
    pub fn get_team_receiver(&self, team_id: u8) -> Option<Receiver<Bytes>> {
        let team_senders = self.team_senders.read();
        team_senders.get(&team_id).map(|sender| sender.subscribe())
    }

    #[inline(always)]
    #[allow(unused)]
    pub fn push_event(&self, _event: &PacketEventData) {
        // TODO: Implement event handling
    }

    #[inline(always)]
    pub fn save_motion(&self, packet: &PacketMotionData) {
        let driver_info = self.driver_info.read();
        let mut general = self.general.write();

        for (i, motion_data) in packet.car_motion_data.iter().enumerate() {
            if motion_data.world_position_x == 0f32 {
                continue;
            }

            if let Some(driver) = driver_info.get(&i) {
                if let Some(player) = general.players.get_mut(driver.name.as_ref()) {
                    player.update_car_motion(motion_data);
                }
            }
        }
    }

    #[inline(always)]
    pub fn save_session(&self, packet: &PacketSessionData) {
        self.general.write().update_session(packet);
    }

    #[inline(always)]
    pub fn save_lap_history(&self, packet: &PacketSessionHistoryData) {
        let driver_info = self.driver_info.read();
        let mut general = self.general.write();

        if let Some(driver) = driver_info.get(&(packet.car_idx as usize)) {
            if let Some(player) = general.players.get_mut(driver.name.as_ref()) {
                player.update_session_history(packet);
            }
        }
    }

    #[inline(always)]
    pub fn save_participants(&self, packet: &PacketParticipantsData) {
        let mut driver_info = self.driver_info.write();
        let mut general = self.general.write();
        let mut telemetry = self.telemetry.write();

        for i in 0..packet.num_active_cars as usize {
            let Some(participant) = packet.participants.get(i) else {
                error!(
                    "num_active_cars ({}) exceeds array bound ({})",
                    packet.num_active_cars,
                    packet.participants.len()
                );
                break;
            };

            let steam_name = match participant.steam_name() {
                Some(name) if name != "Player" => name,
                _ => continue,
            };

            driver_info.entry(i).or_insert_with(|| DriverInfo {
                name: steam_name.into(),
                team_id: participant.team_id,
            });

            general
                .players
                .entry(steam_name.to_string())
                .and_modify(|player| player.update_participant_info(participant))
                .or_insert_with(|| {
                    let mut new_player = PlayerInfo::default();
                    new_player.update_participant_info(participant);
                    telemetry
                        .player_telemetry
                        .entry(steam_name.to_string())
                        .or_default();
                    new_player
                });

            // Ensure there's a sender for this team
            self.team_senders
                .write()
                .entry(participant.team_id)
                .or_insert_with(|| {
                    Sender::new(100) // Adjust buffer size as needed
                });
        }
    }

    #[inline(always)]
    pub fn save_car_damage(&self, packet: &PacketCarDamageData) {
        self.process_telemetry_packet(&packet.car_damage_data, |player_telemetry, data| {
            player_telemetry.update_car_damage(data);
        });
    }

    #[inline(always)]
    pub fn save_car_status(&self, packet: &PacketCarStatusData) {
        self.process_telemetry_packet(&packet.car_status_data, |player_telemetry, data| {
            player_telemetry.update_car_status(data);
        });
    }

    #[inline(always)]
    pub fn save_car_telemetry(&self, packet: &PacketCarTelemetryData) {
        self.process_telemetry_packet(&packet.car_telemetry_data, |player_telemetry, data| {
            player_telemetry.update_car_telemetry(data);
        });
    }

    #[inline(always)]
    pub fn save_final_classification(&self, packet: &PacketFinalClassificationData) {
        self.process_general_packet(
            &packet.classification_data,
            |player_info, classification_data| {
                player_info.update_classification_data(classification_data);
            },
        );
    }

    fn spawn_update_task(&mut self, tx: Sender<Bytes>) {
        let (stop_sender, mut stop_receiver) = oneshot::channel();
        self.stop_sender = Some(stop_sender);

        let driver_info = self.driver_info.clone();
        let general = self.general.clone();
        let telemetry = self.telemetry.clone();
        let last_general = self.last_general.clone();
        let last_telemetry = self.last_telemetry.clone();
        let team_senders = self.team_senders.clone();

        ntex::rt::spawn(async move {
            let general_interval = interval(GENERAL_INTERVAL);
            let telemetry_interval = interval(TELEMETRY_INTERVAL);

            loop {
                tokio::select! {
                    _ = &mut stop_receiver => break,
                    _ = general_interval.tick() => {
                        Self::send_general_updates(&general, &last_general, &tx);
                    }
                    _ = telemetry_interval.tick() => {
                        Self::send_telemetry_updates(&driver_info, &telemetry, &last_telemetry, &team_senders);
                    }
                }
            }
        });
    }

    #[inline(always)]
    fn send_general_updates(
        general: &Arc<RwLock<F1GeneralInfo>>,
        last_general: &Arc<RwLock<F1GeneralInfo>>,
        tx: &Sender<Bytes>,
    ) {
        if tx.receiver_count() == 0 {
            return;
        }

        let current_general = general.read();
        let mut last_general = last_general.write();

        if let Some(diff) = Self::diff_general(&current_general, &last_general) {
            if tx.send(diff).is_err() {
                error!("Failed to send general update");
            }
            *last_general = current_general.clone();
        }
    }

    #[inline(always)]
    fn send_telemetry_updates(
        driver_info: &Arc<RwLock<AHashMap<usize, DriverInfo>>>,
        telemetry: &Arc<RwLock<F1TelemetryInfo>>,
        last_telemetry: &Arc<RwLock<F1TelemetryInfo>>,
        team_senders: &Arc<RwLock<AHashMap<u8, Sender<Bytes>>>>,
    ) {
        let driver_info = driver_info.read();
        let current_telemetry = telemetry.read();
        let mut last_telemetry = last_telemetry.write();
        let team_senders = team_senders.read();

        let mut team_updates: AHashMap<u8, F1TelemetryInfo> = AHashMap::new();

        let active_teams: AHashSet<u8> = team_senders
            .iter()
            .filter(|(_, sender)| sender.receiver_count() > 0)
            .map(|(&team_id, _)| team_id)
            .collect();

        for (_, driver) in driver_info.iter() {
            if active_teams.contains(&driver.team_id) {
                if let (Some(current_player_telemetry), Some(last_player_telemetry)) = (
                    current_telemetry.player_telemetry.get(driver.name.as_ref()),
                    last_telemetry.player_telemetry.get(driver.name.as_ref()),
                ) {
                    if let Some(diff_telemetry) =
                        Self::diff_player_telemetry(current_player_telemetry, last_player_telemetry)
                    {
                        team_updates
                            .entry(driver.team_id)
                            .or_default()
                            .player_telemetry
                            .insert(driver.name.to_string(), diff_telemetry);
                    }
                }
            }
        }

        for (team_id, update) in team_updates {
            if let Some(sender) = team_senders.get(&team_id) {
                let mut buf = BytesMut::with_capacity(update.encoded_len());
                update.encode_raw(&mut buf);

                if sender.send(buf.freeze()).is_err() {
                    error!("Failed to send telemetry update for team {}", team_id);
                }
            }
        }

        *last_telemetry = current_telemetry.clone();
    }

    #[inline(always)]
    fn diff_general(_current: &F1GeneralInfo, _last: &F1GeneralInfo) -> Option<Bytes> {
        // TODO: Implement efficient diff logic
        unimplemented!()
    }

    #[inline(always)]
    fn diff_player_telemetry(
        _current: &PlayerTelemetry,
        _last: &PlayerTelemetry,
    ) -> Option<PlayerTelemetry> {
        // TODO: Implement efficient diff logic
        unimplemented!()
    }

    #[inline(always)]
    fn process_general_packet<T, F>(&self, packet_data: &[T], mut process_fn: F)
    where
        F: FnMut(&mut PlayerInfo, &T),
    {
        let driver_info = self.driver_info.read();
        let mut general = self.general.write();

        for (i, data) in packet_data.iter().enumerate() {
            if let Some(driver) = driver_info.get(&i) {
                if let Some(player) = general.players.get_mut(driver.name.as_ref()) {
                    process_fn(player, data);
                }
            }
        }
    }

    #[inline(always)]
    fn process_telemetry_packet<T, F>(&self, packet_data: &[T], mut process_fn: F)
    where
        F: FnMut(&mut PlayerTelemetry, &T),
    {
        let driver_info = self.driver_info.read();
        let mut telemetry = self.telemetry.write();

        for (i, data) in packet_data.iter().enumerate() {
            if let Some(driver) = driver_info.get(&i) {
                if let Some(player_telemetry) =
                    telemetry.player_telemetry.get_mut(driver.name.as_ref())
                {
                    process_fn(player_telemetry, data);
                }
            }
        }
    }
}

impl Drop for F1SessionDataManager {
    fn drop(&mut self) {
        if let Some(sender) = self.stop_sender.take() {
            let _ = sender.send(());
        }
    }
}
