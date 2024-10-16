use garde::Validate;
use serde::{Deserialize, Serialize};
use serde_trim::{option_string_trim, string_trim};

use entities::{Category, ChampionshipRole, SharedChampionship, SharedRace};

// Championship Management
#[derive(Debug, Deserialize, Validate)]
pub struct ChampionshipCreationData {
    #[garde(ascii, length(min = 3, max = 20))]
    #[serde(deserialize_with = "string_trim")]
    pub name: String,
    #[garde(skip)]
    pub category: Category,
}

#[derive(Debug, Deserialize, Validate)]
pub struct ChampionshipUserAddForm {
    #[serde(deserialize_with = "string_trim")]
    #[garde(email)]
    pub email: String,
    #[serde(default)]
    #[garde(skip)]
    pub role: ChampionshipRole,
    #[garde(skip)]
    pub team_id: Option<i16>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct ChampionshipUpdateData {
    #[garde(ascii, length(min = 3, max = 20))]
    #[serde(default, deserialize_with = "option_string_trim")]
    pub name: Option<String>,
    #[garde(skip)]
    pub category: Option<Category>,
}

// Service Status
#[derive(Default, Debug, Serialize)]
pub struct ServiceStatus {
    pub active: bool,
    pub general_conn: u32,
    pub engineer_conn: u32,
}

// Path Parameters
#[derive(Debug, Deserialize, Validate)]
pub struct ChampionshipId(#[garde(range(min = 700000000, max = 799999999))] pub i32);

#[derive(Debug, Deserialize, Validate)]
pub struct ChampionshipAndUserId {
    #[garde(range(min = 700000000, max = 799999999))]
    pub championship_id: i32,
    #[garde(range(min = 600000000, max = 699999999))]
    pub user_id: i32,
}

#[derive(Serialize)]
pub struct ChampionshipData {
    pub championship: SharedChampionship,
    pub races: Vec<SharedRace>,
}
