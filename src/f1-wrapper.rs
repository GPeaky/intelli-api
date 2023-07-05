use bincode::serialize;
use dotenvy::var;
use scylla::SessionBuilder;
use services::f123::{deserialize_header, deserialize_packet, F123Packet};
use std::{
    collections::HashMap,
    time::{Duration, Instant},
};
use tokio::net::UdpSocket;
use tokio_stream::StreamExt;
use tokio_util::{codec::BytesCodec, udp::UdpFramed};

mod config;
mod dtos;
mod services;

#[tokio::main]
async fn main() {
    let mut last_session_update: HashMap<i64, Instant> = HashMap::new();
    let mut last_car_motion_update: HashMap<i64, Instant> = HashMap::new();
    let socket = UdpSocket::bind("127.0.0.1:20777").await.unwrap();
    let mut stream = UdpFramed::new(socket, BytesCodec::new());
    let session = SessionBuilder::new()
        .known_node(var("DB_URL").unwrap())
        .build()
        .await
        .unwrap();

    // session.query("CREATE KEYSPACE IF NOT EXISTS intelli_api WITH REPLICATION = {'class' : 'NetworkTopologyStrategy', 'replication_factor' : 1}", &[]).await.unwrap();

    // session
    //     .query(
    //         "CREATE TABLE IF NOT EXISTS intelli_api.game_sessions (
    //             id bigint PRIMARY KEY,
    //             data BLOB
    //         )",
    //         &[],
    //     )
    //     .await
    //     .unwrap();

    // session
    //     .query(
    //         "CREATE TABLE IF NOT EXISTS intelli_api.car_motion (
    //             session_id bigint PRIMARY KEY,
    //             data BLOB
    //         )",
    //         &[],
    //     )
    //     .await
    //     .unwrap();

    // session
    //     .query(
    //         "CREATE TABLE IF NOT EXISTS intelli_api.lap_data (
    //             session_id bigint PRIMARY KEY,
    //             lap BLOB,
    //         );",
    //         &[],
    //     )
    //     .await
    //     .unwrap();

    // session
    //     .query(
    //         "CREATE TABLE IF NOT EXISTS intelli_api.event_data (
    //         session_id bigint PRIMARY KEY,
    //         string_code text,
    //         event BLOB
    //     );",
    //         &[],
    //     )
    //     .await
    //     .unwrap();

    // session
    //     .query(
    //         "CREATE TABLE IF NOT EXISTS intelli_api.participants_data (
    //             session_id bigint PRIMARY KEY,
    //             participants BLOB,
    //         );",
    //         &[],
    //     )
    //     .await
    //     .unwrap();

    // session
    //     .query(
    //         "CREATE TABLE IF NOT EXISTS intelli_api.final_classification_data (
    //         session_id bigint PRIMARY KEY,
    //         classification BLOB,
    //     );",
    //         &[],
    //     )
    //     .await
    //     .unwrap();

    let insert_game_session = session
        .prepare("INSERT INTO intelli_api.game_sessions (id, data) VALUES (?,?);")
        .await
        .unwrap();

    let insert_car_motion = session
        .prepare("INSERT INTO intelli_api.car_motion (session_id, data) VALUES (?,?);")
        .await
        .unwrap();

    let insert_lap_data = session
        .prepare("INSERT INTO intelli_api.lap_data (session_id, lap) VALUES (?,?);")
        .await
        .unwrap();

    let insert_event_data = session
        .prepare(
            "INSERT INTO intelli_api.event_data (session_id, string_code, event) VALUES (?,?,?);",
        )
        .await
        .unwrap();

    let insert_participant_data = session
        .prepare(
            "INSERT INTO intelli_api.participants_data (session_id, participants) VALUES (?,?);",
        )
        .await
        .unwrap();

    let insert_final_classification_data = session
        .prepare("INSERT INTO intelli_api.final_classification_data (session_id, classification) VALUES (?,?);")
        .await
        .unwrap();

    while let Some(Ok((bytes, _))) = stream.next().await {
        let data = deserialize_header(&bytes).unwrap();
        let session_id = data.m_sessionUID as i64;

        if data.m_sessionUID == 0 {
            continue;
        }

        if let Ok(packet) = deserialize_packet(data.m_packetId, &bytes) {
            match packet {
                F123Packet::Motion(motion_data) => {
                    let now = Instant::now();

                    if !last_car_motion_update.contains_key(&session_id)
                        || now.duration_since(last_car_motion_update[&session_id])
                            >= Duration::from_millis(500)
                    {
                        let data = serialize(&motion_data).unwrap();

                        session
                            .execute(&insert_car_motion, (session_id, data))
                            .await
                            .unwrap();

                        last_car_motion_update.insert(session_id, now);
                    }
                }

                F123Packet::Session(session_data) => {
                    let now = Instant::now();

                    if !last_session_update.contains_key(&session_id)
                        || now.duration_since(last_session_update[&session_id])
                            >= Duration::from_secs(30)
                    {
                        let data = serialize(&session_data).unwrap();

                        session
                            .execute(&insert_game_session, (session_id, data))
                            .await
                            .unwrap();

                        last_session_update.insert(session_id, now);
                    }
                }

                F123Packet::LapData(lap_data) => {
                    let lap_info = serialize(&lap_data.m_lapData).unwrap();

                    // TODO: Save lap data to database
                    session
                        .execute(&insert_lap_data, (session_id, lap_info))
                        .await
                        .unwrap();
                }

                F123Packet::Event(event_data) => {
                    let event = serialize(&event_data.m_eventDetails).unwrap();

                    session
                        .execute(
                            &insert_event_data,
                            (session_id, event_data.m_eventStringCode, event),
                        )
                        .await
                        .unwrap();
                }

                F123Packet::Participants(participants_data) => {
                    let participants = serialize(&participants_data.m_participants).unwrap();

                    session
                        .execute(&insert_participant_data, (session_id, participants))
                        .await
                        .unwrap();
                }

                F123Packet::FinalClassification(classification_data) => {
                    let classifications =
                        serialize(&classification_data.m_classificationData).unwrap();

                    session
                        .execute(
                            &insert_final_classification_data,
                            (session_id, classifications),
                        )
                        .await
                        .unwrap();
                }

                _ => {}
            }
        }
    }
}
