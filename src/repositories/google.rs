use crate::dtos::{GoogleAuthResponse, GoogleTokenRequest, GoogleUserInfo};
use crate::error::AppResult;
use dotenvy::var;

pub struct GoogleRepository {
    client_id: String,
    client_secret: String,
    redirect_uri: String,
    grant_type: String,
    reqwest_client: reqwest::Client,
}

impl GoogleRepository {
    pub fn new() -> Self {
        Self {
            client_id: var("GOOGLE_CLIENT_ID").unwrap(),
            client_secret: var("GOOGLE_CLIENT_SECRET").unwrap(),
            redirect_uri: var("GOOGLE_REDIRECT_URI").unwrap(),
            grant_type: var("GOOGLE_GRANT_TYPE").unwrap(),
            reqwest_client: reqwest::Client::new(),
        }
    }

    pub async fn account_info(&self, callback_code: &str) -> AppResult<GoogleUserInfo> {
        let token_request = GoogleTokenRequest {
            client_id: &self.client_id,
            client_secret: &self.client_secret,
            code: callback_code,
            grant_type: &self.grant_type,
            redirect_uri: &self.redirect_uri,
        };

        let response: GoogleAuthResponse = self
            .reqwest_client
            .post("https://oauth2.googleapis.com/token")
            .form(&token_request)
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();

        let user_info: GoogleUserInfo = self
            .reqwest_client
            .get("https://www.googleapis.com/oauth2/v2/userinfo")
            .bearer_auth(response.access_token)
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();

        Ok(user_info)
    }
}
