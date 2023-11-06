use tracing_subscriber::{fmt, EnvFilter};

pub fn initialize_tracing_subscriber() {
    let filter = EnvFilter::new("intelli=trace");

    let subscriber = fmt::Subscriber::builder()
        .compact()
        .with_env_filter(filter)
        .without_time()
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("Error setting global subscriber");
}
