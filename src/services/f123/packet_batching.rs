use crate::{
    cache::F123InsiderCache,
    config::constants::BATCHING_INTERVAL,
    error::{AppResult, F123Error},
    protos::{batched::ToProtoMessageBatched, PacketHeader},
};
use log::warn;
use ntex::util::Bytes;
use tokio::{sync::broadcast::Sender, time::Instant};

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
    pub async fn push_and_check(&mut self, packet: PacketHeader) -> AppResult<()> {
        self.buf.push(packet);
        self.check().await?;
        Ok(())
    }

    // This method is used to send the last batch of data
    // Should be not used for other event that is not the end of the session
    #[inline(always)]
    pub async fn final_send(&mut self, packet: PacketHeader) -> AppResult<()> {
        self.buf.push(packet);

        if let Some(batch) = ToProtoMessageBatched::batched_encoded(self.buf.clone()) {
            self.cache.prune().await?;

            if let Err(e) = self.tx.send(batch) {
                warn!("Broadcast Channel: {}", e);
            };
        } else {
            Err(F123Error::BatchedEncoding)?
        }

        self.last_batch_time = Instant::now();
        self.buf.clear();

        Ok(())
    }

    #[inline(always)]
    async fn check(&mut self) -> AppResult<()> {
        if self.last_batch_time.elapsed() < BATCHING_INTERVAL || self.buf.is_empty() {
            return Ok(());
        }

        // TODO: Implement another cache method for events
        if let Some(batch) = ToProtoMessageBatched::batched_encoded(self.buf.clone()) {
            self.cache.set(&batch).await?;

            if let Err(e) = self.tx.send(batch) {
                warn!("Broadcast channel: {}", e);
            };
        } else {
            Err(F123Error::BatchedEncoding)?
        }

        self.last_batch_time = Instant::now();
        self.buf.clear();
        Ok(())
    }
}
