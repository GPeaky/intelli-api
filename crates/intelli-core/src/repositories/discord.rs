use std::fmt::Debug;

use dotenvy::var;
use reqwest::Client;

use error::AppResult;
use structs::{DiscordAuth, DiscordExchangeRequest, DiscordUserInfo};

const DISCORD_API_URL: &str = "https://discord.com/api/v10";

/// Repository for interacting with the Discord API.
#[derive(Clone)]
pub struct DiscordRepository {
    client_id: &'static str,
    client_secret: &'static str,
    redirect_uri: &'static str,
    client: Client,
}

impl DiscordRepository {
    /// Creates a new DiscordRepository instance.
    ///
    /// Initializes the repository with Discord API credentials from environment variables.
    ///
    /// # Panics
    /// if any of the required environment variables are missing.
    ///
    /// # Returns
    /// A new DiscordRepository instance.
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        DiscordRepository {
            client_id: var("DISCORD_CLIENT_ID")
                .expect("Missing DISCORD_CLIENT_ID")
                .leak(),

            client_secret: var("DISCORD_CLIENT_SECRET")
                .expect("Missing DISCORD_CLIENT_SECRET")
                .leak(),

            redirect_uri: var("DISCORD_REDIRECT_URI")
                .expect("Missing DISCORD_REDIRECT_URI")
                .leak(),

            client: Client::new(),
        }
    }

    /// Retrieves Discord account information for a user.
    ///
    /// # Arguments
    /// - `code`: The authorization code received from Discord OAuth flow.
    ///
    /// # Returns
    /// Discord user information wrapped in AppResult.
    #[tracing::instrument(skip(self))]
    pub async fn account_info(&self, code: impl AsRef<str> + Debug) -> AppResult<DiscordUserInfo> {
        let discord_exchange = DiscordExchangeRequest {
            client_id: self.client_id,
            client_secret: self.client_secret,
            grant_type: "authorization_code",
            code: code.as_ref(),
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
