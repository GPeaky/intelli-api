use crate::error::{AppResult, FirewallError};
use ahash::AHashMap;
use regex::Regex;
use std::{str, sync::Arc};
use tokio::process::Command;
use tokio::sync::RwLock;
use tracing::{error, info, warn};

#[derive(Clone, Copy, Debug)]
enum FirewallType {
    Open,
    PartiallyClosed,
}

#[derive(Debug)]
struct FirewallRule {
    port: u16,
    r#type: FirewallType,
    handle: String,
    ip_address: Option<String>,
}

impl FirewallRule {
    pub fn new(port: u16, r#type: FirewallType, handle: String) -> Self {
        FirewallRule {
            port,
            r#type,
            handle,
            ip_address: None,
        }
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

    #[allow(unused)]
    pub async fn open(&self, id: i32, port: u16) -> AppResult<()> {
        if cfg!(target_os = "windows") {
            warn!("Firewall not supported on this platform");
            return Ok(());
        }

        if self.rule_exists(id).await {
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

        let ruleset = Self::ruleset().await?;
        let handle =
            Self::extract_handle_from_ruleset(&ruleset, &format!("udp dport {} accept", port))?;

        info!("Open Handle: {:?}", handle);

        let mut rules = self.rules.write().await;
        rules.insert(id, FirewallRule::new(port, FirewallType::Open, handle));

        info!("Open Get Rule: {:?}", rules.get(&id));

        Ok(())
    }

    #[allow(unused)]
    pub async fn restrict_to_ip(&self, id: i32, ip_address: String) -> AppResult<()> {
        if cfg!(target_os = "windows") {
            warn!("Firewall not supported on this platform");
            return Ok(());
        }

        let mut rules = self.rules.write().await;

        match rules.get_mut(&id) {
            None => Err(FirewallError::RuleNotFound)?,
            Some(rule) => {
                Self::nft_command(&[
                    "delete",
                    "rule",
                    "inet",
                    "nftables_svc",
                    "allow",
                    "handle",
                    &rule.handle,
                ])
                .await?;

                Self::nft_command(&[
                    "add",
                    "rule",
                    "inet",
                    "nftables_svc",
                    "allow",
                    "ip",
                    "saddr",
                    &ip_address,
                    "udp",
                    "dport",
                    &rule.port.to_string(),
                    "accept",
                ])
                .await?;

                let ruleset = Self::ruleset().await?;
                let new_handle = Self::extract_handle_from_ruleset(
                    &ruleset,
                    &format!("ip saddr {} udp dport {} accept", ip_address, rule.port),
                )?;

                rule.handle = new_handle;
                rule.ip_address = Some(ip_address);
                rule.r#type = FirewallType::PartiallyClosed;

                Ok(())
            }
        }
    }

    pub async fn close(&self, id: i32) -> AppResult<()> {
        if cfg!(target_os = "windows") {
            warn!("Firewall not supported on this platform");
            return Ok(());
        }

        let rules = self.rules.read().await;

        match rules.get(&id) {
            None => {
                error!("Not Founded Id");
                Err(FirewallError::RuleNotFound)?
            }

            Some(rule) => {
                Self::nft_command(&[
                    "delete",
                    "rule",
                    "inet",
                    "nftables_svc",
                    "allow",
                    "handle",
                    &rule.handle,
                ])
                .await?;

                drop(rules);
                let mut rules = self.rules.write().await;
                rules.remove(&id);
                Ok(())
            }
        }
    }

    #[allow(unused)]
    pub async fn close_all(&self) -> AppResult<()> {
        let ids = {
            let rules = self.rules.read().await;
            rules.keys().copied().collect::<Vec<_>>()
        };

        // Not concurrency need it because its only for stopping
        for id in ids {
            self.close(id).await?;
        }

        Ok(())
    }

    async fn rule_exists(&self, id: i32) -> bool {
        let rules = self.rules.read().await;
        rules.contains_key(&id)
    }

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

    fn extract_handle_from_ruleset(ruleset: &str, search_pattern: &str) -> AppResult<String> {
        info!("Ruleset: {}", ruleset);
        info!("Search Pattern: {}", search_pattern);

        let pattern = format!(r"{}\s*#\s*handle\s+(\d+)", regex::escape(search_pattern));
        let re = Regex::new(&pattern).map_err(|_| FirewallError::ParseError)?;

        if let Some(caps) = re.captures(ruleset) {
            if let Some(handle) = caps.get(1) {
                return Ok(handle.as_str().to_string());
            }
        }

        error!("Error extracting handler");
        Err(FirewallError::RuleNotFound)?
    }
}
