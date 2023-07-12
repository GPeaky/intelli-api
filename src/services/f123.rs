use crate::{
    config::Database,
    dtos::F123Packet,
    error::{AppResult, SocketError},
};
use bincode::serialize;
use std::{
    collections::HashMap,
    net::IpAddr,
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::{net::UdpSocket, process::Command, sync::RwLock, task::JoinHandle};
use tracing::{error, info};

#[derive(Clone)]
pub struct F123Service {
    db_conn: Arc<Database>,
    sockets: Arc<RwLock<HashMap<String, JoinHandle<()>>>>,
    ip_addresses: Arc<RwLock<HashMap<String, IpAddr>>>,
}

impl F123Service {
    pub fn new(db_conn: &Arc<Database>) -> Self {
        Self {
            db_conn: db_conn.clone(),
            sockets: Arc::new(RwLock::new(HashMap::new())),
            ip_addresses: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn new_socket(&self, port: i16, championship_id: String) {
        let db = self.db_conn.clone();
        let ip_addresses = self.ip_addresses.clone();
        let chm_id = championship_id.clone();

        let socket = tokio::spawn(async move {
            let mut buf = vec![0; 2048];
            let mut closed_ports = false;
            let session = db.get_scylla();
            let mut last_session_update = Instant::now();
            let mut last_car_motion_update = Instant::now();
            let (open_machine_port, close_port_for_all_except) =
                (Self::open_machine_port, Self::close_port_for_all_except);
            let Ok(socket) = UdpSocket::bind(format!("0.0.0.0:{}", port)).await else {
                error!("There was an error binding to the socket");
                return;
            };

            open_machine_port(port).await.unwrap();

            loop {
                match socket.recv_from(&mut buf).await {
                    Ok((size, address)) => {
                        if !closed_ports {
                            close_port_for_all_except(port as u16, address.ip())
                                .await
                                .unwrap();

                            closed_ports = true;

                            {
                                let mut ip_addresses = ip_addresses.write().await;
                                ip_addresses.insert(championship_id.clone(), address.ip());
                            }
                        }

                        let Ok(header) = F123Packet::parse_header(&buf[..size]) else {
                            continue;
                        };
                        let session_id = header.m_sessionUID as i64;

                        if session_id == 0 {
                            continue;
                        }

                        let Ok(Some(packet)) = F123Packet::parse(header.m_packetId, &buf[..size])
                        else {
                            continue;
                        };

                        match packet {
                            F123Packet::Motion(motion_data) => {
                                let now = Instant::now();

                                if now.duration_since(last_car_motion_update)
                                    >= Duration::from_millis(500)
                                {
                                    let Ok(data) = serialize(&motion_data) else {
                                        error!("There was an error serializing the motion data");
                                        continue;
                                    };

                                    session
                                        .execute(
                                            db.statements.get("insert_car_motion").unwrap(),
                                            (session_id, data),
                                        )
                                        .await
                                        .unwrap();

                                    last_car_motion_update = now;
                                }
                            }

                            F123Packet::Session(session_data) => {
                                let now = Instant::now();

                                if now.duration_since(last_session_update)
                                    >= Duration::from_secs(30)
                                {
                                    let Ok(data) = serialize(&session_data) else {
                                        error!("There was an error serializing the session data");
                                        continue;
                                    };

                                    session
                                        .execute(
                                            db.statements.get("insert_game_session").unwrap(),
                                            (session_id, data),
                                        )
                                        .await
                                        .unwrap();

                                    last_session_update = now;
                                }
                            }

                            F123Packet::LapData(lap_data) => {
                                let Ok(lap_info) = serialize(&lap_data.m_lapData) else {
                                    error!("There was an error serializing the lap data");
                                    continue;
                                };

                                // TODO: Save lap data to database
                                session
                                    .execute(
                                        db.statements.get("insert_lap_data").unwrap(),
                                        (session_id, lap_info),
                                    )
                                    .await
                                    .unwrap();
                            }

                            F123Packet::Event(event_data) => {
                                let Ok(event) = serialize(&event_data.m_eventDetails) else {
                                    error!("There was an error serializing the event data");
                                    continue;
                                };

                                session
                                    .execute(
                                        db.statements.get("insert_event_data").unwrap(),
                                        (session_id, event_data.m_eventStringCode, event),
                                    )
                                    .await
                                    .unwrap();
                            }

                            F123Packet::Participants(participants_data) => {
                                let Ok(participants) = serialize(&participants_data.m_participants)
                                else {
                                    error!("There was an error serializing the participants data");
                                    continue;
                                };

                                session
                                    .execute(
                                        db.statements.get("insert_participant_data").unwrap(),
                                        (session_id, participants),
                                    )
                                    .await
                                    .unwrap();
                            }

                            F123Packet::FinalClassification(classification_data) => {
                                let Ok(classifications) =
                                    serialize(&classification_data.m_classificationData)
                                else {
                                    error!("There was an error serializing the final classification data");
                                    continue;
                                };

                                session
                                    .execute(
                                        db.statements
                                            .get("insert_final_classification_data")
                                            .unwrap(),
                                        (session_id, classifications),
                                    )
                                    .await
                                    .unwrap();
                            }

                            // TODO: use unused packets
                            _ => {}
                        }
                    }

                    Err(e) => {
                        error!("Error receiving packet: {}", e);
                    }
                }
            }
        });

        {
            let mut sockets = self.sockets.write().await;
            sockets.insert(chm_id, socket);
        }
    }

    // pub async fn active_sockets(&self) {
    //     let sockets = self.sockets.read().await;

    //     for socket in sockets.iter() {
    //         println!("Socket: {:?}", socket);
    //     }
    // }

    // pub async fn stop_all_sockets(&self) {
    //     let mut sockets = self.sockets.write().await;

    //     for socket in sockets.iter() {
    //         socket.1.abort();
    //     }

    //     sockets.clear();
    // }

    pub async fn stop_socket(&self, championship_id: String, port: i16) -> AppResult<()> {
        {
            let ip_addresses = self.ip_addresses.read().await;
            let ip = ip_addresses.get(&championship_id).unwrap();
            Self::close_machine_port(port, *ip).await.unwrap();
        }

        let mut sockets = self.sockets.write().await;

        let Some(socket) = sockets.remove(&championship_id) else {
            Err(SocketError::NotFound)?
        };

        socket.abort();
        Ok(())
    }

    async fn open_machine_port(port: i16) -> tokio::io::Result<()> {
        let port_str = port.to_string();

        if cfg!(unix) {
            let output = Command::new("sudo")
                .arg("iptables")
                .arg("-A")
                .arg("INPUT")
                .arg("-p")
                .arg("udp")
                .arg("--dport")
                .arg(port_str)
                .arg("-j")
                .arg("ACCEPT")
                .output()
                .await?;

            if !output.status.success() {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "Failed to open port with iptables",
                ));
            }
        } else {
            info!("The machine is not running a unix based OS, so the port will not be opened automatically");
        }

        Ok(())
    }

    async fn close_port_for_all_except(port: u16, ip: IpAddr) -> std::io::Result<()> {
        if cfg!(unix) {
            let port_str = port.to_string();
            let ip_str = ip.to_string();

            // Primero, borramos cualquier regla existente que afecte al puerto especificado.
            let _ = Command::new("sudo")
                .arg("iptables")
                .arg("-D")
                .arg("INPUT")
                .arg("-p")
                .arg("udp")
                .arg("--dport")
                .arg(&port_str)
                .arg("-j")
                .arg("ACCEPT")
                .output()
                .await?;

            let _ = Command::new("sudo")
                .arg("iptables")
                .arg("-D")
                .arg("INPUT")
                .arg("-p")
                .arg("udp")
                .arg("--dport")
                .arg(&port_str)
                .arg("-j")
                .arg("DROP")
                .output()
                .await?;

            // Luego, agregamos las nuevas reglas.
            // Bloquear todas las conexiones a este puerto
            let output = Command::new("sudo")
                .arg("iptables")
                .arg("-A")
                .arg("INPUT")
                .arg("-p")
                .arg("udp")
                .arg("--dport")
                .arg(&port_str)
                .arg("-j")
                .arg("DROP")
                .output()
                .await?;

            if !output.status.success() {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "Failed to close port for all with iptables",
                ));
            }

            // Permitir conexiones desde la IP específica
            let output = Command::new("sudo")
                .arg("iptables")
                .arg("-I")
                .arg("INPUT")
                .arg("1")
                .arg("-p")
                .arg("udp")
                .arg("--dport")
                .arg(&port_str)
                .arg("-s")
                .arg(&ip_str)
                .arg("-j")
                .arg("ACCEPT")
                .output()
                .await?;

            if !output.status.success() {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "Failed to open port for specific IP with iptables",
                ));
            }
        }

        Ok(())
    }

    async fn close_machine_port(port: i16, ip: IpAddr) -> tokio::io::Result<()> {
        if cfg!(unix) {
            let port_str = port.to_string();
            let ip_str = ip.to_string();

            // Elimina la regla que permite las conexiones desde una IP específica
            match Command::new("sudo")
                .arg("iptables")
                .arg("-D")
                .arg("INPUT")
                .arg("-p")
                .arg("udp")
                .arg("--dport")
                .arg(&port_str)
                .arg("-s")
                .arg(&ip_str)
                .arg("-j")
                .arg("ACCEPT")
                .output()
                .await
            {
                Ok(output) => {
                    if !output.status.success() {
                        eprintln!("Failed to remove specific IP rule with iptables");
                    }
                }
                Err(e) => eprintln!("Failed to execute command: {}", e),
            }

            // Elimina la regla que bloquea todas las demás conexiones
            match Command::new("sudo")
                .arg("iptables")
                .arg("-D")
                .arg("INPUT")
                .arg("-p")
                .arg("udp")
                .arg("--dport")
                .arg(&port_str)
                .arg("-j")
                .arg("DROP")
                .output()
                .await
            {
                Ok(output) => {
                    if !output.status.success() {
                        eprintln!("Failed to remove drop rule with iptables");
                    }
                }
                Err(e) => eprintln!("Failed to execute command: {}", e),
            }
        } else {
            info!("The machine is not running a unix based OS, so the port will not be closed automatically");
        }

        Ok(())
    }
}
