use crate::error::{AppResult, FirewallError};
use ahash::AHashMap;
use parking_lot::RwLock;
use std::{str, sync::Arc};
use tokio::process::Command;
use tracing::{error, warn};

#[derive(Clone, Copy)]
enum FirewallType {
    Open,
    #[allow(unused)]
    PartiallyClosed,
}

#[allow(unused)]
struct FirewallRule {
    port: u16,
    r#type: FirewallType,
}

impl FirewallRule {
    pub fn new(port: u16, r#type: FirewallType) -> Self {
        FirewallRule { port, r#type }
    }
}

pub struct FirewallService {
    rules: Arc<RwLock<AHashMap<i32, FirewallRule>>>,
}

impl FirewallService {
    pub fn new() -> Self {
        Self {
            rules: Arc::from(RwLock::from(AHashMap::with_capacity(10))),
        }
    }

    pub async fn open(&self, id: i32, port: u16) -> AppResult<()> {
        if cfg!(target_os = "windows") {
            warn!("Firewall not supported on this platform");
            return Ok(());
        }

        if self.rule_exists(id) {
            Err(FirewallError::RuleExists)?
        }

        Self::nft_command(&[
            "add",
            "rule",
            "inet",
            "nftables_svc",
            "allow",
            "udp",
            "dport",
            &port.to_string(),
            "accept",
        ])
        .await?;

        let mut rules = self.rules.write();
        rules.insert(id, FirewallRule::new(port, FirewallType::Open));

        Ok(())
    }

    pub async fn close(&self, id: i32) -> AppResult<()> {
        if cfg!(target_os = "windows") {
            warn!("Firewall not supported on this platform");
            return Ok(());
        }

        let rules = self.rules.read_arc();

        match rules.get(&id) {
            None => Err(FirewallError::RuleNotFound)?,
            Some(rule) => {
                let search_pattern = format!("udp dport {}", rule.port);
                drop(rules);
                let ruleset = Self::ruleset().await?;

                if let Some(start) = ruleset.find(&search_pattern) {
                    let remainder = &ruleset[start..];
                    if let Some(handler_index) = remainder.find("handle ") {
                        let handle = &remainder[handler_index + 7..]
                            .split_whitespace()
                            .next()
                            .ok_or(FirewallError::ParseError)?;

                        Self::nft_command(&[
                            "delete",
                            "rule",
                            "inet",
                            "nftables_svc",
                            "allow",
                            "handle",
                            handle,
                        ])
                        .await?;

                        let mut rules = self.rules.write();
                        rules.remove(&id);

                        Ok(())
                    } else {
                        Err(FirewallError::ParseError)?
                    }
                } else {
                    Err(FirewallError::RuleNotFound)?
                }
            }
        }
    }

    #[allow(unused)]
    // Todo - Run this function when the server closes
    pub async fn close_all(&self) -> AppResult<()> {
        let ids = {
            let rules = self.rules.read();
            rules.keys().copied().collect::<Vec<_>>()
        };

        // Not concurrency need it because its only for stopping
        for id in ids {
            self.close(id).await?;
        }

        Ok(())
    }

    fn rule_exists(&self, id: i32) -> bool {
        let rules = self.rules.read();
        rules.contains_key(&id)
    }

    #[inline(always)]
    async fn ruleset() -> AppResult<String> {
        let output = Command::new("nft")
            .args(["-a", "list", "ruleset"])
            .output()
            .await
            .expect("Failed to execute process");

        if output.status.success() {
            let ruleset = String::from_utf8(output.stdout).unwrap_or_default();
            Ok(ruleset)
        } else {
            Err(FirewallError::ExecutionError)?
        }
    }

    #[inline(always)]
    async fn nft_command(args: &[&str]) -> AppResult<()> {
        let output = Command::new("nft")
            .args(args)
            .output()
            .await
            .expect("Failed to execute process");

        if output.status.success() {
            Ok(())
        } else {
            error!("{:?}", std::str::from_utf8(&output.stderr));
            Err(FirewallError::ExecutionError)?
        }
    }
}
