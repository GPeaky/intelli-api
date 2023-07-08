use garde::Validate;
use serde::Deserialize;

#[derive(Debug, Deserialize, Validate)]
pub struct CreateChampionshipDto {
    #[garde(length(min = 3, max = 20))]
    pub name: String,
}
