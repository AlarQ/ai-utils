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
    pub fn new(url: String, detail: Option<String>) -> Self {
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
    #[serde(rename = "gpt-1")]
    Gpt1,
}

impl std::fmt::Display for OpenAIModel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OpenAIModel::Gpt4o => write!(f, "gpt-4o"),
            OpenAIModel::Gpt4oMini => write!(f, "gpt-4o-mini"),
            OpenAIModel::Gpt4oTranscribe => write!(f, "gpt-4o-transcribe"),
            OpenAIModel::Gpt1 => write!(f, "gpt-1"),
        }
    }
}
