use async_openai::{
    config::OpenAIConfig,
    types::{
        chat::{
            ChatCompletionRequestMessage, ChatCompletionRequestMessageContentPartImage,
            ChatCompletionRequestMessageContentPartText, ChatCompletionRequestSystemMessage,
            ChatCompletionRequestSystemMessageContent, ChatCompletionRequestUserMessage,
            ChatCompletionRequestUserMessageContent, ChatCompletionRequestUserMessageContentPart,
            CreateChatCompletionRequest, ImageDetail, ImageUrl as OpenAIImageUrl, Role,
            StopConfiguration,
        },
        embeddings::{CreateEmbeddingRequest, EmbeddingInput},
    },
    Client,
};
use async_trait::async_trait;
use reqwest::header::{HeaderMap, HeaderValue};

use crate::error::Error;
use crate::openrouter::types::{
    ChatCompletion, ChatOptions, ContentPart, KeyInfo, Message, MessageContent, MessageRole,
    ModelInfo, ModelsResponse, OpenRouterErrorResponse,
};

const OPENROUTER_BASE_URL: &str = "https://openrouter.ai/api/v1";
const DEFAULT_EMBEDDING_MODEL: &str = "openai/text-embedding-3-large";

/// Trait for AI services that can perform chat completions and embeddings
#[async_trait]
pub trait AIService: Send + Sync {
    /// Perform a chat completion
    async fn chat(
        &self,
        messages: Vec<Message>,
        options: ChatOptions,
    ) -> crate::Result<ChatCompletion>;

    /// Generate embedding for a single text
    async fn embed(&self, text: String) -> crate::Result<Vec<f32>>;

    /// Generate embeddings for multiple texts
    async fn embed_batch(&self, texts: Vec<String>) -> crate::Result<Vec<Vec<f32>>>;

    /// List available models
    async fn list_models(&self) -> crate::Result<Vec<ModelInfo>>;

    /// Get key information
    async fn key_info(&self) -> crate::Result<KeyInfo>;
}

/// Service for interacting with the OpenRouter API
pub struct OpenRouterService {
    client: Client<OpenAIConfig>,
    http_client: reqwest::Client,
    api_base: String,
    api_key: String,
}

impl OpenRouterService {
    /// Create a new OpenRouterService from environment variables
    pub fn new() -> crate::Result<Self> {
        Self::from_env()
    }

    /// Create a new OpenRouterService from environment variables
    fn from_env() -> crate::Result<Self> {
        let api_key = std::env::var("OPENROUTER_API_KEY")
            .map_err(|_| Error::Config("OPENROUTER_API_KEY must be set".to_string()))?;

        // Validate API key format
        if api_key.trim().is_empty() {
            return Err(Error::Config(
                "OPENROUTER_API_KEY cannot be empty".to_string(),
            ));
        }

        let site_url = std::env::var("OPENROUTER_SITE_URL").ok();
        let site_name = std::env::var("OPENROUTER_SITE_NAME").ok();

        Self::with_config(api_key, site_url, site_name)
    }

    /// Create a new OpenRouterService with explicit configuration
    pub fn with_config(
        api_key: String,
        site_url: Option<String>,
        site_name: Option<String>,
    ) -> crate::Result<Self> {
        // Build default headers for the HTTP client
        let mut headers = HeaderMap::new();

        // Add required HTTP-Referer header for attribution
        if let Some(url) = site_url {
            if let Ok(header_value) = HeaderValue::from_str(&url) {
                headers.insert("HTTP-Referer", header_value);
            }
        }

        // Add X-Title header for site name
        if let Some(name) = site_name {
            if let Ok(header_value) = HeaderValue::from_str(&name) {
                headers.insert("X-Title", header_value);
            }
        }

        // Add Authorization header for authenticated endpoints (e.g. /auth/key)
        headers.insert(
            reqwest::header::AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {api_key}"))?,
        );

        // Build HTTP client with custom headers for OpenRouter-specific endpoints
        let http_client = reqwest::Client::builder()
            .default_headers(headers)
            .build()
            .map_err(Error::Request)?;

        // Build OpenAI config pointing to OpenRouter
        let config = OpenAIConfig::new()
            .with_api_key(api_key.clone())
            .with_api_base(OPENROUTER_BASE_URL);

        // Create async-openai client (uses its own reqwest client internally)
        // Note: Custom headers are applied to http_client used for OpenRouter-specific endpoints
        // The async-openai client uses OpenRouter-compatible requests via the base URL
        let client = Client::with_config(config);

        Ok(Self {
            client,
            http_client,
            api_base: OPENROUTER_BASE_URL.to_string(),
            api_key,
        })
    }

