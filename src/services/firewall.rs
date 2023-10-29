use rustc_hash::FxHashMap;
use std::net::IpAddr;
use tokio::sync::RwLock;

use crate::error::{AppResult, SocketError};

#[allow(unused)]
enum FirewallType {
    Open,
    PartiallyOpen,
}

#[allow(unused)]
struct FirewallRule {
    port: u16,
    address: IpAddr,
    firewall_type: FirewallType,
}

pub struct FirewallService {
    rules: RwLock<FxHashMap<u32, FirewallRule>>,
}

#[allow(unused)]
impl FirewallService {
    pub fn new() -> Self {
        Self {
            rules: RwLock::new(FxHashMap::default()),
        }
    }

    async fn rule_exists(&self, id: &u32) -> bool {
        let rules = self.rules.read().await;
        rules.contains_key(id)
    }

    pub async fn open(&self, id: u32, port: u16) -> AppResult<()> {
        let exists = self.rule_exists(&id).await;

        if exists {
            Err(SocketError::RuleAlreadyExists)?;
        }

        todo!()
        // Ok(())
    }

    pub async fn open_partially(&self, id: u32, address: IpAddr) -> AppResult<()> {
        todo!()
    }

    pub async fn close(&self, id: u32) -> AppResult<()> {
        todo!()
    }

    pub fn close_all(&self) -> AppResult<()> {
        todo!()
    }
}
