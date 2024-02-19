use crate::{
    cache::RedisCache,
    config::Database,
    repositories::{
        ChampionshipRepository, F123Repository, GoogleRepository, ServerRepository, UserRepository,
    },
    services::{
        ChampionshipService, EmailService, F123Service, FirewallService, SavedSessionService,
        TokenService, UserService,
    },
};

#[derive(Clone)]
pub struct AppState {
    pub user_svc: &'static UserService,
    pub user_repo: &'static UserRepository,
    pub token_svc: &'static TokenService,
    pub championship_svc: &'static ChampionshipService,
    pub championship_repo: &'static ChampionshipRepository,
    pub email_svc: EmailService,
    pub f123_svc: F123Service,
    pub f123_repo: F123Repository,
    pub saved_session_svc: &'static SavedSessionService,
    pub google_repo: &'static GoogleRepository,
    pub server_repo: ServerRepository,
}

impl AppState {
    pub async fn new(
        db: &'static Database,
        firewall_svc: FirewallService,
        cache: &'static RedisCache,
    ) -> Self {
        // Repositories
        let f123_repo = F123Repository::new(db);
        let user_repo = Box::leak(Box::new(UserRepository::new(db, cache)));
        let championship_repo = Box::leak(Box::new(ChampionshipRepository::new(db, cache)));
        let google_repo = Box::leak(Box::new(GoogleRepository::new()));

        // Services
        let token_svc = Box::leak(Box::from(TokenService::new(cache)));
        let user_svc = Box::leak(Box::from(
            UserService::new(db, cache, user_repo, token_svc).await,
        ));
        let championship_svc = Box::leak(Box::from(
            ChampionshipService::new(db, cache, user_repo, championship_repo).await,
        ));
        let saved_session_svc = Box::leak(Box::from(SavedSessionService::new(db, cache).await));

        Self {
            user_svc,
            f123_svc: F123Service::new(db, firewall_svc),
            f123_repo,
            user_repo,
            token_svc,
            championship_svc,
            championship_repo,
            email_svc: EmailService::new(),
            saved_session_svc,
            google_repo,
            server_repo: ServerRepository::new(db),
        }
    }
}
