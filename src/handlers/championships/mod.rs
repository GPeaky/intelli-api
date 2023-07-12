use crate::{
    dtos::CreateChampionshipDto,
    entity::{Championship, User},
    error::{AppResult, CommonError},
    states::UserState,
};
use axum::{
    extract::{Path, State},
    response::{IntoResponse, Response},
    Extension, Form, Json,
};
use garde::Validate;
use hyper::StatusCode;

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

pub async fn get_championship(
    State(state): State<UserState>,
    Path(championship_id): Path<String>,
) -> AppResult<Json<Championship>> {
    let championship = state.championship_repository.find(&championship_id).await?;

    Ok(Json(championship))
}

pub async fn start_socket(
    State(state): State<UserState>,
    Path(championship_id): Path<String>,
) -> AppResult<Response> {
    let championship = state.championship_repository.find(&championship_id).await?;

    state
        .f123_service
        .new_socket(championship.port, championship_id)
        .await;

    Ok(StatusCode::CREATED.into_response())
}

pub async fn stop_socket(
    State(state): State<UserState>,
    Path(championship_id): Path<String>,
) -> AppResult<Response> {
    state.f123_service.stop_socket(championship_id).await?;

    Ok(StatusCode::OK.into_response())
}
