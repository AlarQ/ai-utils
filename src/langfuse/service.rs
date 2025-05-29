use async_trait::async_trait;
use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use chrono;
use reqwest::Client;
use serde_json::json;
use uuid::Uuid;

use crate::error::Error;
use crate::langfuse::types::LangfuseConfig;
use crate::openai::{ChatCompletion, OpenAIMessage};

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
        input: &[OpenAIMessage],
        output: &[OpenAIMessage],
        conversation_id: &str,
    ) -> Result<String, Error>;

    async fn create_span(
        &self,
        trace_id: &str,
        name: &str,
        input: &[OpenAIMessage],
        output: &[OpenAIMessage],
    ) -> Result<String, Error>;

    async fn finalize_span(
        &self,
        span_id: &str,
        name: &str,
        input: &[OpenAIMessage],
        output: &ChatCompletion,
    ) -> Result<(), Error>;

    async fn finalize_trace(
        &self,
        trace_id: &str,
        input: &[OpenAIMessage],
        output: &[OpenAIMessage],
    ) -> Result<(), Error>;
}

#[async_trait]
impl LangfuseService for LangfuseServiceImpl {
    async fn create_trace(
        &self,
        trace_id: Uuid,
        name: &str,
        input: &[OpenAIMessage],
        output: &[OpenAIMessage],
        conversation_id: &str,
    ) -> Result<String, Error> {
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
            .await?;

        if !response.status().is_success() {
            return Err(Error::Langfuse(format!(
                "Failed to create trace: {}",
                response.text().await?
            )));
        }

        Ok(trace_id.to_string())
    }

    async fn create_span(
        &self,
        trace_id: &str,
        name: &str,
        input: &[OpenAIMessage],
        output: &[OpenAIMessage],
    ) -> Result<String, Error> {
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
            .await?;

        if !response.status().is_success() {
            return Err(Error::Langfuse(format!(
                "Failed to create span: {}",
                response.text().await?
            )));
        }

        Ok(span_id)
    }

    async fn finalize_span(
        &self,
        span_id: &str,
        name: &str,
        input: &[OpenAIMessage],
        output: &ChatCompletion,
    ) -> Result<(), Error> {
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
            .await?;

        if !response.status().is_success() {
            return Err(Error::Langfuse(format!(
                "Failed to finalize span: {}",
                response.text().await?
            )));
        }

        Ok(())
    }

    async fn finalize_trace(
        &self,
        trace_id: &str,
        input: &[OpenAIMessage],
        output: &[OpenAIMessage],
    ) -> Result<(), Error> {
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
            .await?;

        if !response.status().is_success() {
            return Err(Error::Langfuse(format!(
                "Failed to finalize trace: {}",
                response.text().await?
            )));
        }

        Ok(())
    }
}
