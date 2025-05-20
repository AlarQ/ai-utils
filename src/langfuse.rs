use crate::openai::ChatCompletion;
use crate::Message;
use async_trait::async_trait;
use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use chrono;
use reqwest::Client;
use serde::Serialize;
use serde_json::json;
use std::env;
use uuid::Uuid;

#[derive(Serialize)]
pub struct LangfuseTrace {
    pub id: Uuid,
    pub name: String,
    pub input: Vec<Message>,
    pub output: Vec<Message>,
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
            public_key: env::var("LANGFUSE_PUBLIC_KEY").expect("LANGFUSE_PUBLIC_KEY must be set"),
            secret_key: env::var("LANGFUSE_SECRET_KEY").expect("LANGFUSE_SECRET_KEY must be set"),
            api_url: env::var("LANGFUSE_HOST")
                .unwrap_or_else(|_| "https://cloud.langfuse.com".to_string()),
        }
    }
}

pub struct LangfuseServiceImpl {
    config: LangfuseConfig,
    client: Client,
}

impl LangfuseServiceImpl {
    pub fn new(config: LangfuseConfig) -> Self {
        Self {
            config,
            client: Client::new(),
        }
    }

    fn get_auth_header(&self) -> String {
        let credentials = format!("{}:{}", self.config.public_key, self.config.secret_key);
        format!("Basic {}", BASE64.encode(credentials))
    }
}

#[async_trait]
pub trait LangfuseService: Send + Sync {
    async fn create_trace(
        &self,
        trace_id: Uuid,
        name: &str,
        input: &[Message],
        output: &[Message],
        conversation_id: &str,
    ) -> String;
    async fn create_span(
        &self,
        trace_id: &str,
        name: &str,
        input: &[Message],
        output: &[Message],
    ) -> String;
    async fn finalize_span(
        &self,
        span_id: &str,
        name: &str,
        input: &[Message],
        output: &ChatCompletion,
    );
    async fn finalize_trace(&self, trace_id: &str, input: &[Message], output: &[Message]);
}

#[async_trait]
impl LangfuseService for LangfuseServiceImpl {
    async fn create_trace(
        &self,
        trace_id: Uuid,
        name: &str,
        input: &[Message],
        output: &[Message],
        conversation_id: &str,
    ) -> String {
        let url = format!("{}/api/public/ingestion", self.config.api_url);

        let batch = json!({
            "batch": [
                {
                    "type": "trace-create",
                    "id": trace_id.to_string(),
                    "timestamp": chrono::Utc::now().to_rfc3339(),
                    "body": {
                        "id": trace_id.to_string(),
                        "name": name,
                        "input": input,
                        "output": output,
                        "metadata": {
                            "conversation_id": conversation_id
                        }
                    }
                }
            ]
        });

        let response = self
            .client
            .post(&url)
            .header("Authorization", self.get_auth_header())
            .json(&batch)
            .send()
            .await
            .expect("Failed to create trace");

        println!("Response status: {:?}", response.status());
        println!("Response: {:?}", response.text().await.unwrap());
        trace_id.to_string()
    }

    async fn create_span(
        &self,
        trace_id: &str,
        name: &str,
        input: &[Message],
        output: &[Message],
    ) -> String {
        let span_id = uuid::Uuid::new_v4().to_string();
        let url = format!("{}/api/public/ingestion", self.config.api_url);

        let batch = json!({
            "batch": [
                {
                    "type": "span-create",
                    "id": span_id,
                    "timestamp": chrono::Utc::now().to_rfc3339(),
                    "body": {
                        "id": span_id,
                        "name": name,
                        "traceId": trace_id,
                        "input": input,
                        "output": output
                    }
                }
            ]
        });

        let response = self
            .client
            .post(&url)
            .header("Authorization", self.get_auth_header())
            .json(&batch)
            .send()
            .await
            .expect("Failed to create span");

        println!("Response status: {:?}", response.status());
        println!("Response: {:?}", response.text().await.unwrap());
        span_id
    }

    async fn finalize_span(
        &self,
        span_id: &str,
        name: &str,
        input: &[Message],
        output: &ChatCompletion,
    ) {
        let url = format!("{}/api/public/ingestion", self.config.api_url);

        let batch = json!({
            "batch": [
                {
                    "type": "span-update",
                    "id": span_id,
                    "timestamp": chrono::Utc::now().to_rfc3339(),
                    "body": {
                        "output": output,
                        "metadata": {
                            "name": name,
                            "input": input
                        }
                    }
                }
            ]
        });

        let response = self
            .client
            .post(&url)
            .header("Authorization", self.get_auth_header())
            .json(&batch)
            .send()
            .await
            .expect("Failed to finalize span");

        println!("Response status: {:?}", response.status());
        println!("Response: {:?}", response.text().await.unwrap());
    }

