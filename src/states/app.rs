use crate::{
    cache::RedisCache,
    config::Database,
    repositories::{
        ChampionshipRepository, F123Repository, GoogleRepository, ServerRepository, UserRepository,
        UserRepositoryTrait,
    },
    services::{
        ChampionshipService, EmailService, F123Service, FirewallService, SavedSessionService,
        TokenService, TokenServiceTrait, UserService, UserServiceTrait,
    },
};

#[derive(Clone)]
pub struct AppState {
    pub user_svc: UserService,
    pub user_repo: &'static UserRepository,
    pub token_svc: TokenService,
    pub championship_svc: ChampionshipService,
    pub championship_repo: &'static ChampionshipRepository,
    pub email_svc: EmailService,
    pub f123_svc: F123Service,
    pub f123_repo: F123Repository,
    pub saved_session_svc: SavedSessionService,
    pub google_repo: GoogleRepository,
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

        // Services
        let user_svc = UserService::new(db, cache, user_repo).await;
        let championship_svc =
            ChampionshipService::new(db, cache, user_repo, championship_repo).await;

        Self {
            user_svc,
            f123_svc: F123Service::new(db, firewall_svc),
            f123_repo,
            user_repo,
            token_svc: TokenService::new(cache),
            championship_svc,
            championship_repo,
            email_svc: EmailService::new(),
            saved_session_svc: SavedSessionService::new(db, cache).await,
            google_repo: GoogleRepository::new(),
            server_repo: ServerRepository::new(db),
        }
    }
}
