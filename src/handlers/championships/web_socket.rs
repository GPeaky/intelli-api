use crate::{entity::Championship, error::AppResult, states::WebSocketState};
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
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

    Ok(ws.on_upgrade(move |socket| handle_socket(socket, state, championship, session_id)))
}

#[inline(always)]
async fn handle_socket(
    mut socket: WebSocket,
    _state: WebSocketState,
    _championship: Championship,
    _session_id: i64,
) {
    while let Some(msg) = socket.recv().await {
        let Ok(msg) = msg else {
            break;
        };

        // TODO: Check if the user is authenticated
        // TODO: Check if the user is authorized to access the data
        // TODO: handle a message when data is updated and send it to the client
        // TODO: Think about how to handle the data & send it to the client (maybe a stream?)
        match msg {
            Message::Text(text) => match text.as_str() {
                "data" => {
                    tracing::info!("Received data");

                    socket.send(Message::Text("Data".to_owned())).await.unwrap();
                }

                "user" => {
                    tracing::info!("Received user");

                    socket.send(Message::Text("User".to_owned())).await.unwrap();
                }

                _ => {}
            },

            Message::Close(_) => {
                tracing::info!("Received close");
            }

            _ => {}
        }
    }
}
