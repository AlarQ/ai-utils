use base64::Engine;

/// Configuration for OpenTelemetry telemetry
#[derive(Debug, Clone)]
pub struct TelemetryConfig {
    /// OTLP exporter endpoint (e.g., "https://cloud.langfuse.com/api/public/otel")
    pub endpoint: String,
    /// Service name for traces
    pub service_name: String,
    /// Optional authorization header (for Langfuse: base64("public_key:secret_key"))
    pub auth_header: Option<String>,
}

impl TelemetryConfig {
    /// Read configuration from environment variables
    ///
    /// Required:
    /// - OTEL_EXPORTER_OTLP_ENDPOINT
    ///
    /// Optional:
    /// - OTEL_SERVICE_NAME (defaults to "ai-utils")
    /// - LANGFUSE_PUBLIC_KEY + LANGFUSE_SECRET_KEY (builds auth header for Langfuse)
    pub fn from_env() -> crate::Result<Self> {
        use crate::error::Error;

        let endpoint = std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT")
            .map_err(|_| Error::Config("OTEL_EXPORTER_OTLP_ENDPOINT must be set".to_string()))?;

        let service_name =
            std::env::var("OTEL_SERVICE_NAME").unwrap_or_else(|_| "ai-utils".to_string());

        // Build auth header from Langfuse credentials if available
        let auth_header = if let (Ok(public), Ok(secret)) = (
            std::env::var("LANGFUSE_PUBLIC_KEY"),
            std::env::var("LANGFUSE_SECRET_KEY"),
        ) {
            let credentials = format!("{}:{}", public, secret);
            let auth = base64::engine::general_purpose::STANDARD.encode(credentials);
            Some(format!("Basic {}", auth))
        } else {
            None
        };

        Ok(Self {
            endpoint,
            service_name,
            auth_header,
        })
    }
}

/// Guard that holds the tracer provider for graceful shutdown
pub struct TelemetryGuard {
    provider: opentelemetry_sdk::trace::SdkTracerProvider,
}

impl TelemetryGuard {
    /// Create a new telemetry guard
    pub(crate) fn new(provider: opentelemetry_sdk::trace::SdkTracerProvider) -> Self {
        Self { provider }
    }

    /// Shutdown the telemetry pipeline gracefully, flushing all spans
    pub fn shutdown(self) {
        if let Err(e) = self.provider.shutdown() {
            eprintln!("Failed to shutdown telemetry provider: {:?}", e);
        }
    }
}

impl Drop for TelemetryGuard {
    fn drop(&mut self) {
        // Note: We can't use shutdown() here because Drop takes &mut self
        // but shutdown() takes self. The explicit shutdown() method should
        // be called when possible for graceful shutdown.
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_telemetry_config_from_env_success() {
        // Set required environment variables
        std::env::set_var("OTEL_EXPORTER_OTLP_ENDPOINT", "https://otel.example.com");
        std::env::set_var("OTEL_SERVICE_NAME", "test-service");

        // Clear optional auth vars to test without auth
        std::env::remove_var("LANGFUSE_PUBLIC_KEY");
        std::env::remove_var("LANGFUSE_SECRET_KEY");

        let config = TelemetryConfig::from_env().expect("Should parse config");

        assert_eq!(config.endpoint, "https://otel.example.com");
        assert_eq!(config.service_name, "test-service");
        assert!(config.auth_header.is_none());
    }

    #[test]
    fn test_telemetry_config_from_env_with_defaults() {
        // Set only required variable
        std::env::set_var("OTEL_EXPORTER_OTLP_ENDPOINT", "https://otel.example.com");
        std::env::remove_var("OTEL_SERVICE_NAME");
        std::env::remove_var("LANGFUSE_PUBLIC_KEY");
        std::env::remove_var("LANGFUSE_SECRET_KEY");

        let config = TelemetryConfig::from_env().expect("Should parse config");

        assert_eq!(config.endpoint, "https://otel.example.com");
        assert_eq!(config.service_name, "ai-utils"); // Default value
        assert!(config.auth_header.is_none());
    }

    #[test]
    fn test_telemetry_config_from_env_missing_endpoint() {
        // Clear required variable
        std::env::remove_var("OTEL_EXPORTER_OTLP_ENDPOINT");

        let result = TelemetryConfig::from_env();

        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("OTEL_EXPORTER_OTLP_ENDPOINT"));
    }

    #[test]
    fn test_telemetry_config_from_env_with_langfuse_auth() {
        // Set required variables
        std::env::set_var(
            "OTEL_EXPORTER_OTLP_ENDPOINT",
            "https://cloud.langfuse.com/api/public/otel",
        );
        std::env::set_var("OTEL_SERVICE_NAME", "test-service");

        // Set Langfuse credentials
        std::env::set_var("LANGFUSE_PUBLIC_KEY", "pk-test-123");
        std::env::set_var("LANGFUSE_SECRET_KEY", "sk-test-456");

        let config = TelemetryConfig::from_env().expect("Should parse config");

        assert_eq!(
            config.endpoint,
            "https://cloud.langfuse.com/api/public/otel"
        );
        assert_eq!(config.service_name, "test-service");

        // Auth header should be present and start with "Basic "
        let auth_header = config.auth_header.expect("Auth header should be present");
        assert!(auth_header.starts_with("Basic "));

        // Verify the base64 encoding is correct
        let encoded = &auth_header[6..]; // Remove "Basic " prefix
        let decoded = base64::engine::general_purpose::STANDARD
            .decode(encoded.as_bytes())
            .expect("Should decode base64");
        let decoded_str = String::from_utf8(decoded).expect("Should be valid UTF-8");
        assert_eq!(decoded_str, "pk-test-123:sk-test-456");
    }

    #[test]
    fn test_telemetry_config_with_partial_langfuse_credentials() {
        // Set required variables
        std::env::set_var("OTEL_EXPORTER_OTLP_ENDPOINT", "https://otel.example.com");

        // Only set public key, not secret key
        std::env::set_var("LANGFUSE_PUBLIC_KEY", "pk-test-123");
        std::env::remove_var("LANGFUSE_SECRET_KEY");

        let config = TelemetryConfig::from_env().expect("Should parse config");

        // Should not have auth header with partial credentials
        assert!(config.auth_header.is_none());
    }
}
