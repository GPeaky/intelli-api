use axum::{
    extract::ws::{WebSocket, WebSocketUpgrade},
    response::Response, // IntoResponse
};

#[inline(always)]
pub async fn session_socket(ws: WebSocketUpgrade) -> Response {
    tracing::info!("session_socket: {ws:?}");

    ws.on_upgrade(handle_socket)
}

#[inline(always)]
async fn handle_socket(mut socket: WebSocket) {
    while let Some(msg) = socket.recv().await {
        let Ok(msg) = msg else {
            break;
        };

        tracing::info!("Received message from {:?}", msg);
    }
}
