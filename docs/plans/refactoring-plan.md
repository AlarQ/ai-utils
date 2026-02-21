# ai-utils Refactoring Plan

## Context

The ai-utils library currently wraps OpenAI, Langfuse, Qdrant, and text splitting behind feature flags. After evaluating the Rust ecosystem:

- **OpenAI module** adds limited value over `async-openai` and should be replaced with **OpenRouter** (access to 200+ models via a single API)
- **Langfuse module** is a hand-rolled REST client; Langfuse now supports **OpenTelemetry** natively, so we switch to the standard OTEL stack
- **Qdrant, text_splitter, common** modules stay as-is (unique value, no better alternatives)

## Summary of Changes

| Module | Action |
|--------|--------|
| `src/openai/` | **Delete** - replace with `src/openrouter/` |
| `src/langfuse/` | **Delete** - replace with `src/telemetry/` |
| `src/qdrant/` | **Update** - swap OpenAI→OpenRouter dependency, fix `.unwrap()` bugs |
| `src/text_splitter/` | No changes |
| `src/common/` | No changes |
| `src/error/` | **Update** - rename variants, add new ones |
| `src/lib.rs` | **Update** - new feature gates |
| `Cargo.toml` | **Update** - new deps and features |

---

## Phase 1: Cargo.toml — Add New Dependencies and Features

**File:** `Cargo.toml`

Add dependencies:
```toml
# OTEL stack (all optional, gated behind "telemetry" feature)
opentelemetry = { version = "0.28", optional = true }
opentelemetry_sdk = { version = "0.28", features = ["rt-tokio"], optional = true }
opentelemetry-otlp = { version = "0.28", features = ["http-proto"], optional = true }
tracing-opentelemetry = { version = "0.29", optional = true }
tracing-subscriber = { version = "0.3", features = ["registry"], optional = true }
```

