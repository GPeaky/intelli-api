use dashmap::DashMap;
use ntex::util::Bytes;
use tokio::sync::{
    broadcast::{channel, Receiver},
    oneshot,
};
use tracing::{info, warn};

use super::F1State;
use error::{AppResult, F1ServiceError};
use structs::ServiceStatus;

pub use super::{
    firewall::FirewallService,
    manager::F1SessionDataManager,
    service::{F1Service, F1ServiceData},
};

/// Manages F1 championship services, including caching, subscriptions, and service lifecycle.
#[derive(Clone)]
pub struct F1ServiceHandler {
    services: &'static DashMap<i32, F1ServiceData>,
    f1_state: &'static F1State,
}

impl F1ServiceHandler {
    /// Creates a new F1ServiceHandler instance.
    ///
    /// # Returns
    /// A new F1ServiceHandler with initialized services and firewall.
    pub fn new(f1_state: &'static F1State) -> Self {
        let services = Box::leak(Box::new(DashMap::with_capacity(10)));
        Self { services, f1_state }
    }

    /// Subscribes to a team-specific channel for a championship service.
    pub fn subscribe_team(&self, championship_id: &i32, team_id: u8) -> Option<Receiver<Bytes>> {
        let service = self.services.get(championship_id)?;
        service.team_sub(team_id)
    }

    /// Retrieves cache and subscribes to a channel for a specific championship service.
    ///
    /// # Arguments
    /// - `championship_id`: The ID of the championship.
    ///
    /// # Returns
    /// Some((Option<Bytes>, Receiver<Bytes>)) if service exists, None otherwise.
    pub fn cache_and_subscribe(
        &self,
        championship_id: &i32,
    ) -> Option<(Option<Bytes>, Receiver<Bytes>)> {
        let service = self.services.get(championship_id)?;
        Some((service.cache(), service.global_sub()))
    }

    /// Unsubscribes from a championship service.
    ///
    /// # Arguments
    /// - `championship_id`: The ID of the championship.
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
    ///
    /// # Returns
    /// A Vec<i32> containing IDs of active services.
    pub fn services(&self) -> Vec<i32> {
        let mut services = Vec::with_capacity(self.services.len());

        for item in self.services {
            services.push(*item.key())
        }

        services
    }

    /// Retrieves the status of a specific service.
    ///
    /// # Arguments
    /// - `id`: The ID of the service.
    ///
    /// # Returns
    /// ServiceStatus containing activity status and connection count.
    pub fn service_status(&self, id: &i32) -> ServiceStatus {
        match self.services.get(id) {
            Some(service) => ServiceStatus {
                active: true,
                general_conn: service.global_count(),
                engineer_conn: service.all_team_count(),
            },
            None => ServiceStatus {
                active: false,
                general_conn: 0,
                engineer_conn: 0,
            },
        }
    }

    /// Starts a new F1 service for the given championship.
    ///
    /// # Arguments
    /// - `port`: The port number to listen on.
    /// - `championship_id`: The championship ID to associate with the service.
    ///
    /// # Returns
    /// Result indicating success or failure.
    pub async fn start(&self, port: i32, championship_id: i32) -> AppResult<()> {
        if self.service(&championship_id) {
            return Err(F1ServiceError::AlreadyExists)?;
        }

        let (otx, orx) = oneshot::channel::<()>();
        let (tx, _) = channel::<Bytes>(50);
        let session_manager = F1SessionDataManager::new(tx.clone());
        let service_data = F1ServiceData::new(session_manager.clone(), tx, otx);
        let mut service = F1Service::new(session_manager, orx, self.services, self.f1_state).await;

        // TODO: Add real race_id
        service.initialize(port, championship_id, 0).await?;

        ntex::rt::spawn(async move { service.run().await });

        self.services.insert(championship_id, service_data);

        Ok(())
    }

    /// Stops the active F1 service for the given championship.
    ///
    /// # Arguments
    /// - `championship_id`: The championship ID whose service is to be stopped.
    ///
    /// # Returns
    /// Result indicating success or failure.
    pub async fn stop(&self, championship_id: &i32) -> AppResult<()> {
        match self.services.remove(championship_id) {
            Some((_, mut service)) => {
                if service.shutdown().is_err() {
                    Err(F1ServiceError::Shutdown)?
                }
            }

            None => {
                warn!("Trying to remove a non existing service");
                Err(F1ServiceError::NotActive)?
            }
        }

        info!("Service stopped for championship: {}", championship_id);
        Ok(())
    }

    /// Checks if a specific service is active.
    ///
    /// # Arguments
    /// - `id`: The ID of the service to check.
    ///
    /// # Returns
    /// true if the service is active, false otherwise.
    #[inline]
    fn service(&self, id: &i32) -> bool {
        self.services.contains_key(id)
    }
}
