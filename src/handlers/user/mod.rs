use axum::{
    extract::{Extension, Form, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use garde::Validate;

pub(crate) use admin::*;

use crate::{
    entity::UserExtension,
    error::{AppResult, CommonError},
    services::UserServiceTrait,
    states::AppState,
    structs::{UpdateUser, UserData},
};

mod admin;

#[inline(always)]
pub(crate) async fn user_data(
    state: State<AppState>,
    Extension(user): Extension<UserExtension>,
) -> AppResult<Json<UserData>> {
    let championships = state.championship_repository.find_all(&user.id).await?;

    let user_data = UserData {
        user,
        championships,
    };

    Ok(Json(user_data))
}

#[inline(always)]
pub(crate) async fn update_user(
    state: State<AppState>,
    Extension(user): Extension<UserExtension>,
    form: Form<UpdateUser>,
) -> AppResult<Response> {
    if form.validate(&()).is_err() {
        Err(CommonError::ValidationFailed)?
    };

    state.user_service.update(&user, &form).await?;
    Ok(StatusCode::OK.into_response())
}
