use crate::config::application::ApplicationConfig;
use crate::infrastructure::observability::metrics_layer::MetricsLayer;
use opentelemetry::KeyValue;
use opentelemetry::trace::TracerProvider;
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::Resource;
use opentelemetry_sdk::trace::{Sampler, TracerProvider as SdkTracerProvider};
use tracing_subscriber::EnvFilter;
use tracing_subscriber::fmt::format::FmtSpan;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

const SERVICE_NAME: &str = "giteye-bot";

/// Initializes the global tracing subscriber.
///
/// Layers (in order):
/// 1. EnvFilter (reads RUST_LOG, defaults to debug/info based on `config.debug`).
/// 2. fmt (text or JSON, depending on `LOG_FORMAT` env).
/// 3. OpenTelemetry (only when `OTEL_ENABLED=true`; exports OTLP/HTTP to `OTEL_EXPORTER_OTLP_ENDPOINT`).
/// 4. MetricsLayer — auto-increments `errors_total` on every `tracing::error!`.
pub fn init(config: &ApplicationConfig) {
    let debug_filter: &str = if config.debug { "debug" } else { "info" };
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(debug_filter));

    let log_format = std::env::var("LOG_FORMAT").unwrap_or_else(|_| "text".to_string());

    let otel_enabled = std::env::var("OTEL_ENABLED")
        .map(|v| v == "true" || v == "1")
        .unwrap_or(false);

    let otel_tracer = if otel_enabled {
        match build_otel_tracer() {
            Ok(tracer) => Some(tracer),
            Err(e) => {
                eprintln!("Failed to initialize OpenTelemetry exporter: {}", e);
                None
            }
        }
    } else {
        None
    };

    if log_format == "json" {
        let fmt_layer = tracing_subscriber::fmt::layer()
            .json()
            .with_target(true)
            .with_file(true)
            .with_line_number(config.debug)
            .with_span_events(FmtSpan::ENTER | FmtSpan::EXIT);
        let otel_layer = otel_tracer
            .clone()
            .map(|tracer| tracing_opentelemetry::layer().with_tracer(tracer));

        tracing_subscriber::registry()
            .with(env_filter)
            .with(fmt_layer)
            .with(otel_layer)
            .with(MetricsLayer)
            .init();
    } else {
        let fmt_layer = tracing_subscriber::fmt::layer()
            .with_target(true)
            .with_file(true)
            .with_line_number(config.debug)
            .with_span_events(FmtSpan::ENTER | FmtSpan::EXIT);
        let otel_layer = otel_tracer
            .map(|tracer| tracing_opentelemetry::layer().with_tracer(tracer));

        tracing_subscriber::registry()
            .with(env_filter)
            .with(fmt_layer)
            .with(otel_layer)
            .with(MetricsLayer)
            .init();
    }
}

fn build_otel_tracer() -> Result<opentelemetry_sdk::trace::Tracer, Box<dyn std::error::Error>> {
    let endpoint = std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT")
        .unwrap_or_else(|_| "http://localhost:4318".to_string());

    let exporter = opentelemetry_otlp::SpanExporter::builder()
        .with_http()
        .with_endpoint(format!("{}/v1/traces", endpoint.trim_end_matches('/')))
        .with_timeout(std::time::Duration::from_secs(5))
        .build()?;

    let resource = Resource::new(vec![
        KeyValue::new(
            opentelemetry_semantic_conventions::resource::SERVICE_NAME,
            SERVICE_NAME,
        ),
        KeyValue::new(
            opentelemetry_semantic_conventions::resource::SERVICE_VERSION,
            env!("CARGO_PKG_VERSION"),
        ),
    ]);

    let provider = SdkTracerProvider::builder()
        .with_resource(resource)
        .with_sampler(Sampler::AlwaysOn)
        .with_batch_exporter(exporter, opentelemetry_sdk::runtime::Tokio)
        .build();

    let tracer = provider.tracer(SERVICE_NAME);
    opentelemetry::global::set_tracer_provider(provider);

    Ok(tracer)
}

/// Flushes pending spans on shutdown. Call from `main` before exit.
pub fn shutdown() {
    opentelemetry::global::shutdown_tracer_provider();
}
