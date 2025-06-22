# Quick Start

Get up and running with AI Utils in minutes. This guide will walk you through building a simple AI agent that can process text and answer questions.

## Basic Chat Bot

Let's create a simple chat bot that can maintain conversation context:

```rust
use ai_utils::{
    openai::{OpenAIService, OpenAIMessage, OpenAIModel},
    text_splitter::TextSplitter,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();
    
    // Initialize services
    let openai = OpenAIService::new();
    let splitter = TextSplitter::new(None);
    
    // System prompt to define the bot's behavior
    let system_message = OpenAIMessage {
        role: "system".to_string(),
        content: "You are a helpful AI assistant. Provide clear, concise answers.".to_string(),
        name: None,
    };
    
    // User message
    let user_message = OpenAIMessage {
        role: "user".to_string(),
        content: "What is the capital of France?".to_string(),
        name: None,
    };
    
    // Create conversation
    let messages = vec![system_message, user_message];
    
    // Get response
    let response = openai.completion(messages, OpenAIModel::GPT35Turbo).await?;
    
    if let Some(choice) = response.choices.first() {
        println!("AI: {}", choice.message.content);
    }
    
    Ok(())
}
```

## Document Q&A System

Create a system that can answer questions about documents:

```rust
use ai_utils::{
    openai::{OpenAIService, OpenAIMessage, OpenAIModel},
    text_splitter::TextSplitter,
    qdrant::QdrantService,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();
    
    let openai = OpenAIService::new();
    let splitter = TextSplitter::new(None);
    let qdrant = QdrantService::new().await?;
    
    // Sample document
    let document = r#"
    Artificial Intelligence (AI) is a branch of computer science that aims to create 
    intelligent machines that work and react like humans. Some of the activities 
    computers with artificial intelligence are designed for include speech recognition, 
    learning, planning, and problem solving.
    "#;
    
    // Split document into chunks
    let chunks = splitter.split(document, 1000)?;
    
    // Create collection for document chunks
    let collection_name = "documents";
    qdrant.create_collection(collection_name, 1536).await?;
    
    // Process each chunk
    for (i, chunk) in chunks.iter().enumerate() {
        // Generate embeddings
        let embeddings = openai.embed(chunk.content.clone()).await?;
        
        // Store in vector database
        qdrant.upsert_points(
            collection_name,
            &[i.to_string()],
            &[embeddings],
            &[chunk.content.clone()],
        ).await?;
    }
    
    // Query the system
    let question = "What is artificial intelligence?";
    let question_embeddings = openai.embed(question.to_string()).await?;
    
    // Search for relevant chunks
    let search_results = qdrant.search_points(
        collection_name,
        &question_embeddings,
        3,
    ).await?;
    
    // Build context from search results
    let context: String = search_results
        .iter()
        .map(|result| result.payload.get("text").unwrap_or(&"".to_string()))
        .collect::<Vec<_>>()
        .join("\n\n");
    
    // Generate answer
    let messages = vec![
        OpenAIMessage {
            role: "system".to_string(),
            content: format!(
                "Answer the question based on the following context:\n\n{}",
                context
            ),
            name: None,
        },
        OpenAIMessage {
            role: "user".to_string(),
            content: question.to_string(),
            name: None,
        },
    ];
    
    let response = openai.completion(messages, OpenAIModel::GPT35Turbo).await?;
    
    if let Some(choice) = response.choices.first() {
        println!("Question: {}", question);
        println!("Answer: {}", choice.message.content);
    }
    
    Ok(())
}
```

## Image Analysis

Process and analyze images with multimodal capabilities:

