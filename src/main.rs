use config::{initialize_tracing_subscriber, Database};
use dotenvy::{dotenv, var};
use hyper::Server;
use mimalloc::MiMalloc;
use services::FirewallService;
use std::net::TcpListener;
use std::sync::Arc;
use tracing::info;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

mod config;
mod dtos;
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

#[tokio::main]
async fn main() {
    dotenv().ok();
    initialize_tracing_subscriber();
    let db = Database::default().await;
    let firewall_service = Arc::new(FirewallService::new());

    let incoming = TcpListener::bind(var("HOST").unwrap()).unwrap();

    Server::from_tcp(incoming)
        .unwrap()
        .serve(routes::service_routes(Arc::new(db), firewall_service.clone()).await)
        .with_graceful_shutdown(async {
            tokio::signal::ctrl_c()
                .await
                .expect("failed to install CTRL+C signal handler");

            firewall_service.close_all().await.unwrap();
        })
        .await
        .unwrap();
}
