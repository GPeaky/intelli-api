use tracing::subscriber::set_global_default;
use tracing_subscriber::FmtSubscriber;

pub fn initialize_tracing_subscriber() {
    let subscriber = FmtSubscriber::builder().compact().finish();

    set_global_default(subscriber).unwrap();
}
