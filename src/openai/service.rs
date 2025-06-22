use async_openai::{
    config::OpenAIConfig,
    types::{
        AudioInput, ChatCompletionRequestMessage, ChatCompletionRequestMessageContentPartImage,
        ChatCompletionRequestSystemMessage, ChatCompletionRequestSystemMessageContent,
        ChatCompletionRequestUserMessage, ChatCompletionRequestUserMessageContent,
        ChatCompletionRequestUserMessageContentPart, CreateChatCompletionRequest,
        CreateEmbeddingRequestArgs, CreateImageRequestArgs, CreateTranscriptionRequestArgs, Image,
        ImageResponseFormat, ImageSize, ImageUrl,
    },
    Client,
};
use async_trait::async_trait;

use crate::error::Error;
use crate::openai::types::{ChatCompletion, OpenAIImageMessage, OpenAIMessage};

use super::OpenAIModel;

#[async_trait]
pub trait AIService: Send + Sync {
    async fn completion(
        &self,
        messages: Vec<OpenAIMessage>,
        model: OpenAIModel,
    ) -> Result<ChatCompletion, Error>;

    async fn completion_image(
        &self,
        text_messages: Vec<OpenAIMessage>,
        vision_messages: Vec<OpenAIImageMessage>,
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
    pub fn new() -> Self {
        let api_key = std::env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY must be set");
        let config = OpenAIConfig::new().with_api_key(api_key);
        Self {
            client: Client::with_config(config),
        }
    }
}

#[async_trait]
impl AIService for OpenAIService {
    async fn completion(
        &self,
        messages: Vec<OpenAIMessage>,
        model: OpenAIModel,
    ) -> Result<ChatCompletion, Error> {
        let request_messages: Vec<ChatCompletionRequestMessage> = messages
            .iter()
            .map(|msg| match msg.role.as_str() {
                "system" => {
                    ChatCompletionRequestMessage::System(ChatCompletionRequestSystemMessage {
                        content: ChatCompletionRequestSystemMessageContent::Text(
                            msg.content.clone(),
                        ),
                        name: msg.name.clone(),
                    })
                }
                "user" => ChatCompletionRequestMessage::User(ChatCompletionRequestUserMessage {
                    content: ChatCompletionRequestUserMessageContent::Text(msg.content.clone()),
                    name: msg.name.clone(),
                }),
                _ => ChatCompletionRequestMessage::User(ChatCompletionRequestUserMessage {
                    content: ChatCompletionRequestUserMessageContent::Text(msg.content.clone()),
                    name: msg.name.clone(),
                }),
            })
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

        Ok(ChatCompletion {
            choices: response
                .choices
                .into_iter()
                .map(|choice| crate::openai::types::Choice {
                    message: OpenAIMessage {
                        role: choice.message.role.to_string(),
                        content: choice.message.content.unwrap_or_default(),
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
        })
    }

    async fn completion_image(
        &self,
        text_messages: Vec<OpenAIMessage>,
        vision_messages: Vec<OpenAIImageMessage>,
        model: OpenAIModel,
    ) -> Result<ChatCompletion, Error> {
        let text_messages: Vec<ChatCompletionRequestMessage> = text_messages
            .iter()
            .map(|msg| match msg.role.as_str() {
                "system" => {
                    ChatCompletionRequestMessage::System(ChatCompletionRequestSystemMessage {
                        content: ChatCompletionRequestSystemMessageContent::Text(
                            msg.content.clone(),
                        ),
                        name: msg.name.clone(),
                    })
                }
                "user" => ChatCompletionRequestMessage::User(ChatCompletionRequestUserMessage {
                    content: ChatCompletionRequestUserMessageContent::Text(msg.content.clone()),
                    name: msg.name.clone(),
                }),
                _ => ChatCompletionRequestMessage::User(ChatCompletionRequestUserMessage {
                    content: ChatCompletionRequestUserMessageContent::Text(msg.content.clone()),
                    name: msg.name.clone(),
                }),
            })
            .collect();

        let mut all_messages = text_messages;
        for vision_msg in vision_messages {
            let image_parts: Vec<ChatCompletionRequestUserMessageContentPart> = vision_msg
                .content
                .iter()
                .map(|img| {
                    ChatCompletionRequestUserMessageContentPart::ImageUrl(
                        ChatCompletionRequestMessageContentPartImage {
                            image_url: ImageUrl {
                                url: img.url.clone(),
                                detail: None,
                            },
                        },
                    )
                })
                .collect();

            all_messages.push(ChatCompletionRequestMessage::User(
                ChatCompletionRequestUserMessage {
                    content: ChatCompletionRequestUserMessageContent::Array(image_parts),
                    name: vision_msg.name.clone(),
                },
            ));
        }

        let request = CreateChatCompletionRequest {
            model: model.to_string(),
            messages: all_messages,
            ..Default::default()
        };

        println!(">>> Request: {:?}", request);
        let response = self
            .client
            .chat()
            .create(request)
            .await
            .map_err(|e| Error::OpenAI(e))?;

        Ok(ChatCompletion {
            choices: response
                .choices
                .into_iter()
                .map(|choice| crate::openai::types::Choice {
                    message: OpenAIMessage {
                        role: choice.message.role.to_string(),
                        content: choice.message.content.unwrap_or_default(),
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
        })
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
