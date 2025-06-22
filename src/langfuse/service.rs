use async_trait::async_trait;
use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use chrono;
use reqwest::Client;
use serde_json::json;
use uuid::Uuid;

use crate::error::Error;
use crate::langfuse::types::LangfuseConfig;
use crate::langfuse::types::{
    BaseEvent, GenerationCreateBody, GenerationUpdateBody, IngestionBatch, IngestionEvent,
    IngestionResponse, IngestionUsage, OpenAIUsage, SpanCreateBody, SpanUpdateBody, TraceBody,
};
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

    fn serialize_messages(messages: &[OpenAIMessage]) -> serde_json::Value {
        serde_json::to_value(messages).unwrap_or_else(|_| json!(messages))
    }

    fn create_base_event() -> BaseEvent {
        BaseEvent {
            id: Uuid::new_v4().to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            metadata: None,
        }
    }

    fn convert_usage(usage: &crate::openai::Usage) -> IngestionUsage {
        IngestionUsage::OpenAIUsage(OpenAIUsage {
            promptTokens: Some(usage.prompt_tokens),
            completionTokens: Some(usage.completion_tokens),
            totalTokens: Some(usage.total_tokens),
        })
    }

    pub async fn send_batch(&self, batch: IngestionBatch) -> Result<IngestionResponse, Error> {
        let url = format!("{}/api/public/ingestion", self.config.api_url);

        let response = self
            .client
            .post(&url)
            .header("Authorization", self.get_auth_header())
            .json(&batch)
            .send()
            .await?;

        let status = response.status();

        // Langfuse API returns 207 for batch operations with detailed success/error info
        if status == 207 {
            let ingestion_response: IngestionResponse = response.json().await?;

            // Check if there are any errors
            if !ingestion_response.errors.is_empty() {
                let error_messages: Vec<String> = ingestion_response
                    .errors
                    .iter()
                    .map(|e| {
                        format!(
                            "ID {}: {} (status: {})",
                            e.id,
                            e.message.as_deref().unwrap_or("Unknown error"),
                            e.status
                        )
                    })
                    .collect();
                return Err(Error::Langfuse(format!(
                    "Batch ingestion errors: {}",
                    error_messages.join(", ")
                )));
            }

            Ok(ingestion_response)
        } else if status.is_success() {
            // Handle other success status codes
            let ingestion_response: IngestionResponse = response.json().await?;
            Ok(ingestion_response)
        } else {
            let error_text = response.text().await?;
            Err(Error::Langfuse(format!("HTTP {}: {}", status, error_text)))
        }
    }
}

#[async_trait]
pub trait LangfuseService: Send + Sync {
    async fn create_trace(
        &self,
        trace_id: Uuid,
        name: &str,
        input: Option<&[OpenAIMessage]>,
        output: Option<&[OpenAIMessage]>,
        conversation_id: Option<&str>,
    ) -> Result<String, Error>;

    async fn create_generation(
        &self,
        trace_id: &str,
        name: &str,
        model: &str,
        input: &[OpenAIMessage],
    ) -> Result<String, Error>;

    async fn update_generation(
        &self,
        generation_id: &str,
        output: &ChatCompletion,
    ) -> Result<(), Error>;

    async fn create_span(
        &self,
        trace_id: &str,
        name: &str,
        input: Option<&[OpenAIMessage]>,
    ) -> Result<String, Error>;

    async fn update_span(&self, span_id: &str, output: &[OpenAIMessage]) -> Result<(), Error>;
}

#[async_trait]
impl LangfuseService for LangfuseServiceImpl {
    async fn create_trace(
        &self,
        trace_id: Uuid,
        name: &str,
        input: Option<&[OpenAIMessage]>,
        output: Option<&[OpenAIMessage]>,
        conversation_id: Option<&str>,
    ) -> Result<String, Error> {
        let mut metadata = serde_json::Map::new();
        if let Some(conv_id) = conversation_id {
            metadata.insert("conversation_id".to_string(), json!(conv_id));
        }

        let body = TraceBody {
            id: Some(trace_id.to_string()),
            timestamp: Some(chrono::Utc::now().to_rfc3339()),
            name: Some(name.to_string()),
            userId: None,
            input: input.map(Self::serialize_messages),
            output: output.map(Self::serialize_messages),
            sessionId: None,
            release: None,
            version: None,
            metadata: if metadata.is_empty() {
                None
            } else {
                Some(json!(metadata))
            },
            tags: None,
            environment: None,
            public: None,
        };

        let event = IngestionEvent::trace_create(Self::create_base_event(), body);

        let batch = IngestionBatch {
            batch: vec![event],
            metadata: None,
        };

        self.send_batch(batch).await?;
        Ok(trace_id.to_string())
    }

