# Configuration

This guide covers all configuration options for AI Utils, including environment variables, service settings, and performance tuning.

## Environment Variables

AI Utils uses environment variables for configuration. Create a `.env` file in your project root:

```bash
# OpenAI Configuration
OPENAI_API_KEY=your-openai-api-key

# Qdrant Vector Database
QDRANT_URL=https://your-qdrant-instance.com
QDRANT_API_KEY=your-qdrant-api-key

# Langfuse Monitoring (Optional)
LANGFUSE_PUBLIC_KEY=your-langfuse-public-key
LANGFUSE_SECRET_KEY=your-langfuse-secret-key

# Application Settings
RUST_LOG=info
RUST_BACKTRACE=1
```

## Service Configuration

### OpenAI Service

The OpenAI service is configured automatically from environment variables:

```rust
use ai_utils::openai::OpenAIService;

// Automatically reads OPENAI_API_KEY from environment
let openai = OpenAIService::new();
```

**Available Models:**
- `GPT35Turbo` - Fast, cost-effective
- `GPT4` - More capable, better reasoning
- `GPT4Vision` - Multimodal (text + image)

### Qdrant Service

```rust
use ai_utils::qdrant::QdrantService;

// Automatically reads QDRANT_URL and QDRANT_API_KEY
let qdrant = QdrantService::new().await?;
```

**Configuration Options:**
- `QDRANT_URL` - Server endpoint
- `QDRANT_API_KEY` - Authentication key
- Connection timeout (configurable)
- Retry policies

### Text Splitter

```rust
use ai_utils::text_splitter::TextSplitter;

// Default configuration
let splitter = TextSplitter::new(None);

// Custom configuration
let config = TextSplitterConfig {
    chunk_size: 1000,
    chunk_overlap: 200,
    separator: "\n\n".to_string(),
};
let splitter = TextSplitter::new(Some(config));
```

## Performance Configuration

### Connection Pooling

```rust
// Configure HTTP client for better performance
use reqwest::Client;

let client = Client::builder()
    .pool_max_idle_per_host(10)
    .timeout(std::time::Duration::from_secs(30))
    .build()?;
```

### Batch Processing

```rust
// Configure batch sizes for optimal performance
const BATCH_SIZE: usize = 100; // Embeddings
const VECTOR_BATCH_SIZE: usize = 1000; // Vector operations
```

### Caching

```rust
use std::collections::HashMap;
use std::sync::Mutex;

struct Cache {
    embeddings: Mutex<HashMap<String, Vec<f32>>>,
    responses: Mutex<HashMap<String, String>>,
}

impl Cache {
    pub fn new() -> Self {
        Self {
            embeddings: Mutex::new(HashMap::new()),
            responses: Mutex::new(HashMap::new()),
        }
    }
    
    pub fn get_embedding(&self, text: &str) -> Option<Vec<f32>> {
        self.embeddings.lock().unwrap().get(text).cloned()
    }
    
    pub fn set_embedding(&self, text: String, embedding: Vec<f32>) {
        self.embeddings.lock().unwrap().insert(text, embedding);
    }
}
```

## Logging Configuration

### Basic Logging

```rust
use tracing_subscriber;

// Initialize logging
tracing_subscriber::fmt::init();

// Set log level via environment
// RUST_LOG=debug cargo run
```

### Structured Logging

```rust
use tracing::{info, warn, error, instrument};

#[instrument]
async fn process_document(content: &str) -> Result<(), Error> {
    info!(length = content.len(), "Processing document");
    
    // Process document...
    
    info!("Document processed successfully");
    Ok(())
}
```

## Error Handling Configuration

### Custom Error Types

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("OpenAI API error: {0}")]
    OpenAI(#[from] ai_utils::error::Error),
    
    #[error("Configuration error: {0}")]
    Config(String),
    
    #[error("Rate limit exceeded")]
    RateLimit,
}

// Use in your application
async fn process_with_retry() -> Result<(), AppError> {
    // Implementation with retry logic
    Ok(())
}
```

### Retry Configuration

```rust
use tokio::time::{sleep, Duration};

async fn with_retry<F, T, E>(mut f: F, max_retries: usize) -> Result<T, E>
where
    F: FnMut() -> Result<T, E>,
    E: std::fmt::Debug,
{
    let mut attempts = 0;
    loop {
        match f() {
            Ok(result) => return Ok(result),
            Err(e) => {
                attempts += 1;
                if attempts >= max_retries {
                    return Err(e);
                }
                
                let delay = Duration::from_secs(2_u64.pow(attempts as u32));
                sleep(delay).await;
            }
        }
    }
}
```

## Security Configuration

### API Key Management

```rust
// Use environment variables (recommended)
let api_key = std::env::var("OPENAI_API_KEY")
    .expect("OPENAI_API_KEY must be set");

// Or use a secure key management service
let api_key = get_secret_from_vault("openai-api-key").await?;
```

### Input Validation

```rust
use regex::Regex;

fn validate_input(input: &str) -> Result<(), Error> {
    // Check for malicious content
    let malicious_pattern = Regex::new(r"<script|javascript:|data:text/html").unwrap();
    
    if malicious_pattern.is_match(input) {
        return Err(Error::InvalidInput("Potentially malicious input detected".into()));
    }
    
    // Check length limits
    if input.len() > 10000 {
        return Err(Error::InvalidInput("Input too long".into()));
    }
    
    Ok(())
}
```

## Development vs Production

### Development Configuration

```bash
# .env.development
RUST_LOG=debug
RUST_BACKTRACE=1
OPENAI_API_KEY=sk-test-...
QDRANT_URL=http://localhost:6333
```

### Production Configuration

```bash
# .env.production
RUST_LOG=warn
OPENAI_API_KEY=sk-prod-...
QDRANT_URL=https://production-qdrant.com
LANGFUSE_PUBLIC_KEY=prod-key
LANGFUSE_SECRET_KEY=prod-secret
```

### Configuration Loading

```rust
use dotenv::dotenv;

fn load_config() {
    // Load environment-specific config
    let env = std::env::var("RUST_ENV").unwrap_or_else(|_| "development".to_string());
    
    match env.as_str() {
        "production" => dotenv::from_filename(".env.production").ok(),
        "staging" => dotenv::from_filename(".env.staging").ok(),
        _ => dotenv::from_filename(".env.development").ok(),
    };
}
```

## Monitoring Configuration

### Langfuse Integration

```rust
use ai_utils::langfuse::LangfuseService;

let langfuse = LangfuseService::new().await?;

// Track operations
langfuse.trace("document-processing", |trace| {
    trace.span("embedding-generation", |span| {
        // Generate embeddings
        span.end();
    });
    trace.end();
}).await?;
```

### Metrics Collection

```rust
use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Default)]
struct Metrics {
    requests: AtomicU64,
    errors: AtomicU64,
    latency: AtomicU64,
}

impl Metrics {
    pub fn increment_requests(&self) {
        self.requests.fetch_add(1, Ordering::Relaxed);
    }
    
    pub fn record_latency(&self, duration: Duration) {
        self.latency.store(duration.as_millis() as u64, Ordering::Relaxed);
    }
}
```

## Best Practices

### 1. Environment Management
- Never commit API keys to version control
- Use different keys for development and production
- Rotate keys regularly

### 2. Performance Tuning
- Monitor API usage and costs
- Implement appropriate caching strategies
- Use batch operations when possible

### 3. Error Handling
- Implement comprehensive error handling
- Use structured logging for debugging
- Set up monitoring and alerting

### 4. Security
- Validate all inputs
- Use HTTPS for all external communications
- Implement rate limiting
