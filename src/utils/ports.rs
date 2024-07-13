use parking_lot::Mutex;
use tracing_log::log::error;

use crate::{config::constants::PORTS_RANGE, error::AppResult};

#[derive(Clone)]
pub struct MachinePorts {
    ports: &'static Mutex<Vec<i32>>,
}

impl MachinePorts {
    pub async fn new(used_ports: Vec<i32>) -> AppResult<Self> {
        let estimated_len = PORTS_RANGE.len() - used_ports.len();
        let mut ports = Vec::with_capacity(estimated_len);

        for port in PORTS_RANGE {
            if !used_ports.contains(&port) {
                ports.push(port);
            }
        }

        let ports = Box::leak(Box::new(Mutex::new(ports)));

        Ok(MachinePorts { ports })
    }

    pub fn next(&self) -> Option<i32> {
        let mut ports = self.ports.lock();
        ports.pop()
    }

    pub fn return_port(&self, port: i32) {
        if !PORTS_RANGE.contains(&port) {
            error!("Port {} is not in the range", port);
            return;
        }

        let mut ports = self.ports.lock();

        if !ports.contains(&port) {
            ports.push(port);
        }
    }
}
