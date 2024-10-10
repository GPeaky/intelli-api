use ntex::web::{
    types::{Query, State},
    HttpResponse,
};

use entities::Provider;
use error::{AppResult, UserError};
use intelli_core::services::UserServiceOperations;
use structs::{OauthAuthorizationCode, UserRegistrationData};
use token::TokenIntent;

use crate::states::AppState;

const DISCORD_REDIRECT: &str = "https://intellitelemetry.live/auth/discord/callback";

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

            state.user_repo.find(id).await?.unwrap() // Safe to unwrap
        }
    };

    let access_token = state.token_mgr.create(user.id, TokenIntent::Auth);

    let redirect_url = format!(
        "{DISCORD_REDIRECT}?access_token={}",
        access_token.as_base64()
    );

    Ok(HttpResponse::Found()
        .set_header("Location", redirect_url)
        .finish())
}
