use cache::RedisCache;
use config::{initialize_tracing_subscriber, Database};
use dotenvy::{dotenv, var};
use mimalloc::MiMalloc;
use ntex::web;
use services::FirewallService;
use states::AppStateInner;
use std::sync::Arc;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

mod cache;
mod config;
mod dtos;
mod entity;
mod error;
mod handlers;
mod middlewares;
mod protos;
mod repositories;
mod routes;
mod services;
mod states;

#[ntex::main]
async fn main() {
    dotenv().ok();
    initialize_tracing_subscriber();
    let db = Arc::new(Database::default().await);
    let redis_cache = Arc::new(RedisCache::new(&db));
    let firewall_service = Arc::new(FirewallService::new());
    let app_state = Arc::new(AppStateInner::new(&db, firewall_service, &redis_cache).await);

    web::server(move || {
        web::App::new()
            .configure(routes::api_routes)
            .configure(routes::admin_routes)
            .state(app_state.clone())
            .wrap(web::middleware::Logger::default())
    })
    .bind(var("HOST").unwrap())
    .unwrap()
    .run()
    .await
    .unwrap();
}
