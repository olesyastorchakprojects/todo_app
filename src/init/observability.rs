use std::time::Duration;

use opentelemetry::global::{self};
use opentelemetry::trace::TracerProvider;
use opentelemetry_otlp::{MetricExporter, WithExportConfig};
use opentelemetry_sdk::{
    metrics::{PeriodicReader, SdkMeterProvider},
    trace::{Sampler, SdkTracerProvider},
    Resource,
};
use tracing_log::LogTracer;
use tracing_opentelemetry::OpenTelemetryLayer;
#[allow(unused_imports)]
use tracing_subscriber::fmt;
use tracing_subscriber::{layer::SubscriberExt, EnvFilter, Registry};

use crate::config::Settings;
use crate::utils::blocking_task_guard::init_blocking_tasks_metric;
use crate::utils::measure_metrics::init_memory_metrics;
use crate::utils::APP_NAME;

use super::StartupError;

pub fn init_tracer_provider(settings: &Settings) -> Result<SdkTracerProvider, StartupError> {
    LogTracer::init().expect("Failed to set logger");

    let otlp_exporter = opentelemetry_otlp::SpanExporter::builder()
        .with_tonic()
        .with_endpoint(&settings.telemetry.tracing_endpoint)
        .build()?;

    let provider = SdkTracerProvider::builder()
        .with_batch_exporter(otlp_exporter)
        .with_resource(Resource::builder().with_service_name(APP_NAME).build())
        .with_sampler(Sampler::ParentBased(Box::new(Sampler::TraceIdRatioBased(
            settings.telemetry.tracing_sampling_rate,
        ))))
        .build();

    let tracer = provider.tracer(APP_NAME);

    let telemetry_layer = OpenTelemetryLayer::new(tracer);

    let subscriber = Registry::default()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .with(telemetry_layer);

    if settings.telemetry.stdout_tracing {
        let fmt_layer = fmt::layer()
            .pretty()
            .with_level(true)
            .with_target(true)
            .with_thread_ids(true)
            .with_thread_names(true)
            .compact();
        tracing::subscriber::set_global_default(subscriber.with(fmt_layer))?;
    } else {
        tracing::subscriber::set_global_default(subscriber)?;
    }

    global::set_tracer_provider(provider.clone());

    Ok(provider)
}

pub fn init_metrics_provider(settings: &Settings) -> Result<SdkMeterProvider, StartupError> {
    let exporter = MetricExporter::builder()
        .with_tonic()
        .with_endpoint(&settings.telemetry.metrics_endpoint)
        .build()?;

    let provider = SdkMeterProvider::builder()
        .with_resource(Resource::builder().with_service_name(APP_NAME).build())
        .with_reader(
            PeriodicReader::builder(exporter)
                .with_interval(Duration::from_secs(10))
                .build(),
        )
        .build();

    global::set_meter_provider(provider.clone());

    init_memory_metrics();
    init_blocking_tasks_metric();

    Ok(provider)
}
