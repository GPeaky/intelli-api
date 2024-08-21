use garde::Validate;
use ntex::web::{
    types::{Path, State},
    HttpResponse,
};

use crate::{
    error::{AppResult, ChampionshipError, CommonError},
    states::AppState,
    structs::{ChampionshipId, UserId},
};

#[inline(always)]
pub async fn user_championships(
    state: State<AppState>,
    path: Path<UserId>,
) -> AppResult<HttpResponse> {
    if path.validate().is_err() {
        Err(CommonError::ValidationFailed)?
    }

    let championships = state.championship_repo.find_all(path.0).await?;
    Ok(HttpResponse::Ok().json(&championships))
}

#[inline(always)]
pub async fn delete_championship(
    state: State<AppState>,
    path: Path<ChampionshipId>,
) -> AppResult<HttpResponse> {
    let Some(championship) = state.championship_repo.find(path.0).await? else {
        Err(ChampionshipError::NotFound)?
    };

    state.championship_svc.delete(championship.id).await?;
    Ok(HttpResponse::Ok().finish())
}
