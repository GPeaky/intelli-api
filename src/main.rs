use axum::Server;
use config::{initialize_tracing_subscriber, Database};
use dotenvy::{dotenv, var};
use hyper::Error;
use routes::service_routes;
use std::{net::TcpListener, sync::Arc};
use tracing::info;

mod config;
mod dtos;
mod entity;
mod error;
mod handlers;
mod middlewares;
mod repositories;
mod response;
mod routes;
mod services;
mod states;

#[tokio::main]
async fn main() -> Result<(), Error> {
    dotenv().ok();
    initialize_tracing_subscriber();
    let db = Database::default().await;

    let listener = TcpListener::bind(var("HOST").unwrap()).unwrap();

    info!("Server listening on {}", listener.local_addr().unwrap());
    Server::from_tcp(listener)?
        .serve(service_routes(Arc::new(db)).await)
        .await?;

    Ok(())
}
