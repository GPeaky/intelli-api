use std::time::Duration;

use crate::{
    dtos::F123Data,
    entity::Championship,
    error::{AppResult, ChampionshipError},
    states::UserState,
};
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Path, State,
    },
    response::Response, // IntoResponse
};
use tokio::time::sleep;
use tracing::info;
// use rmp_serde::{Deserializer, Serializer};

#[inline(always)]
pub async fn session_socket(
    State(state): State<UserState>,
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
    state: UserState,
    championship: Championship,
    _session_id: i64,
) {
    // TODO: Implement all the socket logic (Only Send data)
    loop {
        let Some(f123_data) = state.f123_service.get_receiver(&championship.id).await else {
            sleep(Duration::from_millis(700)).await;
            continue;
        };

        match f123_data {
            F123Data::Motion(motion_data) => {
                let _ = socket
                    .send(Message::Text(
                        serde_json::to_string(&motion_data.m_carMotionData).unwrap(),
                    ))
                    .await;
            }

            F123Data::SessionHistory(history_data) => {
                let _ = socket
                    .send(Message::Text(serde_json::to_string(&history_data).unwrap()))
                    .await;
            }

            F123Data::Session(session_data) => {
                let _ = socket
                    .send(Message::Text(serde_json::to_string(&session_data).unwrap()))
                    .await;
            }

            F123Data::Event(event) => {
                let _ = socket
                    .send(Message::Text(serde_json::to_string(&event).unwrap()))
                    .await;
            }

            F123Data::Participants(participants) => {
                let _ = socket
                    .send(Message::Text(serde_json::to_string(&participants).unwrap()))
                    .await;
            }

            F123Data::FinalClassification(classification) => {
                let _ = socket
                    .send(Message::Text(
                        serde_json::to_string(&classification).unwrap(),
                    ))
                    .await;
            }
        }

        sleep(Duration::from_millis(700)).await
    }
}
