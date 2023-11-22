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
// use log::{error, info};

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
