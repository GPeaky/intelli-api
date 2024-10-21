mod config;
mod handlers;
mod middlewares;
mod routes;
mod states;

use dashmap::DashMap;
use dotenvy::{dotenv, var};
use ntex::{http::header, time::Seconds, web};
use ntex_cors::Cors;
use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod};
use tracing::info;

use config::{initialize_tracing_subscriber, setup_panic_handler};
use db::Database;
use f1_telemetry::FirewallService;
use states::AppState;

#[cfg(not(test))]
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    initialize_tracing_subscriber();

    let firewall_svc = Box::leak(Box::new(FirewallService::new()));
    setup_panic_handler(firewall_svc);

    ntex::rt::System::new("intelli-api")
        .run_local(async {
            let app_state = {
                let db = Box::leak(Box::from(Database::new().await));
                AppState::new(db, firewall_svc).await.unwrap()
            };

            let mut builder = SslAcceptor::mozilla_intermediate(SslMethod::tls())?;

            builder.set_private_key_file("certs/key.pem", SslFiletype::PEM)?;
            builder.set_certificate_chain_file("certs/cert.pem")?;

            // TODO - Make an recycle function to delete all unused data
            let login_limit_visitors: &'static _ =
                Box::leak(Box::new(DashMap::with_capacity(1_000)));

            web::server(move || {
                web::App::new()
                    .configure(|svc| routes::api_routes(svc, login_limit_visitors))
                    .configure(routes::admin_routes)
                    .state(app_state.clone())
                    .wrap(
                        Cors::new()
                            .allowed_origin("https://intellitelemetry.live")
                            .allowed_origin("http://localhost:5173")
                            .allowed_methods(["GET", "POST", "DELETE"])
                            .allowed_headers([
                                header::ACCEPT,
                                header::CONTENT_TYPE,
                                header::AUTHORIZATION,
                                header::ACCESS_CONTROL_ALLOW_CREDENTIALS,
                            ])
                            .max_age(3600)
                            .finish(),
                    )
            })
            .shutdown_timeout(Seconds::new(10))
            .bind_openssl(var("HOST").unwrap(), builder)?
            .run()
            .await
        })
        .await?;

    info!("Stopping service, cleaning up firewall rules");
    firewall_svc.close_all().await.unwrap();

    Ok(())
}
