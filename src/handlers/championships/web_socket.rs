use crate::{
    entity::Championship,
    error::{AppResult, ChampionshipError},
    states::WebSocketState,
};
use axum::{
    extract::{
        ws::{WebSocket, WebSocketUpgrade},
        Path, State,
    },
    response::Response, // IntoResponse
};
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
    mut _socket: WebSocket,
    _state: WebSocketState,
    _championship: Championship,
    _session_id: i64,
) {
    // TODO: Implement all the socket logic (Only Send data)
}
