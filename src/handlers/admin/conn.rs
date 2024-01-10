use axum::{extract::State, Json};

use crate::{error::AppResult, states::AppState, structs::DatabasesStatus};

#[inline(always)]
pub async fn pool_status(state: State<AppState>) -> AppResult<Json<DatabasesStatus>> {
    let active_pools = state.server_repository.active_pools();

    Ok(Json(active_pools))
}
