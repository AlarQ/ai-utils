use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MessageRole {
    System,
    User,
    Assistant,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageContent {
    Text(String),
    Image(Vec<ImageUrl>),
    Mixed(Vec<ContentPart>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ContentPart {
    Text(String),
    Image(ImageUrl),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: MessageRole,
    pub content: MessageContent,
    pub name: Option<String>,
}

impl Message {
    pub fn system(content: impl Into<String>) -> Self {
        Self {
            role: MessageRole::System,
            content: MessageContent::Text(content.into()),
            name: None,
        }
    }

    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: MessageRole::User,
            content: MessageContent::Text(content.into()),
            name: None,
        }
    }

    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            role: MessageRole::Assistant,
            content: MessageContent::Text(content.into()),
            name: None,
        }
    }

    pub fn with_images(content: impl Into<String>, images: Vec<ImageUrl>) -> Self {
        let mut parts = vec![ContentPart::Text(content.into())];
        parts.extend(images.into_iter().map(ContentPart::Image));

        Self {
            role: MessageRole::User,
            content: MessageContent::Mixed(parts),
            name: None,
        }
    }

    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Validate the message content and structure
    pub fn validate(&self) -> Result<(), crate::error::Error> {
        // Check for empty content
        match &self.content {
            MessageContent::Text(text) => {
                if text.trim().is_empty() {
                    return Err(crate::error::Error::OpenAIValidation(
                        "Message content cannot be empty".to_string(),
                    ));
                }
            }
            MessageContent::Image(images) => {
                if images.is_empty() {
                    return Err(crate::error::Error::OpenAIValidation(
                        "Image message must contain at least one image".to_string(),
                    ));
                }
                // Validate each image URL
                for (i, image) in images.iter().enumerate() {
                    image.validate().map_err(|e| {
                        crate::error::Error::OpenAIValidation(format!("Image {}: {}", i, e))
                    })?;
                }
            }
            MessageContent::Mixed(parts) => {
                if parts.is_empty() {
                    return Err(crate::error::Error::OpenAIValidation(
                        "Mixed content message must contain at least one part".to_string(),
                    ));
                }
                // Validate each part
                for (i, part) in parts.iter().enumerate() {
                    match part {
                        ContentPart::Text(text) => {
                            if text.trim().is_empty() {
                                return Err(crate::error::Error::OpenAIValidation(format!(
                                    "Mixed content text part {} cannot be empty",
                                    i
                                )));
                            }
                        }
                        ContentPart::Image(img) => {
                            img.validate().map_err(|e| {
                                crate::error::Error::OpenAIValidation(format!(
                                    "Mixed content image part {}: {}",
                                    i, e
                                ))
                            })?;
                        }
                    }
                }
            }
        }

        // Validate name if present
        if let Some(name) = &self.name {
            if name.trim().is_empty() {
                return Err(crate::error::Error::OpenAIValidation(
                    "Message name cannot be empty".to_string(),
                ));
            }
        }

        Ok(())
    }

    /// Check if the message contains images
    pub fn has_images(&self) -> bool {
        matches!(
            self.content,
            MessageContent::Image(_) | MessageContent::Mixed(_)
        )
    }

    /// Get the text content if available
    pub fn text_content(&self) -> Option<&str> {
        match &self.content {
            MessageContent::Text(text) => Some(text),
            MessageContent::Mixed(parts) => parts.iter().find_map(|part| match part {
                ContentPart::Text(text) => Some(text.as_str()),
                ContentPart::Image(_) => None,
            }),
            MessageContent::Image(_) => None,
        }
    }
}

// Legacy types for backward compatibility
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
    pub message: Message,
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

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ImageUrl {
    pub url: String,
    pub detail: Option<String>, // e.g., "high"
}

impl ImageUrl {
    pub fn new(url: &str, detail: Option<String>) -> Self {
        Self {
            url: format!("data:image/png;base64,{}", url),
            detail,
        }
    }

