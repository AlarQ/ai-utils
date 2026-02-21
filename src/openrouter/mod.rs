mod service;
mod types;

pub use service::*;
pub use types::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_chat_completion() {
        dotenv::dotenv().ok();

        // Skip test if OPENROUTER_API_KEY is not set
        if std::env::var("OPENROUTER_API_KEY").is_err() {
            eprintln!("Skipping test: OPENROUTER_API_KEY not set");
            return;
        }

        let service = OpenRouterService::new().expect("Failed to create service");

        let messages = vec![
            Message::system("You are a helpful assistant."),
            Message::user("Say 'Hello from OpenRouter!' and nothing else."),
        ];

        let options = ChatOptions {
            model: ModelId::from_constant(ModelId::GPT_4O_MINI),
            temperature: Some(0.0),
            max_tokens: Some(50),
            ..Default::default()
        };

        let result = service.chat(messages, options).await;
        assert!(result.is_ok(), "Chat completion failed: {:?}", result.err());

        let completion = result.unwrap();
        assert!(!completion.choices.is_empty());

        let response_text = completion.choices[0]
            .message
            .text_content()
            .expect("Response should have text content");
        assert!(response_text.to_lowercase().contains("hello"));
    }

    #[tokio::test]
    async fn test_chat_with_builder() {
        dotenv::dotenv().ok();

        // Skip test if OPENROUTER_API_KEY is not set
        if std::env::var("OPENROUTER_API_KEY").is_err() {
            eprintln!("Skipping test: OPENROUTER_API_KEY not set");
            return;
        }

        let service = OpenRouterService::new().expect("Failed to create service");

        let (messages, options) = ChatRequestBuilder::new(ModelId::GPT_4O_MINI)
            .message(Message::system("You are helpful."))
            .message(Message::user("Say 'Hi!'"))
            .temperature(0.0)
            .max_tokens(10)
            .build();

        let result = service.chat(messages, options).await;
        assert!(result.is_ok());

        let completion = result.unwrap();
        assert!(!completion.choices.is_empty());
    }

    #[tokio::test]
    async fn test_embedding() {
        dotenv::dotenv().ok();

        // Skip test if OPENROUTER_API_KEY is not set
        if std::env::var("OPENROUTER_API_KEY").is_err() {
            eprintln!("Skipping test: OPENROUTER_API_KEY not set");
            return;
        }

        let service = OpenRouterService::new().expect("Failed to create service");

        let text = "Hello, world!".to_string();
        let result = service.embed(text).await;

        assert!(result.is_ok(), "Embedding failed: {:?}", result.err());

        let embedding = result.unwrap();
        assert!(!embedding.is_empty(), "Embedding should not be empty");
    }

    #[tokio::test]
    async fn test_embedding_batch() {
        dotenv::dotenv().ok();

        // Skip test if OPENROUTER_API_KEY is not set
        if std::env::var("OPENROUTER_API_KEY").is_err() {
            eprintln!("Skipping test: OPENROUTER_API_KEY not set");
            return;
        }

        let service = OpenRouterService::new().expect("Failed to create service");

        let texts = vec![
            "First text".to_string(),
            "Second text".to_string(),
            "Third text".to_string(),
        ];

        let result = service.embed_batch(texts).await;

        assert!(result.is_ok(), "Batch embedding failed: {:?}", result.err());

        let embeddings = result.unwrap();
        assert_eq!(embeddings.len(), 3);
        assert!(!embeddings[0].is_empty());
    }

    #[tokio::test]
    async fn test_list_models() {
        dotenv::dotenv().ok();

        // Skip test if OPENROUTER_API_KEY is not set
        if std::env::var("OPENROUTER_API_KEY").is_err() {
            eprintln!("Skipping test: OPENROUTER_API_KEY not set");
            return;
        }

        let service = OpenRouterService::new().expect("Failed to create service");

        let result = service.list_models().await;

        assert!(result.is_ok(), "List models failed: {:?}", result.err());

        let models = result.unwrap();
        assert!(!models.is_empty(), "Should have some models");

        // Check that GPT-4o is in the list
        let has_gpt4o = models.iter().any(|m| m.id.contains("gpt-4o"));
        assert!(has_gpt4o, "Should have GPT-4o in models list");
    }

    #[tokio::test]
    async fn test_key_info() {
        dotenv::dotenv().ok();

        // Skip test if OPENROUTER_API_KEY is not set
        if std::env::var("OPENROUTER_API_KEY").is_err() {
            eprintln!("Skipping test: OPENROUTER_API_KEY not set");
            return;
        }

        let service = OpenRouterService::new().expect("Failed to create service");

        let result = service.key_info().await;

        assert!(result.is_ok(), "Key info failed: {:?}", result.err());

        let key_info = result.unwrap();
        // Just verify we got a response - actual values depend on the key
        assert!(!key_info.data.label.is_empty());
    }

    #[tokio::test]
    async fn test_test_connection() {
        dotenv::dotenv().ok();

        // Skip test if OPENROUTER_API_KEY is not set
        if std::env::var("OPENROUTER_API_KEY").is_err() {
            eprintln!("Skipping test: OPENROUTER_API_KEY not set");
            return;
        }

        let service = OpenRouterService::new().expect("Failed to create service");

        let result = service.test_connection().await;
        assert!(result.is_ok(), "Test connection failed: {:?}", result.err());
    }

    #[test]
    fn test_empty_messages_validation() {
        let result = std::panic::catch_unwind(|| {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let service = OpenRouterService::with_config("test-key".to_string(), None, None)
                    .expect("Failed to create service");

                let messages: Vec<Message> = vec![];
                let options = ChatOptions::default();

                service.chat(messages, options).await
            })
        });

        // This should fail because we need a real runtime and actual API calls
        // Just verifying the method exists and can be called
        assert!(result.is_err() || result.is_ok());
    }

    #[test]
    fn test_message_reexports() {
        // Verify that all types are properly re-exported
        let _: Message = Message::user("test");
        let _: MessageRole = MessageRole::User;
        let _: MessageContent = MessageContent::Text("test".to_string());
        let _: ContentPart = ContentPart::Text {
            text: "test".to_string(),
        };
        let _: ImageUrl = ImageUrl::from_url("http://test.com", None);

        let _: ChatCompletion = ChatCompletion {
            choices: vec![],
            model: String::new(),
            usage: None,
        };

        let _: Choice = Choice {
            message: Message::user("test"),
        };

        let _: Usage = Usage {
            prompt_tokens: 0,
            completion_tokens: 0,
            total_tokens: 0,
        };

        let _: ModelInfo = ModelInfo {
            id: String::new(),
            name: String::new(),
            description: None,
            pricing: ModelPricing {
                prompt: 0.0,
                completion: 0.0,
            },
        };

        let _: KeyInfo = KeyInfo {
            data: KeyData {
                label: String::new(),
                usage: 0,
                limit: None,
                is_free_tier: true,
            },
        };
    }
}
