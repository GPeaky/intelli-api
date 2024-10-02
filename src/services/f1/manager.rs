use std::{ops::Deref, sync::Arc};

use ahash::{AHashMap, AHashSet};
use ntex::{
    time::interval,
    util::{Bytes, BytesMut},
};
use parking_lot::{Mutex, RwLock};
use prost::Message;
use tokio::sync::{
    broadcast::{Receiver, Sender},
    oneshot,
};
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
    pub name: Box<str>,
    pub team_id: u8,
}

#[derive(Debug)]
pub struct F1SessionDataManagerInner {
    driver_info: RwLock<AHashMap<usize, DriverInfo>>,
    general: RwLock<F1GeneralInfo>,
    telemetry: RwLock<F1TelemetryInfo>,
    last_general: RwLock<F1GeneralInfo>,
    last_general_encoded: RwLock<Option<Bytes>>,
    last_telemetry: RwLock<F1TelemetryInfo>,
    team_senders: RwLock<AHashMap<u8, Sender<Bytes>>>,
    stop_sender: Mutex<Option<oneshot::Sender<()>>>,
}

#[derive(Clone)]
pub struct F1SessionDataManager {
    inner: Arc<F1SessionDataManagerInner>,
}

impl Deref for F1SessionDataManager {
    type Target = F1SessionDataManagerInner;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl F1SessionDataManager {
    pub fn new(tx: Sender<Bytes>) -> Self {
        let inner = Arc::new(F1SessionDataManagerInner {
            driver_info: RwLock::new(AHashMap::new()),
            general: RwLock::new(F1GeneralInfo::default()),
            telemetry: RwLock::new(F1TelemetryInfo::default()),
            last_general: RwLock::new(F1GeneralInfo::default()),
            last_general_encoded: RwLock::new(None),
            last_telemetry: RwLock::new(F1TelemetryInfo::default()),
            team_senders: RwLock::new(AHashMap::new()),
            stop_sender: Mutex::new(None),
        });

        let mut instance = Self { inner };
        instance.spawn_update_task(tx);
        instance
    }

    #[inline]
    pub fn cache(&self) -> Option<Bytes> {
        self.last_general_encoded.read().clone()
    }

    #[allow(unused)]
    pub fn get_team_receiver(&self, team_id: u8) -> Option<Receiver<Bytes>> {
        self.team_senders
            .read()
            .get(&team_id)
            .map(|sender| sender.subscribe())
    }

    #[inline]
    pub fn push_event(&self, event: &PacketEventData) {
        let driver_info = self.driver_info.read();
        if let Some(event_data) = EventData::from_f1(event, &driver_info) {
            let mut general = self.general.write();
            general
                .events
                .get_or_insert_with(PacketsEventsData::default)
                .events
                .push(event_data);
        }
    }

    #[inline]
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

    #[inline]
    pub fn save_session(&self, packet: &PacketSessionData) {
        let mut general = self.general.write();
        general.update_session(packet);
    }

    #[inline]
    pub fn save_lap_history(&self, packet: &PacketSessionHistoryData) {
        let driver_info = self.driver_info.read();

        if let Some(driver) = driver_info.get(&(packet.car_idx as usize)) {
            let mut general = self.general.write();
            if let Some(player) = general.players.get_mut(driver.name.as_ref()) {
                player.update_session_history(packet);
            }
        }
    }

    #[inline]
    pub fn save_participants(&self, packet: &PacketParticipantsData) {
        let mut driver_info = self.driver_info.write();
        let mut general = self.general.write();
        let mut telemetry = self.telemetry.write();
        let mut team_senders = self.team_senders.write();

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
                .or_insert_with(|| Sender::new(30));
        }
    }

    #[inline]
    pub fn save_car_damage(&self, packet: &PacketCarDamageData) {
        self.process_telemetry_packet(&packet.car_damage_data, |player_telemetry, data| {
            player_telemetry.update_car_damage(data);
        });
    }

    #[inline]
    pub fn save_car_status(&self, packet: &PacketCarStatusData) {
        self.process_telemetry_packet(&packet.car_status_data, |player_telemetry, data| {
            player_telemetry.update_car_status(data);
        });
    }

    #[inline]
    pub fn save_car_telemetry(&self, packet: &PacketCarTelemetryData) {
        self.process_telemetry_packet(&packet.car_telemetry_data, |player_telemetry, data| {
            player_telemetry.update_car_telemetry(data);
        });
    }

    #[inline]
    pub fn save_final_classification(&self, packet: &PacketFinalClassificationData) {
        let driver_info = self.driver_info.read();
        let mut general = self.general.write();

        for (i, classification_data) in packet.classification_data.iter().enumerate() {
            if let Some(driver) = driver_info.get(&i) {
                if let Some(player) = general.players.get_mut(driver.name.as_ref()) {
                    player.update_classification_data(classification_data);
                }
            }
        }
    }

    #[inline]
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

    #[inline]
    fn spawn_update_task(&mut self, tx: Sender<Bytes>) {
        let (stop_sender, mut stop_receiver) = oneshot::channel();
        *self.inner.stop_sender.lock() = Some(stop_sender);

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

    #[inline]
    fn send_general_updates(inner: &Arc<F1SessionDataManagerInner>, tx: &Sender<Bytes>) {
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

    #[inline]
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

    #[inline]
    fn diff_general(current: &F1GeneralInfo, last: &F1GeneralInfo) -> Option<Bytes> {
        let diff = current.diff(last);

        if let Some(diff) = diff {
            let mut buf = BytesMut::with_capacity(diff.encoded_len());
            diff.encode_raw(&mut buf);

            return Some(buf.freeze());
        }

        None
    }

    #[inline]
    fn diff_player_telemetry(
        current: &PlayerTelemetry,
        last: &PlayerTelemetry,
    ) -> Option<PlayerTelemetry> {
        current.diff(last)
    }
}

impl Drop for F1SessionDataManager {
    fn drop(&mut self) {
        if let Some(sender) = self.stop_sender.lock().take() {
            let _ = sender.send(());
        }
    }
}
