use dotenvy::var;
use reqwest::Client;

use crate::{
    config::constants::*,
    error::AppResult,
    structs::{DiscordAuth, DiscordExchangeRequest, DiscordUserInfo},
};

#[derive(Clone)]
pub struct DiscordRepository {
    client_id: &'static str,
    cliente_secret: &'static str,
    redirect_uri: &'static str,
    client: Client,
}

impl DiscordRepository {
    pub fn new() -> Self {
        DiscordRepository {
            client_id: var("DISCORD_CLIENT_ID")
                .expect("Missing DISCORD_CLIENT_ID")
                .leak(),

            cliente_secret: var("DISCORD_CLIENT_SECRET")
                .expect("Missing DISCORD_CLIENT_SECRET")
                .leak(),

            redirect_uri: var("DISCORD_REDIRECT_URI")
                .expect("Missing DISCORD_REDIRECT_URI")
                .leak(),

            client: Client::new(),
        }
    }

    pub async fn account_info(&self, code: &str) -> AppResult<DiscordUserInfo> {
        let discord_exchange = DiscordExchangeRequest {
            client_id: self.client_id,
            client_secret: self.cliente_secret,
            grant_type: "authorization_code",
            code,
            redirect_uri: self.redirect_uri,
        };

        // TODO: This will be important for discord auth
        let auth = self
            .client
            .post(format!("{DISCORD_API_URL}/oauth2/token"))
            .form(&discord_exchange)
            .send()
            .await?
            .json::<DiscordAuth>()
            .await?;

        let user_info = self
            .client
            .get(format!("{DISCORD_API_URL}/users/@me"))
            .header("Authorization", format!("Bearer {}", auth.access_token))
            .send()
            .await?
            .json::<DiscordUserInfo>()
            .await?;

        Ok(user_info)
    }
}
