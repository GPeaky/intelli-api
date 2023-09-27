use config::{initialize_tracing_subscriber, Database};
use dotenvy::{dotenv, var};
use hyper::server::conn::AddrIncoming;
use hyper::Server;
use hyper_rustls::TlsAcceptor;
use mimalloc::MiMalloc;
use rustls::PrivateKey;
use std::io::BufReader;
use std::sync::Arc;
use std::{fs, io};

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

mod capnp;
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
async fn main() {
    dotenv().ok();
    initialize_tracing_subscriber();
    let db = Database::default().await;

    let cert_file = load_certs("./certs/intellitelemetry.live.crt");
    let key_file = load_private_key("./certs/intellitelemetry.live.key").unwrap();

    let incoming = AddrIncoming::bind(&var("HOST").unwrap().parse().unwrap()).unwrap();

    let acceptor = TlsAcceptor::builder()
        .with_single_cert(cert_file, key_file)
        .unwrap()
        .with_alpn_protocols([b"h2".to_vec(), b"http/1.1".to_vec()].to_vec())
        .with_incoming(incoming);

    Server::builder(acceptor)
        .serve(routes::service_routes(Arc::new(db)).await)
        // .with_graceful_shutdown(signal)
        .await
        .unwrap();
}

fn load_certs(filename: &str) -> Vec<rustls::Certificate> {
    // Open certificate file.
    let cert_file = fs::File::open(filename).unwrap();
    let mut reader = io::BufReader::new(cert_file);

    // Load and return certificate.
    let certs = rustls_pemfile::certs(&mut reader).unwrap();

    certs.into_iter().map(rustls::Certificate).collect()
}

// Load private key from file.
fn load_private_key(filename: &str) -> Result<PrivateKey, Box<dyn std::error::Error>> {
    // Open keyfile.
    let key_file = fs::File::open(filename)?;
    let mut reader = BufReader::new(key_file);

    // Load and return a single private key.
    let keys = rustls_pemfile::pkcs8_private_keys(&mut reader)?;

    // Check if we've read at least one private key.
    if keys.is_empty() {
        return Err("No keys found in PEM file.".into());
    }

    Ok(PrivateKey(keys[0].clone()))
}
