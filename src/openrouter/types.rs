use serde::{Deserialize, Serialize};

/// Model identifier newtype with well-known model constants
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ModelId(pub String);

impl ModelId {
    // OpenAI models via OpenRouter
    pub const GPT_4O: &'static str = "openai/gpt-4o";
    pub const GPT_4O_MINI: &'static str = "openai/gpt-4o-mini";
    pub const GPT_4_5_PREVIEW: &'static str = "openai/gpt-4.5-preview";
    pub const O1: &'static str = "openai/o1";
    pub const O1_MINI: &'static str = "openai/o1-mini";
    pub const O3_MINI: &'static str = "openai/o3-mini";

    // Anthropic models
    pub const CLAUDE_SONNET_4: &'static str = "anthropic/claude-sonnet-4-20250514";
    pub const CLAUDE_OPUS_4: &'static str = "anthropic/claude-opus-4";
    pub const CLAUDE_SONNET_3_5: &'static str = "anthropic/claude-3.5-sonnet";
    pub const CLAUDE_HAIKU_3_5: &'static str = "anthropic/claude-3.5-haiku";

    // Google models
    pub const GEMINI_FLASH_2: &'static str = "google/gemini-flash-2.0";
    pub const GEMINI_PRO_2: &'static str = "google/gemini-pro-2";
    pub const GEMINI_FLASH_1_5: &'static str = "google/gemini-flash-1.5";

    // OpenRouter-specific
    pub const OR_LLAMA_3_3_70B: &'static str = "meta-llama/llama-3.3-70b-instruct";
    pub const OR_DEEPSEEK_V3: &'static str = "deepseek/deepseek-v3";
    pub const OR_MISTRAL_LARGE: &'static str = "mistral/mistral-large";

    /// Create a ModelId with a custom model identifier
    pub fn custom(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// Create from a well-known constant
    pub fn from_constant(constant: &'static str) -> Self {
        Self(constant.to_string())
    }
}

impl std::fmt::Display for ModelId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Default for ModelId {
    fn default() -> Self {
        Self::from_constant(Self::GPT_4O)
    }
}

/// Message role for chat completions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MessageRole {
    System,
    User,
    Assistant,
}

/// Content of a message - can be text, image(s), or mixed content
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MessageContent {
    Text(String),
    Image(Vec<ImageUrl>),
    Mixed(Vec<ContentPart>),
}

/// A part of mixed content (text or image)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ContentPart {
    Text { text: String },
    Image { image_url: ImageUrl },
}

/// Image URL with optional detail specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageUrl {
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

impl ImageUrl {
    /// Create an ImageUrl from base64 image data (PNG format)
    pub fn from_base64(base64_data: impl Into<String>, detail: Option<String>) -> Self {
        Self {
            url: format!("data:image/png;base64,{}", base64_data.into()),
            detail,
        }
    }

    /// Create an ImageUrl from a regular HTTP/HTTPS URL
    pub fn from_url(url: impl Into<String>, detail: Option<String>) -> Self {
        Self {
            url: url.into(),
            detail,
        }
    }

