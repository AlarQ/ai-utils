# Qdrant Vector Database Integration

The Qdrant module provides a high-performance interface for working with Qdrant vector databases, including automatic text embedding, batch operations, and feature-flag based functionality.

## Features

- **Vector Database Operations**: Create collections, upsert points, and perform similarity searches
- **Automatic Text Embedding**: Seamless integration with OpenAI for text-to-vector conversion
- **Batch Operations**: Efficient batch upserting with optimized embedding calls
- **Feature Flags**: Optional OpenAI integration for custom embedding workflows
- **Error Handling**: Comprehensive error handling with descriptive messages
- **Configuration**: Flexible configuration via structs instead of environment variables

## Quick Start

### Basic Setup

```rust
use ai_utils::qdrant::{QdrantService, QdrantConfig, PointInput};
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create configuration
    let config = QdrantConfig::from_env()?;
    
    // Initialize service
    let qdrant_service = QdrantService::new(config)?;
    
    // Create a collection
    qdrant_service.create_collection("my_collection", 3072).await?;
    
    // Create a point with metadata
    let mut metadata = HashMap::new();
    metadata.insert("category".to_string(), "technology".to_string());
    metadata.insert("source".to_string(), "blog".to_string());
    
    let point = PointInput::new(
        "1",
        "This is a sample document about artificial intelligence",
        &metadata
    );
    
    // Upsert the point (requires 'openai' feature)
    qdrant_service.upsert_point("my_collection", point).await?;
    
    // Search for similar documents
    let results = qdrant_service.search_points(
        "my_collection".to_string(),
        "machine learning".to_string(),
        10
    ).await?;
    
    println!("Found {} similar documents", results.len());
    
    Ok(())
}
```

## Configuration

### QdrantConfig

The service uses a configuration struct for better testability and flexibility:

```rust
use ai_utils::qdrant::QdrantConfig;

// From environment variables
let config = QdrantConfig::from_env()?;

// Or create manually
let config = QdrantConfig {
    url: "http://localhost:6333".to_string(),
    api_key: "your-api-key".to_string(),
};
```

### Environment Variables

- `QDRANT_URL`: The URL of your Qdrant instance
- `QDRANT_API_KEY`: Your Qdrant API key

## Core Types

### PointInput

Represents a document to be stored in the vector database:

```rust
use ai_utils::qdrant::PointInput;
use std::collections::HashMap;

let mut metadata = HashMap::new();
metadata.insert("author".to_string(), "John Doe".to_string());
metadata.insert("date".to_string(), "2024-01-01".to_string());

let point = PointInput::new(
    "unique_id_123",
    "Document content to be embedded",
    &metadata
);
```

### QueryOutput

Represents search results with metadata:

```rust
use ai_utils::qdrant::QueryOutput;

// QueryOutput implements Debug and Clone
let results: Vec<QueryOutput> = qdrant_service.search_points(...).await?;

for result in &results {
    println!("Result: {:?}", result);
    
    // Access metadata
    if let Some(id) = result.0.get("id") {
        println!("Document ID: {}", id);
    }
}
```

## API Reference

### Collection Management

#### Create Collection

```rust
// Create a collection with cosine distance
qdrant_service.create_collection("my_collection", 3072).await?;
```

#### List Collections

```rust
let collections = qdrant_service.list_collections().await?;
println!("Available collections: {:?}", collections);
```

#### Delete Collection

```rust
// Available in test builds
qdrant_service.delete_collection("my_collection").await?;
```

### Point Operations

#### Single Point Upsert (with OpenAI)

```rust
let point = PointInput::new("1", "Document content", &metadata);
qdrant_service.upsert_point("collection", point).await?;
```

#### Batch Point Upsert (with OpenAI)

```rust
let points = vec![
    PointInput::new("1", "First document", &metadata1),
    PointInput::new("2", "Second document", &metadata2),
    PointInput::new("3", "Third document", &metadata3),
];

qdrant_service.upsert_points("collection", points).await?;
```

#### Vector-Based Operations (without OpenAI)

When the `openai` feature is disabled, you can work with pre-computed vectors:

```rust
// Upsert with pre-computed vector
let vector = vec![0.1; 3072]; // Your embedding vector
let payload = HashMap::new(); // Your metadata
qdrant_service.upsert_point_with_vector("collection", 1, vector, payload).await?;

// Search with pre-computed vector
let query_vector = vec![0.2; 3072]; // Your query embedding
let results = qdrant_service.search_points_with_vector(
    "collection".to_string(),
    query_vector,
    10
).await?;
```

### Search Operations

#### Semantic Search (with OpenAI)

```rust
let results = qdrant_service.search_points(
    "collection".to_string(),
    "natural language query".to_string(),
    10
).await?;

for result in results {
    println!("Score: {:?}", result.0.get("score"));
    println!("Content: {:?}", result.0.get("text"));
}
```

## Search Builder Examples

The new builder pattern allows ergonomic and flexible search configuration:

### Text Search (with OpenAI)

