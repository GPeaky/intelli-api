use std::time::Duration;
use tokio::{sync::broadcast::Sender, time::Instant};

const INTERVAL: Duration = Duration::from_millis(700);

pub struct PacketBatching {
    buf: Vec<Vec<u8>>,
    sender: Sender<Vec<Vec<u8>>>,
    last_batch_time: Instant,
}

impl PacketBatching {
    pub fn new(sender: Sender<Vec<Vec<u8>>>) -> Self {
        Self {
            sender,
            buf: Vec::new(),
            last_batch_time: Instant::now(),
        }
    }

    #[inline(always)]
    pub fn push(&mut self, packet: Vec<u8>) {
        self.buf.push(packet);
    }

    #[inline(always)]
    pub fn maybe_send_batch(&mut self) {
        if self.last_batch_time.elapsed().gt(&INTERVAL) && !self.buf.is_empty() {
            let batch = self.buf.drain(..).collect::<Vec<_>>();

            if let Err(e) = self.sender.send(batch) {
                eprintln!("Error sending batch data: {:?}", e);
            }

            self.last_batch_time = Instant::now();
        }
    }
}
