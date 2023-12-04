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
    sender: Sender<Bytes>,
    last_batch_time: Instant,
    cache: F123InsiderCache,
}

impl PacketBatching {
    pub fn new(sender: Sender<Bytes>, cache: F123InsiderCache) -> Self {
        Self {
            sender,
            buf: Vec::with_capacity(1024),
            last_batch_time: Instant::now(),
            cache,
        }
    }

    #[inline(always)]
    pub fn push(&mut self, packet: PacketHeader) {
        self.buf.push(packet);
    }

    #[inline(always)]
    // TODO: See if we can make this more efficient
    pub async fn check(&mut self) {
        if self.last_batch_time.elapsed() > BATCHING_INTERVAL && !self.buf.is_empty() {
            // TODO: Implement another cache method for events
            if let Some(batch) = ToProtoMessageBatched::batched_encoded(self.buf.clone()) {
                self.cache.set_cache(&batch).await.unwrap();

                if let Err(e) = self.sender.send(batch).await {
                    error!("Error sending batch data: {:?}", e);
                } else {
                    self.last_batch_time = Instant::now();
                }
            } else {
                error!("Error converting and encoding data");
            }

            self.buf.clear();
        }
    }

    #[inline(always)]
    pub async fn push_and_check(&mut self, packet: PacketHeader) {
        self.push(packet);
        self.check().await;
    }
}