    /// Test the connection to OpenRouter API
    pub async fn test_connection(&self) -> crate::Result<()> {
        self.list_models().await.map(|_| ())
    }

    /// Convert our Message type to async-openai's ChatCompletionRequestMessage
    fn convert_message_to_openai(
        &self,
        message: &Message,
    ) -> crate::Result<ChatCompletionRequestMessage> {
        match (&message.role, &message.content) {
            (MessageRole::System, MessageContent::Text(text)) => Ok(
                ChatCompletionRequestMessage::System(ChatCompletionRequestSystemMessage {
                    content: ChatCompletionRequestSystemMessageContent::Text(text.clone()),
                    name: message.name.clone(),
                }),
            ),
            (MessageRole::User, MessageContent::Text(text)) => Ok(
                ChatCompletionRequestMessage::User(ChatCompletionRequestUserMessage {
                    content: ChatCompletionRequestUserMessageContent::Text(text.clone()),
                    name: message.name.clone(),
                }),
            ),
            (MessageRole::User, MessageContent::Image(images)) => {
                let image_parts: Vec<ChatCompletionRequestUserMessageContentPart> = images
                    .iter()
                    .map(|img| {
                        ChatCompletionRequestUserMessageContentPart::ImageUrl(
                            ChatCompletionRequestMessageContentPartImage {
                                image_url: OpenAIImageUrl {
                                    url: img.url.clone(),
                                    detail: img.detail.as_ref().map(|d| match d.as_str() {
                                        "high" => ImageDetail::High,
                                        "low" => ImageDetail::Low,
                                        _ => ImageDetail::Auto,
                                    }),
                                },
                            },
                        )
                    })
                    .collect();

                Ok(ChatCompletionRequestMessage::User(
                    ChatCompletionRequestUserMessage {
                        content: ChatCompletionRequestUserMessageContent::Array(image_parts),
                        name: message.name.clone(),
                    },
                ))
            }
            (MessageRole::User, MessageContent::Mixed(parts)) => {
                let content_parts: Vec<ChatCompletionRequestUserMessageContentPart> = parts
                    .iter()
                    .map(|part| match part {
                        ContentPart::Text { text } => {
                            ChatCompletionRequestUserMessageContentPart::Text(
                                ChatCompletionRequestMessageContentPartText { text: text.clone() },
                            )
                        }
                        ContentPart::Image { image_url: img } => {
                            ChatCompletionRequestUserMessageContentPart::ImageUrl(
                                ChatCompletionRequestMessageContentPartImage {
                                    image_url: OpenAIImageUrl {
                                        url: img.url.clone(),
                                        detail: img.detail.as_ref().map(|d| match d.as_str() {
                                            "high" => ImageDetail::High,
                                            "low" => ImageDetail::Low,
                                            _ => ImageDetail::Auto,
                                        }),
                                    },
                                },
                            )
                        }
                    })
                    .collect();

                Ok(ChatCompletionRequestMessage::User(
                    ChatCompletionRequestUserMessage {
                        content: ChatCompletionRequestUserMessageContent::Array(content_parts),
                        name: message.name.clone(),
                    },
                ))
            }
            (role, content) => Err(Error::Validation(format!(
                "Unsupported message role/content combination: {role:?} with {content:?}"
            ))),
        }
    }

    /// Convert async-openai response to our ChatCompletion type
    fn convert_response_to_chat_completion(
        &self,
        response: async_openai::types::chat::CreateChatCompletionResponse,
    ) -> ChatCompletion {
        ChatCompletion {
            choices: response
                .choices
                .into_iter()
                .map(|choice| crate::openrouter::types::Choice {
                    message: Message {
                        role: match choice.message.role {
                            Role::System => MessageRole::System,
                            Role::User => MessageRole::User,
                            Role::Assistant | Role::Tool | Role::Function => MessageRole::Assistant,
                        },
                        content: MessageContent::Text(choice.message.content.unwrap_or_default()),
                        name: None,
                    },
                })
                .collect(),
            model: response.model,
            usage: response.usage.map(|usage| crate::openrouter::types::Usage {
                prompt_tokens: usage.prompt_tokens,
                completion_tokens: usage.completion_tokens,
                total_tokens: usage.total_tokens,
            }),
        }
    }

