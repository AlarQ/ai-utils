use crate::openai::OpenAIMessage;
use serde::Serialize;
use uuid::Uuid;

#[derive(Serialize)]
pub struct LangfuseTrace {
    pub id: Uuid,
    pub name: String,
    pub input: Vec<OpenAIMessage>,
    pub output: Vec<OpenAIMessage>,
    pub conversation_id: String,
}

pub struct LangfuseConfig {
    pub public_key: String,
    pub secret_key: String,
    pub api_url: String,
}

impl LangfuseConfig {
    pub fn new() -> Self {
        Self {
            public_key: std::env::var("LANGFUSE_PUBLIC_KEY")
                .expect("LANGFUSE_PUBLIC_KEY must be set"),
            secret_key: std::env::var("LANGFUSE_SECRET_KEY")
                .expect("LANGFUSE_SECRET_KEY must be set"),
            api_url: std::env::var("LANGFUSE_HOST")
                .unwrap_or_else(|_| "https://cloud.langfuse.com".to_string()),
        }
    }
}