    /// Validate the URL format
    pub fn validate(&self) -> crate::Result<()> {
        if self.url.trim().is_empty() {
            return Err(crate::Error::Validation(
                "Image URL cannot be empty".to_string(),
            ));
        }

        // Basic URL validation
        if !self.url.starts_with("http") && !self.url.starts_with("data:") {
            return Err(crate::Error::Validation(
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

/// A message in a conversation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: MessageRole,
    pub content: MessageContent,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

impl Message {
    /// Create a system message
    pub fn system(content: impl Into<String>) -> Self {
        Self {
            role: MessageRole::System,
            content: MessageContent::Text(content.into()),
            name: None,
        }
    }

    /// Create a user message
    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: MessageRole::User,
            content: MessageContent::Text(content.into()),
            name: None,
        }
    }

    /// Create an assistant message
    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            role: MessageRole::Assistant,
            content: MessageContent::Text(content.into()),
            name: None,
        }
    }

    /// Create a user message with images
    pub fn with_images(text: impl Into<String>, images: Vec<ImageUrl>) -> Self {
        let mut parts = vec![ContentPart::Text { text: text.into() }];
        parts.extend(
            images
                .into_iter()
                .map(|img| ContentPart::Image { image_url: img }),
        );

        Self {
            role: MessageRole::User,
            content: MessageContent::Mixed(parts),
            name: None,
        }
    }

    /// Add a name to the message
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Validate the message content and structure
    pub fn validate(&self) -> crate::Result<()> {
        // Check for empty content
        match &self.content {
            MessageContent::Text(text) => {
                if text.trim().is_empty() {
                    return Err(crate::Error::Validation(
                        "Message content cannot be empty".to_string(),
                    ));
                }
            }
            MessageContent::Image(images) => {
                if images.is_empty() {
                    return Err(crate::Error::Validation(
                        "Image message must contain at least one image".to_string(),
                    ));
                }
                // Validate each image URL
                for (i, image) in images.iter().enumerate() {
                    image
                        .validate()
                        .map_err(|e| crate::Error::Validation(format!("Image {}: {e}", i)))?;
                }
            }
            MessageContent::Mixed(parts) => {
                if parts.is_empty() {
                    return Err(crate::Error::Validation(
                        "Mixed content message must contain at least one part".to_string(),
                    ));
                }
                // Validate each part
                for (i, part) in parts.iter().enumerate() {
                    match part {
                        ContentPart::Text { text } => {
                            if text.trim().is_empty() {
                                return Err(crate::Error::Validation(format!(
                                    "Mixed content text part {} cannot be empty",
                                    i
                                )));
                            }
                        }
                        ContentPart::Image { image_url: img } => {
                            img.validate().map_err(|e| {
                                crate::Error::Validation(format!(
                                    "Mixed content image part {}: {e}",
                                    i
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
                return Err(crate::Error::Validation(
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
                ContentPart::Text { text } => Some(text.as_str()),
                ContentPart::Image { .. } => None,
            }),
            MessageContent::Image(_) => None,
        }
    }
}

/// Provider preferences for routing requests
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProviderPreferences {
    /// Ordered list of provider IDs to try
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order: Option<Vec<String>>,
    /// Whether to allow fallbacks to other providers
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_fallbacks: Option<bool>,
    /// Require parameters to be supported
    #[serde(skip_serializing_if = "Option::is_none")]
    pub require_parameters: Option<bool>,
}

/// OpenRouter-specific options
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OpenRouterOptions {
    /// Provider routing preferences
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<ProviderPreferences>,
    /// Route to use (e.g., "fallback" for cheapest available)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub route: Option<String>,
    /// Transforms to apply (e.g., ["middle-out"])
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transforms: Option<Vec<String>>,
}

/// Options for chat completion requests
#[derive(Debug, Clone)]
pub struct ChatOptions {
    pub model: ModelId,
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
    pub top_p: Option<f32>,
    pub stop: Option<Vec<String>>,
    pub user: Option<String>,
    pub openrouter: Option<OpenRouterOptions>,
}

impl Default for ChatOptions {
    fn default() -> Self {
        Self {
            model: ModelId::default(),
            temperature: None,
            max_tokens: None,
            top_p: None,
            stop: None,
            user: None,
            openrouter: None,
        }
    }
}

/// Builder for chat requests
pub struct ChatRequestBuilder {
    messages: Vec<Message>,
    options: ChatOptions,
}

impl ChatRequestBuilder {
    /// Create a new builder with the specified model
    pub fn new(model: impl Into<ModelId>) -> Self {
        Self {
            messages: Vec::new(),
            options: ChatOptions {
                model: model.into(),
                ..Default::default()
            },
        }
    }

    /// Add a single message
    pub fn message(mut self, message: Message) -> Self {
        self.messages.push(message);
        self
    }

    /// Add multiple messages
    pub fn messages(mut self, messages: Vec<Message>) -> Self {
        self.messages = messages;
        self
    }

    /// Set temperature (0.0 to 2.0)
    pub fn temperature(mut self, temperature: f32) -> Self {
        self.options.temperature = Some(temperature);
        self
    }

    /// Set maximum tokens
    pub fn max_tokens(mut self, max_tokens: u32) -> Self {
        self.options.max_tokens = Some(max_tokens);
        self
    }

    /// Set top_p (nucleus sampling)
    pub fn top_p(mut self, top_p: f32) -> Self {
        self.options.top_p = Some(top_p);
        self
    }

    /// Set stop sequences
    pub fn stop(mut self, stop: Vec<String>) -> Self {
        self.options.stop = Some(stop);
        self
    }

    /// Set user identifier
    pub fn user(mut self, user: impl Into<String>) -> Self {
        self.options.user = Some(user.into());
        self
    }

    /// Set OpenRouter-specific options
    pub fn openrouter_options(mut self, options: OpenRouterOptions) -> Self {
        self.options.openrouter = Some(options);
        self
    }

    /// Build the request
    pub fn build(self) -> (Vec<Message>, ChatOptions) {
        (self.messages, self.options)
    }
}

impl From<&str> for ModelId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl From<String> for ModelId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

/// Chat completion response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatCompletion {
    pub choices: Vec<Choice>,
    pub model: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<Usage>,
}

/// A choice in the chat completion response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Choice {
    pub message: Message,
}

/// Token usage information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Usage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

/// Model information from /api/v1/models
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub pricing: ModelPricing,
}

/// Model pricing information (cost per 1K tokens)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelPricing {
    pub prompt: f64,
    pub completion: f64,
}

/// Response from /api/v1/auth/key endpoint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyInfo {
    pub data: KeyData,
}

/// Key data within KeyInfo
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyData {
    pub label: String,
    pub usage: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u64>,
    pub is_free_tier: bool,
}

/// OpenRouter error response structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenRouterErrorResponse {
    pub error: OpenRouterErrorDetail,
}

/// Error detail within OpenRouter error response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenRouterErrorDetail {
    pub message: String,
    #[serde(rename = "type")]
    pub error_type: String,
    pub code: Option<String>,
}

/// List of models response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelsResponse {
    pub data: Vec<ModelInfo>,
}
