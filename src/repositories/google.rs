use dotenvy::var;
use reqwest::Client;

use crate::{
    config::constants::*,
    error::AppResult,
    structs::{GoogleAuthTokens, GoogleTokenExchangeRequest, GoogleUserInfo},
};

/// Manages interactions with Google's OAuth2 API and user information endpoints.
///
/// This struct is responsible for handling OAuth2 authentication and retrieval of user
/// information from Google. It encapsulates the necessary details like client ID, client
/// secret, redirect URI, grant type, and an HTTP client for making requests.
#[derive(Clone)]
pub struct GoogleRepository {
    client_id: &'static str,
    client_secret: &'static str,
    redirect_uri: &'static str,
    grant_type: &'static str,
    reqwest_client: reqwest::Client,
}

impl GoogleRepository {
    /// Constructs a new `GoogleRepository` with environment-specific credentials and an HTTP client.
    ///
    /// The necessary authentication details (client ID, client secret, redirect URI, grant type)
    /// are loaded from environment variables. An HTTP client for making requests is also initialized.
    ///
    /// # Panics
    /// if any of the required environment variables are missing.
    pub fn new() -> Self {
        Self {
            client_id: var("GOOGLE_CLIENT_ID")
                .expect("Missing GOOGLE_CLIENT_ID")
                .leak(),

            client_secret: var("GOOGLE_CLIENT_SECRET")
                .expect("Missing GOOGLE_CLIENT_SECRET")
                .leak(),

            redirect_uri: var("GOOGLE_REDIRECT_URI")
                .expect("Missing GOOGLE_REDIRECT_URI")
                .leak(),

            grant_type: var("GOOGLE_GRANT_TYPE")
                .expect("Missing GOOGLE_GRANT_TYPE")
                .leak(),

            reqwest_client: Client::new(),
        }
    }

    /// Retrieves Google account information for a user given a callback code.
    ///
    /// This method exchanges a callback code for an access token using Google's OAuth2 API,
    /// then uses the access token to fetch the user's account information.
    ///
    /// # Arguments
    /// - `callback_code`: The callback code received from Google after user authorization.
    ///
    /// # Returns
    /// An `AppResult<GoogleUserInfo>` containing the user's Google account information if successful.
    ///
    /// # Errors
    /// Returns an error if the request to Google's API fails or if the response cannot be parsed.
    pub async fn account_info(&self, callback_code: &str) -> AppResult<GoogleUserInfo> {
        let access_token = {
            let token_request = GoogleTokenExchangeRequest {
                client_id: self.client_id,
                client_secret: self.client_secret,
                code: callback_code,
                grant_type: self.grant_type,
                redirect_uri: self.redirect_uri,
            };

            let response = self
                .reqwest_client
                .post(GOOGLE_TOKEN_URL)
                .form(&token_request)
                .send()
                .await?
                .json::<GoogleAuthTokens>()
                .await?;

            response.access_token
        };

        let user_info = self
            .reqwest_client
            .get(GOOGLE_USER_INFO)
            .bearer_auth(access_token)
            .send()
            .await?
            .json::<GoogleUserInfo>()
            .await?;

        Ok(user_info)
    }
}
