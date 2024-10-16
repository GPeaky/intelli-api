use opentelemetry::global;
use opentelemetry::KeyValue;
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::{
    runtime::Tokio,
    trace::{self, Tracer},
    Resource,
};
use tracing_log::LogTracer;
use tracing_subscriber::{fmt, prelude::*, EnvFilter, Registry};

pub fn initialize_tracing_and_telemetry(
) -> Result<Tracer, Box<dyn std::error::Error + Send + Sync + 'static>> {
    global::set_text_map_propagator(opentelemetry_sdk::propagation::TraceContextPropagator::new());

    let tracer = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(
            opentelemetry_otlp::new_exporter()
                .tonic()
                .with_endpoint("http://localhost:4317"),
        )
        .with_trace_config(
            trace::config().with_resource(Resource::new(vec![KeyValue::new(
                "service.name",
                "f1-telemetry-api",
            )])),
        )
        .install_batch(Tokio)?;

    let filter = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new("info"))
        .unwrap();

    let fmt_layer = fmt::layer().with_ansi(true).compact();

    let telemetry_layer = tracing_opentelemetry::layer().with_tracer(tracer.clone());

    let subscriber = Registry::default()
        .with(filter)
        .with(fmt_layer)
        .with(telemetry_layer);

    LogTracer::init()?;

    tracing::subscriber::set_global_default(subscriber)?;

    Ok(tracer)
}