    /// Handle OpenRouter-specific errors from HTTP responses
    async fn handle_error_response(&self, response: reqwest::Response) -> crate::Result<()> {
        if response.status().is_success() {
            return Ok(());
        }

        let status = response.status();
        let body = response.text().await.unwrap_or_else(|_| {
            format!("<failed to read response body: {}>", status)
        });

        // Try to parse OpenRouter error format
        if let Ok(error_response) = serde_json::from_str::<OpenRouterErrorResponse>(&body) {
            return Err(Error::Validation(format!(
                "OpenRouter error ({}): {}",
                error_response.error.error_type, error_response.error.message
            )));
        }

        Err(Error::Validation(format!(
            "OpenRouter HTTP error {}: {body}",
            status
        )))
    }
}

#[async_trait]
impl AIService for OpenRouterService {
    async fn chat(
        &self,
        messages: Vec<Message>,
        options: ChatOptions,
    ) -> crate::Result<ChatCompletion> {
        // Validate messages
        if messages.is_empty() {
            return Err(Error::MissingParameter {
                param: "messages".to_string(),
            });
        }

        // Validate each message
        for (i, message) in messages.iter().enumerate() {
            message
                .validate()
                .map_err(|e| Error::Validation(format!("Message {i}: {e}")))?;
        }

        // Convert messages to OpenAI format
        let request_messages: Vec<ChatCompletionRequestMessage> = messages
            .iter()
            .map(|msg| self.convert_message_to_openai(msg))
            .collect::<crate::Result<Vec<_>>>()?;

        // Build request
        let mut request = CreateChatCompletionRequest {
            model: options.model.to_string(),
            messages: request_messages,
            ..Default::default()
        };

        // Apply optional parameters
        if let Some(temp) = options.temperature {
            request.temperature = Some(temp);
        }
        if let Some(max_tokens) = options.max_tokens {
            request.max_completion_tokens = Some(max_tokens);
        }
        if let Some(top_p) = options.top_p {
            request.top_p = Some(top_p);
        }
        if let Some(stop) = options.stop {
            request.stop = Some(StopConfiguration::StringArray(stop));
        }
        // Note: The `user` field is deprecated in async-openai 0.33
        // OpenRouter will identify requests by API key instead

        // Apply OpenRouter-specific options if present
        if let Some(or_options) = options.openrouter {
            // Provider preferences can be set via extra headers or request body modifications
            // For now, we rely on the HTTP client headers set during construction
            // and the route/transform options would need custom request handling
            if or_options.route.is_some() || or_options.transforms.is_some() {
                // Note: OpenRouter-specific options like route and transforms
                // would require custom request serialization or header-based approach
                // This is a placeholder for future enhancement
            }
        }

        // Execute request
        let response = self
            .client
            .chat()
            .create(request)
            .await
            .map_err(Error::OpenRouter)?;

        Ok(self.convert_response_to_chat_completion(response))
    }

    async fn embed(&self, text: String) -> crate::Result<Vec<f32>> {
        // Validate text
        if text.trim().is_empty() {
            return Err(Error::Validation(
                "Text for embedding cannot be empty".to_string(),
            ));
        }

        let request = CreateEmbeddingRequest {
            model: DEFAULT_EMBEDDING_MODEL.to_string(),
            input: EmbeddingInput::String(text),
            dimensions: None,
            user: None,
            encoding_format: None,
        };

        let response = self
            .client
            .embeddings()
            .create(request)
            .await
            .map_err(Error::OpenRouter)?;

        // Extract embedding from first (and only) result
        response
            .data
            .into_iter()
            .next()
            .map(|data| data.embedding)
            .ok_or_else(|| Error::Validation("No embedding returned from API".to_string()))
    }

    async fn embed_batch(&self, texts: Vec<String>) -> crate::Result<Vec<Vec<f32>>> {
        // Validate texts
        if texts.is_empty() {
            return Err(Error::Validation(
                "Texts for batch embedding cannot be empty".to_string(),
            ));
        }

        // Validate each text is non-empty
        for (i, text) in texts.iter().enumerate() {
            if text.trim().is_empty() {
                return Err(Error::Validation(format!(
                    "Text at index {i} cannot be empty"
                )));
            }
        }

        let request = CreateEmbeddingRequest {
            model: DEFAULT_EMBEDDING_MODEL.to_string(),
            input: EmbeddingInput::StringArray(texts),
            dimensions: None,
            user: None,
            encoding_format: None,
        };

        let response = self
            .client
            .embeddings()
            .create(request)
            .await
            .map_err(Error::OpenRouter)?;

        Ok(response
            .data
            .into_iter()
            .map(|data| data.embedding)
            .collect())
    }

