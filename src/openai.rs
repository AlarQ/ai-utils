use async_openai::{
    config::OpenAIConfig,
    types::{
        ChatCompletionRequestMessage, ChatCompletionRequestMessageContentPartImage, ChatCompletionRequestMessageContentPartText, ChatCompletionRequestSystemMessage, ChatCompletionRequestSystemMessageContent, ChatCompletionRequestUserMessage, ChatCompletionRequestUserMessageContent, ChatCompletionRequestUserMessageContentPart, CreateChatCompletionRequest, ImageUrl, Role
    },
    Client,
};
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

// TODO add Langfuse tracing
pub async fn completion(
    client: &Client<OpenAIConfig>,
    messages: &[OpenAIMessage],
    model: &str,
) -> Result<ChatCompletion, OpenAiError> {
    let request_messages: Vec<ChatCompletionRequestMessage> = messages
        .iter()
        .map(|msg| match msg.role.as_str() {
            "system" => ChatCompletionRequestMessage::System(ChatCompletionRequestSystemMessage {
                content: ChatCompletionRequestSystemMessageContent::Text(msg.content.clone()),
                name: msg.name.clone(),
            }),
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

    let response = client
        .chat()
        .create(request)
        .await
        .map_err(|e| OpenAiError::OpenAIError(e.to_string()))?;

    Ok(ChatCompletion {
        choices: response
            .choices
            .into_iter()
            .map(|choice| Choice {
                message: OpenAIMessage {
                    role: choice.message.role.to_string(),
                    content: choice.message.content.unwrap_or_default(),
                    name: None,
                },
            })
            .collect(),
        model: response.model,
        usage: response.usage.map(|usage| Usage {
            prompt_tokens: usage.prompt_tokens,
            completion_tokens: usage.completion_tokens,
            total_tokens: usage.total_tokens,
        }),
    })
}

pub async fn completion_vision(
    client: &Client<OpenAIConfig>,
    text_messages: &[OpenAIMessage],
    vision_messages: &[OpenAIImageMessage],
    model: &str,
) -> Result<ChatCompletion, OpenAiError> {
    let text_messages: Vec<ChatCompletionRequestMessage> = text_messages
        .iter()
        .map(|msg| match msg.role.as_str() {
            "system" => ChatCompletionRequestMessage::System(ChatCompletionRequestSystemMessage {
                content: ChatCompletionRequestSystemMessageContent::Text(msg.content.clone()),
                name: msg.name.clone(),
            }),
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

    let image_messages = ChatCompletionRequestMessage::User(ChatCompletionRequestUserMessage {
        content: ChatCompletionRequestUserMessageContent::Array(
            vision_messages.iter().map(|msg| ChatCompletionRequestUserMessageContentPart::ImageUrl(ChatCompletionRequestMessageContentPartImage {
                image_url: ImageUrl {
                    url: msg.content.url.clone(),
                    detail: Some(ImageDetail::High),
                }
            })).collect()
        ),
        name: None,
    });

    let request = CreateChatCompletionRequest {
        model: model.to_string(),
        messages: request_messages.extend(image_messages),
        ..Default::default()
    };

    let response = client
        .chat()
        .create(request)
        .await
        .map_err(|e| OpenAiError::OpenAIError(e.to_string()))?;

    Ok(ChatCompletion {
        choices: response
            .choices
            .into_iter()
            .map(|choice| Choice {
                message: OpenAIMessage {
                    role: choice.message.role.to_string(),
                    content: choice.message.content.unwrap_or_default(),
                    name: None,
                },
            })
            .collect(),
        model: response.model,
        usage: response.usage.map(|usage| Usage {
            prompt_tokens: usage.prompt_tokens,
            completion_tokens: usage.completion_tokens,
            total_tokens: usage.total_tokens,
        }),
    })
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

#[derive(Serialize, Deserialize, Clone)]
pub struct OpenAIImageMessage {
    pub role: String,
    pub content: Vec<OpenAIImageUrl>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}
