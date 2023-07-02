use tracing::subscriber::set_global_default;
use tracing_subscriber::FmtSubscriber;

pub fn initialize_tracing_subscriber() {
    let subscriber = FmtSubscriber::builder()
        .with_env_filter("intelli-telemetry-f123=trace")
        .finish();

    set_global_default(subscriber).unwrap();
}
