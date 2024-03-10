use crate::error::{AppResult, FirewallError};
use ahash::AHashMap;
use std::{net::IpAddr, sync::Arc};
use tokio::process::Command;
use tokio::sync::RwLock;
use tracing::{info, warn};

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
            let rules = self.rules.read().await;
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

            let chain_name = id.to_string();
            self.create_chain(&chain_name).await?;

            let output = Command::new("nft")
                .args([
                    "add",
                    "rule",
                    "inet",
                    "nftables_svc",
                    &chain_name,
                    "udp",
                    "dport",
                    port.to_string().as_str(),
                    "accept",
                ])
                .output()
                .await
                .expect("Failed to execute command");

            info!("status: {}", output.status);
            info!("stdout: {}", String::from_utf8_lossy(&output.stdout));
            info!("stderr: {}", String::from_utf8_lossy(&output.stderr));

            match output.status.success() {
                true => {
                    let mut rules = self.rules.write().await;
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
            let rules = self.rules.read().await;

            if let Some(rule) = rules.get(&id) {
                let output = Command::new("sudo")
                    .args([
                        "nft",
                        "delete",
                        "chain",
                        "inet",
                        "nftables_svc",
                        &id.to_string(),
                    ])
                    .output()
                    .await
                    .expect("Error executing command");

                match output.status.success() {
                    true => {
                        drop(rules); // Release the lock
                        let mut rules = self.rules.write().await;
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

    async fn create_chain(&self, id: &str) -> AppResult<()> {
        let output = Command::new("nft")
            .args([
                "add",
                "chain",
                "inet",
                "nftables_svc",
                id,
                "{ type filter hook input priority 0; }",
            ])
            .output()
            .await
            .expect("Error executing command");

        match output.status.success() {
            true => Ok(()),
            false => Err(FirewallError::CreatingChain)?,
        }
    }
}
