//! Telemetry initialisation for the daemon.
//!
//! - With `--features otel`: configures an OTLP span exporter (gRPC/tonic)
//!   alongside the standard `tracing_subscriber::fmt` layer.
//! - Without `otel`: plain `tracing_subscriber::fmt` only.

use tracing_subscriber::EnvFilter;

/// Initialise tracing.
///
/// `mcp_mode` controls whether log output goes to stderr (required when
/// stdout is reserved for MCP stdio transport).
#[cfg(not(feature = "otel"))]
pub fn init_tracing(mcp_mode: bool) -> Result<(), Box<dyn std::error::Error>> {
    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    if mcp_mode {
        tracing_subscriber::fmt()
            .with_env_filter(env_filter)
            .with_writer(std::io::stderr)
            .init();
    } else {
        tracing_subscriber::fmt()
            .with_env_filter(env_filter)
            .init();
    }
    Ok(())
}

/// Initialise tracing with OpenTelemetry OTLP export.
///
/// The OTLP exporter reads `OTEL_EXPORTER_OTLP_ENDPOINT` from the
/// environment (default: `http://localhost:4317`) and sends spans over
/// gRPC via tonic.  A local `fmt` layer is always active as well so
/// that developers still get human-readable console output.
#[cfg(feature = "otel")]
pub fn init_tracing(mcp_mode: bool) -> Result<(), Box<dyn std::error::Error>> {
    use opentelemetry::trace::TracerProvider as _;
    use opentelemetry_otlp::SpanExporter;
    use opentelemetry_sdk::trace::SdkTracerProvider;
    use tracing_subscriber::layer::SubscriberExt;
    use tracing_subscriber::util::SubscriberInitExt;

    // --- OpenTelemetry tracer provider ---
    let otlp_exporter = SpanExporter::builder()
        .with_tonic()
        .build()?;

    let tracer_provider = SdkTracerProvider::builder()
        .with_batch_exporter(otlp_exporter)
        .build();

    opentelemetry::global::set_tracer_provider(tracer_provider.clone());

    let tracer = tracer_provider.tracer("aletheiad");

    // --- fmt layer (human-readable logs) ---
    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    // The otel layer must be constructed in each branch because the
    // subscriber type parameter `S` differs depending on the fmt writer.
    if mcp_mode {
        let fmt_layer = tracing_subscriber::fmt::layer()
            .with_writer(std::io::stderr);
        let otel_layer = tracing_opentelemetry::layer().with_tracer(tracer);

        tracing_subscriber::registry()
            .with(env_filter)
            .with(fmt_layer)
            .with(otel_layer)
            .init();
    } else {
        let fmt_layer = tracing_subscriber::fmt::layer();
        let otel_layer = tracing_opentelemetry::layer().with_tracer(tracer);

        tracing_subscriber::registry()
            .with(env_filter)
            .with(fmt_layer)
            .with(otel_layer)
            .init();
    }

    Ok(())
}
