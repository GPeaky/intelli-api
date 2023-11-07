use crate::{dtos::UserData, entity::UserExtension, error::AppResult, states::UserState};
pub(crate) use admin::*;
use axum::{extract::State, Extension, Json};

mod admin;

#[inline(always)]
pub(crate) async fn user_data(
    State(state): State<UserState>,
    Extension(user): Extension<UserExtension>,
) -> AppResult<Json<UserData>> {
    let championships = state.championship_repository.find_all(&user.id).await?;

    Ok(Json(UserData {
        user,
        championships,
    }))
}
