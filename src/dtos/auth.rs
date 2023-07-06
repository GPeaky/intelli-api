use garde::Validate;
use serde::Deserialize;
use serde_trim::string_trim;

#[derive(Deserialize, Debug, Validate)]
pub struct RegisterUserDto {
    #[garde(length(min = 3, max = 20))]
    #[serde(deserialize_with = "string_trim")]
    pub username: String,
    #[garde(email)]
    #[serde(deserialize_with = "string_trim")]
    pub email: String,
    #[garde(length(min = 8, max = 20))]
    #[serde(deserialize_with = "string_trim")]
    pub password: String,
}

#[derive(Deserialize, Validate)]
pub struct LoginUserDto {
    #[garde(length(min = 3, max = 20))]
    #[serde(deserialize_with = "string_trim")]
    pub username: String,
    #[garde(length(min = 8, max = 20))]
    #[serde(deserialize_with = "string_trim")]
    pub password: String,
}
