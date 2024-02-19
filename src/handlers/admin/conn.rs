use ntex::web::{types::State, HttpResponse, Responder};

use crate::{error::AppResult, states::AppState};

#[inline(always)]
pub async fn pool_status(state: State<AppState>) -> AppResult<impl Responder> {
    let active_pools = &state.server_repo.active_pools();

    Ok(HttpResponse::Ok().json(active_pools))
}
