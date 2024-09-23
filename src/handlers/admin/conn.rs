use ntex::web::{types::State, HttpResponse};

use crate::states::AppState;

#[inline(always)]
pub async fn server_active_pools(state: State<AppState>) -> HttpResponse {
    let active_pools = state.server_repo.active_pools();
    HttpResponse::Ok().json(&active_pools)
}
