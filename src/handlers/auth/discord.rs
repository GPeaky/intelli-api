use ntex::web::{
    types::{Query, State},
    HttpResponse,
};

use crate::{
    config::constants::*,
    entity::Provider,
    error::{AppResult, UserError},
    services::UserServiceOperations,
    states::AppState,
    structs::{OauthAuthorizationCode, TokenPurpose, UserRegistrationData},
};

pub async fn discord_callback(
    state: State<AppState>,
    query: Query<OauthAuthorizationCode>,
) -> AppResult<HttpResponse> {
    let discord_info = state.discord_repo.account_info(&query.code).await?;
    let user = state.user_repo.find_by_email(&discord_info.email).await?;

    let user = match user {
        Some(user) => {
            // TODO: Implement a redirect to website to show a ui error
            if user.provider != Provider::Discord {
                Err(UserError::WrongProvider)?
            }

            user
        }

        None => {
            let id = state
                .user_svc
                .create(UserRegistrationData::from_discord_user_info(discord_info))
                .await?;

            state.user_repo.find(id).await.unwrap().unwrap()
        }
    };

    let access_token = state
        .token_svc
        .generate_token(user.id, TokenPurpose::Authentication)?;

    let redirect_url = format!("{DISCORD_REDIRECT}?access_token={}", access_token);

    Ok(HttpResponse::Found()
        .set_header("Location", redirect_url)
        .finish())
}
