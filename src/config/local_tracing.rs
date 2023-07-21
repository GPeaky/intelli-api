use tracing_subscriber::FmtSubscriber;

pub fn initialize_tracing_subscriber() {
    let subscriber = FmtSubscriber::builder()
        .with_env_filter("daily_api=trace")
        .without_time()
        .finish();

    tracing::subscriber::set_global_default(subscriber).unwrap();
}
