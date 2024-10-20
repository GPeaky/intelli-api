use telemetry::FirewallService;
use tokio::runtime::Builder;
use tracing::error;
use tracing_log::LogTracer;
use tracing_subscriber::{EnvFilter, FmtSubscriber};

pub fn setup_panic_handler(firewall_svc: &'static FirewallService) {
    let default_panic = std::panic::take_hook();

    std::panic::set_hook(Box::new(move |panic_info| {
        default_panic(panic_info);
        error!("Panic ocurred, cleaning up firewall rules");

        std::thread::spawn(move || {
            match Builder::new_multi_thread()
                .enable_io()
                .enable_time()
                .build()
            {
                Ok(rt) => {
                    if let Err(e) = rt.block_on(firewall_svc.close_all()) {
                        error!("Error during firewall cleanup: {:?}", e);
                    }
                }

                Err(_) => {
                    error!("Failed to create Tokio runtime for firewall cleanup")
                }
            }
        })
        .join()
        .unwrap();
    }));
}

pub fn initialize_tracing_subscriber() {
    let filter = EnvFilter::new("info,intelli=trace");

    let subscriber = FmtSubscriber::builder()
        .with_ansi(true)
        .with_max_level(tracing::Level::TRACE)
        .with_env_filter(filter)
        .compact()
        .finish();

    LogTracer::init().unwrap();
    tracing::subscriber::set_global_default(subscriber).unwrap();
}
