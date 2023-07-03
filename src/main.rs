use axum::Server;
use config::{initialize_tracing_subscriber, Database};
use dotenvy::{dotenv, var};
use hyper::Error;
use routes::service_routes;
use std::net::TcpListener;
use tracing::info;

mod config;
mod dtos;
mod handlers;
mod routes;
mod services;

#[tokio::main]
async fn main() -> Result<(), Error> {
    dotenv().ok();
    initialize_tracing_subscriber();
    let db = Database::default();

    let listener = TcpListener::bind(var("HOST").unwrap()).unwrap();

    info!("Server listening on {}", listener.local_addr().unwrap());
    Server::from_tcp(listener)?.serve(service_routes()).await?;

    Ok(())
}