    /// Create an ImageUrl from a regular URL
    pub fn from_url(url: &str, detail: Option<String>) -> Self {
        Self {
            url: url.to_string(),
            detail,
        }
    }

    /// Create an ImageUrl from base64 data
    pub fn from_base64(base64_data: &str, detail: Option<String>) -> Self {
        Self {
            url: format!("data:image/png;base64,{}", base64_data),
            detail,
        }
    }

    /// Validate the URL format
    pub fn validate(&self) -> Result<(), crate::error::Error> {
        if self.url.trim().is_empty() {
            return Err(crate::error::Error::OpenAIValidation(
                "Image URL cannot be empty".to_string(),
            ));
        }

        // Basic URL validation
        if !self.url.starts_with("http") && !self.url.starts_with("data:") {
            return Err(crate::error::Error::OpenAIValidation(
                "Image URL must be a valid HTTP URL or data URI".to_string(),
            ));
        }

        Ok(())
    }

    /// Check if the URL is a data URI
    pub fn is_data_uri(&self) -> bool {
        self.url.starts_with("data:")
    }

    /// Check if the URL is an HTTP URL
    pub fn is_http_url(&self) -> bool {
        self.url.starts_with("http")
    }
}

// Legacy type for backward compatibility
#[derive(Serialize, Deserialize, Clone)]
pub struct OpenAIImageMessage {
    pub role: String,
    pub content: Vec<ImageUrl>,
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
    Custom(String),
}

impl std::fmt::Display for OpenAIModel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OpenAIModel::Gpt4o => write!(f, "gpt-4o"),
            OpenAIModel::Gpt4oMini => write!(f, "gpt-4o-mini"),
            OpenAIModel::Gpt4oTranscribe => write!(f, "gpt-4o-transcribe"),
            OpenAIModel::Gpt41 => write!(f, "gpt-4.1"),
            OpenAIModel::TextEmbedding3Large => write!(f, "text-embedding-3-large"),
            OpenAIModel::Custom(model) => write!(f, "{}", model),
        }
    }
}

impl OpenAIModel {
    /// Check if the model supports chat completions
    pub fn supports_chat(&self) -> bool {
        matches!(
            self,
            OpenAIModel::Gpt4o
                | OpenAIModel::Gpt4oMini
                | OpenAIModel::Gpt41
                | OpenAIModel::Custom(_)
        )
    }

    /// Check if the model supports vision (image analysis)
    pub fn supports_vision(&self) -> bool {
        matches!(self, OpenAIModel::Gpt4o | OpenAIModel::Custom(_))
    }

    /// Check if the model supports audio transcription
    pub fn supports_transcription(&self) -> bool {
        matches!(self, OpenAIModel::Gpt4oTranscribe)
    }

    /// Check if the model supports embeddings
    pub fn supports_embeddings(&self) -> bool {
        matches!(
            self,
            OpenAIModel::TextEmbedding3Large | OpenAIModel::Custom(_)
        )
    }

    /// Get the maximum tokens for the model
    pub fn max_tokens(&self) -> Option<u32> {
        match self {
            OpenAIModel::Gpt4o => Some(128000),
            OpenAIModel::Gpt4oMini => Some(128000),
            OpenAIModel::Gpt41 => Some(128000),
            OpenAIModel::Gpt4oTranscribe => None,
            OpenAIModel::TextEmbedding3Large => None,
            OpenAIModel::Custom(_) => None, // Unknown for custom models
        }
    }

    /// Validate that the model supports the given operation
    pub fn validate_operation(&self, operation: &str) -> Result<(), crate::error::Error> {
        let supported = match operation {
            "chat" => self.supports_chat(),
            "vision" => self.supports_vision(),
            "transcription" => self.supports_transcription(),
            "embeddings" => self.supports_embeddings(),
            _ => false,
        };

        if !supported {
            return Err(crate::error::Error::OpenAIUnsupportedModel {
                model: self.to_string(),
                operation: operation.to_string(),
            });
        }

        Ok(())
    }
}
