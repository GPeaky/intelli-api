use garde::Validate;
use serde::{Deserialize, Serialize};
use serde_trim::{option_string_trim, string_trim};

use entities::Provider;
use utils::deserialize_i64_from_string;

// Authentication Structures
#[derive(Deserialize, Validate)]
pub struct LoginCredentials {
    #[garde(email)]
    #[serde(deserialize_with = "string_trim")]
    pub email: String,
    #[garde(length(min = 8, max = 40))]
    #[serde(deserialize_with = "string_trim")]
    pub password: String,
}

#[derive(Serialize)]
pub struct AuthTokens {
    pub access_token: String,
    pub refresh_token: String,
}

#[derive(Serialize)]
pub struct NewAccessToken {
    pub access_token: String,
}

// User Registration and Management
#[derive(Deserialize, Debug, Validate)]
pub struct UserRegistrationData {
    #[garde(ascii, length(min = 3, max = 20))]
    #[serde(deserialize_with = "string_trim")]
    pub username: String,
    #[garde(email)]
    #[serde(deserialize_with = "string_trim")]
    pub email: String,
    #[serde(default, deserialize_with = "option_string_trim")]
    #[garde(length(min = 8, max = 40))]
    pub password: Option<String>,
    #[garde(inner(length(min = 10, max = 100)))]
    pub avatar: Option<String>,
    #[garde(skip)]
    pub provider: Option<Provider>,
    #[garde(skip)]
    pub discord_id: Option<i64>,
}

impl UserRegistrationData {
    pub fn from_discord_user_info(discord_info: DiscordUserInfo) -> Self {
        UserRegistrationData {
            avatar: discord_info.avatar_url(),
            username: discord_info.username,
            email: discord_info.email,
            provider: Some(Provider::Discord),
            password: None,
            discord_id: Some(discord_info.id),
        }
    }
}

// Password Management
#[derive(Deserialize, Validate)]
pub struct PasswordResetRequest {
    #[garde(email)]
    #[serde(deserialize_with = "string_trim")]
    pub email: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct PasswordUpdateData {
    #[garde(length(min = 8, max = 40))]
    #[serde(deserialize_with = "string_trim")]
    pub password: String,
}

// Token and Security
#[derive(Debug, Deserialize)]
pub struct TokenVerification {
    pub token: String,
}

#[derive(Debug, Deserialize)]
pub struct RefreshTokenRequest {
    pub fingerprint: String,
    pub refresh_token: String,
}

#[derive(Debug, Deserialize)]
pub struct ClientFingerprint {
    pub fingerprint: String,
}

#[derive(Deserialize)]
pub struct OauthAuthorizationCode {
    pub code: String,
}

#[derive(Debug, Serialize)]
pub struct DiscordExchangeRequest<'a> {
    pub client_id: &'a str,
    pub client_secret: &'a str,
    pub grant_type: &'a str,
    pub code: &'a str,
    pub redirect_uri: &'a str,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct DiscordAuth {
    pub access_token: String,
    pub expires_in: u32,
    pub refresh_token: String,
    pub scope: String,
    pub token_type: String,
}

#[derive(Deserialize, Debug)]
pub struct DiscordUserInfo {
    #[serde(deserialize_with = "deserialize_i64_from_string")]
    pub id: i64,
    pub username: String,
    pub email: String,
    pub avatar: Option<String>,
}

impl DiscordUserInfo {
    pub fn avatar_url(&self) -> Option<String> {
        self.avatar.as_ref().map(|avatar| {
            format!(
                "https://cdn.discordapp.com/avatars/{}/{}.png",
                self.id, avatar
            )
        })
    }
}
