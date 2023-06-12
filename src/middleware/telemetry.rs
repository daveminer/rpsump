use anyhow::Error;
use opentelemetry::sdk::{trace as sdktrace, Resource};
use opentelemetry::KeyValue;
use opentelemetry::{global, sdk::propagation::TraceContextPropagator};
use opentelemetry_otlp::WithExportConfig;
use std::str::FromStr;
use tonic::metadata::{MetadataKey, MetadataMap};
use tracing_subscriber::{prelude::*, EnvFilter, Registry};

use crate::config::Settings;

// Configure a global `tracing` subscriber. `actix-web-opentelemetry` will use this
// for spanning on requests.
pub fn init_tracer(settings: &Settings) -> Result<(), Error> {
    global::set_text_map_propagator(TraceContextPropagator::new());

    let otlp_exporter = opentelemetry_otlp::new_exporter()
        .tonic()
        .with_env()
        .with_metadata(headers(settings))
        .with_endpoint(&settings.telemetry.receiver_url);

    // Export traces in batches
    let tracer = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_trace_config(
            sdktrace::config()
                .with_resource(Resource::new(vec![KeyValue::new("service.name", "rpsump")]))
                .with_max_events_per_span(64),
        )
        .with_exporter(otlp_exporter)
        .install_batch(opentelemetry::runtime::Tokio)?;

    // TODO: remove add_directive
    let env_filter = EnvFilter::new("info").add_directive("my_crate::internal=off".parse()?);

    Registry::default()
        // Uncomment to output tracing debug logs to terminal
        //.with(tracing_subscriber::fmt::layer())
        .with(env_filter)
        .with(tracing_opentelemetry::layer().with_tracer(tracer))
        .init();

    Ok(())
}

// Configure the headers for the telemetry exporter, including external receiver
// authentication
fn headers(settings: &Settings) -> MetadataMap {
    let mut metadata = MetadataMap::new();
    metadata.insert(
        MetadataKey::from_str("x-honeycomb-team").unwrap(),
        settings.telemetry.api_key.parse().unwrap(),
    );

    metadata
}
