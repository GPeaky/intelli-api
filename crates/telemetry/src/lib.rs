mod f1_structs;
mod firewall;
mod handler;
mod manager;
mod service;
mod utils;

pub use handler::*;
pub use manager::DriverInfo;

use intelli_core::{
    repositories::{ChampionshipRepository, DriverRepository},
    services::{ChampionshipService, DriverService},
};

pub struct F1State {
    pub driver_svc: &'static DriverService,
    pub firewall: &'static FirewallService,
    pub driver_repo: &'static DriverRepository,
    pub championship_repo: &'static ChampionshipRepository,
    pub championship_svc: &'static ChampionshipService,
}

impl F1State {
    pub fn new(
        driver_svc: &'static DriverService,
        driver_repo: &'static DriverRepository,
        championship_repo: &'static ChampionshipRepository,
        championship_svc: &'static ChampionshipService,
    ) -> Self {
        let firewall = Box::leak(Box::new(FirewallService::new()));

        F1State {
            firewall,
            driver_svc,
            driver_repo,
            championship_repo,
            championship_svc,
        }
    }
}
