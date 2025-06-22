use async_openai::{
    config::OpenAIConfig,
    types::{
        AudioInput, ChatCompletionRequestMessage, ChatCompletionRequestMessageContentPartImage,
        ChatCompletionRequestSystemMessage, ChatCompletionRequestSystemMessageContent,
        ChatCompletionRequestUserMessage, ChatCompletionRequestUserMessageContent,
        ChatCompletionRequestUserMessageContentPart, CreateChatCompletionRequest,
        CreateEmbeddingRequestArgs, CreateImageRequestArgs, CreateTranscriptionRequestArgs, Image,
        ImageResponseFormat, ImageSize, ImageUrl as OpenAIImageUrl,
    },
    Client,
};
use async_trait::async_trait;

use crate::error::Error;
use crate::openai::types::{
    ChatCompletion, ChatOptions, Message, MessageContent, MessageRole, OpenAIModel,
};

#[async_trait]
pub trait AIService: Send + Sync {
    async fn completion(
        &self,
        messages: Vec<Message>,
        model: OpenAIModel,
    ) -> Result<ChatCompletion, Error>;

    async fn generate_image_url(&self, prompt: String) -> Result<String, Error>;

    async fn transcribe(&self, audio: Vec<u8>) -> Result<String, Error>;

    async fn embed(&self, text: String) -> Result<Vec<f32>, Error>;
}

pub struct OpenAIService {
    client: Client<OpenAIConfig>,
}

impl OpenAIService {
    pub fn new() -> Result<Self, Error> {
        let api_key = std::env::var("OPENAI_API_KEY")
            .map_err(|_| Error::Config("OPENAI_API_KEY must be set".to_string()))?;

        // Validate API key format
        if api_key.trim().is_empty() {
            return Err(Error::Config("OPENAI_API_KEY cannot be empty".to_string()));
        }

        if !api_key.starts_with("sk-") {
            return Err(Error::Config(
                "OPENAI_API_KEY must start with 'sk-'".to_string(),
            ));
        }

        let config = OpenAIConfig::new().with_api_key(api_key);
        Ok(Self {
            client: Client::with_config(config),
        })
    }

    /// Validate the service configuration
    pub fn validate_config(&self) -> Result<(), Error> {
        // This could be extended to test the connection or validate other config
        Ok(())
    }

    /// Test the connection to OpenAI API
    pub async fn test_connection(&self) -> Result<(), Error> {
        // Simple test by trying to list models
        self.client
            .models()
            .list()
            .await
            .map_err(|e| Error::OpenAI(e))?;

        Ok(())
    }

