use redis::JsonCommands;
use services::f123::{deserialize_header, deserialize_packet, F123Packet};
use tokio::net::UdpSocket;
use tokio_stream::StreamExt;
use tokio_util::{codec::BytesCodec, udp::UdpFramed};

mod config;
mod dtos;
mod services;

#[tokio::main]
async fn main() {
    let db = config::Database::default();
    let socket = UdpSocket::bind("127.0.0.1:20777").await.unwrap();
    let mut stream = UdpFramed::new(socket, BytesCodec::new());
    let mut redis = db.get_redis();

    while let Some(Ok((bytes, _))) = stream.next().await {
        let data = deserialize_header(&bytes).unwrap();

        if data.m_sessionUID == 0 {
            continue;
        }

        if let Ok(packet) = deserialize_packet(data.m_packetId, &bytes) {
            match packet {
                F123Packet::Motion(motion_data) => {
                    let session_id = motion_data.m_header.m_sessionUID;

                    motion_data
                        .m_carMotionData
                        .iter()
                        .enumerate()
                        .for_each(|(index, motion)| {
                            let _: () = redis
                                .json_set(
                                    format!("sessions:{}:car:{}:live-position", session_id, index),
                                    ".",
                                    motion,
                                )
                                .unwrap();
                        });
                }

                F123Packet::Session(session_data) => {
                    let session_id = session_data.m_header.m_sessionUID;

                    let _: () = redis
                        .json_set(
                            format!("sessions:{}:session_data", session_id),
                            ".",
                            &session_data,
                        )
                        .unwrap();
                }
                F123Packet::LapData(lap_data) => {
                    let session_id = lap_data.m_header.m_sessionUID;

                    lap_data
                        .m_lapData
                        .iter()
                        .enumerate()
                        .for_each(|(index, lap)| {
                            let lap_n = lap.m_currentLapNum;

                            let _: () = redis
                                .json_set(
                                    format!("sessions:{}:car:{}:lap:{}", session_id, index, lap_n),
                                    ".",
                                    lap,
                                )
                                .unwrap();
                        });
                }

                F123Packet::Event(event_data) => {
                    let session_id = event_data.m_header.m_sessionUID;

                    let _: () = redis
                        .json_set(
                            format!("sessions:{}:event", session_id),
                            ".",
                            &event_data.m_eventDetails,
                        )
                        .unwrap();
                }

                F123Packet::Participants(participants_data) => {
                    let session_id = participants_data.m_header.m_sessionUID;

                    participants_data
                        .m_participants
                        .iter()
                        .enumerate()
                        .for_each(|(index, participant)| {
                            let _: () = redis
                                .json_set(
                                    format!("sessions:{}:participants:{}", session_id, index),
                                    ".",
                                    participant,
                                )
                                .unwrap();
                        })
                }

                F123Packet::FinalClassification(classification_data) => {
                    let session_id = classification_data.m_header.m_sessionUID;

                    classification_data
                        .m_classificationData
                        .iter()
                        .enumerate()
                        .for_each(|(index, classification)| {
                            let _: () = redis
                                .json_set(
                                    format!("sessions:{}:car:{}:classification", session_id, index),
                                    ".",
                                    classification,
                                )
                                .unwrap();
                        })
                }

                F123Packet::LobbyInfo(lobby_info) => {
                    let session_id = lobby_info.m_header.m_sessionUID;

                    let _: () = redis
                        .json_set(
                            format!("sessions:{}:lobby", session_id),
                            ".",
                            &lobby_info.m_lobbyPlayers,
                        )
                        .unwrap();
                }

                _ => {}
            }
        }
    }
}
