use std::{cell::Cell, sync::Arc};

use crate::{
    cache::F123InsiderCache,
    config::constants::BATCHING_INTERVAL,
    error::{AppResult, F123Error},
    protos::{batched::ToProtoMessageBatched, packet_header::PacketType, PacketHeader},
    structs::OptionalMessage,
};
use ntex::util::Bytes;
use parking_lot::Mutex;
use prost::Message;
use tokio::{
    sync::{broadcast::Sender, oneshot},
    time::interval,
};
use tracing::{info, warn};

pub struct PacketBatching {
    buf: Arc<Mutex<Vec<PacketHeader>>>,
    shutdown: Option<oneshot::Sender<()>>,
    cache: F123InsiderCache,
}

impl PacketBatching {
    pub fn new(tx: Sender<Bytes>, cache: F123InsiderCache) -> Self {
        let (stx, srx) = oneshot::channel::<()>();
        let buf = Arc::from(Mutex::from(Vec::with_capacity(2048)));

        let instance = Self {
            buf: buf.clone(),
            shutdown: Some(stx),
            cache,
        };

        let mut srx = Cell::from(srx);
        let mut interval_timer = interval(BATCHING_INTERVAL);
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    _ = interval_timer.tick() => {
                        if let Err(e) = Self::send_data(&buf, &tx).await {
                            warn!("Packet batching: {}", e);
                        }
                    }

                    _ = srx.get_mut() => {
                        info!("Packet batching: shutdown");
                        break;
                    }
                }
            }
        });

        instance
    }

    #[inline(always)]
    pub async fn push(&mut self, packet: PacketHeader) -> AppResult<()> {
        self.push_with_optional_parameter(packet, None).await
    }

    pub async fn push_with_optional_parameter(
        &mut self,
        packet: PacketHeader,
        second_param: Option<OptionalMessage<'_>>,
    ) -> AppResult<()> {
        let encoded_package = packet.encode_to_vec();
        let packet_type = PacketType::try_from(packet.r#type).unwrap();

        self.save_cache(packet_type, &encoded_package, second_param)
            .await?;

        self.buf.lock().push(packet);
        Ok(())
    }

    async fn send_data(buf: &Arc<Mutex<Vec<PacketHeader>>>, tx: &Sender<Bytes>) -> AppResult<()> {
        let buf = {
            let mut buf = buf.lock();

            if buf.is_empty() {
                return Ok(());
            }

            let buf = buf.drain(..).collect::<Vec<_>>();
            buf
        };

        // TODO: Implement another cache method for events
        if let Some(batch) = ToProtoMessageBatched::batched_encoded(buf) {
            let encoded_batch = Self::compress(&batch)?;
            // cache.set(&encoded_batch).await?;

            if tx.receiver_count() == 0 {
                return Ok(());
            }

            if let Err(e) = tx.send(encoded_batch) {
                warn!("Broadcast channel: {}", e);
            };
        } else {
            Err(F123Error::BatchedEncoding)?
        }

        Ok(())
    }

    #[inline(always)]
    async fn save_cache(
        &mut self,
        packet_type: PacketType,
        encoded_package: &[u8],
        second_param: Option<OptionalMessage<'_>>,
    ) -> AppResult<()> {
        match packet_type {
            PacketType::CarMotion => {
                if let Err(e) = self.cache.set_motion_data(encoded_package).await {
                    warn!("F123 cache: {}", e);
                }
            }

            PacketType::SessionData => {
                if let Err(e) = self.cache.set_session_data(encoded_package).await {
                    warn!("F123 cache: {}", e);
                }
            }

            PacketType::SessionHistoryData => {
                let car_id = match second_param.unwrap() {
                    OptionalMessage::Number(car_id) => car_id,
                    _ => unreachable!(),
                };

                if let Err(e) = self
                    .cache
                    .set_session_history(encoded_package, car_id)
                    .await
                {
                    warn!("F123 cache: {}", e);
                }
            }

            PacketType::Participants => {
                if let Err(e) = self.cache.set_participants_data(encoded_package).await {
                    warn!("F123 cache: {}", e);
                }
            }

            PacketType::EventData => {
                let string_code = match second_param.unwrap() {
                    OptionalMessage::Text(string_code) => string_code,
                    _ => unreachable!(),
                };

                if let Err(e) = self
                    .cache
                    .push_event_data(encoded_package, string_code)
                    .await
                {
                    warn!("F123 cache: {}", e);
                }
            }

            PacketType::FinalClassificationData => {
                info!("Final classification data");

                self.cache.prune().await?;

                // if let Err(e) = self
                //     .cache
                //     .set_final_classification_data(&encoded_package)
                //     .await
                // {
                //     warn!("F123 cache: {}", e);
                // }
            }
        }

        Ok(())
    }

    // This method is used to compress the batched data
    #[inline(always)]
    fn compress(data: &[u8]) -> AppResult<Bytes> {
        match zstd::stream::encode_all(data, 3) {
            Ok(compressed_data) => Ok(Bytes::from(compressed_data)),
            Err(e) => {
                warn!("Zstd compression: {}", e);
                Err(F123Error::Compressing)?
            }
        }
    }
}

impl Drop for PacketBatching {
    fn drop(&mut self) {
        if let Some(tx) = self.shutdown.take() {
            let _ = tx.send(());
        }
    }
}
