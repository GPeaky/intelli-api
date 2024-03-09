use crate::error::AppResult;
use ahash::AHashMap;
use parking_lot::RwLock;
use std::{net::IpAddr, sync::Arc};
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
            todo!()
        } else {
            warn!("Firewall service is not supported on this platform");
            Ok(())
        }
    }

    pub async fn open_partially(&self, id: i32, address: IpAddr) -> AppResult<()> {
        if cfg!(target_os = "linux") {
            todo!()
        } else {
            warn!("Firewall service is not supported on this platform");
            Ok(())
        }
    }

    pub async fn close(&self, id: i32) -> AppResult<()> {
        if cfg!(target_os = "linux") {
            todo!()
        } else {
            warn!("Firewall service is not supported on this platform");
            Ok(())
        }
    }

    pub async fn close_all(&self) -> AppResult<()> {
        if cfg!(target_os = "linux") {
            todo!()
        } else {
            warn!("Firewall service is not supported on this platform");
            Ok(())
        }
    }
}
