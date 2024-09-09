use ahash::{AHashMap, AHashSet};
use ntex::util::Bytes;
use parking_lot::RwLock;
use prost::bytes::{Bytes as ProstBytes, BytesMut as ProstBytesMut};
use tokio::time::Instant;
use tracing::error;

use crate::{
    config::constants::F1_CACHING_DUR,
    protos::{batched::ToProtoMessageBatched, packet_header::PacketType, PacketHeader},
    structs::PacketExtraData,
    utils::compress_async,
};

struct CachedData(Bytes, Instant);

/// Manages caching of various packet types for F1 telemetry data.
pub struct PacketCaching {
    car_motion: Option<ProstBytes>,
    session_data: Option<ProstBytes>,
    participants: Option<ProstBytes>,
    history_data: AHashMap<u8, ProstBytes>,
    event_data: AHashSet<ProstBytes>,
    cache: RwLock<Option<CachedData>>,
}

impl PacketCaching {
    /// Creates a new PacketCaching instance.
    pub fn new() -> Self {
        Self {
            car_motion: None,
            session_data: None,
            participants: None,
            history_data: AHashMap::with_capacity(20),
            event_data: AHashSet::with_capacity(10),
            cache: RwLock::new(None),
        }
    }

    /// Retrieves all cached data, compressing it if necessary.
    ///
    /// # Returns
    /// Compressed bytes of all cached data, or None if cache is empty or compression fails.
    pub async fn get(&self) -> Option<Bytes> {
        {
            let cache_read = self.cache.read();
            if let Some(cached) = &*cache_read {
                if cached.1.elapsed() < F1_CACHING_DUR {
                    return Some(cached.0.clone());
                }
            }
        }

        let mut headers = Vec::with_capacity(self.total_headers());

        if let Some(header) = self.get_car_motion() {
            headers.push(header);
        }

        if let Some(header) = self.get_session_data() {
            headers.push(header)
        };

        if let Some(header) = self.get_participants() {
            headers.push(header)
        };

        if let Some(mut history_headers) = self.get_history_data() {
            headers.append(&mut history_headers)
        };

        if let Some(mut events_headers) = self.get_events_data() {
            headers.append(&mut events_headers)
        };

        if headers.is_empty() {
            return None;
        }

        match ToProtoMessageBatched::batched_encoded(headers) {
            Some(bytes) => match compress_async(bytes).await {
                Ok(compressed) => {
                    let mut cache_write = self.cache.write();

                    *cache_write = Some(CachedData(compressed.clone(), Instant::now()));
                    Some(compressed)
                }

                Err(e) => {
                    error!("Error while compressing data: {}", e);
                    None
                }
            },
            None => None,
        }
    }

    /// Saves a packet to the appropriate cache based on its type.
    ///
    /// # Arguments
    /// - `packet_type`: Type of the packet being saved.
    /// - `payload`: Raw data of the packet.
    /// - `extra_data`: Additional data specific to certain packet types.
    pub fn save(
        &mut self,
        packet_type: PacketType,
        payload: &[u8],
        extra_data: Option<PacketExtraData>,
    ) {
        match packet_type {
            PacketType::CarMotion => self.set_car_motion(payload),
            PacketType::SessionData => self.set_session_data(payload),
            PacketType::Participants => self.set_participants(payload),

            PacketType::EventData => {
                debug_assert!(extra_data.is_some());

                if let Some(PacketExtraData::EventCode(code)) = extra_data {
                    self.push_event(payload, code);
                } else {
                    error!("Error Receiving OptionalMessage");
                }
            }

            PacketType::SessionHistoryData => {
                debug_assert!(extra_data.is_some());

                if let Some(PacketExtraData::CarNumber(car_id)) = extra_data {
                    self.set_history_data(payload, car_id)
                } else {
                    error!("Error Receiving OptionalMessage");
                }
            }

            PacketType::FinalClassificationData => {
                todo!()
            }
        }
    }

    /// Calculates the total number of headers across all packet types.
    ///
    /// # Returns
    /// Total count of headers in the cache.
    fn total_headers(&self) -> usize {
        let base_count = 3;
        let history_estimate = self.history_data.len();
        let events_estimate = self.event_data.len();

        base_count + history_estimate + events_estimate
    }

    // Various getter methods for different packet types
    #[inline(always)]
    fn get_car_motion(&self) -> Option<PacketHeader> {
        self.car_motion
            .as_ref()
            .map(|car_motion_data| PacketHeader {
                r#type: PacketType::CarMotion.into(),
                payload: car_motion_data.clone(),
            })
    }

    #[inline(always)]
    fn get_participants(&self) -> Option<PacketHeader> {
        self.participants.as_ref().map(|participants| PacketHeader {
            r#type: PacketType::Participants.into(),
            payload: participants.clone(),
        })
    }

    #[inline(always)]
    fn get_session_data(&self) -> Option<PacketHeader> {
        self.session_data.as_ref().map(|session_data| PacketHeader {
            r#type: PacketType::SessionData.into(),
            payload: session_data.clone(),
        })
    }

    #[inline(always)]
    fn get_history_data(&self) -> Option<Vec<PacketHeader>> {
        let len = self.history_data.len();
        if len == 0 {
            return None;
        }

        let mut vec = Vec::with_capacity(len);
        for (_, session_history) in &self.history_data {
            vec.push(PacketHeader {
                r#type: PacketType::SessionHistoryData.into(),
                payload: session_history.clone(),
            })
        }

        Some(vec)
    }

    #[inline(always)]
    fn get_events_data(&self) -> Option<Vec<PacketHeader>> {
        let len = self.event_data.len();

        if len == 0 {
            return None;
        }

        let mut vec = Vec::with_capacity(len);
        for event in &self.event_data {
            vec.push(PacketHeader {
                r#type: PacketType::EventData.into(),
                payload: event.clone(),
            })
        }

        Some(vec)
    }

    // Various setter methods for different packet types
    #[inline(always)]
    fn set_car_motion(&mut self, payload: &[u8]) {
        let mut data = ProstBytesMut::with_capacity(payload.len());
        data.extend_from_slice(payload);

        self.car_motion = Some(data.freeze());
    }

    #[inline(always)]
    fn set_session_data(&mut self, payload: &[u8]) {
        let mut data = ProstBytesMut::with_capacity(payload.len());
        data.extend_from_slice(payload);

        self.session_data = Some(data.freeze());
    }

    #[inline(always)]
    fn set_participants(&mut self, payload: &[u8]) {
        let mut data = ProstBytesMut::with_capacity(payload.len());
        data.extend_from_slice(payload);

        self.participants = Some(data.freeze());
    }

    #[inline(always)]
    fn set_history_data(&mut self, payload: &[u8], car_idx: u8) {
        self.history_data
            .insert(car_idx, ProstBytes::copy_from_slice(payload));
    }

    #[inline(always)]
    fn push_event(&mut self, payload: &[u8], _code: [u8; 4]) {
        self.event_data.insert(ProstBytes::copy_from_slice(payload));
    }
}
