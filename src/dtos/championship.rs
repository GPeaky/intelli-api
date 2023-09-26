use garde::Validate;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Validate)]
pub struct CreateChampionshipDto {
    #[garde(length(min = 3, max = 20))]
    pub name: String,
}

#[derive(Debug, Serialize)]
pub struct SocketStatus {
    pub active: bool,
    pub connections: usize,
}
