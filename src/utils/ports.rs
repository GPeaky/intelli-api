use std::{collections::VecDeque, ops::Range, sync::Arc};

use parking_lot::Mutex;

use crate::{error::AppResult, repositories::ChampionshipRepository};

pub const PORTS_RANGE: Range<i32> = 27700..27800;

#[derive(Clone)]
pub struct MachinePorts {
    ports: Arc<Mutex<VecDeque<i32>>>,
}

impl MachinePorts {
    pub async fn new(championship_repo: &ChampionshipRepository) -> AppResult<Self> {
        let ports_used = championship_repo.ports_in_use().await?;
        let estimated_len = PORTS_RANGE.len() - ports_used.len();
        let mut ports = VecDeque::with_capacity(estimated_len);

        for port in PORTS_RANGE {
            if !ports_used.contains(&port) {
                ports.push_back(port);
            }
        }

        let ports = Arc::new(Mutex::new(ports));

        Ok(MachinePorts { ports })
    }

    // TODO: Make sure that championship is created before eliminating the port from the list
    pub fn get(&self) -> Option<i32> {
        let mut ports = self.ports.lock();
        ports.pop_front()
    }
}