    fn convert_message_to_openai(
        &self,
        message: &Message,
    ) -> Result<ChatCompletionRequestMessage, Error> {
        match (&message.role, &message.content) {
            (MessageRole::System, MessageContent::Text(text)) => {
                Ok(ChatCompletionRequestMessage::System(ChatCompletionRequestSystemMessage {
                    content: ChatCompletionRequestSystemMessageContent::Text(text.clone()),
                    name: message.name.clone(),
                }))
            }
            (MessageRole::User, MessageContent::Text(text)) => {
                Ok(ChatCompletionRequestMessage::User(ChatCompletionRequestUserMessage {
                    content: ChatCompletionRequestUserMessageContent::Text(text.clone()),
                    name: message.name.clone(),
                }))
            }
            (MessageRole::User, MessageContent::Image(images)) => {
                let image_parts: Vec<ChatCompletionRequestUserMessageContentPart> = images
                    .iter()
                    .map(|img| {
                        ChatCompletionRequestUserMessageContentPart::ImageUrl(
                            ChatCompletionRequestMessageContentPartImage {
                                image_url: OpenAIImageUrl {
                                    url: img.url.clone(),
                                    detail: img.detail.as_ref().map(|d| match d.as_str() {
                                        "high" => async_openai::types::ImageDetail::High,
                                        "low" => async_openai::types::ImageDetail::Low,
                                        _ => async_openai::types::ImageDetail::Auto,
                                    }),
                                },
                            },
                        )
                    })
                    .collect();

                Ok(ChatCompletionRequestMessage::User(ChatCompletionRequestUserMessage {
                    content: ChatCompletionRequestUserMessageContent::Array(image_parts),
                    name: message.name.clone(),
                }))
            }
            (MessageRole::User, MessageContent::Mixed(parts)) => {
                let content_parts: Vec<ChatCompletionRequestUserMessageContentPart> = parts
                    .iter()
                    .map(|part| match part {
                        crate::openai::types::ContentPart::Text(text) => {
                            ChatCompletionRequestUserMessageContentPart::Text(
                                async_openai::types::ChatCompletionRequestMessageContentPartText {
                                    text: text.clone(),
                                },
                            )
                        }
                        crate::openai::types::ContentPart::Image(img) => {
                            ChatCompletionRequestUserMessageContentPart::ImageUrl(
                                ChatCompletionRequestMessageContentPartImage {
                                    image_url: OpenAIImageUrl {
                                        url: img.url.clone(),
                                        detail: img.detail.as_ref().map(|d| match d.as_str() {
                                            "high" => async_openai::types::ImageDetail::High,
                                            "low" => async_openai::types::ImageDetail::Low,
                                            _ => async_openai::types::ImageDetail::Auto,
                                        }),
                                    },
                                },
                            )
                        }
                    })
                    .collect();

                Ok(ChatCompletionRequestMessage::User(ChatCompletionRequestUserMessage {
                    content: ChatCompletionRequestUserMessageContent::Array(content_parts),
                    name: message.name.clone(),
                }))
            }
            (role, content) => {
                Err(Error::OpenAIValidation(format!(
                    "Unsupported message role/content combination: {:?} with {:?}. Only User and System roles are supported.",
                    role, content
                )))
            }
        }
    }

    fn convert_response_to_chat_completion(
        &self,
        response: async_openai::types::CreateChatCompletionResponse,
    ) -> ChatCompletion {
        ChatCompletion {
            choices: response
                .choices
                .into_iter()
                .map(|choice| crate::openai::types::Choice {
                    message: Message {
                        role: match choice.message.role {
                            async_openai::types::Role::System => MessageRole::System,
                            async_openai::types::Role::User => MessageRole::User,
                            async_openai::types::Role::Tool => MessageRole::User, // fallback
                            async_openai::types::Role::Function => MessageRole::User, // fallback
                            _ => MessageRole::User, // fallback for any other roles
                        },
                        content: MessageContent::Text(choice.message.content.unwrap_or_default()),
                        name: None,
                    },
                })
                .collect(),
            model: response.model,
            usage: response.usage.map(|usage| crate::openai::types::Usage {
                prompt_tokens: usage.prompt_tokens,
                completion_tokens: usage.completion_tokens,
                total_tokens: usage.total_tokens,
            }),
        }
    }

    /// Unified chat completion API using builder/options pattern
    pub async fn chat(
        &self,
        messages: Vec<Message>,
        options: ChatOptions,
    ) -> Result<ChatCompletion, Error> {
        // Validate model supports chat
        options.model.validate_operation("chat")?;

        // Validate messages
        if messages.is_empty() {
            return Err(Error::OpenAIMissingParameter {
                param: "messages".to_string(),
            });
        }

        for (i, message) in messages.iter().enumerate() {
            message
                .validate()
                .map_err(|e| Error::OpenAIValidation(format!("Message {}: {}", i, e)))?;
        }

        let has_images = messages.iter().any(|msg| msg.has_images());
        if has_images {
            options.model.validate_operation("vision")?;
        }

        let request_messages: Vec<ChatCompletionRequestMessage> = messages
            .iter()
            .map(|msg| self.convert_message_to_openai(msg))
            .collect::<Result<Vec<_>, _>>()?;

        let mut request = CreateChatCompletionRequest {
            model: options.model.to_string(),
            messages: request_messages,
            ..Default::default()
        };

        if let Some(temp) = options.temperature {
            request.temperature = Some(temp);
        }
        if let Some(max_tokens) = options.max_tokens {
            request.max_tokens = Some(max_tokens);
        }
        if let Some(top_p) = options.top_p {
            request.top_p = Some(top_p);
        }
        if let Some(stop) = options.stop {
            request.stop = Some(async_openai::types::Stop::StringArray(stop));
        }
        if let Some(user) = options.user {
            request.user = Some(user);
        }

        let response = self
            .client
            .chat()
            .create(request)
            .await
            .map_err(|e| Error::OpenAI(e))?;

        Ok(self.convert_response_to_chat_completion(response))
    }

