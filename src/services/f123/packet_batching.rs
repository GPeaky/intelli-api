use crate::{
    cache::F123InsiderCache,
    config::constants::BATCHING_INTERVAL,
    error::{AppResult, F123Error},
    protos::{batched::ToProtoMessageBatched, PacketHeader},
};
use ntex::util::Bytes;
use tokio::{sync::broadcast::Sender, time::Instant};
use tracing::warn;

// Packet Batching implementation
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
    //
    // Should be not used for other event that is not the end of the session
    #[inline(always)]
    pub async fn final_send(&mut self, packet: PacketHeader) -> AppResult<()> {
        self.buf.push(packet);

        let buf = self.buf.drain(..).collect::<Vec<_>>();
        if let Some(batch) = ToProtoMessageBatched::batched_encoded(buf) {
            self.cache.prune().await?;
            let encoded_batch = Self::compress(&batch).await.unwrap();

            // Todo: Check the subscribers count and only send if is at least 1 receiver `self.tx.receiver_count()`
            if let Err(e) = self.tx.send(encoded_batch) {
                warn!("Broadcast Channel: {}", e);
            };
        } else {
            Err(F123Error::BatchedEncoding)?
        }

        self.last_batch_time = Instant::now();
        Ok(())
    }

    #[inline(always)]
    async fn check(&mut self) -> AppResult<()> {
        if self.last_batch_time.elapsed() < BATCHING_INTERVAL || self.buf.is_empty() {
            return Ok(());
        }

        // TODO: Implement another cache method for events
        if let Some(batch) = ToProtoMessageBatched::batched_encoded(self.buf.clone()) {
            let encoded_batch = Self::compress(&batch).await.unwrap();
            self.cache.set(&encoded_batch).await?;

            // Todo: Check the subscribers count and only send if is at least 1 receiver `self.tx.receiver_count()`
            if let Err(e) = self.tx.send(encoded_batch) {
                warn!("Broadcast channel: {}", e);
            };
        } else {
            Err(F123Error::BatchedEncoding)?
        }

        self.last_batch_time = Instant::now();
        self.buf.clear();
        Ok(())
    }

    // Testing brotli compression algorithm for batched data
    // This method is used to compress the batched data
    #[inline(always)]
    async fn compress(data: &[u8]) -> Result<Bytes, Box<dyn std::error::Error>> {
        // todo: Decide between level 3 or 9 for compression 280us(level3) vs 1ms(level9)
        let compressed_data: Vec<u8> = zstd::stream::encode_all(data, 9).unwrap();
        Ok(Bytes::from(compressed_data))
    }
}
