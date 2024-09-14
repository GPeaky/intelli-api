use std::sync::Arc;

use dashmap::DashMap;
use ntex::util::Bytes;
use tokio::sync::{
    broadcast::{channel, Receiver},
    oneshot,
};
use tracing::{info, warn};

use crate::{
    error::{AppResult, F1ServiceError},
    states::F1State,
    structs::ServiceStatus,
};

pub use super::{
    firewall::FirewallService,
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

    /// Retrieves the cache for a specific championship service.
    ///
    /// # Arguments
    /// - `championship_id`: The ID of the championship.
    ///
    /// # Returns
    /// Some(Bytes) if cache exists, None otherwise.
    #[allow(unused)]
    pub async fn cache(&self, championship_id: &i32) -> Option<Bytes> {
        // if let Some(service) = self.services.get(championship_id) {
        //     let cache = service.cache.read_arc();
        //     return cache.get().await;
        // }

        None
    }

    /// Subscribes to a channel for a specific championship service.
    ///
    /// # Arguments
    /// - `championship_id`: The ID of the championship.
    ///
    /// # Returns
    /// Some(Receiver<Bytes>) if service exists, None otherwise.
    #[allow(unused)]
    pub fn subscribe(&self, championship_id: &i32) -> Option<Receiver<Bytes>> {
        let service = self.services.get(championship_id)?;
        Some(service.subscribe())
    }

    /// Retrieves cache and subscribes to a channel for a specific championship service.
    ///
    /// # Arguments
    /// - `championship_id`: The ID of the championship.
    ///
    /// # Returns
    /// Some((Option<Bytes>, Receiver<Bytes>)) if service exists, None otherwise.
    pub async fn cache_and_subscribe(
        &self,
        _championship_id: &i32,
    ) -> Option<(Option<Bytes>, Receiver<Bytes>)> {
        // if let Some(service) = self.services.get(championship_id) {
        //     let cache = service.cache.read_arc();
        //     let data = cache.get().await;
        //     let receiver = service.subscribe();

        //     return Some((data, receiver));
        // }

        None
    }

    /// Unsubscribes from a championship service.
    ///
    /// # Arguments
    /// - `championship_id`: The ID of the championship.
    pub fn unsubscribe(&self, championship_id: &i32) {
        if let Some(service) = self.services.get(championship_id) {
            service.unsubscribe();
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

    /// Checks if a specific service is active.
    ///
    /// # Arguments
    /// - `id`: The ID of the service to check.
    ///
    /// # Returns
    /// true if the service is active, false otherwise.
    pub fn service(&self, id: &i32) -> bool {
        self.services.contains_key(id)
    }

    /// Retrieves the status of a specific service.
    ///
    /// # Arguments
    /// - `id`: The ID of the service.
    ///
    /// # Returns
    /// ServiceStatus containing activity status and connection count.
    pub fn service_status(&self, id: &i32) -> ServiceStatus {
        let Some(service) = self.services.get(id) else {
            return ServiceStatus {
                active: false,
                connections: 0,
            };
        };

        let connections = service.subscribers_count();

        ServiceStatus {
            active: true,
            connections,
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
        let (tx, rx) = channel::<Bytes>(50);
        let service_data = F1ServiceData::new(Arc::new(rx), otx);
        let mut service = F1Service::new(
            tx,
            orx,
            // service_data.cache.clone(),
            self.services,
            self.f1_state,
        )
        .await;

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
        if !self.service(championship_id) {
            return Err(F1ServiceError::NotActive)?;
        }

        if let Some((_, mut service)) = self.services.remove(championship_id) {
            if service.shutdown().is_err() {
                return Err(F1ServiceError::Shutdown)?;
            }
        } else {
            warn!("Trying to remove a non existing service");
        }

        info!("Service stopped for championship: {}", championship_id);
        Ok(())
    }
}
