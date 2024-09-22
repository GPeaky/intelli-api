use std::sync::Arc;

use ahash::{AHashMap, AHashSet};
use ntex::{
    time::interval,
    util::{Bytes, BytesMut},
};
use parking_lot::RwLock;
use prost::Message;
use tokio::sync::{broadcast, oneshot};
use tracing::error;

use crate::{
    config::constants::{GENERAL_INTERVAL, TELEMETRY_INTERVAL},
    structs::{
        protos::*, PacketCarDamageData, PacketCarStatusData, PacketCarTelemetryData,
        PacketEventData, PacketFinalClassificationData, PacketMotionData, PacketParticipantsData,
        PacketSessionData, PacketSessionHistoryData,
    },
};

#[derive(Debug)]
pub struct DriverInfo {
    pub(crate) name: Box<str>,
    pub(crate) team_id: u8,
}

struct F1SessionDataManagerInner {
    driver_info: RwLock<AHashMap<usize, DriverInfo>>,
    general: RwLock<F1GeneralInfo>,
    telemetry: RwLock<F1TelemetryInfo>,
    last_general: RwLock<F1GeneralInfo>,
    last_general_encoded: RwLock<Option<Bytes>>,
    last_telemetry: RwLock<F1TelemetryInfo>,
    team_senders: RwLock<AHashMap<u8, broadcast::Sender<Bytes>>>,
}

pub struct F1SessionDataManager {
    inner: Arc<F1SessionDataManagerInner>,
    stop_sender: Option<oneshot::Sender<()>>,
}

impl F1SessionDataManager {
    pub fn new(tx: broadcast::Sender<Bytes>) -> Self {
        let inner = Arc::new(F1SessionDataManagerInner {
            driver_info: RwLock::new(AHashMap::new()),
            general: RwLock::new(F1GeneralInfo::default()),
            telemetry: RwLock::new(F1TelemetryInfo::default()),
            last_general: RwLock::new(F1GeneralInfo::default()),
            last_general_encoded: RwLock::new(None),
            last_telemetry: RwLock::new(F1TelemetryInfo::default()),
            team_senders: RwLock::new(AHashMap::new()),
        });

        let mut instance = Self {
            inner,
            stop_sender: None,
        };

        instance.spawn_update_task(tx);
        instance
    }

    #[inline(always)]
    pub fn cache(&self) -> Option<Bytes> {
        self.inner.last_general_encoded.read().clone()
    }

    #[allow(unused)]
    pub fn get_team_receiver(&self, team_id: u8) -> Option<broadcast::Receiver<Bytes>> {
        self.inner
            .team_senders
            .read()
            .get(&team_id)
            .map(|sender| sender.subscribe())
    }

    #[inline(always)]
    pub fn push_event(&self, event: &PacketEventData) {
        let driver_info = self.inner.driver_info.read();
        if let Some(event_data) = EventData::from_f1(event, &driver_info) {
            let mut general = self.inner.general.write();
            general
                .events
                .get_or_insert_with(PacketsEventsData::default)
                .events
                .push(event_data);
        }
    }

    #[inline(always)]
    pub fn save_motion(&self, packet: &PacketMotionData) {
        let driver_info = self.inner.driver_info.read();
        let mut general = self.inner.general.write();
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
        let mut general = self.inner.general.write();
        general.update_session(packet);
    }

    #[inline(always)]
    pub fn save_lap_history(&self, packet: &PacketSessionHistoryData) {
        let driver_info = self.inner.driver_info.read();
        if let Some(driver) = driver_info.get(&(packet.car_idx as usize)) {
            let mut general = self.inner.general.write();
            if let Some(player) = general.players.get_mut(driver.name.as_ref()) {
                player.update_session_history(packet);
            }
        }
    }

    #[inline(always)]
    pub fn save_participants(&self, packet: &PacketParticipantsData) {
        let mut driver_info = self.inner.driver_info.write();
        let mut general = self.inner.general.write();
        let mut telemetry = self.inner.telemetry.write();
        let mut team_senders = self.inner.team_senders.write();

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

            team_senders
                .entry(participant.team_id)
                .or_insert_with(|| broadcast::Sender::new(50));
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
        let driver_info = self.inner.driver_info.read();
        let mut general = self.inner.general.write();
        for (i, classification_data) in packet.classification_data.iter().enumerate() {
            if let Some(driver) = driver_info.get(&i) {
                if let Some(player) = general.players.get_mut(driver.name.as_ref()) {
                    player.update_classification_data(classification_data);
                }
            }
        }
    }

    #[inline(always)]
    fn process_telemetry_packet<T, F>(&self, packet_data: &[T], mut process_fn: F)
    where
        F: FnMut(&mut PlayerTelemetry, &T),
    {
        let driver_info = self.inner.driver_info.read();
        let mut telemetry = self.inner.telemetry.write();
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

    #[inline(always)]
    fn spawn_update_task(&mut self, tx: broadcast::Sender<Bytes>) {
        let (stop_sender, mut stop_receiver) = oneshot::channel();
        self.stop_sender = Some(stop_sender);

        let inner = self.inner.clone();

        ntex::rt::spawn(async move {
            let general_interval = interval(GENERAL_INTERVAL);
            let telemetry_interval = interval(TELEMETRY_INTERVAL);

            loop {
                tokio::select! {
                    _ = &mut stop_receiver => break,
                    _ = general_interval.tick() => {
                        Self::send_general_updates(&inner, &tx);
                    }
                    _ = telemetry_interval.tick() => {
                        Self::send_telemetry_updates(&inner);
                    }
                }
            }
        });
    }

    #[inline(always)]
    fn send_general_updates(inner: &Arc<F1SessionDataManagerInner>, tx: &broadcast::Sender<Bytes>) {
        if tx.receiver_count() == 0 {
            return;
        }

        let general = inner.general.read();
        let mut last_general = inner.last_general.write();
        let mut last_general_encoded = inner.last_general_encoded.write();

        if let Some(diff) = Self::diff_general(&general, &last_general) {
            if tx.send(diff.clone()).is_err() {
                error!("Failed to send general update");
            }

            *last_general = general.clone();
            *last_general_encoded = Some(diff);
        }
    }

    #[inline(always)]
    fn send_telemetry_updates(inner: &Arc<F1SessionDataManagerInner>) {
        let driver_info = inner.driver_info.read();
        let telemetry = inner.telemetry.read();
        let mut last_telemetry = inner.last_telemetry.write();
        let team_senders = inner.team_senders.read();

        let mut team_updates: AHashMap<u8, F1TelemetryInfo> = AHashMap::new();

        let active_teams: AHashSet<u8> = team_senders
            .iter()
            .filter(|(_, sender)| sender.receiver_count() > 0)
            .map(|(&team_id, _)| team_id)
            .collect();

        for (_, driver) in driver_info.iter() {
            if active_teams.contains(&driver.team_id) {
                if let (Some(current_player_telemetry), Some(last_player_telemetry)) = (
                    telemetry.player_telemetry.get(driver.name.as_ref()),
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

        *last_telemetry = telemetry.clone();
    }

    fn diff_general(_current: &F1GeneralInfo, _last: &F1GeneralInfo) -> Option<Bytes> {
        None
    }

    fn diff_player_telemetry(
        _current: &PlayerTelemetry,
        _last: &PlayerTelemetry,
    ) -> Option<PlayerTelemetry> {
        None
    }
}

impl Drop for F1SessionDataManager {
    fn drop(&mut self) {
        if let Some(sender) = self.stop_sender.take() {
            let _ = sender.send(());
        }
    }
}
