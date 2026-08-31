use std::str::FromStr;

use anyhow::Error;
use opentelemetry::{global, trace::TracerProvider as _};
use opentelemetry_otlp::{SpanExporter, WithExportConfig, WithTonicConfig};
use opentelemetry_sdk::{propagation::TraceContextPropagator, trace::SdkTracerProvider, Resource};
use tokio::time::Duration;
use tonic::metadata::{MetadataKey, MetadataMap};
use tracing_subscriber::{prelude::*, EnvFilter, Registry};

use crate::config::Settings;

/// Configure a global `tracing` subscriber. `actix-web-opentelemetry` will use this
/// for spanning on requests.
pub fn init_tracer(settings: &Settings) -> Result<SdkTracerProvider, Error> {
    global::set_text_map_propagator(TraceContextPropagator::new());

    let exporter = SpanExporter::builder()
        .with_tonic()
        .with_metadata(headers(settings))
        .with_endpoint(&settings.telemetry.receiver_url)
        .with_timeout(Duration::from_secs(10))
        .build()?;

    // Export traces in batches
    let provider = SdkTracerProvider::builder()
        .with_batch_exporter(exporter)
        .with_resource(Resource::builder().with_service_name("rpsump").build())
        .build();

    let tracer = provider.tracer("rpsump");
    global::set_tracer_provider(provider.clone());

    // TODO: remove add_directive
    let env_filter = EnvFilter::new("info").add_directive("my_crate::internal=off".parse()?);

    Registry::default()
        // Uncomment to output tracing debug logs to terminal
        //.with(tracing_subscriber::fmt::layer())
        .with(env_filter)
        .with(tracing_opentelemetry::layer().with_tracer(tracer))
        .init();

    Ok(provider)
}

// Configure the headers for the telemetry exporter, including external receiver
// authentication
fn headers(settings: &Settings) -> MetadataMap {
    let mut metadata = MetadataMap::with_capacity(1);
    metadata.insert(
        MetadataKey::from_str("x-honeycomb-team").unwrap(),
        settings.telemetry.api_key.parse().unwrap(),
    );

    metadata
}
