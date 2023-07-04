use dotenvy::var;
use scylla::SessionBuilder;
use services::f123::{deserialize_header, deserialize_packet, F123Packet};
use tokio::net::UdpSocket;
use tokio_stream::StreamExt;
use tokio_util::{codec::BytesCodec, udp::UdpFramed};

mod config;
mod dtos;
mod services;

#[tokio::main]
async fn main() {
    let socket = UdpSocket::bind("127.0.0.1:20777").await.unwrap();
    let mut stream = UdpFramed::new(socket, BytesCodec::new());
    let session = SessionBuilder::new()
        .known_node(var("DB_URL").unwrap())
        .build()
        .await
        .unwrap();

    session.query("CREATE KEYSPACE IF NOT EXISTS intelli_api WITH REPLICATION = {'class' : 'NetworkTopologyStrategy', 'replication_factor' : 1}", &[]).await.unwrap();

    // session
    //     .query(
    //         "CREATE TYPE intelli_api.marshal_zone (
    //             m_zoneStart float,
    //             m_zoneFlag tinyint,
    //         );
    //     ",
    //         &[],
    //     )
    //     .await
    //     .unwrap();

    // session
    //     .query(
    //         " CREATE TYPE intelli_api.weather_forecast_sample (
    //             m_sessionType tinyint,
    //             m_timeOffset tinyint,
    //             m_weather tinyint,
    //             m_trackTemperature tinyint,
    //             m_trackTemperatureChange tinyint,
    //             m_airTemperature tinyint,
    //             m_airTemperatureChange tinyint,
    //             m_rainPercentage tinyint,
    //         );",
    //         &[],
    //     )
    //     .await
    //     .unwrap();

    // session
    //     .query(
    //         "CREATE TYPE intelli_api.packet_header (
    //             m_packetFormat smallint,
    //             m_gameYear tinyint,
    //             m_gameMajorVersion tinyint,
    //             m_gameMinorVersion tinyint,
    //             m_packetVersion tinyint,
    //             m_packetId tinyint,
    //             m_sessionUID bigint,
    //             m_sessionTime float,
    //             m_frameIdentifier int,
    //             m_overallFrameIdentifier int,
    //             m_playerCarIndex tinyint,
    //             m_secondaryPlayerCarIndex tinyint,
    //         );",
    //         &[],
    //     )
    //     .await
    //     .unwrap();

    // session
    //     .query(
    //         "CREATE TYPE intelli_api.packet_session_data (
    //             m_header frozen<packet_header>,
    //             m_weather tinyint,
    //             m_trackTemperature tinyint,
    //             m_airTemperature tinyint,
    //             m_totalLaps tinyint,
    //             m_trackLength smallint,
    //             m_sessionType tinyint,
    //             m_trackId tinyint,
    //             m_formula tinyint,
    //             m_sessionTimeLeft smallint,
    //             m_sessionDuration smallint,
    //             m_pitSpeedLimit tinyint,
    //             m_gamePaused tinyint,
    //             m_isSpectating tinyint,
    //             m_spectatorCarIndex tinyint,
    //             m_sliProNativeSupport tinyint,
    //             m_numMarshalZones tinyint,
    //             m_marshalZones frozen<list<frozen<marshal_zone>>>,
    //             m_safetyCarStatus tinyint,
    //             m_networkGame tinyint,
    //             m_numWeatherForecastSamples tinyint,
    //             m_forecastAccuracy tinyint,
    //             m_aiDifficulty tinyint,
    //             m_seasonLinkIdentifier int,
    //             m_weekendLinkIdentifier int,
    //             m_sessionLinkIdentifier int,
    //             m_pitStopWindowIdealLap tinyint,
    //             m_pitStopWindowLatestLap tinyint,
    //             m_pitStopRejoinPosition tinyint,
    //             m_steeringAssist tinyint,
    //             m_brakingAssist tinyint,
    //             m_gearboxAssist tinyint,
    //             m_pitAssist tinyint,
    //             m_pitReleaseAssist tinyint,
    //             m_ERSAssist tinyint,
    //             m_DRSAssist tinyint,
    //             m_dynamicRacingLine tinyint,
    //             m_dynamicRacingLineType tinyint,
    //             m_gameMode tinyint,
    //             m_ruleSet tinyint,
    //             m_timeOfDay int,
    //             m_sessionLength tinyint,
    //             m_speedUnitsLeadPlayer tinyint,
    //             m_temperatureUnitsLeadPlayer tinyint,
    //             m_speedUnitsSecondaryPlayer tinyint,
    //             m_temperatureUnitsSecondaryPlayer tinyint,
    //             m_numSafetyCarPeriods tinyint,
    //             m_numVirtualSafetyCarPeriods tinyint,
    //             m_numRedFlagPeriods tinyint,
    //             m_weatherForecastSamples frozen<list<frozen<weather_forecast_sample>>>,
    //         );",
    //         &[],
    //     )
    //     .await
    //     .unwrap();

    // session
    //     .query(
    //         "CREATE TABLE IF NOT EXISTS intelli_api.game_sessions (id bigint PRIMARY KEY, data intelli_api.packet_session_data);",
    //         &[],
    //     )
    //     .await
    //     .unwrap();

    while let Some(Ok((bytes, _))) = stream.next().await {
        let data = deserialize_header(&bytes).unwrap();

        if data.m_sessionUID == 0 {
            continue;
        }

        if let Ok(packet) = deserialize_packet(data.m_packetId, &bytes) {
            match packet {
                F123Packet::Motion(motion_data) => {
                    let session_id = motion_data.m_header.m_sessionUID;

                    // session.execute_iter(prepared, values)

                    // motion_data
                    //     .m_carMotionData
                    //     .iter()
                    //     .enumerate()
                    //     .for_each(|(index, motion)| {
                    //         let _: () = redis
                    //             .json_set(
                    //                 format!("sessions:{}:car:{}:live-position", session_id, index),
                    //                 ".",
                    //                 motion,
                    //             )
                    //             .unwrap();
                    //     });
                }

                F123Packet::Session(session_data) => {
                    let session_id = session_data.m_header.m_sessionUID as i64;
                    let data = serde_json::to_string(&session_data).unwrap();

                    session
                        .query(
                            "INSERT INTO intelli_api.game_sessions (id, data) VALUES (?,?);",
                            (session_id, data),
                        )
                        .await
                        .unwrap();

                    let result_data = session
                        .query(
                            "SELECT data FROM intelli_api.game_sessions WHERE id = ?;",
                            (session_id,),
                        )
                        .await
                        .unwrap();

                    dbg!(result_data);

                    // let _: () = redis
                    //     .json_set(
                    //         format!("sessions:{}:session_data", session_id),
                    //         ".",
                    //         &session_data,
                    //     )
                    //     .unwrap();
                }

                F123Packet::LapData(lap_data) => {
                    let session_id = lap_data.m_header.m_sessionUID;

                    // lap_data
                    //     .m_lapData
                    //     .iter()
                    //     .enumerate()
                    //     .for_each(|(index, lap)| {
                    //         let lap_n = lap.m_currentLapNum;

                    //         let _: () = redis
                    //             .json_set(
                    //                 format!("sessions:{}:car:{}:lap:{}", session_id, index, lap_n),
                    //                 ".",
                    //                 lap,
                    //             )
                    //             .unwrap();
                    //     });
                }

                F123Packet::Event(event_data) => {
                    let session_id = event_data.m_header.m_sessionUID;

                    // let _: () = redis
                    //     .json_set(
                    //         format!("sessions:{}:event", session_id),
                    //         ".",
                    //         &event_data.m_eventDetails,
                    //     )
                    //     .unwrap();
                }

                F123Packet::Participants(participants_data) => {
                    let session_id = participants_data.m_header.m_sessionUID;

                    // participants_data
                    //     .m_participants
                    //     .iter()
                    //     .enumerate()
                    //     .for_each(|(index, participant)| {
                    //         let _: () = redis
                    //             .json_set(
                    //                 format!("sessions:{}:participants:{}", session_id, index),
                    //                 ".",
                    //                 participant,
                    //             )
                    //             .unwrap();
                    //     })
                }

                F123Packet::FinalClassification(classification_data) => {
                    let session_id = classification_data.m_header.m_sessionUID;

                    // classification_data
                    //     .m_classificationData
                    //     .iter()
                    //     .enumerate()
                    //     .for_each(|(index, classification)| {
                    //         let _: () = redis
                    //             .json_set(
                    //                 format!("sessions:{}:car:{}:classification", session_id, index),
                    //                 ".",
                    //                 classification,
                    //             )
                    //             .unwrap();
                    //     })
                }

                F123Packet::LobbyInfo(lobby_info) => {
                    let session_id = lobby_info.m_header.m_sessionUID;

                    // let _: () = redis
                    //     .json_set(
                    //         format!("sessions:{}:lobby", session_id),
                    //         ".",
                    //         &lobby_info.m_lobbyPlayers,
                    //     )
                    //     .unwrap();
                }

                _ => {}
            }
        }
    }
}
