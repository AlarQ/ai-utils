# Langfuse Module

The Langfuse module provides comprehensive monitoring and observability for AI applications through integration with the Langfuse platform. It enables tracking of traces, spans, generations, scores, and other observability events to monitor AI application performance and behavior.

## Overview

Langfuse is an open-source LLM observability platform that helps you monitor, debug, and improve your LLM applications. This module provides a Rust client that implements the Langfuse ingestion API, allowing you to:

- Track complete AI workflows (traces)
- Monitor individual operations (spans)
- Record model outputs (generations)
- Evaluate performance with scores
- Log custom events and observations

## Configuration

The module uses environment variables for configuration:

```bash
LANGFUSE_PUBLIC_KEY=your_public_key
LANGFUSE_SECRET_KEY=your_secret_key
LANGFUSE_HOST=https://cloud.langfuse.com  # Optional, defaults to cloud
```

```rust
use ai_utils::langfuse::{LangfuseConfig, LangfuseServiceImpl};

let config = LangfuseConfig::new();
let service = LangfuseServiceImpl::new(config);
```

## Core Concepts

### Traces
A trace represents a complete AI workflow or session. It's the top-level container that groups related operations together.

```rust
use uuid::Uuid;
use ai_utils::openai::OpenAIMessage;

let trace_id = Uuid::new_v4();
let input_messages = vec![OpenAIMessage::new("user", "Hello", None)];
let output_messages = vec![OpenAIMessage::new("assistant", "Hi there!", None)];

let trace_id_str = service
    .create_trace(
        trace_id,
        "chat_conversation",
        Some(&input_messages),
        Some(&output_messages),
        Some("conversation_123"),
    )
    .await?;
```

### Generations
A generation represents a single model output (e.g., an LLM call) within a trace. It tracks input, output, model parameters, and usage statistics.

```rust
// Create a generation
let generation_id = service
    .create_generation(
        &trace_id_str,
        "gpt_response",
        "gpt-4",
        &input_messages,
    )
    .await?;

// Update with output
let completion = ChatCompletion {
    choices: vec![Choice {
        message: Message::assistant("The answer is 42".to_string()),
    }],
    model: "gpt-4".to_string(),
    usage: Some(Usage {
        prompt_tokens: 10,
        completion_tokens: 5,
        total_tokens: 15,
    }),
};

service.update_generation(&generation_id, &completion).await?;
```

### Spans
Spans represent individual operations or steps within a trace. They can be used to track custom logic, API calls, or any measurable operation.

```rust
// Create a span
let span_id = service
    .create_span(
        &trace_id_str,
        "data_processing",
        Some(&input_messages),
    )
    .await?;

// Update with results
let output_messages = vec![OpenAIMessage::new("system", "Processed", None)];
service.update_span(&span_id, &output_messages).await?;
```

### Scores
Scores are quantitative or qualitative evaluations attached to traces or observations. They're useful for tracking performance metrics or user feedback.

```rust
use ai_utils::langfuse::types::{ScoreBody, BaseEvent, IngestionEvent};

let score_body = ScoreBody {
    id: Some(Uuid::new_v4().to_string()),
    traceId: Some(trace_id_str.clone()),
    name: "accuracy".to_string(),
    value: json!(0.95),
    comment: Some("High accuracy score".to_string()),
    sessionId: None,
    observationId: None,
    environment: None,
    metadata: None,
};

let base_event = BaseEvent {
    id: Uuid::new_v4().to_string(),
    timestamp: chrono::Utc::now().to_rfc3339(),
    metadata: None,
};

let event = IngestionEvent::score_create(base_event, score_body);
let batch = IngestionBatch {
    batch: vec![event],
    metadata: None,
};

service.send_batch(batch).await?;
```

## Event Types

The module supports all Langfuse ingestion event types:

### Core Events
- **trace-create**: Create a new trace
- **span-create**: Create a new span
- **span-update**: Update an existing span
- **generation-create**: Create a new generation
- **generation-update**: Update an existing generation

### Evaluation Events
- **score-create**: Create a score for evaluation

### Custom Events
- **event-create**: Create a custom event
- **sdk-log**: Log SDK-specific information
- **observation-create**: Create an observation
- **observation-update**: Update an observation

## Data Types

### BaseEvent
Common fields for all events:
```rust
pub struct BaseEvent {
    pub id: String,
    pub timestamp: String,
    pub metadata: Option<serde_json::Value>,
}
```

### TraceBody
Trace-specific data:
```rust
pub struct TraceBody {
    pub id: Option<String>,
    pub name: Option<String>,
    pub input: Option<serde_json::Value>,
    pub output: Option<serde_json::Value>,
    pub sessionId: Option<String>,
    pub metadata: Option<serde_json::Value>,
    // ... other optional fields
}
```

### GenerationCreateBody
Generation creation data:
```rust
pub struct GenerationCreateBody {
    pub span: SpanCreateBody,  // Flattened span data
    pub model: Option<String>,
    pub modelParameters: Option<serde_json::Value>,
    pub usage: Option<IngestionUsage>,
    // ... other optional fields
}
```

### ScoreBody
Score evaluation data:
```rust
pub struct ScoreBody {
    pub name: String,
    pub value: serde_json::Value,
    pub traceId: Option<String>,
    pub observationId: Option<String>,
    pub comment: Option<String>,
    // ... other optional fields
}
```

