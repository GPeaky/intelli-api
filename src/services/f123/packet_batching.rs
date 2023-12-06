use crate::{
    cache::F123InsiderCache,
    config::constants::BATCHING_INTERVAL,
    protos::{batched::ToProtoMessageBatched, PacketHeader},
};
use async_channel::Sender;
use log::error;
use ntex::util::Bytes;
use tokio::time::Instant;

pub struct PacketBatching {
    buf: Vec<PacketHeader>,
    tx: Sender<Bytes>,
    last_batch_time: Instant,
    cache: F123InsiderCache,
}

impl PacketBatching {
    pub fn new(tx: Sender<Bytes>, cache: F123InsiderCache) -> Self {
        Self {
            tx,
            buf: Vec::with_capacity(2048),
            last_batch_time: Instant::now(),
            cache,
        }
    }

    #[inline(always)]
    pub async fn check(&mut self) {
        if self.last_batch_time.elapsed() < BATCHING_INTERVAL || self.buf.is_empty() {
            return;
        }

        // TODO: Implement another cache method for events
        if let Some(batch) = ToProtoMessageBatched::batched_encoded(self.buf.clone()) {
            let batch_cloned = batch.clone();
            let cache_update_fut = self.cache.set(&batch_cloned);
            let channel_send_fut = self.tx.send(batch);

            let (cache_res, channel_res) = tokio::join!(cache_update_fut, channel_send_fut);

            if let Err(e) = cache_res {
                error!("Error Updating Batching Cache: {:?}", e);
            }

            if let Err(e) = channel_res {
                error!("Error sending batch data: {:?}", e);
            }
        } else {
            error!("Error converting and encoding data");
        }

        self.last_batch_time = Instant::now();
        self.buf.clear();
    }

    // This method is used to send the last batch of data
    // Should be not used for other event that is not the end of the session
    #[inline(always)]
    pub async fn final_send(&mut self, packet: PacketHeader) {
        self.buf.push(packet);

        if let Some(batch) = ToProtoMessageBatched::batched_encoded(self.buf.clone()) {
            let cache_prune_fut = self.cache.prune();
            let channel_send_fut = self.tx.send(batch);

            let (cache_res, channel_res) = tokio::join!(cache_prune_fut, channel_send_fut);

            if let Err(e) = cache_res {
                error!("Error Pruning Batching Cache: {:?}", e);
            }

            if let Err(e) = channel_res {
                error!("Error sending batch data: {:?}", e);
            }
        } else {
            error!("Error converting and encoding data");
        }

        self.last_batch_time = Instant::now();
        self.buf.clear();
    }

    #[inline(always)]
    pub async fn push_and_check(&mut self, packet: PacketHeader) {
        self.buf.push(packet);
        self.check().await;
    }
}
