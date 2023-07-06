use axum::Server;
use config::{initialize_tracing_subscriber, Database};
use dotenvy::{dotenv, var};
use hyper::Error;
use routes::service_routes;
use std::net::TcpListener;
use tracing::info;

mod config;
mod dtos;
mod entity;
mod error;
mod handlers;
mod response;
mod routes;
mod services;

#[tokio::main]
async fn main() -> Result<(), Error> {
    dotenv().ok();
    initialize_tracing_subscriber();
    let db = Database::default().await;

    //     db.get_scylla()
    //         .query(
    //             "CREATE TABLE IF NOT EXISTS intelli_api.users (
    //     id UUID,
    //     username varchar,
    //     password varchar,
    //     email varchar,
    //     created_at timestamp,
    //     updated_at timestamp,
    //     PRIMARY KEY (id, username, email)
    // )",
    //             &[],
    //         )
    //         .await
    //         .unwrap();

    let listener = TcpListener::bind(var("HOST").unwrap()).unwrap();

    info!("Server listening on {}", listener.local_addr().unwrap());
    Server::from_tcp(listener)?
        .serve(service_routes(db))
        .await?;

    Ok(())
}
