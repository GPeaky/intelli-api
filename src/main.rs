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
mod utils;

use cache::RedisCache;
use config::{initialize_tracing_subscriber, Database};
use dotenvy::{dotenv, var};
use ntex::{http, web};
use ntex_cors::Cors;
use services::FirewallService;
use states::AppState;

#[cfg(not(test))]
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

#[ntex::main]
async fn main() {
    dotenv().ok();
    initialize_tracing_subscriber();
    let app_state = {
        let db = Database::default().await;
        let redis_cache = RedisCache::new(&db);
        let firewall_service = FirewallService::new();

        AppState::new(&db, firewall_service, &redis_cache).await
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
                        http::header::ACCESS_CONTROL_ALLOW_ORIGIN,
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