```rust
use ai_utils::qdrant::qdrant_service::QdrantService;

let results = qdrant_service
    .search_builder("my_collection")
    .query_text("machine learning")
    .limit(5)
    .hnsw_ef(64)
    .exact(true)
    .with_payload(true)
    .search()
    .await?;

for result in results {
    println!("Result: {:?}", result);
}
```

### Vector Search (no OpenAI required)

```rust
use ai_utils::qdrant::qdrant_service::QdrantService;

let query_vector = vec![0.1; 3072];
let results = qdrant_service
    .search_builder("my_collection")
    .query_vector(query_vector)
    .limit(3)
    .search()
    .await?;

for result in results {
    println!("Result: {:?}", result);
}
```

### Using Filters

```rust
use ai_utils::qdrant::qdrant_service::QdrantService;
use qdrant_client::qdrant::{Filter, FieldCondition, Match, MatchValue};

let filter = Filter::must([FieldCondition {
    key: "category".to_string(),
    r#match: Some(Match::Value(MatchValue::from("technology"))),
    ..Default::default()
}]);

let results = qdrant_service
    .search_builder("my_collection")
    .query_text("AI")
    .filter(filter)
    .search()
    .await?;

for result in results {
    println!("Filtered result: {:?}", result);
}
```

## Feature Flags

The Qdrant module supports feature flags for flexible compilation:

### Available Features

- **`qdrant`** (default): Core Qdrant functionality
- **`openai`** (default): OpenAI integration for text embedding

### Build Configurations

#### Minimal Build (Qdrant only, no OpenAI)

```toml
[dependencies]
ai_utils = { version = "0.1.0", default-features = false, features = ["qdrant"] }
```

#### Full Build (with OpenAI)

```toml
[dependencies]
ai_utils = { version = "0.1.0", features = ["qdrant", "openai"] }
```

### Feature-Specific API

When `openai` is disabled, the service provides vector-based methods:

```rust
// These methods work without OpenAI:
// - upsert_point_with_vector()
// - search_points_with_vector()
// - create_collection()
// - list_collections()

// These methods require OpenAI:
// - upsert_point()
// - upsert_points()
// - search_points()
```

## Constants

The module provides several constants for configuration:

```rust
use ai_utils::qdrant::qdrant_service::{
    DEFAULT_HNSW_EF,
    DEFAULT_SEARCH_LIMIT,
    TEXT_EMBEDDING_3_LARGE_DIMENSION
};

// Use in your code
let vector_size = TEXT_EMBEDDING_3_LARGE_DIMENSION; // 3072
let search_limit = DEFAULT_SEARCH_LIMIT; // 10
```

## Error Handling

The service provides comprehensive error handling:

```rust
use ai_utils::Error;

match qdrant_service.upsert_point("collection", point).await {
    Ok(()) => println!("Point upserted successfully"),
    Err(Error::Config(msg)) => println!("Configuration error: {}", msg),
    Err(Error::Other(msg)) => println!("Operation failed: {}", msg),
}
```

## Performance Considerations

### Batch Operations

For large datasets, use batch operations:

```rust
// Efficient: Single API call for embeddings
let points = vec![/* many points */];
qdrant_service.upsert_points("collection", points).await?;

// Less efficient: Multiple API calls
for point in points {
    qdrant_service.upsert_point("collection", point).await?;
}
```

### Vector Dimensions

- Use `TEXT_EMBEDDING_3_LARGE_DIMENSION` (3072) for OpenAI's text-embedding-3-large model
- Ensure your collection is created with the correct vector size
- Mismatched dimensions will cause errors

## Testing

The module includes comprehensive tests that demonstrate usage:

```rust
#[tokio::test]
async fn test_basic_operations() {
    let config = QdrantConfig::from_env().expect("Config from env");
    let service = QdrantService::new(config).expect("Service creation");
    
    // Test collection operations
    service.create_collection("test", 3072).await.expect("Create collection");
    
    // Test point operations
    let point = PointInput::new("1", "test content", &HashMap::new());
    service.upsert_point("test", point).await.expect("Upsert point");
    
    // Test search
    let results = service.search_points("test".to_string(), "test".to_string(), 10).await.expect("Search");
    assert_eq!(results.len(), 1);
}
```

## Migration Guide

### From Direct Environment Variables

**Before:**
```rust
// Old way - direct env access
let service = QdrantService::new()?;
```

**After:**
```rust
// New way - config struct
let config = QdrantConfig::from_env()?;
let service = QdrantService::new(config)?;
```

### From Magic Numbers

**Before:**
```rust
service.create_collection("collection", 3072).await?;
```

**After:**
```rust
use ai_utils::qdrant::qdrant_service::TEXT_EMBEDDING_3_LARGE_DIMENSION;
service.create_collection("collection", TEXT_EMBEDDING_3_LARGE_DIMENSION).await?;
```

## Best Practices

1. **Use batch operations** for multiple points
2. **Handle errors gracefully** with proper error types
3. **Use constants** instead of magic numbers
4. **Configure features** based on your needs
5. **Test with feature flags** to ensure compatibility
6. **Monitor performance** with batch operations
7. **Use appropriate vector dimensions** for your embedding model 