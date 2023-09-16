use crate::{
    entity::Championship,
    error::{AppResult, SocketError},
    states::SafeUserState,
};
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Path, State,
    },
    response::Response, // IntoResponse
};
use tracing::{error, info};

#[inline(always)]
pub async fn session_socket(
    State(state): State<SafeUserState>,
    Path(championship_id): Path<u32>,
    ws: WebSocketUpgrade,
) -> AppResult<Response> {
    let championship = state.championship_repository.find(&championship_id).await?;

    let socket_active = state
        .f123_service
        .championship_socket(&championship.id)
        .await;

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

    loop {
        tokio::select! {
            result = rx.recv() => {
                match result {
                    Ok(data) => {
                        if let Err(e) = socket.send(Message::Binary(rmp_serde::to_vec(&data).unwrap())).await {
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
}
