use crate::{
    config::constants::BATCHING_INTERVAL,
    protos::{packet_header::PacketType, ToProtoMessage},
};
use tokio::{sync::broadcast::Sender, time::Instant};
use tracing::error;

pub struct PacketBatching {
    buf: Vec<Vec<u8>>,
    sender: Sender<Vec<u8>>,
    last_batch_time: Instant,
}

impl PacketBatching {
    pub fn new(sender: Sender<Vec<u8>>) -> Self {
        Self {
            sender,
            buf: Vec::with_capacity(2048),
            last_batch_time: Instant::now(),
        }
    }

    #[inline(always)]
    pub fn push(&mut self, packet: Vec<u8>) {
        self.buf.push(packet);
    }

    #[inline(always)]
    // TODO: Check if this is the best way to do this
    pub fn check(&mut self) {
        if self.last_batch_time.elapsed() > BATCHING_INTERVAL && !self.buf.is_empty() {
            if let Some(batch) = self.buf.convert_and_encode(PacketType::SessionData) {
                if let Err(e) = self.sender.send(batch) {
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
    pub fn push_and_check(&mut self, packet: Vec<u8>) {
        self.push(packet);
        self.check();
    }
}
