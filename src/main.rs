use cache::RedisCache;
use config::{initialize_tracing_subscriber, Database};
use dotenvy::{dotenv, var};
use mimalloc::MiMalloc;
use ntex::{http, web};
use ntex_cors::Cors;
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
    let app_state = {
        let db = Arc::new(Database::default().await);
        let redis_cache = Arc::new(RedisCache::new(&db));
        let firewall_service = Arc::new(FirewallService::new());

        Arc::new(AppStateInner::new(&db, firewall_service, &redis_cache).await)
    };

    web::server(move || {
        web::App::new()
            .configure(routes::api_routes)
            .configure(routes::admin_routes)
            .state(app_state.clone())
            .wrap(
                Cors::new()
                    .allowed_origin("https://intellitelemetry.live")
                    .allowed_origin("http://localhost:5173")
                    .allowed_methods(vec!["GET", "POST", "DELETE"])
                    .allowed_headers(vec![
                        http::header::AUTHORIZATION,
                        http::header::ACCEPT,
                        http::header::ACCESS_CONTROL_ALLOW_CREDENTIALS,
                    ])
                    .allowed_header(http::header::CONTENT_TYPE)
                    .max_age(3600)
                    .finish(),
            )
    })
    .bind(var("HOST").unwrap())
    .unwrap()
    .run()
    .await
    .unwrap();
}
