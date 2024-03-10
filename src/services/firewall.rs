use crate::error::{AppResult, FirewallError};
use ahash::AHashMap;
use parking_lot::RwLock;
use std::{net::IpAddr, sync::Arc};
use tokio::process::Command;
use tracing::warn;

#[allow(unused)]
#[derive(Clone, Copy)]
enum FirewallType {
    Open,
    PartiallyClosed,
}

#[allow(unused)]
struct FirewallRule {
    port: i32,
    r#type: FirewallType,
    address: Option<IpAddr>,
}

// TODO: Implement the firewall
#[derive(Clone)]
pub struct FirewallService {
    rules: Arc<RwLock<AHashMap<i32, FirewallRule>>>,
}

#[allow(unused)]
impl FirewallService {
    pub fn new() -> Self {
        Self {
            rules: Arc::from(RwLock::from(AHashMap::with_capacity(100))),
        }
    }

    async fn rule_exists(&self, id: i32) -> bool {
        if cfg!(target_os = "linux") {
            let rules = self.rules.read();
            rules.contains_key(&id)
        } else {
            warn!("Firewall service is not supported on this platform");
            false
        }
    }

    pub async fn open(&self, id: i32, port: i32) -> AppResult<()> {
        if cfg!(target_os = "linux") {
            if self.rule_exists(id).await {
                Err(FirewallError::RuleExists)?
            }

            let output = Command::new("nft")
                .args(&[
                    "add",
                    "rule",
                    "ip",
                    "filter",
                    "input",
                    "udp",
                    "dport",
                    &port.to_string(),
                    "accept",
                ])
                .output()
                .await
                .expect("Failed to execute command");

            match output.status.success() {
                true => {
                    let mut rules = self.rules.write();
                    rules.insert(
                        id,
                        FirewallRule {
                            port,
                            r#type: FirewallType::Open,
                            address: None,
                        },
                    );
                    Ok(())
                }
                false => Err(FirewallError::OpeningPort)?,
            }
        } else {
            warn!("Firewall service is not supported on this platform");
            Ok(())
        }
    }

    // Todo: Implement the open_partially method
    pub async fn open_partially(&self, id: i32, address: IpAddr) -> AppResult<()> {
        if cfg!(target_os = "linux") {
            Ok(())
        } else {
            warn!("Firewall service is not supported on this platform");
            Ok(())
        }
    }

    pub async fn close(&self, id: i32) -> AppResult<()> {
        if cfg!(target_os = "linux") {
            let rules = self.rules.read();

            if let Some(rule) = rules.get(&id) {
                let output = Command::new("nft")
                    .args(&[
                        "delete",
                        "rule",
                        "ip",
                        "filter",
                        "input",
                        "udp",
                        "dport",
                        &rule.port.to_string(),
                        "accept",
                    ])
                    .output()
                    .await
                    .expect("Error executing command");

                match output.status.success() {
                    true => {
                        let mut rules = self.rules.write();
                        rules.remove(&id);
                        Ok(())
                    }
                    false => Err(FirewallError::ClosingPort)?,
                }
            } else {
                Err(FirewallError::RuleNotFound)?
            }
        } else {
            warn!("Firewall service is not supported on this platform");
            Ok(())
        }
    }

    // TODO: Implement the close_all method
    pub async fn close_all(&self) -> AppResult<()> {
        if cfg!(target_os = "linux") {
            Ok(())
        } else {
            warn!("Firewall service is not supported on this platform");
            Ok(())
        }
    }
}
