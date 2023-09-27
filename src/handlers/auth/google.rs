use crate::dtos::TokenType;
use crate::repositories::UserRepositoryTrait;
use crate::services::{TokenServiceTrait, UserServiceTrait};
use crate::states::AuthState;
use crate::{dtos::GoogleCallbackQuery, error::AppResult};
use axum::extract::{Query, State};
use axum::response::Response;

const WEB_REDIRECT_URL: &str = "https://intellitelemetry.live/auth/google/callback";

pub async fn callback(
    State(state): State<AuthState>,
    query: Query<GoogleCallbackQuery>,
) -> AppResult<Response<String>> {
    let google_user = state
        .google_repository
        .account_info(&query.code)
        .await
        .unwrap();

    // TODO: Check if the provided type is the same as the one in the database
    let user = state
        .user_repository
        .find_by_email(&google_user.email)
        .await?;

    let user = match user {
        Some(user) => user,
        None => {
            let id = state.user_service.new_user(&google_user.into()).await?;
            state.user_repository.find(&id).await?.unwrap()
        }
    };

    let user_id = user.id.to_string();

    let access_token_task = state
        .token_service
        .generate_token(&user_id, TokenType::Bearer);

    let refresh_token_task = state
        .token_service
        .generate_refresh_token(&user_id, "google");

    let (access_token, refresh_token) = tokio::join!(access_token_task, refresh_token_task);

    let redirect_url = format!(
        "{WEB_REDIRECT_URL}?access_token={}&refresh_token={}",
        access_token.unwrap(),
        refresh_token.unwrap()
    );

    let resp = Response::builder()
        .header("Location", redirect_url)
        .status(302)
        .body("Redirecting...".into());

    Ok(resp.unwrap())
}
