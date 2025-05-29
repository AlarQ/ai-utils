use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("OpenAI error: {0}")]
    OpenAI(#[from] async_openai::error::OpenAIError),

    #[error("Langfuse error: {0}")]
    Langfuse(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Request error: {0}")]
    Request(#[from] reqwest::Error),

    #[error("Other error: {0}")]
    Other(String),
}
