# AI Utils

A comprehensive Rust library for AI and machine learning utilities, including OpenAI integration, Qdrant vector database operations, Langfuse observability, and text processing.

## Prerequisites

- Rust (latest stable version)
- OpenAI API key
- Langfuse API keys (public and secret)

## Setup

1. Add the following to your `Cargo.toml`:
   ```toml
   [dependencies]
   ai_utils = { path = "path/to/ai_utils" }
   ```

2. Create a `.env` file in your project root with the following variables:
   ```
   OPENAI_API_KEY=your_openai_api_key
   LANGFUSE_SECRET_KEY=your_langfuse_secret_key
   LANGFUSE_PUBLIC_KEY=your_langfuse_public_key
   LANGFUSE_HOST=your_langfuse_host  # Optional
   NODE_ENV=development  # Optional, for debug mode
   ```

## Project Structure

The library is organized into the following modules:

- `common`: Shared utilities and common functionality
- `error`: Error handling and custom error types
- `langfuse`: Langfuse integration for monitoring and analytics
- `openai`: OpenAI API integration

## Features

This library supports feature flags to allow you to compile only the functionality you need, reducing binary size and dependencies.

### Available Features

- **`openai`** (default): OpenAI API integration for embeddings and chat completions
- **`qdrant`** (default): Qdrant vector database client and operations
- **`langfuse`** (default): Langfuse observability and tracing
- **`text-splitter`** (default): Text splitting and tokenization utilities
- **`full`**: All features enabled (same as default)

### Feature Usage

#### Minimal build (no AI features):
```toml
[dependencies]
ai_utils = { version = "0.1.0", default-features = false }
```

#### Only Qdrant (without OpenAI):
```toml
[dependencies]
ai_utils = { version = "0.1.0", default-features = false, features = ["qdrant"] }
```

#### Only OpenAI:
```toml
[dependencies]
ai_utils = { version = "0.1.0", default-features = false, features = ["openai"] }
```

#### Full build (all features):
```toml
[dependencies]
ai_utils = { version = "0.1.0", features = ["full"] }
```

### Feature-Specific API

When the `openai` feature is disabled, the Qdrant service provides alternative methods that work with pre-computed vectors:

```rust
// With OpenAI feature (default)
qdrant_service.upsert_point("collection", point).await?;
qdrant_service.search_points("collection", "query", 10).await?;

// Without OpenAI feature
qdrant_service.upsert_point_with_vector("collection", 1, vector, payload).await?;
qdrant_service.search_points_with_vector("collection", vector, 10).await?;
```

## Dependencies

The library uses the following main dependencies:
- tokio for async runtime
- async-openai for OpenAI API integration
- serde for serialization
- reqwest for HTTP requests
- uuid for unique identifier generation
- image for image processing
- chrono for datetime handling

## Usage

```rust
use ai_utils::{openai, langfuse, Result};

// Example usage will be added as the library matures
```

## Development

The project uses strict linting with clippy. To run the linter:

```bash
cargo clippy
```

## License

[Add your license information here] 