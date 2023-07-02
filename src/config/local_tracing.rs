use tracing::subscriber::set_global_default;
use tracing_subscriber::FmtSubscriber;

pub fn initialize_tracing_subscriber() {
    let subscriber = FmtSubscriber::builder().without_time().finish();

    set_global_default(subscriber).unwrap();

    tracing::info!("Tracing initialized");
}
