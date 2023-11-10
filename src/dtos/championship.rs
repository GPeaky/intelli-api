use crate::entity::Category;
use garde::Validate;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Validate)]
pub struct CreateChampionshipDto {
    #[garde(length(min = 3, max = 20))]
    pub name: String,
    #[garde(skip)]
    pub category: Category,
    #[garde(skip)]
    pub season: u16,
}

#[derive(Debug, Serialize)]
pub struct SocketStatus {
    pub active: bool,
    pub connections: usize,
}

pub struct ChampionshipCacheData {
    pub session_data: Vec<u8>,
    pub motion_data: Vec<u8>,
    pub participants_data: Vec<u8>,
    pub history_data: Vec<Vec<u8>>,
}
