use std::{cell::Cell, sync::Arc};

use crate::{
    cache::F123InsiderCache,
    config::constants::BATCHING_INTERVAL,
    error::{AppResult, F123Error},
    protos::{batched::ToProtoMessageBatched, PacketHeader},
};
use ntex::util::Bytes;
use parking_lot::Mutex;
use tokio::{
    sync::{broadcast::Sender, oneshot},
    time::interval,
};
use tracing::{info, warn};

pub struct PacketBatching {
    buf: Arc<Mutex<Vec<PacketHeader>>>,
    shutdown: Option<oneshot::Sender<()>>,
}

impl PacketBatching {
    pub fn new(tx: Sender<Bytes>, mut cache: F123InsiderCache) -> Self {
        let (stx, srx) = oneshot::channel::<()>();
        let mut srx = Cell::from(srx);
        let buf = Arc::from(Mutex::from(Vec::with_capacity(2048)));

        let instance = Self {
            buf: buf.clone(),
            shutdown: Some(stx),
        };

        let mut interval_timer = interval(BATCHING_INTERVAL);
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    _ = interval_timer.tick() => {
                        if let Err(e) = Self::send_data(&buf, &tx, &mut cache).await {
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
    pub fn push(&mut self, packet: PacketHeader) {
        self.buf.lock().push(packet);
    }

    async fn send_data(
        buf: &Arc<Mutex<Vec<PacketHeader>>>,
        tx: &Sender<Bytes>,
        cache: &mut F123InsiderCache,
    ) -> AppResult<()> {
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
            cache.set(&encoded_batch).await?;

            // Todo: Check the subscribers count and only send if is at least 1 receiver `self.tx.receiver_count()`
            if let Err(e) = tx.send(encoded_batch) {
                warn!("Broadcast channel: {}", e);
            };
        } else {
            Err(F123Error::BatchedEncoding)?
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
