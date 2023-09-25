use tracing_subscriber::fmt;
use tracing_subscriber::EnvFilter;

pub fn initialize_tracing_subscriber() {
    let filter = EnvFilter::new("intelli=trace"); // Solo registra errores

    let subscriber = fmt::Subscriber::builder()
        .compact() // Formato compacto
        .with_env_filter(filter) // Filtrado para solo errores
        .without_time() // No incluir marcas de tiempo para reducir el formato
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("Error setting global subscriber");
}
