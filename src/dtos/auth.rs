use garde::Validate;
use serde::{Deserialize, Serialize};
use serde_trim::string_trim;

#[derive(Serialize)]
pub struct AuthResponse {
    pub access_token: String,
    pub refresh_token: String,
}

#[derive(Serialize)]
pub struct RefreshResponse {
    pub access_token: String,
}

#[derive(Clone, Deserialize, Debug, Validate)]
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
    #[garde(email)]
    #[serde(deserialize_with = "string_trim")]
    pub email: String,
    #[garde(length(min = 8, max = 20))]
    #[serde(deserialize_with = "string_trim")]
    pub password: String,
}

#[derive(Deserialize, Validate)]
pub struct ForgotPasswordDto {
    #[garde(email)]
    #[serde(deserialize_with = "string_trim")]
    pub email: String,
}
