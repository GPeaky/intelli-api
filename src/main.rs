use axum::Server;
use config::{initialize_tracing_subscriber, Database};
use dotenvy::{dotenv, var};
use hyper::Error;
use routes::service_routes;
use std::net::TcpListener;
use tracing::info;

mod config;
mod dtos;
mod handlers;
mod routes;
mod services;

#[tokio::main]
async fn main() -> Result<(), Error> {
    dotenv().ok();
    initialize_tracing_subscriber();
    Database::default();

    let listener = TcpListener::bind(var("HOST").unwrap()).unwrap();

    info!("Server listening on {}", listener.local_addr().unwrap());
    Server::from_tcp(listener)?.serve(service_routes()).await?;

    Ok(())
}

// let socket = UdpSocket::bind("127.0.0.1:20777").await?;
// let mut buff = vec![0; 2048];

// loop {
//     let (size, _) = socket.recv_from(&mut buff).await?;

//     if size >= 1 {
//         let data = deserialize_header(&buff[0..size])?;

//         match deserialize_packet(data.m_packetId, &buff[0..size]) {
//             Ok(packet) => match packet {
//                 F123Packet::CarDamage(car_damage) => {
//                     println!("CarDamage: {:#?}", car_damage);
//                 }
//                 F123Packet::Motion(motion_data) => {
//                     println!("Motion: {:#?}", motion_data);
//                 }
//                 F123Packet::Session(session_data) => {
//                     println!("Session: {:#?}", session_data);
//                 }
//                 F123Packet::LapData(lap_data) => {
//                     println!("LapData: {:#?}", lap_data);
//                 }
//                 F123Packet::Event(event_data) => {
//                     println!("Event: {:#?}", event_data);
//                 }
//                 F123Packet::Participants(participants_data) => {
//                     println!("Participants: {:#?}", participants_data);
//                 }
//                 F123Packet::CarSetups(car_setups) => {
//                     println!("CarSetups: {:#?}", car_setups);
//                 }
//                 F123Packet::CarTelemetry(car_telemetry) => {
//                     println!("CarTelemetry: {:#?}", car_telemetry);
//                 }
//                 F123Packet::CarStatus(car_status) => {
//                     println!("CarStatus: {:#?}", car_status);
//                 }
//                 F123Packet::FinalClassification(classification_data) => {
//                     println!("FinalClassification: {:#?}", classification_data);
//                 }
//                 F123Packet::LobbyInfo(lobby_info) => {
//                     println!("LobbyInfo: {:#?}", lobby_info);
//                 }
//                 F123Packet::SessionHistory(session_history) => {
//                     println!("SessionHistory: {:#?}", session_history);
//                 }
//                 F123Packet::TyresSets(tyres_sets) => {
//                     println!("TyresSets: {:#?}", tyres_sets);
//                 }
//                 F123Packet::MotionExData(motion_ex_data) => {
//                     println!("MotionExData: {:#?}", motion_ex_data);
//                 }
//             },
//             Err(e) => {
//                 eprintln!("Error al deserializar el paquete: {}", e);
//             }
//         }
//     } else {
//         eprintln!("Tamaño del paquete incorrecto: {} bytes", size);
//     }
// }
