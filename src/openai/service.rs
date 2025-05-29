use async_openai::{
    config::OpenAIConfig,
    types::{
        ChatCompletionRequestMessage, ChatCompletionRequestMessageContentPartImage,
        ChatCompletionRequestSystemMessage, ChatCompletionRequestSystemMessageContent,
        ChatCompletionRequestUserMessage, ChatCompletionRequestUserMessageContent,
        ChatCompletionRequestUserMessageContentPart, CreateChatCompletionRequest, ImageUrl,
    },
    Client,
};
use async_trait::async_trait;

use crate::error::Error;
use crate::openai::types::{ChatCompletion, OpenAIImageMessage, OpenAIMessage};

#[async_trait]
pub trait OpenAIService: Send + Sync {
    async fn completion(
        &self,
        messages: &[OpenAIMessage],
        model: &str,
    ) -> Result<ChatCompletion, Error>;

    async fn completion_image(
        &self,
        text_messages: &[OpenAIMessage],
        vision_messages: &[OpenAIImageMessage],
        model: &str,
    ) -> Result<ChatCompletion, Error>;
}

pub struct OpenAIServiceImpl {
    client: Client<OpenAIConfig>,
}

impl OpenAIServiceImpl {
    pub fn new() -> Self {
        let api_key = std::env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY must be set");
        let config = OpenAIConfig::new().with_api_key(api_key);
        Self {
            client: Client::with_config(config),
        }
    }
}

#[async_trait]
impl OpenAIService for OpenAIServiceImpl {
    async fn completion(
        &self,
        messages: &[OpenAIMessage],
        model: &str,
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
        text_messages: &[OpenAIMessage],
        vision_messages: &[OpenAIImageMessage],
        model: &str,
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
}
