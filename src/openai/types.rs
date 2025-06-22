use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct OpenAIMessage {
    pub role: String,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

impl OpenAIMessage {
    pub fn new(role: &str, content: String, name: Option<String>) -> Self {
        Self {
            role: role.to_string(),
            content,
            name,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct ChatCompletion {
    pub choices: Vec<Choice>,
    pub model: String,
    pub usage: Option<Usage>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Choice {
    pub message: OpenAIMessage,
}

#[derive(Serialize, Deserialize)]
pub struct Usage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

#[derive(Debug)]
pub enum OpenAiError {
    OpenAIError(String),
    SerdeError(String),
    RequestError(String),
    ResponseError(String),
}

#[derive(Serialize, Deserialize, Clone)]
pub struct OpenAIImageUrl {
    pub url: String,
    pub detail: Option<String>, // e.g., "high"
}

impl OpenAIImageUrl {
    pub fn new(url: &str, detail: Option<String>) -> Self {
        Self {
            url: format!("data:image/png;base64,{}", url),
            detail,
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct OpenAIImageMessage {
    pub role: String,
    pub content: Vec<OpenAIImageUrl>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}
#[derive(Serialize, Deserialize, Clone)]
pub enum ImageType {
    #[serde(rename = "url")]
    Url,
    #[serde(rename = "b64_json")]
    B64Json,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct OpenAIImageGenMessage {
    pub prompt: String,
    pub n: u32,
    pub size: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub enum OpenAIModel {
    #[serde(rename = "gpt-4o")]
    Gpt4o,
    #[serde(rename = "gpt-4o-mini")]
    Gpt4oMini,
    #[serde(rename = "gpt-4o-transcribe")]
    Gpt4oTranscribe,
    #[serde(rename = "gpt-4.1")]
    Gpt41,
    #[serde(rename = "text-embedding-3-large")]
    TextEmbedding3Large,
    #[serde(rename = "ft:gpt-4.1-mini-2025-04-14:alarqai:my-validated-data:BgbHnf3n")]
    MyValidatedDataModel,
}

impl std::fmt::Display for OpenAIModel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OpenAIModel::Gpt4o => write!(f, "gpt-4o"),
            OpenAIModel::Gpt4oMini => write!(f, "gpt-4o-mini"),
            OpenAIModel::Gpt4oTranscribe => write!(f, "gpt-4o-transcribe"),
            OpenAIModel::Gpt41 => write!(f, "gpt-4.1"),
            OpenAIModel::TextEmbedding3Large => write!(f, "text-embedding-3-large"),
            OpenAIModel::MyValidatedDataModel => write!(f, "ft:gpt-4.1-mini-2025-04-14:alarqai:my-validated-data:BgbHnf3n"),
        }
    }
}
