use crate::{
    error::{AppResult, ChampionshipError, SocketError},
    states::AppState,
};
use counter::*;
use log::info;
use ntex::web;
use once_cell::sync::Lazy;

mod counter;

#[allow(unused)]
static COUNTER: Lazy<WebSocketCounter> = Lazy::new(WebSocketCounter::new);

pub async fn session_socket(
    _req: web::HttpRequest,
    state: web::types::State<AppState>,
    championship_id: web::types::Path<i32>,
) -> AppResult<impl web::Responder> {
    let Some(championship) = state.championship_repository.find(&championship_id).await? else {
        Err(ChampionshipError::NotFound)?
    };

    let socket_active = state
        .f123_service
        .is_championship_socket_active(&championship.id)
        .await;

    if !socket_active {
        Err(SocketError::NotActive)?
    }

    //* Testing counter
    COUNTER.increment(championship.id);

    let get = COUNTER.get(championship.id);
    info!("Counter Size: {:?}", get);

    COUNTER.decrement(championship.id);

    info!("Implement socket");
    // web::ws::start(req, fn_factory_with_config())
    Ok(web::HttpResponse::Ok())
}
