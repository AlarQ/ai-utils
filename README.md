# AI Utils

A Rust library that provides utilities for working with OpenAI and Langfuse, including monitoring and analytics capabilities.

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

- Integration with OpenAI's GPT models
- Langfuse integration for monitoring and analytics
- Error handling with custom error types
- Async/await support
- Image processing capabilities
- UUID generation and management
- JSON serialization/deserialization

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