use ntex::web::{
    types::{Path, State},
    HttpResponse,
};

use error::{AppResult, ChampionshipError};
use intelli_core::services::ChampionshipAdminServiceOperations;
use structs::ChampionshipId;

use crate::states::AppState;

// TODO: implement a method to update championship info
// TODO: implement an addUser and removeUser for admin

#[inline]
pub async fn active_championships(state: State<AppState>) -> AppResult<HttpResponse> {
    let services = state.f1_svc.services();
    Ok(HttpResponse::Ok().json(&services))
}

#[inline]
pub async fn delete_championship(
    state: State<AppState>,
    path: Path<ChampionshipId>,
) -> AppResult<HttpResponse> {
    let Some(championship) = state.championship_repo.find(path.0).await? else {
        Err(ChampionshipError::NotFound)?
    };

    state
        .championship_svc
        .admin_delete_championship(championship.id)
        .await?;

    Ok(HttpResponse::Ok().finish())
}
