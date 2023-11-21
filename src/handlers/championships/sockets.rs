use crate::{
    dtos::SocketStatus,
    error::{AppResult, ChampionshipError},
    // handlers::championships::websocket_active_connections,
    states::AppState,
};
use ntex::web;
use std::sync::Arc;

#[inline(always)]
pub async fn active_sockets(state: web::types::State<AppState>) -> AppResult<impl web::Responder> {
    let sockets = state.f123_service.get_active_socket_ids().await;
    Ok(web::HttpResponse::Ok().json(&sockets))
}

#[inline(always)]
pub async fn start_socket(
    state: web::types::State<AppState>,
    championship_id: web::types::Path<i32>,
) -> AppResult<impl web::Responder> {
    let Some(championship) = state.championship_repository.find(&championship_id).await? else {
        Err(ChampionshipError::NotFound)?
    };

    state
        .f123_service
        .setup_championship_listening_socket(championship.port, Arc::new(championship.id))
        .await?;

    Ok(web::HttpResponse::Created())
}

#[inline(always)]
pub async fn socket_status(
    state: web::types::State<AppState>,
    championship_id: web::types::Path<i32>,
) -> AppResult<impl web::Responder> {
    let Some(championship) = state.championship_repository.find(&championship_id).await? else {
        Err(ChampionshipError::NotFound)?
    };

    #[allow(unused_mut)]
    let mut num_connections = 0;
    let socket_active = state
        .f123_service
        .is_championship_socket_active(&championship.id)
        .await;

    // if socket_active {
    // num_connections = websocket_active_connections(*championship_id).await;
    // }

    let socket_status = SocketStatus {
        active: socket_active,
        connections: num_connections,
    };

    Ok(web::HttpResponse::Ok().json(&socket_status))
}

#[inline(always)]
pub async fn stop_socket(
    state: web::types::State<AppState>,
    championship_id: web::types::Path<i32>,
) -> AppResult<impl web::Responder> {
    state.f123_service.stop_socket(*championship_id).await?;

    Ok(web::HttpResponse::Ok())
}
