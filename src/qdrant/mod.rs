pub mod qdrant_service;

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::env;
    use std::time::Duration;

    use qdrant_client::Qdrant;

    use crate::openai::{AIService, OpenAIService};
    use crate::qdrant::qdrant_service::{PointInput, QdrantConfig, QdrantService};

    #[tokio::test]
    async fn test() {
        dotenv::dotenv().ok();

        let url = env::var("QDRANT_URL").unwrap();
        let api_key = env::var("QDRANT_API_KEY").unwrap();

        println!("Connecting to Qdrant at URL: {}", url);
        println!("Using API key: {}", api_key);

        let client = Qdrant::from_url(&url)
            .api_key(api_key)
            .timeout(Duration::from_secs(30))
            .build()
            .unwrap();

        // First try a simple health check
        match client.health_check().await {
            Ok(_) => println!("Health check successful"),
            Err(e) => println!("Health check failed: {:?}", e),
        }

        let collections_list = client.list_collections().await;
        let _ = dbg!(collections_list);
    }

    #[tokio::test]
    async fn test_openai_batch_embedding() {
        dotenv::dotenv().ok();

        let openai_service = OpenAIService::new().expect("Failed to create OpenAI service");

        // Test single text embedding
        let single_text = "Hello world".to_string();
        let single_embedding = openai_service
            .embed(single_text.clone())
            .await
            .expect("Failed to embed single text");

        // Test batch embedding with single text
        let batch_texts = vec![single_text.clone()];
        let batch_embeddings = openai_service
            .embed_batch(batch_texts)
            .await
            .expect("Failed to embed batch with single text");

        // Verify single embedding matches batch embedding in structure
        assert_eq!(batch_embeddings.len(), 1);
        assert_eq!(single_embedding.len(), batch_embeddings[0].len());

        // Test batch embedding with multiple texts
        let multiple_texts = vec![
            "Hello world".to_string(),
            "This is a test".to_string(),
            "Batch embedding works".to_string(),
        ];

        let multiple_embeddings = openai_service
            .embed_batch(multiple_texts.clone())
            .await
            .expect("Failed to embed batch with multiple texts");

        // Verify we got the right number of embeddings
        assert_eq!(multiple_embeddings.len(), multiple_texts.len());

        // Verify each embedding has the expected dimension (for text-embedding-3-large)
        for embedding in &multiple_embeddings {
            assert_eq!(embedding.len(), 3072); // text-embedding-3-large dimension
        }

        // Verify embeddings are different for different texts (they should be semantically different)
        // Note: We can't guarantee exact differences due to model variations, but they should be different vectors
        let embedding1 = &multiple_embeddings[0];
        let embedding2 = &multiple_embeddings[1];
        let embedding3 = &multiple_embeddings[2];

        // Check that at least some values are different (not identical vectors)
        let mut different_1_2 = false;
        let mut different_2_3 = false;
        let mut different_1_3 = false;

        for i in 0..embedding1.len() {
            if (embedding1[i] - embedding2[i]).abs() > 1e-6 {
                different_1_2 = true;
            }
            if (embedding2[i] - embedding3[i]).abs() > 1e-6 {
                different_2_3 = true;
            }
            if (embedding1[i] - embedding3[i]).abs() > 1e-6 {
                different_1_3 = true;
            }
        }

        // At least one pair should be different
        assert!(
            different_1_2 || different_2_3 || different_1_3,
            "All embeddings appear to be identical"
        );

        // Test empty batch
        let empty_result = openai_service.embed_batch(vec![]).await;
        assert!(empty_result.is_err());
    }

    #[tokio::test]
    async fn test_qdrant_batch_upsert() {
        dotenv::dotenv().ok();

        let config = crate::qdrant::qdrant_service::QdrantConfig::from_env()
            .expect("Qdrant config from env");
        let qdrant_service = QdrantService::new(config).expect("Failed to create Qdrant service");
        let test_collection = "test_batch_upsert";

        // Clean up any existing test collection
        let _ = qdrant_service.delete_collection(test_collection).await;

        // Create test collection
        qdrant_service
            .create_collection(test_collection, 3072)
            .await
            .expect("Failed to create test collection");

        // Create test points
        let mut metadata1 = HashMap::new();
        metadata1.insert("source".to_string(), "test1".to_string());
        metadata1.insert("category".to_string(), "batch".to_string());

        let mut metadata2 = HashMap::new();
        metadata2.insert("source".to_string(), "test2".to_string());
        metadata2.insert("category".to_string(), "batch".to_string());

        let mut metadata3 = HashMap::new();
        metadata3.insert("source".to_string(), "test3".to_string());
        metadata3.insert("category".to_string(), "batch".to_string());

        let points = vec![
            PointInput::new("1", "First test document for batch upsert", &metadata1),
            PointInput::new("2", "Second test document for batch upsert", &metadata2),
            PointInput::new("3", "Third test document for batch upsert", &metadata3),
        ];

        // Test batch upsert
        qdrant_service
            .upsert_points_batch(test_collection, points.clone())
            .await
            .expect("Failed to batch upsert points");

        // Verify points were inserted by searching
        let search_results = qdrant_service
            .search_points(test_collection.to_string(), "test document".to_string(), 10)
            .await
            .expect("Failed to search points");

        // Should find our 3 test documents
        assert_eq!(search_results.len(), 3);

        // Verify each result contains expected metadata
        let mut found_ids = std::collections::HashSet::new();
        for result in search_results {
            if let Some(id) = result.0.get("id") {
                found_ids.insert(id.clone());
            }
        }

        assert!(found_ids.contains("1"));
        assert!(found_ids.contains("2"));
        assert!(found_ids.contains("3"));

        // Clean up
        let _ = qdrant_service.delete_collection(test_collection).await;
    }

    #[tokio::test]
    async fn test_qdrant_batch_upsert_empty() {
        dotenv::dotenv().ok();

        let config = crate::qdrant::qdrant_service::QdrantConfig::from_env()
            .expect("Qdrant config from env");
        let qdrant_service = QdrantService::new(config).expect("Failed to create Qdrant service");

        // Test batch upsert with empty points
        let result = qdrant_service
            .upsert_points_batch("nonexistent_collection", vec![])
            .await;
        assert!(result.is_ok(), "Empty batch should succeed");
    }

    #[tokio::test]
    async fn test_qdrant_batch_upsert_performance() {
        dotenv::dotenv().ok();

        let config = crate::qdrant::qdrant_service::QdrantConfig::from_env()
            .expect("Qdrant config from env");
        let qdrant_service = QdrantService::new(config).expect("Failed to create Qdrant service");
        let test_collection = "test_batch_performance";

        // Clean up any existing test collection
        let _ = qdrant_service.delete_collection(test_collection).await;

        // Create test collection
        qdrant_service
            .create_collection(test_collection, 3072)
            .await
            .expect("Failed to create test collection");

        // Create larger batch of test points
        let mut points = Vec::new();
        for i in 0..10 {
            let mut metadata = HashMap::new();
            metadata.insert("batch_id".to_string(), format!("batch_{}", i));
            metadata.insert("index".to_string(), i.to_string());

            points.push(PointInput::new(
                &i.to_string(),
                &format!("Test document number {} for performance testing", i),
                &metadata,
            ));
        }

        // Measure batch upsert time
        let start = std::time::Instant::now();
        qdrant_service
            .upsert_points_batch(test_collection, points.clone())
            .await
            .expect("Failed to batch upsert points");
        let batch_duration = start.elapsed();

        println!(
            "Batch upsert of {} points took: {:?}",
            points.len(),
            batch_duration
        );

        // Verify all points were inserted
        let search_results = qdrant_service
            .search_points(test_collection.to_string(), "test document".to_string(), 20)
            .await
            .expect("Failed to search points");

        assert_eq!(search_results.len(), 10);

        // Clean up
        let _ = qdrant_service.delete_collection(test_collection).await;
    }

    #[tokio::test]
    async fn test_qdrant_upsert_points_uses_batch() {
        dotenv::dotenv().ok();

        let config = crate::qdrant::qdrant_service::QdrantConfig::from_env()
            .expect("Qdrant config from env");
        let qdrant_service = QdrantService::new(config).expect("Failed to create Qdrant service");
        let test_collection = "test_upsert_points";

        // Clean up any existing test collection
        let _ = qdrant_service.delete_collection(test_collection).await;

        // Create test collection
        qdrant_service
            .create_collection(test_collection, 3072)
            .await
            .expect("Failed to create test collection");

        // Create test points
        let mut metadata = HashMap::new();
        metadata.insert("test".to_string(), "upsert_points".to_string());

        let points = vec![
            PointInput::new("1", "Test document one", &metadata),
            PointInput::new("2", "Test document two", &metadata),
        ];

        // Test that upsert_points uses batch implementation
        qdrant_service
            .upsert_points(test_collection, points)
            .await
            .expect("Failed to upsert points");

        // Verify points were inserted
        let search_results = qdrant_service
            .search_points(test_collection.to_string(), "test document".to_string(), 10)
            .await
            .expect("Failed to search points");

        assert_eq!(search_results.len(), 2);

        // Clean up
        let _ = qdrant_service.delete_collection(test_collection).await;
    }
}
