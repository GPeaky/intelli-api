use db::Database;
use error::AppResult;

use f1_telemetry::{F1ChampionshipManager, F1State, FirewallService};
use intelli_core::{
    repositories::{
        ChampionshipRepository, DiscordRepository, DriverRepository, ServerRepository,
        UserRepository,
    },
    services::{ChampionshipService, DriverService, EmailService, UserService},
};
use token_manager::TokenManager;

#[derive(Clone)]
pub struct AppState {
    pub user_svc: &'static UserService,
    pub user_repo: &'static UserRepository,
    pub token_mgr: &'static TokenManager,
    pub championship_svc: &'static ChampionshipService,
    pub championship_repo: &'static ChampionshipRepository,
    #[allow(unused)]
    pub driver_repo: &'static DriverRepository,
    #[allow(unused)]
    pub driver_svc: &'static DriverService,
    pub email_svc: EmailService,
    pub f1_svc: F1ChampionshipManager,
    pub discord_repo: &'static DiscordRepository,
    pub server_repo: ServerRepository,
}

impl AppState {
    pub async fn new(db: &'static Database, firewall: &'static FirewallService) -> AppResult<Self> {
        // Repositories
        let user_repo = Box::leak(Box::new(UserRepository::new(db)));
        let discord_repo = Box::leak(Box::new(DiscordRepository::new()));
        let championship_repo = Box::leak(Box::new(ChampionshipRepository::new(db)));
        let driver_repo = Box::leak(Box::new(DriverRepository::new(db)));

        // Services
        let token_mgr = Box::leak(Box::from(TokenManager::load_from_file().unwrap()));
        let driver_svc = Box::leak(Box::new(DriverService::new(db, driver_repo).await));
        let user_svc = Box::leak(Box::from(UserService::new(db, user_repo, token_mgr).await));
        let championship_svc = Box::leak(Box::from(
            ChampionshipService::new(db, user_repo, championship_repo).await?,
        ));

        token_mgr.start_purge_thread();

        // Inner states
        let f1_state = Box::leak(Box::new(F1State::new(
            driver_svc,
            driver_repo,
            championship_repo,
            championship_svc,
            firewall,
        )));

        Ok(Self {
            user_svc,
            f1_svc: F1ChampionshipManager::new(f1_state),
            user_repo,
            token_mgr,
            championship_svc,
            championship_repo,
            driver_repo,
            driver_svc,
            email_svc: EmailService::new(),
            discord_repo,
            server_repo: ServerRepository::new(db),
        })
    }
}
