use std::ops::Range;

use parking_lot::Mutex;

use error::AppResult;

const PORTS_RANGE: Range<i32> = 27700..27800;

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
        self.ports.lock().pop()
    }

    pub fn return_port(&self, port: i32) {
        assert!(PORTS_RANGE.contains(&port));
        let mut ports = self.ports.lock();
        assert!(!ports.contains(&port));
        ports.push(port);
    }
}