    async fn list_models(&self) -> crate::Result<Vec<ModelInfo>> {
        let url = format!("{}/models", self.api_base);
        let response = self
            .http_client
            .get(&url)
            .send()
            .await
            .map_err(Error::Request)?;

        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_else(|_| format!("<failed to read body: {}>", status));
            return Err(Error::Validation(format!(
                "OpenRouter API error ({}): {}",
                status, body
            )));
        }

        let models_response: ModelsResponse = response
            .json()
            .await
            .map_err(|e| Error::Validation(format!("Failed to parse models response: {}", e)))?;

        Ok(models_response.data)
    }

    async fn key_info(&self) -> crate::Result<KeyInfo> {
        let url = format!("{}/auth/key", self.api_base);

        let response = self
            .http_client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .send()
            .await
            .map_err(Error::Request)?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_else(|_| {
                format!("<failed to read response body: {}>", status)
            });

            // Try to parse OpenRouter error format
            if let Ok(error_response) = serde_json::from_str::<OpenRouterErrorResponse>(&body) {
                return Err(Error::Validation(format!(
                    "OpenRouter auth error: {}",
                    error_response.error.message
                )));
            }

            return Err(Error::Validation(format!(
                "OpenRouter auth HTTP error {status}: {body}"
            )));
        }

        let key_info: KeyInfo = response
            .json()
            .await
            .map_err(|e| Error::Validation(format!("Failed to parse key info response: {e}")))?;

        Ok(key_info)
    }
}

#[async_trait]
impl crate::qdrant::EmbeddingService for OpenRouterService {
    async fn embed(&self, text: String) -> crate::Result<Vec<f32>> {
        AIService::embed(self, text).await
    }

    async fn embed_batch(&self, texts: Vec<String>) -> crate::Result<Vec<Vec<f32>>> {
        AIService::embed_batch(self, texts).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::openrouter::types::{ChatRequestBuilder, ImageUrl, ModelId};

    #[test]
    fn test_model_id_constants() {
        assert_eq!(ModelId::GPT_4O, "openai/gpt-4o");
        assert_eq!(
            ModelId::CLAUDE_SONNET_4,
            "anthropic/claude-sonnet-4-20250514"
        );
    }

    #[test]
    fn test_model_id_custom() {
        let model = ModelId::custom("custom/model");
        assert_eq!(model.0, "custom/model");
    }

    #[test]
    fn test_message_creation() {
        let msg = Message::system("You are a helpful assistant");
        assert!(matches!(msg.role, MessageRole::System));
        assert!(matches!(msg.content, MessageContent::Text(_)));

        let msg = Message::user("Hello");
        assert!(matches!(msg.role, MessageRole::User));

        let msg = Message::assistant("Hi there");
        assert!(matches!(msg.role, MessageRole::Assistant));
    }

    #[test]
    fn test_message_with_images() {
        let images = vec![ImageUrl::from_url("http://example.com/image.png", None)];
        let msg = Message::with_images("Describe this", images);
        assert!(matches!(msg.content, MessageContent::Mixed(_)));
        assert!(msg.has_images());
    }

    #[test]
    fn test_message_validation() {
        let valid_msg = Message::user("Hello");
        assert!(valid_msg.validate().is_ok());

        let invalid_msg = Message {
            role: MessageRole::User,
            content: MessageContent::Text("".to_string()),
            name: None,
        };
        assert!(invalid_msg.validate().is_err());
    }

    #[test]
    fn test_chat_request_builder() {
        let (_, options) = ChatRequestBuilder::new(ModelId::GPT_4O)
            .message(Message::user("Hello"))
            .temperature(0.7)
            .max_tokens(100)
            .build();

        assert_eq!(options.model.0, "openai/gpt-4o");
        assert_eq!(options.temperature, Some(0.7));
        assert_eq!(options.max_tokens, Some(100));
    }

    #[tokio::test]
    async fn test_empty_messages_validation() {
        let service = match OpenRouterService::with_config("test-key".to_string(), None, None) {
            Ok(s) => s,
            Err(_) => {
                eprintln!("Skipping test: Failed to create OpenRouterService");
                return;
            }
        };

        let messages: Vec<Message> = vec![];
        let options = ChatOptions::default();

        let result = service.chat(messages, options).await;
        assert!(result.is_err());
        let err_msg = format!("{}", result.unwrap_err());
        assert!(err_msg.contains("messages") || err_msg.contains("Missing parameter"));
    }
}
