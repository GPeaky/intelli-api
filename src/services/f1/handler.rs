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
    structs::ServiceStatus,
};

use super::{
    firewall::FirewallService,
    service::{F1Service, F1ServiceData},
};

/// `F1ServiceHandler` manages services related to F1 championships, including caching, subscriptions, and service management.
#[derive(Clone)]
pub struct F1ServiceHandler {
    services: &'static DashMap<i32, F1ServiceData>,
    firewall: &'static FirewallService,
}

impl F1ServiceHandler {
    /// Creates a new instance of `F1ServiceHandler`.
    ///
    /// This function initializes the handler with a firewall service and a map of services, allowing for
    /// efficient management and access to these services. The `services` map is pre-allocated with a capacity of 100.
    ///
    /// # Returns
    ///
    /// A new `F1ServiceHandler` instance.
    pub fn new() -> Self {
        let firewall = Box::leak(Box::new(FirewallService::new()));
        let services = Box::leak(Box::new(DashMap::with_capacity(100)));

        Self { services, firewall }
    }

    /// Retrieves the cache for a specific championship service.
    ///
    /// This asynchronous function checks if a service for the given `championship_id` exists. If it does,
    /// it returns the cached data for that service. If no service exists for the given ID, it returns `Ok(None)`.
    ///
    /// # Parameters
    ///
    /// - `championship_id`: A reference to the ID of the championship.
    ///
    /// # Returns
    ///
    /// An `AppResult` containing an `Option<Bytes>` which is:
    /// - `Some(Bytes)` if the cache data is available.
    /// - `None` if the service or cache data does not exist.
    pub async fn cache(&self, championship_id: &i32) -> AppResult<Option<Bytes>> {
        if let Some(service) = self.services.get(championship_id) {
            let cache = service.cache.read_arc();
            return cache.get().await;
        }

        Ok(None)
    }

    /// Subscribes to a channel for a specific championship service.
    ///
    /// This function checks if a service for the given `championship_id` exists. If it does,
    /// it subscribes to the service's channel and returns a `Receiver` that can be used to receive messages.
    ///
    /// # Parameters
    ///
    /// - `championship_id`: A reference to the ID of the championship.
    ///
    /// # Returns
    ///
    /// An `Option<Receiver<Bytes>>` which is:
    /// - `Some(Receiver)` if the service exists and subscription is successful.
    /// - `None` if no service exists for the given ID.
    pub fn subscribe(&self, championship_id: &i32) -> Option<Receiver<Bytes>> {
        let service = self.services.get(championship_id)?;
        Some(service.subscribe())
    }

    pub fn unsubscribe(&self, championship_id: &i32) {
        if let Some(service) = self.services.get(championship_id) {
            service.unsubscribe();
        }
    }

    /// Retrieves a list of all active services.
    ///
    /// This function returns a vector containing the IDs of all currently active services.
    ///
    /// # Returns
    ///
    /// A `Vec<i32>` containing the IDs of active services.
    pub fn services(&self) -> Vec<i32> {
        let mut services = Vec::with_capacity(self.services.len());

        for item in self.services {
            services.push(*item.key())
        }

        services
    }

    /// Checks if a specific service is active.
    ///
    /// This function returns `true` if a service with the given ID is currently active, and `false` otherwise.
    ///
    /// # Parameters
    ///
    /// - `id`: The ID of the service to check.
    ///
    /// # Returns
    ///
    /// `true` if the service is active, `false` otherwise.
    pub fn service(&self, id: &i32) -> bool {
        self.services.contains_key(id)
    }

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
    /// Initializes and runs a service on the specified `port` if it doesn't already exist.
    ///
    /// # Parameters
    /// - `port`: The port number to listen on.
    /// - `championship_id`: The championship ID to associate with the service.
    ///
    /// # Returns
    /// - `Ok(())` if the service starts successfully.
    /// - `Err(F1ServiceError::AlreadyExists)` if the service already exists.
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
            service_data.cache.clone(),
            self.firewall,
            self.services,
        )
        .await;

        service.initialize(port, championship_id).await?;

        ntex::rt::spawn(async move { service.run().await });

        self.services.insert(championship_id, service_data);

        Ok(())
    }

    /// Stops the active F1 service for the given championship.
    ///
    /// Shuts down the service and removes it if it's active.
    ///
    /// # Parameters
    /// - `championship_id`: The championship ID whose service is to be stopped.
    ///
    /// # Returns
    /// - `Ok(())` if the service stops successfully.
    /// - `Err(F1ServiceError::NotActive)` if no active service is found.
    /// - `Err(F1ServiceError::Shutdown)` if shutdown fails.
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
