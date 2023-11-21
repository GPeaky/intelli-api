// use crate::{
//     entity::Championship,
//     error::{AppResult, ChampionshipError, SocketError},
//     protos::{packet_header::PacketType, ToProtoMessage},
//     states::UserState,
// };
// use axum::{
//     extract::{
//         ws::{Message, WebSocket, WebSocketUpgrade},
//         Path, State,
//     },
//     response::Response,
// };
// use once_cell::sync::Lazy;
// use rustc_hash::FxHashMap;
// use std::sync::{
//     atomic::{AtomicUsize, Ordering},
//     Arc,
// };
// use tokio::sync::RwLock;
// use tracing::{error, info};

// static DEFAULT_COUNTER: AtomicUsize = AtomicUsize::new(0);
// static ACTIVE_CONNECTIONS: Lazy<Arc<RwLock<FxHashMap<i32, AtomicUsize>>>> =
//     Lazy::new(|| Arc::new(RwLock::new(FxHashMap::default())));

// #[inline(always)]
// pub async fn session_socket(
//     State(state): State<UserState>,
//     Path(championship_id): Path<i32>,
//     ws: WebSocketUpgrade,
// ) -> AppResult<Response> {
//     let Some(championship) = state.championship_repository.find(&championship_id).await? else {
//         Err(ChampionshipError::NotFound)?
//     };

//     let socket_active = state
//         .f123_service
//         .is_championship_socket_active(&championship.id)
//         .await;

//     if !socket_active {
//         Err(SocketError::NotActive)?
//     }

//     Ok(ws.on_upgrade(move |socket| handle_socket(socket, state, championship)))
// }

// #[inline(always)]
// async fn increment_counter(championship_id: i32) {
//     let mut active_connections = ACTIVE_CONNECTIONS.write().await;
//     let counter = active_connections
//         .entry(championship_id)
//         .or_insert(AtomicUsize::new(0));

//     counter.fetch_add(1, Ordering::Relaxed);
// }

// #[inline(always)]
// async fn decrement_counter(championship_id: i32) {
//     let mut active_connections = ACTIVE_CONNECTIONS.write().await;
//     let counter = active_connections
//         .entry(championship_id)
//         .or_insert(AtomicUsize::new(1));

//     counter.fetch_sub(1, Ordering::Relaxed);
// }

// pub async fn websocket_active_connections(championship_id: i32) -> usize {
//     let active_connection = ACTIVE_CONNECTIONS.read().await;
//     let counter = active_connection
//         .get(&championship_id)
//         .unwrap_or(&DEFAULT_COUNTER);

//     counter.load(Ordering::Relaxed)
// }

// #[inline(always)]
// async fn handle_socket(mut socket: WebSocket, state: UserState, championship: Championship) {
//     let Some(mut rx) = state
//         .f123_service
//         .subscribe_to_championship_events(&championship.id)
//         .await
//     else {
//         error!("Receiver not Found");
//         return;
//     };

//     increment_counter(championship.id).await;

//     //* Loading cache messages and sending to the new connection
//     {
//         let cache = state
//             .championship_repository
//             .session_data(&championship.id)
//             .await
//             .unwrap();

//         let data = vec![
//             cache.session_data,
//             cache.motion_data,
//             cache.participants_data,
//         ];

//         // TODO: Remove packet_type from here
//         let Some(data) = data.convert_and_encode(PacketType::SessionData) else {
//             error!("Failed converting data to proto");
//             return;
//         };

//         if let Err(e) = socket.send(Message::Binary(data)).await {
//             error!("Failed sending message:{}", e);
//             return;
//         };
//     }

//     loop {
//         tokio::select! {
//             result = rx.recv() => {
//                 match result {
//                     Ok(data) => {
//                         if let Err(e) = socket.send(Message::Binary(data)).await {
//                             error!("Failed sending message:{}", e);
//                             break;
//                         };
//                     }

//                     Err(err) => {
//                         error!("Error receiving from broadcast channel: {}", err);
//                         break;
//                     }
//                 }
//             }

//             result = socket.recv() => {
//                 match result {
//                     Some(_) => {}
//                     None => {
//                         info!("Connection Closed");
//                         break;
//                     }
//                 }
//             }
//         }
//     }

//     decrement_counter(championship.id).await;
// }
