use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use garde::Validate;

use crate::{
    entity::Championship,
    error::{AppResult, ChampionshipError, CommonError},
    states::AppState,
    structs::{ChampionshipIdPath, UserIdPath},
};

#[inline(always)]
pub async fn user_championships(
    state: State<AppState>,
    path: Path<UserIdPath>,
) -> AppResult<Json<Vec<Championship>>> {
    if path.validate(&()).is_err() {
        Err(CommonError::ValidationFailed)?
    }

    let championships = state.championship_repository.find_all(&path.id).await?;
    Ok(Json(championships))
}

#[inline(always)]
pub async fn delete_championship(
    state: State<AppState>,
    path: Path<ChampionshipIdPath>,
) -> AppResult<Response> {
    let Some(championship) = state.championship_repository.find(&path.id).await? else {
        Err(ChampionshipError::NotFound)?
    };

    state.championship_service.delete(&championship.id).await?;
    Ok(StatusCode::OK.into_response())
}
