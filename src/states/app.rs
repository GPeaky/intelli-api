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
    pub user_service: UserService,
    pub user_repository: UserRepository,
    pub token_service: TokenService,
    pub championship_service: ChampionshipService,
    pub championship_repository: ChampionshipRepository,
    pub email_service: EmailService,
    pub f123_service: F123Service,
    pub f123_repository: F123Repository,
    pub saved_session_service: SavedSessionService,
    pub google_repository: GoogleRepository,
    pub server_repository: ServerRepository,
}

impl AppState {
    pub async fn new(
        db_conn: &Database,
        firewall_service: FirewallService,
        cache: &RedisCache,
    ) -> Self {
        Self {
            user_service: UserService::new(db_conn, cache),
            f123_service: F123Service::new(db_conn, firewall_service),
            f123_repository: F123Repository::new(db_conn),
            user_repository: UserRepository::new(db_conn, cache),
            token_service: TokenService::new(cache),
            championship_service: ChampionshipService::new(db_conn, cache).await,
            championship_repository: ChampionshipRepository::new(db_conn, cache).await,
            email_service: EmailService::new(),
            saved_session_service: SavedSessionService::new(db_conn, cache),
            google_repository: GoogleRepository::new(),
            server_repository: ServerRepository::new(db_conn),
        }
    }
}
