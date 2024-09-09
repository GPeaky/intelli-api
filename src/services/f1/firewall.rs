use crate::error::{AppResult, FirewallError};
use ahash::AHashMap;
use regex::Regex;
use std::{str, sync::Arc};
use tokio::process::Command;
use tokio::sync::RwLock;
use tracing::{error, warn};

/// Represents a single firewall rule.
struct FirewallRule {
    port: u16,
    handle: String,
    ip_address: Option<String>,
}

impl FirewallRule {
    /// Creates a new FirewallRule.
    ///
    /// # Arguments
    /// - `port`: The port number for the rule.
    /// - `handle`: A unique identifier for the rule.
    pub fn new(port: u16, handle: String) -> Self {
        FirewallRule {
            port,
            handle,
            ip_address: None,
        }
    }
}

/// Manages firewall rules for the application.
pub struct FirewallService {
    rules: Arc<RwLock<AHashMap<i32, FirewallRule>>>,
}

// TODO: this must check on initialization if the server has de firewall service installed and active to use it
impl FirewallService {
    /// Creates a new FirewallService instance.
    pub fn new() -> Self {
        Self {
            rules: Arc::from(RwLock::from(AHashMap::with_capacity(10))),
        }
    }

    /// Opens a port in the firewall.
    ///
    /// # Arguments
    /// - `id`: Unique identifier for the rule.
    /// - `port`: Port number to open.
    ///
    /// # Returns
    /// Result indicating success or failure.
    pub async fn open(&self, id: i32, port: u16) -> AppResult<()> {
        if cfg!(not(target_os = "linux")) {
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

        let mut rules = self.rules.write().await;
        rules.insert(id, FirewallRule::new(port, handle));

        Ok(())
    }

    /// Restricts an open port to a specific IP address.
    ///
    /// # Arguments
    /// - `id`: Unique identifier for the rule.
    /// - `ip_address`: IP address to restrict the rule to.
    ///
    /// # Returns
    /// Result indicating success or failure.
    pub async fn restrict_to_ip(&self, id: i32, ip_address: String) -> AppResult<()> {
        if cfg!(not(target_os = "linux")) {
            warn!("Firewall not supported on this platform");
            return Ok(());
        }

        let mut rules = self.rules.write().await;
        let rule = rules.get_mut(&id).ok_or(FirewallError::RuleNotFound)?;

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

        Ok(())
    }

    /// Closes a previously opened port.
    ///
    /// # Arguments
    /// - `id`: Unique identifier for the rule to close.
    ///
    /// # Returns
    /// Result indicating success or failure.
    pub async fn close(&self, id: i32) -> AppResult<()> {
        if cfg!(not(target_os = "linux")) {
            warn!("Firewall not supported on this platform");
            return Ok(());
        }

        if !self.rule_exists(id).await {
            return Err(FirewallError::RuleNotFound)?;
        };

        let mut rules = self.rules.write().await;
        let rule = rules.get(&id).unwrap();

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

        rules.remove(&id);
        Ok(())
    }

    /// Closes all open ports managed by this service.
    ///
    /// # Returns
    /// Result indicating success or failure.
    #[allow(unused)]
    // TODO: Use it when the server instance goes down to clear all the firewall rules seated
    pub async fn close_all(&self) -> AppResult<()> {
        if cfg!(not(target_os = "linux")) {
            warn!("Firewall not supported on this platform");
            return Ok(());
        }

        let ids = {
            let rules = self.rules.read().await;
            rules.keys().copied().collect::<Vec<_>>()
        };

        for id in ids {
            self.close(id).await?;
        }

        Ok(())
    }

    /// Checks if a rule with the given ID exists.
    ///
    /// # Arguments
    /// - `id`: Unique identifier for the rule.
    ///
    /// # Returns
    /// Boolean indicating whether the rule exists.
    #[inline(always)]
    async fn rule_exists(&self, id: i32) -> bool {
        let rules = self.rules.read().await;
        rules.contains_key(&id)
    }

    /// Retrieves the current firewall ruleset.
    ///
    /// # Returns
    /// String representation of the current ruleset or an error.
    #[inline(always)]
    async fn ruleset() -> AppResult<String> {
        let output = Command::new("nft")
            .args(["-a", "list", "ruleset"])
            .output()
            .await
            .expect("Failed to execute process");

        if !output.status.success() {
            Err(FirewallError::ExecutionError)?
        }

        let ruleset = String::from_utf8(output.stdout).unwrap_or_default();
        Ok(ruleset)
    }

    /// Executes an nft command with the given arguments.
    ///
    /// # Arguments
    /// - `args`: Slice of command arguments.
    ///
    /// # Returns
    /// Result indicating success or failure.
    #[inline(always)]
    async fn nft_command(args: &[&str]) -> AppResult<()> {
        let output = Command::new("nft")
            .args(args)
            .output()
            .await
            .expect("Failed to execute process");

        if !output.status.success() {
            error!("{:?}", std::str::from_utf8(&output.stderr));
            Err(FirewallError::ExecutionError)?
        }

        Ok(())
    }

    /// Extracts the handle from a ruleset for a given search pattern.
    ///
    /// # Arguments
    /// - `ruleset`: String representation of the current ruleset.
    /// - `search_pattern`: Pattern to search for in the ruleset.
    ///
    /// # Returns
    /// The extracted handle as a string or an error.
    #[inline(always)]
    fn extract_handle_from_ruleset(ruleset: &str, search_pattern: &str) -> AppResult<String> {
        let pattern = format!(r"{}\s+#\s+handle\s+(\d+)", regex::escape(search_pattern));
        let re = Regex::new(&pattern).map_err(|_| FirewallError::ParseError)?;

        if let Some(caps) = re.captures(ruleset) {
            if let Some(handle) = caps.get(1) {
                return Ok(handle.as_str().to_string());
            }
        }

        Err(FirewallError::RuleNotFound)?
    }
}
