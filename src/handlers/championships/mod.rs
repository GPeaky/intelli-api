use axum::{
    extract::State,
    response::{IntoResponse, Response},
    Extension, Form,
};
use garde::Validate;
use hyper::StatusCode;

use crate::{
    dtos::CreateChampionshipDto,
    entity::User,
    error::{AppResult, CommonError},
    states::UserState,
};

pub async fn create_championship(
    Extension(user): Extension<User>,
    State(mut state): State<UserState>,
    Form(form): Form<CreateChampionshipDto>,
) -> AppResult<Response> {
    if form.validate(&()).is_err() {
        return Err(CommonError::FormValidationFailed)?;
    }

    state
        .championship_service
        .create_championship(form, &user.id)
        .await?;

    Ok(StatusCode::OK.into_response())
}