    async fn create_generation(
        &self,
        trace_id: &str,
        name: &str,
        model: &str,
        input: &[OpenAIMessage],
    ) -> Result<String, Error> {
        let generation_id = Uuid::new_v4().to_string();

        let span_body = SpanCreateBody {
            id: Some(generation_id.clone()),
            traceId: trace_id.to_string(),
            name: Some(name.to_string()),
            startTime: Some(chrono::Utc::now().to_rfc3339()),
            endTime: None,
            input: Some(Self::serialize_messages(input)),
            output: None, // Will be set on update
            metadata: None,
            level: None,
            statusMessage: None,
            parentObservationId: None,
            version: None,
            environment: None,
        };

        let body = GenerationCreateBody {
            span: span_body,
            completionStartTime: Some(chrono::Utc::now().to_rfc3339()),
            model: Some(model.to_string()),
            modelParameters: None,
            usage: None, // Will be set on update
            promptName: None,
            promptVersion: None,
        };

        let event = IngestionEvent::generation_create(Self::create_base_event(), body);

        let batch = IngestionBatch {
            batch: vec![event],
            metadata: None,
        };

        self.send_batch(batch).await?;
        Ok(generation_id)
    }

    async fn update_generation(
        &self,
        generation_id: &str,
        output: &ChatCompletion,
    ) -> Result<(), Error> {
        let span_body = SpanUpdateBody {
            id: generation_id.to_string(),
            endTime: Some(chrono::Utc::now().to_rfc3339()),
            input: None,
            output: Some(serde_json::to_value(output)?),
            metadata: None,
            level: None,
            statusMessage: None,
        };

        let body = GenerationUpdateBody {
            span: span_body,
            completionStartTime: None,
            model: None,
            modelParameters: None,
            usage: output.usage.as_ref().map(Self::convert_usage),
            promptName: None,
            promptVersion: None,
        };

        let event = IngestionEvent::generation_update(Self::create_base_event(), body);

        let batch = IngestionBatch {
            batch: vec![event],
            metadata: None,
        };

        self.send_batch(batch).await?;
        Ok(())
    }

    async fn create_span(
        &self,
        trace_id: &str,
        name: &str,
        input: Option<&[OpenAIMessage]>,
    ) -> Result<String, Error> {
        let span_id = Uuid::new_v4().to_string();

        let body = SpanCreateBody {
            id: Some(span_id.clone()),
            traceId: trace_id.to_string(),
            name: Some(name.to_string()),
            startTime: Some(chrono::Utc::now().to_rfc3339()),
            endTime: None,
            input: input.map(Self::serialize_messages),
            output: None, // Will be set on update
            metadata: None,
            level: None,
            statusMessage: None,
            parentObservationId: None,
            version: None,
            environment: None,
        };

        let event = IngestionEvent::span_create(Self::create_base_event(), body);

        let batch = IngestionBatch {
            batch: vec![event],
            metadata: None,
        };

        self.send_batch(batch).await?;
        Ok(span_id)
    }

    async fn update_span(&self, span_id: &str, output: &[OpenAIMessage]) -> Result<(), Error> {
        let body = SpanUpdateBody {
            id: span_id.to_string(),
            endTime: Some(chrono::Utc::now().to_rfc3339()),
            input: None,
            output: Some(Self::serialize_messages(output)),
            metadata: None,
            level: None,
            statusMessage: None,
        };

        let event = IngestionEvent::span_update(Self::create_base_event(), body);

        let batch = IngestionBatch {
            batch: vec![event],
            metadata: None,
        };

        self.send_batch(batch).await?;
        Ok(())
    }
}
