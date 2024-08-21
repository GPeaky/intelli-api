use deadpool_postgres::Status as DeadPoolStatus;
use serde::Serialize;

// Database Connection Pool Status
#[derive(Serialize)]
pub struct ConnectionPoolStatus {
    pub max_size: usize,
    pub size: usize,
    pub available: usize,
    pub waiting: usize,
}

impl From<DeadPoolStatus> for ConnectionPoolStatus {
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
pub struct DatabaseStatus {
    pub postgres: ConnectionPoolStatus,
}