## Batch Operations

The module supports batch ingestion for efficient event processing:

```rust
use ai_utils::langfuse::types::{IngestionBatch, IngestionEvent};

let events = vec![
    IngestionEvent::trace_create(base_event1, trace_body),
    IngestionEvent::generation_create(base_event2, generation_body),
    IngestionEvent::score_create(base_event3, score_body),
];

let batch = IngestionBatch {
    batch: events,
    metadata: Some(json!({"source": "ai_utils"})),
};

let response = service.send_batch(batch).await?;
```

## Error Handling

The module provides detailed error handling for batch operations:

```rust
match service.send_batch(batch).await {
    Ok(response) => {
        println!("Successes: {}", response.successes.len());
        println!("Errors: {}", response.errors.len());
        
        for error in &response.errors {
            eprintln!("Error ID {}: {} (status: {})", 
                error.id, 
                error.message.as_deref().unwrap_or("Unknown"), 
                error.status
            );
        }
    }
    Err(e) => eprintln!("Batch failed: {}", e),
}
```

## Helper Methods

The module provides helper methods for creating properly typed events:

```rust
// Create events with proper type fields
let trace_event = IngestionEvent::trace_create(base_event, trace_body);
let generation_event = IngestionEvent::generation_create(base_event, generation_body);
let score_event = IngestionEvent::score_create(base_event, score_body);
let span_event = IngestionEvent::span_create(base_event, span_body);
```

## Testing

The module includes comprehensive tests that demonstrate usage patterns:

```rust
#[tokio::test]
async fn test_create_trace() {
    let config = LangfuseConfig::new();
    let service = LangfuseServiceImpl::new(config);
    
    let trace_id = Uuid::new_v4();
    let result = service
        .create_trace(trace_id, "test_trace", None, None, None)
        .await;
    
    assert!(result.is_ok());
}
```

## Integration Examples

### Chat Application Monitoring
```rust
async fn handle_chat_message(
    service: &LangfuseServiceImpl,
    user_message: &str,
) -> Result<String, Error> {
    let trace_id = Uuid::new_v4();
    
    // Create trace
    let trace_id_str = service
        .create_trace(
            trace_id,
            "chat_interaction",
            Some(&[OpenAIMessage::new("user", user_message, None)]),
            None,
            None,
        )
        .await?;
    
    // Create generation
    let generation_id = service
        .create_generation(
            &trace_id_str,
            "gpt_response",
            "gpt-4",
            &[OpenAIMessage::new("user", user_message, None)],
        )
        .await?;
    
    // Get response from OpenAI
    let completion = openai_client.chat_completion(/* ... */).await?;
    
    // Update generation with response
    service.update_generation(&generation_id, &completion).await?;
    
    // Create score for response quality
    let score_body = ScoreBody {
        name: "response_quality".to_string(),
        value: json!(0.8),
        traceId: Some(trace_id_str),
        id: None,
        sessionId: None,
        observationId: None,
        environment: None,
        comment: None,
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
    
    service.send_batch(batch).await?;
    
    Ok(completion.choices[0].message.content.clone())
}
```

### Workflow Monitoring
```rust
async fn monitor_workflow(
    service: &LangfuseServiceImpl,
    workflow_name: &str,
) -> Result<(), Error> {
    let trace_id = Uuid::new_v4();
    let trace_id_str = service
        .create_trace(trace_id, workflow_name, None, None, None)
        .await?;
    
    // Monitor data preprocessing
    let preprocess_span = service
        .create_span(&trace_id_str, "data_preprocessing", None)
        .await?;
    
    // ... perform preprocessing ...
    
    service.update_span(&preprocess_span, &[/* results */]).await?;
    
    // Monitor model inference
    let generation_id = service
        .create_generation(&trace_id_str, "model_inference", "gpt-4", &[/* input */])
        .await?;
    
    // ... perform inference ...
    
    service.update_generation(&generation_id, &completion).await?;
    
    // Add performance score
    let score_body = ScoreBody {
        name: "workflow_performance".to_string(),
        value: json!(0.95),
        traceId: Some(trace_id_str),
        id: None,
        sessionId: None,
        observationId: None,
        environment: None,
        comment: None,
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
    
    service.send_batch(batch).await?;
    
    Ok(())
}
```

## Best Practices

1. **Use meaningful names**: Give traces, spans, and generations descriptive names for better observability.

2. **Include metadata**: Add relevant metadata to help with debugging and analysis.

3. **Batch operations**: Use batch ingestion for multiple events to improve performance.

4. **Error handling**: Always check batch responses for individual event failures.

5. **Consistent IDs**: Use consistent ID patterns across your application for easier correlation.

6. **Environment separation**: Use different Langfuse projects for development, staging, and production.

## Current Implementation Status

The module currently supports:

✅ **Core Features**
- Trace creation and management
- Generation creation and updates
- Span creation and updates
- Score creation
- Batch ingestion with detailed error handling
- All Langfuse event types

✅ **Type Safety**
- Proper Rust types matching Langfuse API spec
- Helper methods for event creation
- Comprehensive error handling

✅ **Testing**
- Integration tests for core functionality
- Environment-aware test skipping

🔄 **Future Enhancements**
- Additional helper methods for all event types
- More comprehensive documentation
- Advanced observability features
- Performance optimizations
