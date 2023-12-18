use deadpool_redis::Status;
use serde::Serialize;

#[derive(Serialize)]
pub struct DatabasesStatus {
    pub redis: Status,
    pub pg: Status,
}