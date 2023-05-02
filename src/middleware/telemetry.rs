use crate::config::Settings;
use anyhow::Error;
use opentelemetry::{global, sdk::propagation::TraceContextPropagator};
use opentelemetry_otlp::WithExportConfig;
use std::str::FromStr;
use tonic::metadata::{MetadataKey, MetadataMap};

use tracing_subscriber::prelude::*;
use tracing_subscriber::Registry;

pub fn init_tracer(settings: &Settings) -> Result<(), Error> {
    let mut metadata = MetadataMap::new();
    metadata.insert(
        MetadataKey::from_str("x-honeycomb-team").unwrap(),
        "12yIxejTjCHyf4iuMCVn0P".parse().unwrap(),
    );

    let otlp_exporter = opentelemetry_otlp::new_exporter()
        .tonic()
        .with_env()
        .with_metadata(headers(settings))
        .with_endpoint(&settings.telemetry.receiver_url);

    // Spans are exported in batch - recommended setup for a production application.
    global::set_text_map_propagator(TraceContextPropagator::new());
    let _tracer = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(otlp_exporter)
        .install_batch(opentelemetry::runtime::Tokio)?;

    let subscriber = Registry::default().with(tracing_subscriber::fmt::layer());

    tracing::subscriber::set_global_default(subscriber)
        .expect("Failed to install `tracing` subscriber.");

    Ok(())
}

fn headers(settings: &Settings) -> MetadataMap {
    let mut metadata = MetadataMap::new();
    metadata.insert(
        MetadataKey::from_str("x-honeycomb-team").unwrap(),
        settings.telemetry.api_key.parse().unwrap(),
    );

    metadata
}
