use crate::{
    config::constants::*,
    dtos::{GoogleAuthResponse, GoogleTokenRequest, GoogleUserInfo},
    error::AppResult,
};
use dotenvy::var;

pub struct GoogleRepository {
    client_id: String,
    client_secret: String,
    redirect_uri: String,
    grant_type: String,
    reqwest_client: reqwest::Client,
}

// FIX: this isn't  working, probably something changed in google api
impl GoogleRepository {
    pub fn new() -> Self {
        Self {
            client_id: var("GOOGLE_CLIENT_ID").expect("GOOGLE_CLIENT_ID secret not found"),
            client_secret: var("GOOGLE_CLIENT_SECRET").expect("GOOGLE_CLIENT_SECRET secret not found"),
            redirect_uri: var("GOOGLE_REDIRECT_URI").expect("GOOGLE_REDIRECT_URI secret not found"),
            grant_type: var("GOOGLE_GRANT_TYPE").expect("GOOGLE_GRANT_TYPE secret not found"),
            reqwest_client: reqwest::Client::new(),
        }
    }

    pub async fn account_info(&self, callback_code: &str) -> AppResult<GoogleUserInfo> {
        let access_token = {
            let token_request = GoogleTokenRequest {
                client_id: &self.client_id,
                client_secret: &self.client_secret,
                code: callback_code,
                grant_type: &self.grant_type,
                redirect_uri: &self.redirect_uri,
            };

            let response: GoogleAuthResponse = self
                .reqwest_client
                .post(GOOGLE_TOKEN_URL)
                .form(&token_request)
                .send()
                .await?
                .json()
                .await?;

            response.access_token
        };

        let user_info: GoogleUserInfo = self
            .reqwest_client
            .get(GOOGLE_USER_INFO)
            .bearer_auth(access_token)
            .send()
            .await?
            .json()
            .await?;

        Ok(user_info)
    }
}
