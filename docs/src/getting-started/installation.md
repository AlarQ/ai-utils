# Installation

This guide will help you set up AI Utils in your Rust project.

## Prerequisites

- Rust 1.70+ (latest stable recommended)
- Cargo package manager
- API keys for external services

## Adding to Your Project

### 1. Add Dependency

Add AI Utils to your `Cargo.toml`:

```toml
[dependencies]
ai_utils = "0.1.0"
tokio = { version = "1.45.1", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
```

### 2. Install mdBook (for documentation)

```bash
cargo install mdbook
```

## Environment Configuration

### Required Environment Variables

Create a `.env` file in your project root:

```bash
# OpenAI Configuration
OPENAI_API_KEY=your-openai-api-key

# Qdrant Vector Database
QDRANT_URL=https://your-qdrant-instance.com
QDRANT_API_KEY=your-qdrant-api-key

# Langfuse Monitoring (Optional)
LANGFUSE_PUBLIC_KEY=your-langfuse-public-key
LANGFUSE_SECRET_KEY=your-langfuse-secret-key
```

### Getting API Keys

#### OpenAI API Key
1. Visit [OpenAI Platform](https://platform.openai.com/)
2. Create an account or sign in
3. Navigate to API Keys section
4. Create a new API key
5. Copy the key to your `.env` file

#### Qdrant Setup
1. **Cloud Option**: Sign up at [Qdrant Cloud](https://cloud.qdrant.io/)
2. **Self-hosted**: Follow [Qdrant installation guide](https://qdrant.tech/documentation/guides/installation/)
3. Get your API key and endpoint URL

#### Langfuse (Optional)
1. Visit [Langfuse](https://langfuse.com/)
2. Create an account
3. Get your public and secret keys from the dashboard

## Verification

Test your installation with a simple example:

```rust
use ai_utils::openai::OpenAIService;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();
    
    let openai = OpenAIService::new();
    let embeddings = openai.embed("Hello, world!".to_string()).await?;
    
    println!("Successfully generated {} embeddings", embeddings.len());
    Ok(())
}
```

Run with:
```bash
cargo run
```

## Next Steps

- [Quick Start Guide](quick-start.md) - Build your first AI agent
- [Configuration](configuration.md) - Learn about advanced configuration options
- [Examples](../examples/) - See real-world usage patterns
