use crate::{entity::User, error::AppResult};
use axum::{Extension, Json};

#[inline(always)]
pub(crate) async fn user_data(Extension(user): Extension<User>) -> AppResult<Json<User>> {
    Ok(Json(user))
}
