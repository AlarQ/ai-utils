use async_openai::{
    config::OpenAIConfig,
    types::{
        ChatCompletionRequestMessage, ChatCompletionRequestMessageContentPartText,
        ChatCompletionRequestSystemMessage, ChatCompletionRequestSystemMessageContent,
        ChatCompletionRequestUserMessage, ChatCompletionRequestUserMessageContent,
        CreateChatCompletionRequest, Role,
    },
    Client,
};
use serde::{Deserialize, Serialize};

use crate::Message;

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

pub async fn completion(
    client: &Client<OpenAIConfig>,
    messages: &[Message],
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

    let response = client.chat().create(request).await
    .map_err(|e| OpenAiError::OpenAIError(e.to_string()))?;

    Ok(ChatCompletion {
        choices: response
            .choices
            .into_iter()
            .map(|choice| Choice {
                message: Message {
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

pub enum OpenAiError {
    OpenAIError(String),
    SerdeError(String),
    RequestError(String),
    ResponseError(String),
}
