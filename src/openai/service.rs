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
use crate::openai::types::{ChatCompletion, Message, MessageContent, MessageRole, OpenAIModel};

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
        let config = OpenAIConfig::new().with_api_key(api_key);
        Ok(Self {
            client: Client::with_config(config),
        })
    }

    fn convert_message_to_openai(&self, message: &Message) -> ChatCompletionRequestMessage {
        match (&message.role, &message.content) {
            (MessageRole::System, MessageContent::Text(text)) => {
                ChatCompletionRequestMessage::System(ChatCompletionRequestSystemMessage {
                    content: ChatCompletionRequestSystemMessageContent::Text(text.clone()),
                    name: message.name.clone(),
                })
            }
            (MessageRole::User, MessageContent::Text(text)) => {
                ChatCompletionRequestMessage::User(ChatCompletionRequestUserMessage {
                    content: ChatCompletionRequestUserMessageContent::Text(text.clone()),
                    name: message.name.clone(),
                })
            }
            (MessageRole::Assistant, MessageContent::Text(text)) => {
                ChatCompletionRequestMessage::Assistant(
                    async_openai::types::ChatCompletionRequestAssistantMessage {
                        content: Some(
                            async_openai::types::ChatCompletionRequestAssistantMessageContent::Text(
                                text.clone(),
                            ),
                        ),
                        name: message.name.clone(),
                        tool_calls: None,
                        function_call: None,
                        audio: None,
                        refusal: None,
                    },
                )
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

                ChatCompletionRequestMessage::User(ChatCompletionRequestUserMessage {
                    content: ChatCompletionRequestUserMessageContent::Array(image_parts),
                    name: message.name.clone(),
                })
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

                ChatCompletionRequestMessage::User(ChatCompletionRequestUserMessage {
                    content: ChatCompletionRequestUserMessageContent::Array(content_parts),
                    name: message.name.clone(),
                })
            }
            _ => {
                // Fallback for unsupported combinations
                ChatCompletionRequestMessage::User(ChatCompletionRequestUserMessage {
                    content: ChatCompletionRequestUserMessageContent::Text(
                        "Unsupported message format".to_string(),
                    ),
                    name: message.name.clone(),
                })
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
                            async_openai::types::Role::Assistant => MessageRole::Assistant,
                            async_openai::types::Role::Tool => MessageRole::User, // fallback
                            async_openai::types::Role::Function => MessageRole::User, // fallback
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
}

#[async_trait]
impl AIService for OpenAIService {
    async fn completion(
        &self,
        messages: Vec<Message>,
        model: OpenAIModel,
    ) -> Result<ChatCompletion, Error> {
        let request_messages: Vec<ChatCompletionRequestMessage> = messages
            .iter()
            .map(|msg| self.convert_message_to_openai(msg))
            .collect();

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
            Image::B64Json { .. } => Err(Error::Other("Expected URL, got b64_json".to_string())),
        }
    }

    async fn transcribe(&self, audio: Vec<u8>) -> Result<String, Error> {
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
