use crate::states::UserState;
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::Response, // IntoResponse
};

#[inline(always)]
pub async fn session_socket(State(state): State<UserState>, ws: WebSocketUpgrade) -> Response {
    tracing::info!("session_socket: {ws:?}");

    ws.on_upgrade(move |socket| handle_socket(state, socket))
}

#[inline(always)]
async fn handle_socket(_state: UserState, mut socket: WebSocket) {
    while let Some(msg) = socket.recv().await {
        let Ok(msg) = msg else {
            break;
        };

        // let x = socket.protocol();

        // println!("Protocol: {:?}", x);

        tracing::info!("Received message from {:?}", msg);

        match msg {
            Message::Text(text) => {
                match text.as_str() {
                    "data" => {
                        tracing::info!("Received data");

                        socket.send(Message::Text("Data".to_owned())).await.unwrap();
                    }

                    "user" => {
                        tracing::info!("Received user");

                        socket.send(Message::Text("User".to_owned())).await.unwrap();
                    }

                    _ => {}
                }

                // tracing::info!("Received message from {:?}", text);
            }

            Message::Close(_) => {
                tracing::info!("Received close");
            }

            // Message::Binary(bin) => {
            //     tracing::info!("Received message from {:?}", bin);
            // }
            // Message::Ping(_) => {
            //     tracing::info!("Received ping");
            // }
            // Message::Pong(_) => {
            //     tracing::info!("Received pong");
            // }
            _ => {}
        }
    }
}
