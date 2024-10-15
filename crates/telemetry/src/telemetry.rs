use dashmap::DashMap;
use intelli_core::{
    repositories::{ChampionshipRepository, DriverRepository},
    services::{ChampionshipService, DriverService},
};
use ntex::util::Bytes;
use tokio::sync::{
    broadcast::{channel, Receiver},
    oneshot,
};
use tracing::{info, warn};

mod f1;
mod firewall;
mod manager;
mod service;

use firewall::FirewallService;
use manager::F1SessionDataManager;
use service::{F1Service, F1ServiceData};

pub use manager::DriverInfo;

use error::{AppResult, F1ServiceError};
use structs::ServiceStatus;

/// Manages F1 championship services, including caching, subscriptions, and service lifecycle.
#[derive(Clone)]
pub struct F1ServiceHandler {
    services: &'static DashMap<i32, F1ServiceData>,
    f1_state: &'static F1State,
}

/// Represents the global state for F1 services.
pub struct F1State {
    pub driver_svc: &'static DriverService,
    pub firewall: &'static FirewallService,
    pub driver_repo: &'static DriverRepository,
    pub championship_repo: &'static ChampionshipRepository,
    pub championship_svc: &'static ChampionshipService,
}

impl F1ServiceHandler {
    /// Creates a new F1ServiceHandler instance.
    pub fn new(f1_state: &'static F1State) -> Self {
        let services = Box::leak(Box::new(DashMap::with_capacity(10)));
        Self { services, f1_state }
    }

    /// Subscribes to a team-specific channel for a championship service.
    pub fn subscribe_team(&self, championship_id: &i32, team_id: u8) -> Option<Receiver<Bytes>> {
        self.services.get(championship_id)?.team_sub(team_id)
    }

    /// Retrieves cache and subscribes to a channel for a specific championship service.
    pub fn cache_and_subscribe(
        &self,
        championship_id: &i32,
    ) -> Option<(Option<Bytes>, Receiver<Bytes>)> {
        let service = self.services.get(championship_id)?;
        Some((service.cache(), service.global_sub()))
    }

    /// Unsubscribes from a championship service.
    #[inline]
    pub fn unsubscribe(&self, championship_id: &i32) {
        if let Some(service) = self.services.get(championship_id) {
            service.global_unsub();
        }
    }

    /// Unsubscribes from the team-specific channel of a championship service.
    #[inline]
    pub fn unsubscribe_team(&self, championship_id: &i32, team_id: u8) {
        if let Some(service) = self.services.get(championship_id) {
            service.team_unsub(team_id);
        }
    }

    /// Retrieves a list of all active service IDs.
    pub fn services(&self) -> Vec<i32> {
        self.services.iter().map(|item| *item.key()).collect()
    }

    /// Retrieves the status of a specific service.
    pub fn service_status(&self, id: &i32) -> ServiceStatus {
        self.services
            .get(id)
            .map(|service| ServiceStatus {
                active: true,
                general_conn: service.global_count(),
                engineer_conn: service.all_team_count(),
            })
            .unwrap_or_default()
    }

    /// Starts a new F1 service for the given championship.
    pub async fn start(&self, port: i32, championship_id: i32) -> AppResult<()> {
        if self.service(&championship_id) {
            return Err(F1ServiceError::AlreadyExists.into());
        }

        let (otx, orx) = oneshot::channel::<()>();
        let (tx, _) = channel::<Bytes>(50);
        let session_manager = F1SessionDataManager::new(tx.clone());
        let service_data = F1ServiceData::new(session_manager.clone(), tx, otx);
        let mut service = F1Service::new(session_manager, orx, self.services, self.f1_state).await;

        service.initialize(port, championship_id, 0).await?;

        ntex::rt::spawn(async move { service.run().await });

        self.services.insert(championship_id, service_data);

        Ok(())
    }

    /// Stops the active F1 service for the given championship.
    pub async fn stop(&self, championship_id: &i32) -> AppResult<()> {
        match self.services.remove(championship_id) {
            Some((_, mut service)) => {
                service.shutdown().map_err(|_| F1ServiceError::Shutdown)?;
            }
            None => {
                warn!("Trying to remove a non-existing service");
                return Err(F1ServiceError::NotActive.into());
            }
        }

        info!("Service stopped for championship: {}", championship_id);
        Ok(())
    }

    /// Checks if a specific service is active.
    #[inline]
    fn service(&self, id: &i32) -> bool {
        self.services.contains_key(id)
    }
}

impl F1State {
    /// Creates a new F1State instance.
    pub fn new(
        driver_svc: &'static DriverService,
        driver_repo: &'static DriverRepository,
        championship_repo: &'static ChampionshipRepository,
        championship_svc: &'static ChampionshipService,
    ) -> Self {
        let firewall = Box::leak(Box::new(FirewallService::new()));

        Self {
            firewall,
            driver_svc,
            driver_repo,
            championship_repo,
            championship_svc,
        }
    }
}
