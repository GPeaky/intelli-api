use crate::{entity::User, error::AppResult};
pub(crate) use admin::*;
use axum::{Extension, Json};

mod admin;

#[inline(always)]
pub(crate) async fn user_data(Extension(user): Extension<User>) -> AppResult<Json<User>> {
    Ok(Json(user))
}
