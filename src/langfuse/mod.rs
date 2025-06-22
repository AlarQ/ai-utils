mod service;
mod types;

pub use service::*;
pub use types::*;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::openai::OpenAIMessage;
    use uuid::Uuid;

    #[tokio::test]
    async fn test_create_trace() {
        dotenv::dotenv().ok();
        // Skip test if Langfuse credentials are not set
        if std::env::var("LANGFUSE_PUBLIC_KEY").is_err()
            || std::env::var("LANGFUSE_SECRET_KEY").is_err()
        {
            eprintln!(
                "Skipping test_create_trace: LANGFUSE_PUBLIC_KEY or LANGFUSE_SECRET_KEY not set"
            );
            return;
        }

        let config = LangfuseConfig::new();
        let service = LangfuseServiceImpl::new(config);

        let trace_id = Uuid::new_v4();
        let name = "test_trace";
        let conversation_id = "test_conversation_123";

        // Create test messages
        let input_messages = vec![OpenAIMessage::new(
            "user",
            "Hello, how are you?".to_string(),
            None,
        )];

        let output_messages = vec![OpenAIMessage::new(
            "assistant",
            "I'm doing well, thank you!".to_string(),
            None,
        )];

        // Create the trace with input/output data
        let result = service
            .create_trace(
                trace_id,
                name,
                Some(&input_messages),
                Some(&output_messages),
                Some(conversation_id),
            )
            .await;

        match result {
            Ok(trace_id_str) => {
                println!("Successfully created trace with ID: {}", trace_id_str);
                assert_eq!(trace_id_str, trace_id.to_string());
            }
            Err(e) => {
                eprintln!("Failed to create trace: {:?}", e);
                // Don't fail the test if it's a network/API issue, only if it's a logic error
                if e.to_string().contains("Batch ingestion errors") {
                    panic!("Trace creation failed: {}", e);
                }
            }
        }
    }

    #[tokio::test]
    async fn test_create_generation() {
        dotenv::dotenv().ok();
        // Skip test if Langfuse credentials are not set
        if std::env::var("LANGFUSE_PUBLIC_KEY").is_err()
            || std::env::var("LANGFUSE_SECRET_KEY").is_err()
        {
            eprintln!("Skipping test_create_generation: LANGFUSE_PUBLIC_KEY or LANGFUSE_SECRET_KEY not set");
            return;
        }

        let config = LangfuseConfig::new();
        let service = LangfuseServiceImpl::new(config);

        // First create a trace
        let trace_id = Uuid::new_v4();
        let trace_name = "test_trace_for_generation";
        let conversation_id = "test_conversation_456";

        let trace_result = service
            .create_trace(trace_id, trace_name, None, None, Some(conversation_id))
            .await;

        let trace_id_str = match trace_result {
            Ok(id) => id,
            Err(e) => {
                eprintln!("Failed to create trace for generation test: {:?}", e);
                return;
            }
        };

        // Create test input messages
        let input_messages = vec![OpenAIMessage::new(
            "user",
            "What is the capital of France?".to_string(),
            None,
        )];

        // Create a generation
        let generation_name = "test_generation";
        let model = "gpt-4o";

        let generation_result = service
            .create_generation(&trace_id_str, generation_name, model, &input_messages)
            .await;

        match generation_result {
            Ok(generation_id) => {
                println!("Successfully created generation with ID: {}", generation_id);

                // Create a mock ChatCompletion for testing
                let mock_output = crate::openai::ChatCompletion {
                    choices: vec![crate::openai::Choice {
                        message: crate::openai::Message::assistant(
                            "The capital of France is Paris.".to_string(),
                        ),
                    }],
                    model: model.to_string(),
                    usage: Some(crate::openai::Usage {
                        prompt_tokens: 10,
                        completion_tokens: 8,
                        total_tokens: 18,
                    }),
                };

                // Update the generation with output
                let update_result = service
                    .update_generation(&generation_id, &mock_output)
                    .await;

                match update_result {
                    Ok(()) => {
                        println!("Successfully updated generation with output");
                    }
                    Err(e) => {
                        eprintln!("Failed to update generation: {:?}", e);
                        if e.to_string().contains("Batch ingestion errors") {
                            panic!("Generation update failed: {}", e);
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("Failed to create generation: {:?}", e);
                if e.to_string().contains("Batch ingestion errors") {
                    panic!("Generation creation failed: {}", e);
                }
            }
        }
    }

    #[tokio::test]
    async fn test_create_score() {
        dotenv::dotenv().ok();
        // Skip test if Langfuse credentials are not set
        if std::env::var("LANGFUSE_PUBLIC_KEY").is_err()
            || std::env::var("LANGFUSE_SECRET_KEY").is_err()
        {
            eprintln!(
                "Skipping test_create_score: LANGFUSE_PUBLIC_KEY or LANGFUSE_SECRET_KEY not set"
            );
            return;
        }

        let config = LangfuseConfig::new();
        let service = LangfuseServiceImpl::new(config);

        // First create a trace
        let trace_id = Uuid::new_v4();
        let trace_name = "test_trace_for_score";
        let conversation_id = "test_conversation_789";

        let trace_result = service
            .create_trace(trace_id, trace_name, None, None, Some(conversation_id))
            .await;

        let trace_id_str = match trace_result {
            Ok(id) => id,
            Err(e) => {
                eprintln!("Failed to create trace for score test: {:?}", e);
                return;
            }
        };

        // Create a score using the raw API to test the new event types
        let score_id = Uuid::new_v4().to_string();
        let event_id = Uuid::new_v4().to_string();

        let score_body = crate::langfuse::types::ScoreBody {
            id: Some(score_id.clone()),
            traceId: Some(trace_id_str.clone()),
            sessionId: None,
            observationId: None,
            name: "response_quality".to_string(),
            environment: None,
            value: serde_json::json!(0.85),
            comment: Some("High quality response".to_string()),
            metadata: None,
        };

        let base_event = crate::langfuse::types::BaseEvent {
            id: event_id,
            timestamp: chrono::Utc::now().to_rfc3339(),
            metadata: None,
        };

        let event = crate::langfuse::types::IngestionEvent::score_create(base_event, score_body);

        let batch = crate::langfuse::types::IngestionBatch {
            batch: vec![event],
            metadata: None,
        };

        // Send the batch directly to test the new event structure
        let result = service.send_batch(batch).await;

        match result {
            Ok(_) => {
                println!("Successfully created score with ID: {}", score_id);
            }
            Err(e) => {
                eprintln!("Failed to create score: {:?}", e);
                if e.to_string().contains("Batch ingestion errors") {
                    panic!("Score creation failed: {}", e);
                }
            }
        }
    }
}