    async fn finalize_trace(&self, trace_id: &str, input: &[Message], output: &[Message]) {
        let url = format!("{}/api/public/ingestion", self.config.api_url);

        let batch = json!({
            "batch": [
                {
                    "type": "trace-update",
                    "id": trace_id,
                    "timestamp": chrono::Utc::now().to_rfc3339(),
                    "body": {
                        "metadata": {
                            "input": input,
                            "output": output
                        }
                    }
                }
            ]
        });

        let response = self
            .client
            .post(&url)
            .header("Authorization", self.get_auth_header())
            .json(&batch)
            .send()
            .await
            .expect("Failed to finalize trace");

        println!("Response status: {:?}", response.status());
        println!("Response: {:?}", response.text().await.unwrap());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::openai::ChatCompletion;
    use crate::Message;

    #[tokio::test]
    async fn test_create_trace() {
        dotenv::dotenv().expect("Failed to load .env file");

        env::set_var(
            "LANGFUSE_PUBLIC_KEY",
            env::var("LANGFUSE_PUBLIC_KEY").expect("LANGFUSE_PUBLIC_KEY not set"),
        );
        env::set_var(
            "LANGFUSE_SECRET_KEY",
            env::var("LANGFUSE_SECRET_KEY").expect("LANGFUSE_SECRET_KEY not set"),
        );
        env::set_var(
            "LANGFUSE_HOST",
            env::var("LANGFUSE_HOST").expect("LANGFUSE_HOST not set"),
        );

        let config = LangfuseConfig::new();
        let langfuse_service = LangfuseServiceImpl::new(config);
        let trace_id = langfuse_service
            .create_trace(
                Uuid::new_v4(),
                "Main Completion",
                &vec![Message {
                    role: "user".to_string(),
                    content: "test_ups_message".to_string(),
                    name: None,
                }],
                &vec![Message {
                    role: "assistant".to_string(),
                    content: "test_message".to_string(),
                    name: None,
                }],
                "test_conversation_id",
            )
            .await;
        assert!(!trace_id.is_empty());
    }

    #[tokio::test]
    async fn test_create_span() {
        dotenv::dotenv().expect("Failed to load .env file");

        env::set_var(
            "LANGFUSE_PUBLIC_KEY",
            env::var("LANGFUSE_PUBLIC_KEY").expect("LANGFUSE_PUBLIC_KEY not set"),
        );
        env::set_var(
            "LANGFUSE_SECRET_KEY",
            env::var("LANGFUSE_SECRET_KEY").expect("LANGFUSE_SECRET_KEY not set"),
        );
        env::set_var(
            "LANGFUSE_HOST",
            env::var("LANGFUSE_HOST").expect("LANGFUSE_HOST not set"),
        );

        let config = LangfuseConfig::new();
        let langfuse_service = LangfuseServiceImpl::new(config);
        let trace_id = langfuse_service
            .create_trace(
                Uuid::new_v4(),
                "New Trace",
                &vec![Message {
                    role: "user".to_string(),
                    content: "test_ups_message".to_string(),
                    name: None,
                }],
                &vec![Message {
                    role: "assistant".to_string(),
                    content: "test_message".to_string(),
                    name: None,
                }],
                "test_conversation_id",
            )
            .await;
        let span_id = langfuse_service
            .create_span(
                &trace_id,
                "First Span",
                &vec![Message {
                    role: "user".to_string(),
                    content: "test_ups_message".to_string(),
                    name: None,
                }],
                &vec![Message {
                    role: "assistant".to_string(),
                    content: "test_message".to_string(),
                    name: None,
                }],
            )
            .await;

        let next_span_id = langfuse_service
            .create_span(
                &trace_id,
                "Second Span",
                &vec![Message {
                    role: "user".to_string(),
                    content: "test_ups_message".to_string(),
                    name: None,
                }],
                &vec![Message {
                    role: "assistant".to_string(),
                    content: "test_message".to_string(),
                    name: None,
                }],
            )
            .await;
        assert!(!span_id.is_empty());
    }
}
