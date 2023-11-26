use crate::{
    config::constants::BATCHING_INTERVAL,
    protos::{packet_header::PacketType, ToProtoMessage},
};
use flume::Sender;
use log::error;
use ntex::util::Bytes;
use tokio::time::Instant;

pub struct PacketBatching {
    buf: Vec<Bytes>,
    sender: Sender<Bytes>,
    last_batch_time: Instant,
}

impl PacketBatching {
    pub fn new(sender: Sender<Bytes>) -> Self {
        Self {
            sender,
            buf: Vec::with_capacity(1024),
            last_batch_time: Instant::now(),
        }
    }

    #[inline(always)]
    pub fn push(&mut self, packet: Bytes) {
        self.buf.push(packet);
    }

    #[inline(always)]
    // TODO: Check if this is the best way to do this
    pub async fn check(&mut self) {
        if self.last_batch_time.elapsed() > BATCHING_INTERVAL && !self.buf.is_empty() {
            if let Some(batch) = self.buf.convert_and_encode(PacketType::SessionData) {
                if let Err(e) = self.sender.send_async(batch).await {
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
    pub async fn push_and_check(&mut self, packet: Bytes) {
        self.push(packet);
        self.check().await;
    }
}