    /// Deprecated: use chat() with builder/options instead
    #[deprecated(note = "Use chat() with builder/options instead")]
    pub async fn completion(
        &self,
        messages: Vec<Message>,
        model: OpenAIModel,
    ) -> Result<ChatCompletion, Error> {
        self.chat(
            messages,
            ChatOptions {
                model,
                ..Default::default()
            },
        )
        .await
    }
}

#[async_trait]
impl AIService for OpenAIService {
    async fn completion(
        &self,
        messages: Vec<Message>,
        model: OpenAIModel,
    ) -> Result<ChatCompletion, Error> {
        // Validate model supports chat
        model.validate_operation("chat")?;

        // Validate messages
        if messages.is_empty() {
            return Err(Error::OpenAIMissingParameter {
                param: "messages".to_string(),
            });
        }

        // Validate each message
        for (i, message) in messages.iter().enumerate() {
            message
                .validate()
                .map_err(|e| Error::OpenAIValidation(format!("Message {}: {}", i, e)))?;
        }

        // Check for vision requirements
        let has_images = messages.iter().any(|msg| msg.has_images());
        if has_images {
            model.validate_operation("vision")?;
        }

        let request_messages: Vec<ChatCompletionRequestMessage> = messages
            .iter()
            .map(|msg| self.convert_message_to_openai(msg))
            .collect::<Result<Vec<_>, _>>()?;

        let request = CreateChatCompletionRequest {
            model: model.to_string(),
            messages: request_messages,
            ..Default::default()
        };

        let response = self
            .client
            .chat()
            .create(request)
            .await
            .map_err(|e| Error::OpenAI(e))?;

        Ok(self.convert_response_to_chat_completion(response))
    }

    async fn generate_image_url(&self, prompt: String) -> Result<String, Error> {
        // Validate prompt
        if prompt.trim().is_empty() {
            return Err(Error::OpenAIValidation(
                "Image generation prompt cannot be empty".to_string(),
            ));
        }

        let request = CreateImageRequestArgs::default()
            .prompt(prompt)
            .n(1)
            .response_format(ImageResponseFormat::Url)
            .size(ImageSize::S1024x1024)
            .user("async-openai")
            .build()?;

        let response = self
            .client
            .images()
            .create(request)
            .await
            .map_err(|e| Error::OpenAI(e))?;

        let image = &response.data[0];
        match &**image {
            Image::Url { url, .. } => Ok(url.clone()),
            Image::B64Json { .. } => Err(Error::OpenAIValidation(
                "Expected URL response format, got b64_json".to_string(),
            )),
        }
    }

    async fn transcribe(&self, audio: Vec<u8>) -> Result<String, Error> {
        // Validate audio data
        if audio.is_empty() {
            return Err(Error::OpenAIValidation(
                "Audio data cannot be empty".to_string(),
            ));
        }

        let request: async_openai::types::CreateTranscriptionRequest =
            CreateTranscriptionRequestArgs::default()
                .file(AudioInput::from_vec_u8("audio.mp3".to_string(), audio))
                .model(OpenAIModel::Gpt4oTranscribe.to_string())
                .build()?;

        let response = self
            .client
            .audio()
            .transcribe(request)
            .await
            .map_err(|e| Error::OpenAI(e))?;

        Ok(response.text)
    }

    async fn embed(&self, text: String) -> Result<Vec<f32>, Error> {
        // Validate text
        if text.trim().is_empty() {
            return Err(Error::OpenAIValidation(
                "Text for embedding cannot be empty".to_string(),
            ));
        }

        let request = CreateEmbeddingRequestArgs::default()
            .model(OpenAIModel::TextEmbedding3Large.to_string())
            .input(text)
            .build()?;

        let response = self
            .client
            .embeddings()
            .create(request)
            .await
            .map_err(|e| Error::OpenAI(e))?;

        Ok(response.data[0].embedding.clone())
    }
}
