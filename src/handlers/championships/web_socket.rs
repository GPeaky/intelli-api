use std::time::Duration;

use crate::{
    entity::Championship,
    error::{AppResult, ChampionshipError},
    states::WebSocketState,
};
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Path, State,
    },
    response::Response, // IntoResponse
};
use tokio::time::sleep;
// use rmp_serde::{Deserializer, Serializer};

#[inline(always)]
pub async fn session_socket(
    State(state): State<WebSocketState>,
    Path((championship_id, session_id)): Path<(String, i64)>,
    ws: WebSocketUpgrade,
) -> AppResult<Response> {
    let championship = state.championship_repository.find(&championship_id).await?;
    let session_exists = state
        .championship_repository
        .session_exists(&championship_id, session_id)
        .await?;

    if !session_exists {
        return Err(ChampionshipError::SessionNotFound)?;
    }

    Ok(ws.on_upgrade(move |socket| handle_socket(socket, state, championship, session_id)))
}

#[inline(always)]
async fn handle_socket(
    mut socket: WebSocket,
    state: WebSocketState,
    championship: Championship,
    _session_id: i64,
) {
    // TODO: Implement all the socket logic (Only Send data)
    loop {
        let Some(message) = state.f123_service.get_receiver(&championship.id).await else {
            sleep(Duration::from_millis(700)).await;
            continue;
        };

        let _ = socket.send(Message::Text(message)).await;

        sleep(Duration::from_millis(700)).await
    }
}
