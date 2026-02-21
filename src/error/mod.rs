use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("OpenRouter error: {0}")]
    OpenRouter(#[from] async_openai::error::OpenAIError),

    #[error("Qdrant error: {0}")]
    Qdrant(#[from] qdrant_client::QdrantError),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Rate limited: retry after {retry_after:?}")]
    RateLimited {
        retry_after: Option<std::time::Duration>,
    },

    #[error("Missing parameter: {param}")]
    MissingParameter { param: String },

    #[error("Telemetry error: {0}")]
    Telemetry(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Request error: {0}")]
    Request(#[from] reqwest::Error),

    #[error("Invalid header value: {0}")]
    InvalidHeader(#[from] reqwest::header::InvalidHeaderValue),

    #[error("Other error: {0}")]
    Other(String),
}
