use super::counter::get;
use crate::dtos::ChampionshipIdPath;
use crate::error::CommonError;
use crate::{
    dtos::SocketStatus,
    error::{AppResult, ChampionshipError},
    states::AppState,
};
use garde::Validate;
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
    path: web::types::Path<ChampionshipIdPath>,
) -> AppResult<impl web::Responder> {
    if path.validate(&()).is_err() {
        Err(CommonError::ValidationFailed)?
    }

    let Some(championship) = state.championship_repository.find(&path.id).await? else {
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
    path: web::types::Path<ChampionshipIdPath>,
) -> AppResult<impl web::Responder> {
    if path.validate(&()).is_err() {
        Err(CommonError::ValidationFailed)?
    }

    let Some(championship) = state.championship_repository.find(&path.id).await? else {
        Err(ChampionshipError::NotFound)?
    };

    let mut num_connections = 0;
    let socket_active = state
        .f123_service
        .is_championship_socket_active(&championship.id)
        .await;

    if socket_active {
        if let Some(count) = get(path.id) {
            num_connections = count;
        };
    }

    let socket_status = SocketStatus {
        active: socket_active,
        connections: num_connections,
    };

    Ok(web::HttpResponse::Ok().json(&socket_status))
}

#[inline(always)]
pub async fn stop_socket(
    state: web::types::State<AppState>,
    path: web::types::Path<ChampionshipIdPath>,
) -> AppResult<impl web::Responder> {
    if path.validate(&()).is_err() {
        Err(CommonError::ValidationFailed)?
    }

    state.f123_service.stop_socket(path.id).await?;

    Ok(web::HttpResponse::Ok())
}
