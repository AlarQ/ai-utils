use opentelemetry::trace::TracerProvider as _;
use opentelemetry::KeyValue;
use opentelemetry_otlp::{Protocol, WithExportConfig, WithHttpConfig};
use opentelemetry_sdk::resource::Resource;
use opentelemetry_sdk::trace::SdkTracerProvider;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

use crate::error::Error;
use crate::telemetry::types::{TelemetryConfig, TelemetryGuard};

/// Initialize the OpenTelemetry tracing pipeline
///
/// Call this once at application startup
///
/// # Errors
///
/// Returns `Error::Telemetry` if initialization fails
///
/// # Example
///
/// ```rust,no_run
/// use ai_utils::telemetry::{init_telemetry, TelemetryConfig};
///
/// # fn main() -> ai_utils::Result<()> {
/// let config = TelemetryConfig::from_env()?;
/// let _guard = init_telemetry(config)?;
/// // ... application runs here ...
/// // Guard will shutdown telemetry on drop
/// # Ok(())
/// # }
/// ```
pub fn init_telemetry(config: TelemetryConfig) -> crate::Result<TelemetryGuard> {
    // Build headers map for auth if needed
    let mut headers = std::collections::HashMap::new();
    if let Some(auth) = &config.auth_header {
        headers.insert("Authorization".to_string(), auth.clone());
    }

    // Create OTLP HTTP exporter
    let exporter = opentelemetry_otlp::SpanExporter::builder()
        .with_http()
        .with_protocol(Protocol::HttpBinary)
        .with_endpoint(&config.endpoint)
        .with_headers(headers)
        .build()
        .map_err(|e| Error::Telemetry(format!("Failed to create exporter: {e}")))?;

    // Build resource
    let resource = Resource::builder()
        .with_attributes(vec![
            KeyValue::new("service.name", config.service_name.clone()),
            KeyValue::new("service.version", env!("CARGO_PKG_VERSION")),
        ])
        .build();

    // Build TracerProvider with resource
    let provider = SdkTracerProvider::builder()
        .with_batch_exporter(exporter)
        .with_resource(resource)
        .build();

    // Create tracing-opentelemetry layer
    let tracer = provider.tracer("ai-utils");
    let telemetry = tracing_opentelemetry::layer().with_tracer(tracer);

    // Initialize tracing subscriber with the OTEL layer
    // Note: This will fail if a subscriber is already set, which is expected behavior
    tracing_subscriber::registry()
        .with(telemetry)
        .try_init()
        .map_err(|e| Error::Telemetry(format!("Failed to init subscriber: {e}")))?;

    Ok(TelemetryGuard::new(provider))
}

/// Create a span for LLM operations with standard attributes
///
/// Usage:
/// ```rust
/// use ai_utils::telemetry::llm_span;
///
/// let span = llm_span("chat_completion", "gpt-4o");
/// let _enter = span.enter();
/// // ... do work
/// ```
#[must_use]
pub fn llm_span(operation: &str, model: &str) -> tracing::Span {
    tracing::info_span!(
        "llm.operation",
        otel.name = format!("llm.{}", operation),
        llm.operation = operation,
        llm.model = model,
        llm.system = "openrouter",
    )
}

/// Create a span for embedding operations
///
/// Usage:
/// ```rust
/// use ai_utils::telemetry::embedding_span;
///
/// let span = embedding_span("text-embedding-3-small");
/// let _enter = span.enter();
/// // ... do work
/// ```
#[must_use]
pub fn embedding_span(model: &str) -> tracing::Span {
    tracing::info_span!(
        "llm.embedding",
        otel.name = "llm.embedding",
        llm.operation = "embedding",
        llm.model = model,
        llm.system = "openrouter",
    )
}

/// Create a span for generic vector operations (search, upsert, etc.)
///
/// Usage:
/// ```rust
/// use ai_utils::telemetry::vector_span;
///
/// let span = vector_span("vector_search", "qdrant");
/// let _enter = span.enter();
/// // ... do work
/// ```
#[must_use]
pub fn vector_span(operation: &str, provider: &str) -> tracing::Span {
    tracing::info_span!(
        "vector.operation",
        otel.name = format!("vector.{}", operation),
        vector.operation = operation,
        vector.provider = provider,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_llm_span_creation() {
        let span = llm_span("chat_completion", "gpt-4o");
        // Verify the span was created - we can't inspect internals easily,
        // but we can verify it's a valid span by entering it
        let _enter = span.enter();
        // Span should be entered without panicking
    }

    #[test]
    fn test_embedding_span_creation() {
        let span = embedding_span("text-embedding-3-small");
        let _enter = span.enter();
        // Span should be entered without panicking
    }

    #[test]
    fn test_vector_span_creation() {
        let span = vector_span("search", "qdrant");
        let _enter = span.enter();
        // Span should be entered without panicking
    }

    // Note: We can't easily test init_telemetry in unit tests because:
    // 1. It sets a global subscriber which can only be set once per process
    // 2. It requires a real OTLP endpoint or complex mocking
    // Integration tests should test the full initialization flow
}
