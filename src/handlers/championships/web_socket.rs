use crate::{
    entity::Championship,
    error::{AppResult, SocketError, UserError},
    states::SafeUserState,
};
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Path, State,
    },
    response::Response,
};
use dashmap::DashMap;
use once_cell::sync::Lazy;
use std::sync::atomic::AtomicUsize;
use tracing::{error, info};

static ACTIVE_CONNECTIONS: Lazy<DashMap<u32, AtomicUsize>> = Lazy::new(DashMap::new);

#[inline(always)]
pub async fn session_socket(
    State(state): State<SafeUserState>,
    Path(championship_id): Path<u32>,
    ws: WebSocketUpgrade,
) -> AppResult<Response> {
    let Some(championship) = state.championship_repository.find(&championship_id).await? else {
        Err(UserError::ChampionshipNotFound)?
    };

    let socket_active = state.f123_service.championship_socket(&championship.id);

    if !socket_active {
        Err(SocketError::NotActive)?
    }

    Ok(ws.on_upgrade(move |socket| handle_socket(socket, state, championship)))
}

#[inline(always)]
async fn handle_socket(mut socket: WebSocket, state: SafeUserState, championship: Championship) {
    let Some(mut rx) = state.f123_service.get_receiver(&championship.id).await else {
        error!("Receiver not Found");
        return;
    };

    increment_counter(championship.id);

    loop {
        tokio::select! {
            result = rx.recv() => {
                match result {
                    Ok(data) => {
                        if let Err(e) = socket.send(Message::Binary(data)).await {
                            error!("Failed sending message:{}", e);
                            break;
                        };
                    }

                    Err(err) => {
                        error!("Error receiving from broadcast channel: {}", err);
                        break;
                    }
                }
            }

            result = socket.recv() => {
                match result {
                    Some(_) => {}
                    None => {
                        info!("Connection Closed");
                        break;
                    }
                }
            }
        }
    }

    decrement_counter(championship.id);
}

#[inline(always)]
fn increment_counter(championship_id: u32) {
    let counter = ACTIVE_CONNECTIONS
        .entry(championship_id)
        .or_insert(AtomicUsize::new(0));
    counter.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
}

#[inline(always)]
fn decrement_counter(championship_id: u32) {
    let counter = ACTIVE_CONNECTIONS
        .entry(championship_id)
        .or_insert(AtomicUsize::new(0));
    counter.fetch_sub(1, std::sync::atomic::Ordering::Relaxed);
}

pub fn websocket_active_connections(championship_id: u32) -> usize {
    let counter = ACTIVE_CONNECTIONS
        .entry(championship_id)
        .or_insert(AtomicUsize::new(0));
    counter.load(std::sync::atomic::Ordering::Relaxed)
}
