use crate::openai::OpenAIMessage;
use serde::{Deserialize, Serialize};
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

// Proper Langfuse API types based on the ingestion API specification

#[derive(Debug, Serialize)]
pub struct IngestionBatch {
    pub batch: Vec<IngestionEvent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
}

// Base event structure that all events extend
#[derive(Debug, Serialize)]
pub struct BaseEvent {
    pub id: String,
    pub timestamp: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
}

// IngestionEvent as a tagged union matching the spec
#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum IngestionEvent {
    TraceCreate {
        #[serde(rename = "type")]
        event_type: String,
        #[serde(flatten)]
        base: BaseEvent,
        body: TraceBody,
    },
    ScoreCreate {
        #[serde(rename = "type")]
        event_type: String,
        #[serde(flatten)]
        base: BaseEvent,
        body: ScoreBody,
    },
    SpanCreate {
        #[serde(rename = "type")]
        event_type: String,
        #[serde(flatten)]
        base: BaseEvent,
        body: SpanCreateBody,
    },
    SpanUpdate {
        #[serde(rename = "type")]
        event_type: String,
        #[serde(flatten)]
        base: BaseEvent,
        body: SpanUpdateBody,
    },
    GenerationCreate {
        #[serde(rename = "type")]
        event_type: String,
        #[serde(flatten)]
        base: BaseEvent,
        body: GenerationCreateBody,
    },
    GenerationUpdate {
        #[serde(rename = "type")]
        event_type: String,
        #[serde(flatten)]
        base: BaseEvent,
        body: GenerationUpdateBody,
    },
    EventCreate {
        #[serde(rename = "type")]
        event_type: String,
        #[serde(flatten)]
        base: BaseEvent,
        body: EventCreateBody,
    },
    SDKLog {
        #[serde(rename = "type")]
        event_type: String,
        #[serde(flatten)]
        base: BaseEvent,
        body: SDKLogBody,
    },
    ObservationCreate {
        #[serde(rename = "type")]
        event_type: String,
        #[serde(flatten)]
        base: BaseEvent,
        body: ObservationBody,
    },
    ObservationUpdate {
        #[serde(rename = "type")]
        event_type: String,
        #[serde(flatten)]
        base: BaseEvent,
        body: ObservationBody,
    },
}

// Helper functions to create properly typed events
impl IngestionEvent {
    pub fn trace_create(base: BaseEvent, body: TraceBody) -> Self {
        Self::TraceCreate {
            event_type: "trace-create".to_string(),
            base,
            body,
        }
    }

    pub fn score_create(base: BaseEvent, body: ScoreBody) -> Self {
        Self::ScoreCreate {
            event_type: "score-create".to_string(),
            base,
            body,
        }
    }

    pub fn span_create(base: BaseEvent, body: SpanCreateBody) -> Self {
        Self::SpanCreate {
            event_type: "span-create".to_string(),
            base,
            body,
        }
    }

    pub fn span_update(base: BaseEvent, body: SpanUpdateBody) -> Self {
        Self::SpanUpdate {
            event_type: "span-update".to_string(),
            base,
            body,
        }
    }

    pub fn generation_create(base: BaseEvent, body: GenerationCreateBody) -> Self {
        Self::GenerationCreate {
            event_type: "generation-create".to_string(),
            base,
            body,
        }
    }

    pub fn generation_update(base: BaseEvent, body: GenerationUpdateBody) -> Self {
        Self::GenerationUpdate {
            event_type: "generation-update".to_string(),
            base,
            body,
        }
    }

    pub fn event_create(base: BaseEvent, body: EventCreateBody) -> Self {
        Self::EventCreate {
            event_type: "event-create".to_string(),
            base,
            body,
        }
    }

    pub fn sdk_log(base: BaseEvent, body: SDKLogBody) -> Self {
        Self::SDKLog {
            event_type: "sdk-log".to_string(),
            base,
            body,
        }
    }

    pub fn observation_create(base: BaseEvent, body: ObservationBody) -> Self {
        Self::ObservationCreate {
            event_type: "observation-create".to_string(),
            base,
            body,
        }
    }

    pub fn observation_update(base: BaseEvent, body: ObservationBody) -> Self {
        Self::ObservationUpdate {
            event_type: "observation-update".to_string(),
            base,
            body,
        }
    }
}

#[derive(Debug, Serialize)]
#[allow(non_snake_case)]
pub struct TraceBody {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub userId: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sessionId: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub release: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub environment: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub public: Option<bool>,
}

#[derive(Debug, Serialize)]
#[allow(non_snake_case)]
pub struct ScoreBody {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub traceId: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sessionId: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub observationId: Option<String>,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub environment: Option<String>,
    pub value: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Serialize)]
#[allow(non_snake_case)]
pub struct SpanCreateBody {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub traceId: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub startTime: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub endTime: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub level: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub statusMessage: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parentObservationId: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub environment: Option<String>,
}

#[derive(Debug, Serialize)]
#[allow(non_snake_case)]
pub struct SpanUpdateBody {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub endTime: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub level: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub statusMessage: Option<String>,
}

#[derive(Debug, Serialize)]
#[allow(non_snake_case)]
pub struct GenerationCreateBody {
    #[serde(flatten)]
    pub span: SpanCreateBody,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completionStartTime: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modelParameters: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<IngestionUsage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub promptName: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub promptVersion: Option<i32>,
}

#[derive(Debug, Serialize)]
#[allow(non_snake_case)]
pub struct GenerationUpdateBody {
    #[serde(flatten)]
    pub span: SpanUpdateBody,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completionStartTime: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modelParameters: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<IngestionUsage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub promptName: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub promptVersion: Option<i32>,
}

#[derive(Debug, Serialize)]
pub struct EventCreateBody {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(flatten)]
    pub observation: OptionalObservationBody,
}

#[derive(Debug, Serialize)]
pub struct SDKLogBody {
    pub log: serde_json::Value,
}

#[derive(Debug, Serialize)]
#[allow(non_snake_case)]
pub struct ObservationBody {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub traceId: Option<String>,
    pub r#type: ObservationType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub startTime: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub endTime: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completionStartTime: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modelParameters: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<IngestionUsage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub level: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub statusMessage: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parentObservationId: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub environment: Option<String>,
}

#[derive(Debug, Serialize)]
#[allow(non_snake_case)]
pub struct OptionalObservationBody {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub traceId: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub startTime: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub level: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub statusMessage: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parentObservationId: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub environment: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum ObservationType {
    Span,
    Generation,
    Event,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum IngestionUsage {
    Usage(Usage),
    OpenAIUsage(OpenAIUsage),
}

#[derive(Debug, Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct Usage {
    pub promptTokens: Option<u32>,
    pub completionTokens: Option<u32>,
    pub totalTokens: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct OpenAIUsage {
    pub promptTokens: Option<u32>,
    pub completionTokens: Option<u32>,
    pub totalTokens: Option<u32>,
}

#[derive(Debug, Deserialize)]
pub struct IngestionResponse {
    pub successes: Vec<IngestionSuccess>,
    pub errors: Vec<IngestionError>,
}

#[derive(Debug, Deserialize)]
pub struct IngestionSuccess {
    pub id: String,
    pub status: u16,
}

#[derive(Debug, Deserialize)]
pub struct IngestionError {
    pub id: String,
    pub status: u16,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<serde_json::Value>,
}