> Note: Pin exact OTEL generation during implementation — these crates have strict cross-version coupling. Verify compatible versions from [opentelemetry-rust releases](https://github.com/open-telemetry/opentelemetry-rust/releases).

Update features:
```toml
[features]
default = ["openrouter", "qdrant", "telemetry", "text-splitter"]
openrouter = ["async-openai"]
telemetry = ["opentelemetry", "opentelemetry_sdk", "opentelemetry-otlp", "tracing-opentelemetry", "tracing-subscriber"]
qdrant = ["qdrant-client", "openrouter"]
text-splitter = ["tiktoken-rs"]
full = ["openrouter", "qdrant", "telemetry", "text-splitter"]
```

Keep old `openai` and `langfuse` features temporarily until Phase 6 cleanup.

---

## Phase 2: OpenRouter Module (new `src/openrouter/`)

Uses `async-openai` with custom base URL (`https://openrouter.ai/api/v1`) + `reqwest` for OpenRouter-specific endpoints.

### `src/openrouter/types.rs`

**Keep from current openai/types.rs** (domain types, not provider-specific):
- `Message`, `MessageRole`, `MessageContent`, `ContentPart`, `ImageUrl`
- `ChatCompletion`, `Choice`, `Usage`
- `ChatRequestBuilder` (adapted)

**Replace:**
- `OpenAIModel` enum → `ModelId` newtype over `String` with well-known constants:
  ```rust
  pub struct ModelId(pub String);
  impl ModelId {
      pub const GPT_4O: &'static str = "openai/gpt-4o";
      pub const CLAUDE_SONNET_4: &'static str = "anthropic/claude-sonnet-4";
      // etc.
      pub fn custom(id: impl Into<String>) -> Self { ... }
  }
  ```
- `ChatOptions` — add `openrouter: Option<OpenRouterOptions>` field

**Add:**
- `OpenRouterOptions` — provider preferences, route, transforms
- `ModelInfo` — for `/api/v1/models` response
- `ModelPricing` — prompt/completion cost per token
- `KeyInfo` — for `/api/v1/auth/key` response (credits, limits, rate limits)

**Remove:**
- All legacy types: `OpenAIMessage`, `OpenAIImageMessage`, `OpenAIImageGenMessage`, `ImageType`, `OpenAiError`
- Model capability guards (`supports_chat`, `supports_vision`, etc.) — OpenRouter handles routing

### `src/openrouter/service.rs`

```rust
pub struct OpenRouterService {
    client: Client<OpenAIConfig>,    // async-openai pointed at OpenRouter
    http_client: reqwest::Client,    // for OpenRouter-specific endpoints
    api_base: String,
}
```

**Construction** (`new()`):
- Read `OPENROUTER_API_KEY` from env (required)
- Read `OPENROUTER_SITE_URL` and `OPENROUTER_SITE_NAME` from env (optional, for attribution)
- Build `reqwest::Client` with default headers: `HTTP-Referer`, `X-Title`
- Build `OpenAIConfig::new().with_api_key().with_api_base("https://openrouter.ai/api/v1")`
- Use `Client::with_config(config).with_http_client(http_client)` — confirmed supported by async-openai

**`AIService` trait** (updated):
```rust
#[async_trait]
pub trait AIService: Send + Sync {
    async fn chat(&self, messages: Vec<Message>, options: ChatOptions) -> Result<ChatCompletion>;
    async fn embed(&self, text: String) -> Result<Vec<f32>>;
    async fn embed_batch(&self, texts: Vec<String>) -> Result<Vec<Vec<f32>>>;
    async fn list_models(&self) -> Result<Vec<ModelInfo>>;
    async fn key_info(&self) -> Result<KeyInfo>;
}
```

- `chat()` — delegates to async-openai `chat().create()` after converting messages. OpenRouter-specific options (provider prefs) injected via extra body fields if needed, otherwise via reqwest fallback
- `embed()` / `embed_batch()` — delegates to async-openai embeddings endpoint via OpenRouter
- `list_models()` — `GET {api_base}/models` via reqwest, deserialize into `Vec<ModelInfo>`
- `key_info()` — `GET {api_base}/auth/key` via reqwest, deserialize into `KeyInfo`
- Remove `generate_image_url()` and `transcribe()` from trait (not supported by OpenRouter)
- Remove deprecated `completion()` method

**Message conversion** — adapt `convert_message_to_openai` (rename to `convert_message`), keep the same logic mapping `Message` → async-openai request types.

### `src/openrouter/mod.rs`

Glob re-exports. Integration tests with early return when `OPENROUTER_API_KEY` is missing.

---

## Phase 3: Error Module Update

**File:** `src/error/mod.rs`

Rename variants:
- `OpenAI(...)` → `OpenRouter(#[from] async_openai::error::OpenAIError)`
- `OpenAIValidation(String)` → `Validation(String)`
- `OpenAIRateLimited { .. }` → `RateLimited { retry_after: Option<Duration> }`
- `OpenAIMissingParameter { .. }` → `MissingParameter { param: String }`
- `OpenAIUnsupportedModel { .. }` → **Remove** (OpenRouter doesn't need this)
- `Langfuse(String)` → `Telemetry(String)`

Add:
- `InvalidHeader(#[from] reqwest::header::InvalidHeaderValue)` — for header construction

---

## Phase 4: Telemetry Module (new `src/telemetry/`)

Replaces `src/langfuse/` with standard OpenTelemetry pipeline.

### `src/telemetry/types.rs`

```rust
pub struct TelemetryConfig {
    pub endpoint: String,         // OTEL_EXPORTER_OTLP_ENDPOINT env var
    pub service_name: String,     // defaults to "ai-utils"
    pub auth_header: Option<String>, // For Langfuse: base64("public_key:secret_key")
}

pub struct TelemetryGuard {
    // Holds the provider for graceful shutdown
}
```

`TelemetryConfig::from_env()` reads:
- `OTEL_EXPORTER_OTLP_ENDPOINT` (required)
- `OTEL_SERVICE_NAME` (optional, default "ai-utils")
- `LANGFUSE_PUBLIC_KEY` + `LANGFUSE_SECRET_KEY` (optional — if both set, builds Basic Auth header)

### `src/telemetry/service.rs`

```rust
/// Initialize OTEL tracing pipeline. Call once at startup.
pub fn init_telemetry(config: TelemetryConfig) -> Result<TelemetryGuard>;

/// Helper to create an LLM-annotated span
pub fn llm_span(operation: &str, model: &str) -> tracing::Span;
```

- `init_telemetry()` builds OTLP HTTP exporter → TracerProvider → bridges with `tracing` via `tracing-opentelemetry` layer
- No Langfuse-specific types — callers use standard `tracing` macros
- `TelemetryGuard::shutdown()` for graceful flush on app exit

### `src/telemetry/mod.rs`

Re-exports. Unit test with in-memory/no-op exporter (no network).

---

## Phase 5: Qdrant Module Update

**File:** `src/qdrant/qdrant_service.rs`

### 5a. Decouple from OpenAI

Define a minimal embedding trait:
```rust
#[async_trait]
pub trait EmbeddingService: Send + Sync {
    async fn embed(&self, text: String) -> crate::Result<Vec<f32>>;
}
```

`OpenRouterService` implements this (it already has `embed()` via `AIService`).

Update `QdrantService`:
```rust
pub struct QdrantService {
    client: Qdrant,
    embedding_service: Box<dyn EmbeddingService>,
}

impl QdrantService {
    pub fn new() -> Result<Self> { ... }  // uses OpenRouterService by default
    pub fn with_embedding_service(service: Box<dyn EmbeddingService>) -> Result<Self> { ... }
}
```

### 5b. Fix `.unwrap()` calls

Replace all 5 `.unwrap()` calls with proper `?` error propagation:
- `self.openai_service.embed(...)` → `self.embedding_service.embed(...).await?`
- `.as_object().unwrap()` → `.as_object().ok_or_else(|| Error::Other(...))?`
- `point.id.parse::<u64>().unwrap()` → `.parse().map_err(|e| Error::Other(...))?`
- Search points `.await.unwrap()` → `.await?`

---

## Phase 6: Cleanup

1. Update `src/lib.rs`:
   - Remove `#[cfg(feature = "openai")] pub mod openai;`
   - Remove `#[cfg(feature = "langfuse")] pub mod langfuse;`
   - Add `#[cfg(feature = "openrouter")] pub mod openrouter;`
   - Add `#[cfg(feature = "telemetry")] pub mod telemetry;`

2. Delete old directories:
   - `src/openai/` (3 files)
   - `src/langfuse/` (3 files)

3. Remove from `Cargo.toml`:
   - Old `openai = [...]` and `langfuse = [...]` feature entries

---

## Environment Variables (new)

| Variable | Required | Used by |
|----------|----------|---------|
| `OPENROUTER_API_KEY` | Yes (for openrouter) | `OpenRouterService` |
| `OPENROUTER_SITE_URL` | No | Attribution header |
| `OPENROUTER_SITE_NAME` | No | Attribution header |
| `OTEL_EXPORTER_OTLP_ENDPOINT` | Yes (for telemetry) | `TelemetryConfig` |
| `OTEL_SERVICE_NAME` | No | `TelemetryConfig` |
| `LANGFUSE_PUBLIC_KEY` | No | OTEL auth to Langfuse |
| `LANGFUSE_SECRET_KEY` | No | OTEL auth to Langfuse |
| `QDRANT_URL` | Yes (for qdrant) | `QdrantService` |
| `QDRANT_API_KEY` | Yes (for qdrant) | `QdrantService` |

---

## Verification

1. `cargo build --all-features` — must compile clean
2. `cargo clippy --all-features` — pedantic + nursery, zero warnings
3. `cargo fmt -- --check` — formatting check
4. `cargo test --lib` — unit tests pass
5. Manual integration test with `OPENROUTER_API_KEY` set:
   - Chat completion with a model (e.g., `openai/gpt-4o-mini`)
   - Embedding generation
   - `list_models()` returns non-empty list
   - `key_info()` returns valid response
6. Manual Qdrant test with `QDRANT_URL`, `QDRANT_API_KEY`, `OPENROUTER_API_KEY`:
   - Upsert and search points work through OpenRouter embeddings
7. Telemetry test: initialize with a local OTEL collector or Langfuse OTEL endpoint, verify spans arrive

## Execution Order

Phases 1→2→3→4→5→6, strictly sequential. The codebase should compile after each phase (old and new modules coexist until Phase 6 cleanup).

## Key Technical Risks

1. **OTEL version coupling** — `opentelemetry`, `opentelemetry_sdk`, `opentelemetry-otlp`, `tracing-opentelemetry` must all be from the same release generation. Pin versions carefully.
2. **OpenRouter provider preferences** — `async-openai` doesn't natively support OpenRouter's `provider` field in request body. May need to fall back to raw reqwest for requests that use provider routing.
3. **Breaking API change** — Consumers of `LangfuseService` trait must migrate to `tracing` spans. This is intentional.
