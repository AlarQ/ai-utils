# Task Context: ai-utils Refactoring

Session ID: 2025-02-20-refactoring
Created: 2025-02-20T00:00:00Z
Status: in_progress

## Current Request
Implement the refactoring plan from @docs/plans/refactoring-plan.md:
- Replace OpenAI module with OpenRouter (access to 200+ models via single API)
- Replace Langfuse module with OpenTelemetry-based telemetry
- Update Qdrant to use OpenRouter for embeddings and fix .unwrap() bugs
- Update error module and lib.rs with new feature gates

## Context Files (Standards to Follow)
- /Users/ernestbednarczyk/.config/opencode/context/core/standards/rust/rust-code-quality.md
- /Users/ernestbednarczyk/.config/opencode/context/core/standards/rust/rust-testing.md
- /Users/ernestbednarczyk/.config/opencode/context/core/standards/code-quality.md
- /Users/ernestbednarczyk/.config/opencode/context/core/workflows/feature-breakdown.md

## Reference Files (Source Material to Look At)
- /Users/ernestbednarczyk/Desktop/projects/ai-utils/Cargo.toml
- /Users/ernestbednarczyk/Desktop/projects/ai-utils/src/openai/mod.rs
- /Users/ernestbednarczyk/Desktop/projects/ai-utils/src/openai/types.rs
- /Users/ernestbednarczyk/Desktop/projects/ai-utils/src/openai/service.rs
- /Users/ernestbednarczyk/Desktop/projects/ai-utils/src/langfuse/mod.rs
- /Users/ernestbednarczyk/Desktop/projects/ai-utils/src/langfuse/types.rs
- /Users/ernestbednarczyk/Desktop/projects/ai-utils/src/langfuse/service.rs
- /Users/ernestbednarczyk/Desktop/projects/ai-utils/src/qdrant/qdrant_service.rs
- /Users/ernestbednarczyk/Desktop/projects/ai-utils/src/error/mod.rs
- /Users/ernestbednarczyk/Desktop/projects/ai-utils/src/lib.rs
- /Users/ernestbednarczyk/Desktop/projects/ai-utils/docs/plans/refactoring-plan.md

## External Docs Fetched
- OpenTelemetry Rust ecosystem: opentelemetry 0.28, opentelemetry_sdk 0.28, opentelemetry-otlp 0.28, tracing-opentelemetry 0.29
- async-openai will be used with custom base URL for OpenRouter

## Components
1. **Cargo.toml** - Add OTEL dependencies, update feature flags
2. **OpenRouter module** (src/openrouter/types.rs, service.rs, mod.rs) - New module replacing openai
3. **Error module** (src/error/mod.rs) - Rename variants, add new ones
4. **Telemetry module** (src/telemetry/types.rs, service.rs, mod.rs) - Replace langfuse with OTEL
5. **Qdrant module** (src/qdrant/qdrant_service.rs) - Decouple from OpenAI, fix unwraps
6. **lib.rs** - Update feature gates
7. **Cleanup** - Delete src/openai/ and src/langfuse/

## Constraints
- OTEL crates must use same generation (0.28.x) for compatibility
- Keep old openai/langfuse features temporarily until Phase 6 cleanup
- All .unwrap() calls must be replaced with proper error propagation
- Follow Rust code quality standards (no shared mutability, proper async patterns)
- Must pass cargo build --all-features, cargo clippy --all-features, cargo test --lib

## Exit Criteria
- [ ] cargo build --all-features compiles clean
- [ ] cargo clippy --all-features has zero warnings
- [ ] cargo fmt -- --check passes
- [ ] cargo test --lib passes
- [ ] Old openai/ and langfuse/ directories deleted
- [ ] lib.rs updated with new feature gates
