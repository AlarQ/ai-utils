# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

`ai_utils` is a Rust library (edition 2021) providing async service abstractions for OpenAI, Langfuse, Qdrant, and text splitting. All modules are feature-gated behind Cargo feature flags.

## Commands

```bash
cargo build                                    # Build with default features
cargo build --all-features                     # Build with all features
cargo build --no-default-features --features openai  # Build specific feature

cargo test                                     # Run all tests (integration tests need env vars)
cargo test --lib langfuse                      # Run tests for a specific module
cargo test --lib text_splitter::tests::test_name  # Run a single test

cargo clippy                                   # Lint (pedantic + nursery enabled)
cargo fmt                                      # Format code
cargo fmt -- --check                           # Check formatting

mdbook build                                   # Build docs (from repo root)
mdbook serve --open                            # Serve docs with live reload
```

## Feature Flags

Defined in `Cargo.toml`. All enabled by default:
- `openai` — OpenAI client via `async-openai`
- `qdrant` — Qdrant vector DB client (composes `OpenAIService` internally for embeddings)
- `langfuse` — Langfuse observability client via `reqwest`
- `text-splitter` — Markdown-aware text chunking via `tiktoken-rs`
- `full` — alias for all features

## Architecture

**Module structure:** Each module (`openai/`, `langfuse/`, `qdrant/`, `text_splitter/`) follows the same pattern: `mod.rs` with glob re-exports (`pub use service::*; pub use types::*;`), a `types.rs` for data structures, and a `service.rs` for the trait + implementation.

**Key patterns:**
- **Trait-based services**: `AIService` (openai) and `LangfuseService` (langfuse) are `async_trait` traits for mockability
- **Builder pattern**: `ChatRequestBuilder`, `Base64ImageBuilder` — consuming-builder style (methods take `self`, return `Self`)
- **Global `Result<T>`**: Defined in `lib.rs` as `Result<T, crate::error::Error>` — use this throughout
- **Error types**: Top-level `Error` enum in `src/error/` using `thiserror`; `CommonError` in `src/common/errors.rs`
- **Feature gating**: Module declarations in `lib.rs` use `#[cfg(feature = "...")]`

**Module dependencies:**
- `common` and `error` — standalone, no feature deps
- `openai` — depends on `error`
- `langfuse` — depends on `error`, references openai types
- `qdrant` — depends on `error`, embeds `OpenAIService` directly
- `text_splitter` — largely independent, uses `anyhow`

**Langfuse specifics:** Types use `#[allow(non_snake_case)]` to match the Langfuse REST API's camelCase field names. `IngestionEvent` uses `#[serde(untagged)]` with factory methods for batch ingestion.

## Coding Conventions

- **Clippy**: Both `clippy::pedantic` and `clippy::nursery` are enabled as warnings in `lib.rs` — all code must pass these strict lints
- **Formatting**: `rustfmt.toml` sets `imports_granularity = "Crate"` and `brace_style = "SameLineWhere"`
- **Async**: Use `tokio` runtime. Use `async_trait` for async trait definitions. Use `tokio::sync::Semaphore` for concurrency control
- **Error handling**: Use `thiserror` for custom error types, `?` for propagation
- **Tests**: Integration tests live inline in `mod.rs` files (not a separate `tests/` dir). Use `#[tokio::test]` for async tests. Integration tests gracefully skip via early `return` when env vars are missing
- **Naming**: snake_case for variables/functions, PascalCase for types/structs
- **Env vars**: Use `dotenv` crate. Config is read from environment at service construction time

## Environment Variables

Required per feature (create a `.env` file):
- **openai**: `OPENAI_API_KEY`
- **langfuse**: `LANGFUSE_PUBLIC_KEY`, `LANGFUSE_SECRET_KEY`, optionally `LANGFUSE_HOST`
- **qdrant**: `QDRANT_URL`, `QDRANT_API_KEY`
- **common**: optionally `IMAGE_PROCESSING_CONCURRENCY` (default: 4)
