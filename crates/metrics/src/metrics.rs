use std::env;

use opentelemetry::{global, trace::TraceError, KeyValue};
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::{self as sdk, propagation::TraceContextPropagator, runtime, Resource};
use opentelemetry_semantic_conventions::resource::{SERVICE_NAME, SERVICE_VERSION};

pub fn init_trace() -> Result<sdk::trace::TracerProvider, TraceError> {
    global::set_text_map_propagator(TraceContextPropagator::new());

    opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(
            opentelemetry_otlp::new_exporter()
                .tonic()
                .with_endpoint("http://localhost:4317"),
        )
        .with_trace_config(
            sdk::trace::Config::default().with_resource(Resource::new(vec![
                KeyValue::new(SERVICE_NAME, "intelli-api"),
                KeyValue::new(SERVICE_VERSION, env!("CARGO_PKG_VERSION")),
            ])),
        )
        .install_batch(runtime::Tokio)
}
