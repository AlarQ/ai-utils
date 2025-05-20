# Langfuse Rust Chat API

This is a Rust implementation of a chat API that integrates with OpenAI and Langfuse for monitoring and analytics.

## Prerequisites

- Rust (latest stable version)
- OpenAI API key
- Langfuse API keys (public and secret)

## Setup

1. Clone the repository
2. Create a `.env` file in the project root with the following variables:
   ```
   OPENAI_API_KEY=your_openai_api_key
   LANGFUSE_SECRET_KEY=your_langfuse_secret_key
   LANGFUSE_PUBLIC_KEY=your_langfuse_public_key
   LANGFUSE_HOST=your_langfuse_host  # Optional
   NODE_ENV=development  # Optional, for debug mode
   ```

## Running the Server

```bash
cargo run
```

The server will start on `http://localhost:3000`.

## API Endpoints

### POST /api/chat

Send a chat request with the following JSON body:

```json
{
  "messages": [
    {
      "role": "user",
      "content": "Your message here"
    }
  ],
  "conversation_id": "optional-conversation-id"
}
```

The response will include three completions:

```json
{
  "completion": "Main completion response",
  "completion2": "Secondary completion response",
  "completion3": "Third completion response",
  "conversation_id": "conversation-id"
}
```

## Features

- Integration with OpenAI's GPT models
- Langfuse integration for monitoring and analytics
- Conversation tracking with unique IDs
- Multiple completion generation
- Error handling and logging 