use dotenvy::{dotenv, var};
use tokio::net::TcpListener;

use services::FirewallService;

use crate::{
    cache::RedisCache,
    config::{initialize_tracing_subscriber, Database},
    routes::routes,
    states::AppState,
};

mod cache;
mod config;
mod entity;
mod error;
mod handlers;
mod middlewares;
mod protos;
mod repositories;
mod response;
mod routes;
mod services;
mod states;
mod structs;
mod utils;

#[cfg(not(test))]
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

#[tokio::main]
async fn main() {
    dotenv().ok();
    initialize_tracing_subscriber();
    let app_state = {
        let db = Database::default().await;
        let redis_cache = RedisCache::new(&db);
        let firewall_service = FirewallService::new();

        AppState::new(&db, firewall_service, &redis_cache).await
    };

    let listener = TcpListener::bind(var("HOST").unwrap()).await.unwrap();

    axum::serve(listener, routes(app_state).into_make_service())
        .await
        .unwrap();
}
