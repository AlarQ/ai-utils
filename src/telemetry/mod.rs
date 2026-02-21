//! OpenTelemetry-based telemetry module
//!
//! This module provides OpenTelemetry integration for tracing LLM operations,
//! replacing the legacy Langfuse-specific implementation.
//!
//! # Quick Start
//!
//! ```rust,no_run
//! use ai_utils::telemetry::{init_telemetry, TelemetryConfig, llm_span};
//!
//! # fn main() -> ai_utils::Result<()> {
//! // Initialize telemetry from environment variables
//! let config = TelemetryConfig::from_env()?;
//! let _guard = init_telemetry(config)?;
//!
//! // Create spans for LLM operations
//! let span = llm_span("chat_completion", "gpt-4o");
//! let _enter = span.enter();
//!
//! // Your LLM operations here...
//!
//! // Telemetry is automatically flushed when _guard is dropped
//! # Ok(())
//! # }
//! ```
//!
//! # Configuration
//!
//! Required environment variables:
//! - `OTEL_EXPORTER_OTLP_ENDPOINT` - OTLP HTTP endpoint URL
//!
//! Optional environment variables:
//! - `OTEL_SERVICE_NAME` - Service name for traces (default: "ai-utils")
//! - `LANGFUSE_PUBLIC_KEY` + `LANGFUSE_SECRET_KEY` - For Langfuse OTEL endpoint auth

mod service;
mod types;

pub use service::*;
pub use types::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_telemetry_config_from_env() {
        // Set required environment variable
        std::env::set_var("OTEL_EXPORTER_OTLP_ENDPOINT", "https://otel.example.com");
        std::env::set_var("OTEL_SERVICE_NAME", "test-service");

        // Clear optional auth vars
        std::env::remove_var("LANGFUSE_PUBLIC_KEY");
        std::env::remove_var("LANGFUSE_SECRET_KEY");

        let config = TelemetryConfig::from_env().expect("Should parse config from env");
        assert_eq!(config.endpoint, "https://otel.example.com");
        assert_eq!(config.service_name, "test-service");
    }

    // Note: Testing init_telemetry is challenging because:
    // 1. It sets a global subscriber which can only be set once per test process
    // 2. It requires an OTLP endpoint or complex mocking infrastructure
    //
    // For integration testing, use a dedicated test binary or:
    // - Use the test in types.rs which validates config parsing
    // - Use manual testing with a real OTLP collector

    #[test]
    fn test_span_helpers_compile() {
        // This test just ensures the span helper functions compile and return valid spans
        let _llm = llm_span("test", "gpt-4");
        let _embedding = embedding_span("text-embedding-3-small");
        let _vector = vector_span("search", "qdrant");
    }

    #[test]
    fn test_module_re_exports() {
        // Verify all public types are re-exported
        let _: TelemetryConfig = TelemetryConfig {
            endpoint: "test".to_string(),
            service_name: "test".to_string(),
            auth_header: None,
        };

        // TelemetryGuard cannot be constructed directly (only via init_telemetry)
        // but we verify it's exported by checking it can be named in a type context
        fn _check_guard_exported(_: Option<TelemetryGuard>) {}
    }
}
