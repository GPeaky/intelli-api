use crate::{entity::UserExtension, error::AppResult};
pub(crate) use admin::*;
use axum::{Extension, Json};

mod admin;

#[inline(always)]
pub(crate) async fn user_data(
    Extension(user): Extension<UserExtension>,
) -> AppResult<Json<UserExtension>> {
    Ok(Json(user))
}