```rust
use ai_utils::openai::{OpenAIService, OpenAIMessage, OpenAIImageMessage, OpenAIModel};
use std::fs;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();
    
    let openai = OpenAIService::new();
    
    // Read image file
    let image_data = fs::read("path/to/your/image.jpg")?;
    let base64_image = base64::encode(&image_data);
    let image_url = format!("data:image/jpeg;base64,{}", base64_image);
    
    // Create image message
    let image_message = OpenAIImageMessage {
        role: "user".to_string(),
        content: vec![image_url],
        name: None,
    };
    
    // Text question about the image
    let text_message = OpenAIMessage {
        role: "user".to_string(),
        content: "What do you see in this image?".to_string(),
        name: None,
    };
    
    // Get multimodal response
    let response = openai.completion_image(
        vec![text_message],
        vec![image_message],
        OpenAIModel::GPT4Vision,
    ).await?;
    
    if let Some(choice) = response.choices.first() {
        println!("Image Analysis: {}", choice.message.content);
    }
    
    Ok(())
}
```

## AI Application Monitoring with Langfuse

Monitor your AI applications with comprehensive observability:

```rust
use ai_utils::{
    openai::{OpenAIService, OpenAIMessage, OpenAIModel},
    langfuse::{LangfuseConfig, LangfuseServiceImpl},
};
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();
    
    let openai = OpenAIService::new();
    let langfuse = LangfuseServiceImpl::new(LangfuseConfig::new());
    
    // Create a trace for this conversation
    let trace_id = Uuid::new_v4();
    let user_message = "What is the weather like today?";
    
    let trace_id_str = langfuse
        .create_trace(
            trace_id,
            "weather_inquiry",
            Some(&[OpenAIMessage::new("user", user_message, None)]),
            None,
            Some("session_123"),
        )
        .await?;
    
    // Create a generation to track the AI response
    let generation_id = langfuse
        .create_generation(
            &trace_id_str,
            "weather_response",
            "gpt-4",
            &[OpenAIMessage::new("user", user_message, None)],
        )
        .await?;
    
    // Get AI response
    let messages = vec![
        OpenAIMessage::new("system", "You are a helpful weather assistant.", None),
        OpenAIMessage::new("user", user_message, None),
    ];
    
    let response = openai.completion(messages, OpenAIModel::GPT4).await?;
    
    // Update generation with the response
    langfuse.update_generation(&generation_id, &response).await?;
    
    // Create a score for response quality
    use ai_utils::langfuse::types::{ScoreBody, BaseEvent, IngestionEvent, IngestionBatch};
    use serde_json::json;
    use chrono;
    
    let score_body = ScoreBody {
        id: Some(Uuid::new_v4().to_string()),
        traceId: Some(trace_id_str.clone()),
        name: "response_quality".to_string(),
        value: json!(0.85),
        comment: Some("Good response quality".to_string()),
        sessionId: None,
        observationId: None,
        environment: None,
        metadata: None,
    };
    
    let batch = IngestionBatch {
        batch: vec![IngestionEvent::score_create(
            BaseEvent {
                id: Uuid::new_v4().to_string(),
                timestamp: chrono::Utc::now().to_rfc3339(),
                metadata: None,
            },
            score_body,
        )],
        metadata: None,
    };
    
    langfuse.send_batch(batch).await?;
    
    if let Some(choice) = response.choices.first() {
        println!("AI Response: {}", choice.message.content);
        println!("Trace ID: {}", trace_id_str);
        println!("Generation ID: {}", generation_id);
    }
    
    Ok(())
}
```

## Key Concepts

### 1. Service Initialization
All services are initialized with environment variables:
- `OpenAIService::new()` - Uses `OPENAI_API_KEY`
- `QdrantService::new()` - Uses `QDRANT_URL` and `QDRANT_API_KEY`

### 2. Async Operations
All operations are async and should be awaited:
```rust
let result = service.operation().await?;
```

### 3. Error Handling
Use the `?` operator for error propagation:
```rust
let response = openai.completion(messages, model).await?;
```

## Next Steps

- [Configuration](../getting-started/configuration.md) - Learn about advanced configuration options
- [Installation](../getting-started/installation.md) - Set up your development environment
