use deadpool_redis::Status as DeadPoolStatus;
use serde::Serialize;

#[derive(Serialize)]
pub struct Status {
    pub max_size: usize,
    pub size: usize,
    pub available: usize,
    pub waiting: usize,
}

impl From<DeadPoolStatus> for Status {
    fn from(status: DeadPoolStatus) -> Self {
        Self {
            max_size: status.max_size,
            size: status.size,
            available: status.available,
            waiting: status.waiting,
        }
    }
}

#[derive(Serialize)]
pub struct DatabasesStatus {
    pub redis: Status,
    pub pg: Status,
}
