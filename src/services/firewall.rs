use crate::error::AppResult;
// #[cfg(target_os = "linux")]
// use crate::error::SocketError;
// #[cfg(target_os = "linux")]
// use rustc_hash::FxHashMap;
use std::net::IpAddr;
// #[cfg(target_os = "linux")]
// use tokio::{process::Command, sync::RwLock};

// #[derive(Clone, Copy)]
// #[cfg(target_os = "linux")]
// enum FirewallType {
//     Open,
//     PartiallyOpen,
// }
//
// #[cfg(target_os = "linux")]
// struct FirewallRule {
//     port: i32,
//     address: Option<IpAddr>,
//     firewall_type: FirewallType,
// }

#[derive(Clone)]
pub struct FirewallService {
    // #[cfg(target_os = "linux")]
    // rules: RwLock<FxHashMap<i32, FirewallRule>>,
}

#[allow(unused)]
impl FirewallService {
    pub fn new() -> Self {
        Self {
            // #[cfg(target_os = "linux")]
            // rules: RwLock::new(FxHashMap::default()),
        }
    }

    async fn rule_exists(&self, id: &i32) -> bool {
        // #[cfg(target_os = "linux")]
        // {
        //     let rules = self.rules.read().await;
        //     rules.contains_key(id)
        // }
        //
        // #[cfg(not(target_os = "linux"))]
        false
    }

    pub async fn open(&self, id: i32, port: i32) -> AppResult<()> {
        // #[cfg(target_os = "linux")]
        // {
        //     let exists = self.rule_exists(&id).await;
        //
        //     if exists {
        //         Err(SocketError::RuleAlreadyExists)?;
        //     }
        //
        //     let status = Command::new("sudo")
        //         .arg("ufw")
        //         .arg("allow")
        //         .arg(format!("{port}/udp"))
        //         .status()
        //         .await
        //         .unwrap();
        //
        //     if !status.success() {
        //         Err(SocketError::CommandFailed)?;
        //     }
        //
        //     {
        //         let mut rules = self.rules.write().await;
        //         rules.insert(
        //             id,
        //             FirewallRule {
        //                 port,
        //                 address: None,
        //                 firewall_type: FirewallType::Open,
        //             },
        //         );
        //     }
        // }

        Ok(())
    }

    pub async fn open_partially(&self, id: i32, address: IpAddr) -> AppResult<()> {
        // #[cfg(target_os = "linux")]
        // {
        //     let port;
        //     let exists = self.rule_exists(&id).await;
        //
        //     if !exists {
        //         Err(SocketError::NotFound)?;
        //     }
        //
        //     {
        //         let rules = self.rules.read().await;
        //         let rule = rules.get(&id).unwrap();
        //         port = rule.port;
        //     }
        //
        //     let delete_status = Command::new("sudo")
        //         .arg("ufw")
        //         .arg("delete")
        //         .arg(format!("{}/udp", port))
        //         .status()
        //         .await
        //         .unwrap();
        //
        //     if !delete_status.success() {
        //         Err(SocketError::CommandFailed)?;
        //     }
        //
        //     let status = Command::new("sudo")
        //         .arg("ufw")
        //         .arg("allow")
        //         .arg("from")
        //         .arg(address.to_string())
        //         .arg("to")
        //         .arg("any")
        //         .arg("port")
        //         .arg(format!("{}/udp", port))
        //         .status()
        //         .await
        //         .unwrap();
        //
        //     if !status.success() {
        //         Err(SocketError::CommandFailed)?
        //     }
        //
        //     {
        //         let mut rules = self.rules.write().await;
        //         let rule = rules.get_mut(&id).unwrap();
        //
        //         rule.address = Some(address);
        //         rule.firewall_type = FirewallType::PartiallyOpen;
        //     }
        // }

        Ok(())
    }

    pub async fn close(&self, id: &i32) -> AppResult<()> {
        // #[cfg(target_os = "linux")]
        // {
        //     let (port, firewall_type, address);
        //
        //     {
        //         let rules = self.rules.read().await;
        //         let rule = rules.get(id).unwrap();
        //
        //         port = rule.port;
        //         firewall_type = rule.firewall_type;
        //         address = rule.address;
        //     }
        //
        //     let delete_status = match firewall_type {
        //         FirewallType::Open => {
        //             Command::new("sudo")
        //                 .arg("ufw")
        //                 .arg("delete")
        //                 .arg("allow")
        //                 .arg(format!("{}/udp", port))
        //                 .status()
        //                 .await
        //         }
        //
        //         FirewallType::PartiallyOpen => {
        //             let Some(addr) = address else {
        //                 Err(SocketError::CommandFailed)?
        //             };
        //
        //             Command::new("sudo")
        //                 .arg("ufw")
        //                 .arg("delete")
        //                 .arg("allow")
        //                 .arg("from")
        //                 .arg(addr.to_string())
        //                 .arg("to")
        //                 .arg("any")
        //                 .arg("port")
        //                 .arg(format!("{}/udp", port))
        //                 .status()
        //                 .await
        //         }
        //     }
        //     .unwrap();
        //
        //     if !delete_status.success() {
        //         Err(SocketError::CommandFailed)?;
        //     }
        //
        //     {
        //         let mut rules = self.rules.write().await;
        //         rules.remove(id);
        //     }
        // }

        Ok(())
    }

    pub async fn close_all(&self) -> AppResult<()> {
        // #[cfg(target_os)]
        // {
        //     let mut rules = self.rules.write().await;
        //
        //     for (id, _) in rules.iter() {
        //         self.close(id).await.unwrap();
        //     }
        //
        //     rules.clear();
        // }

        Ok(())
    }
}
