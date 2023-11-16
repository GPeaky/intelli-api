use crate::{
    config::constants::*,
    dtos::GoogleCallbackQuery,
    dtos::TokenType,
    error::AppResult,
    repositories::UserRepositoryTrait,
    services::{TokenServiceTrait, UserServiceTrait},
    states::AuthState,
};
use axum::{
    extract::{Query, State},
    response::Response,
};

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

    let access_token_task = state
        .token_service
        .generate_token(user.id, TokenType::Bearer);

    let refresh_token_task = state
        .token_service
        .generate_refresh_token(&user.id, "google");

    let (access_token, refresh_token) = tokio::try_join!(access_token_task, refresh_token_task)?;

    let redirect_url = format!(
        "{GOOGLE_REDIRECT}?access_token={}&refresh_token={}",
        access_token, refresh_token
    );

    let resp = Response::builder()
        .header("Location", redirect_url)
        .status(302)
        .body("Redirecting...".into());

    Ok(resp.unwrap())
}
