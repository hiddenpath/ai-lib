## Module tree and import patterns

This guide explains the public module layout, the recommended import styles, and when to choose each approach. It is designed to be skimmable and copy-paste friendly.

### Import strategies at a glance

- Preferred for applications: use the prelude for the minimal, ergonomic set.
- Convenient alternative: use top-level re-exports for common items.
- For library authors and advanced use: import from explicit domain modules.

### Recommended for applications: prelude

Use the prelude to get the 80% most used items without deep paths.

```rust
use ai_lib::prelude::*;

// Included in prelude:
// AiClient, AiClientBuilder, Provider
// ChatCompletionRequest, ChatCompletionResponse, Choice
// Content, Message, Role
// Usage, UsageStatus
// AiLibError
```

### Top-level re-exports (crate root)

Prefer these when you want explicit names but still avoid deep module paths.

```rust
use ai_lib::{AiClient, AiClientBuilder, Provider};
use ai_lib::{ChatCompletionRequest, ChatCompletionResponse};
use ai_lib::{Content, Message, Role};
use ai_lib::{Usage, UsageStatus};
```

### Explicit domain imports (for library authors)

Use module paths when you need fine-grained control, or when building libraries and frameworks that extend ai-lib.

```rust
use ai_lib::types::request::ChatCompletionRequest;
use ai_lib::types::response::{ChatCompletionResponse, Usage, UsageStatus};
use ai_lib::types::common::{Content, Message, Role};
use ai_lib::types::error::AiLibError;
```

### Errors and configuration

```rust
use ai_lib::types::error::AiLibError;
use ai_lib::config::ConnectionOptions;
```

### Streaming and tools

```rust
use ai_lib::api::ChatCompletionChunk;
use ai_lib::types::{FunctionCall, FunctionCallPolicy, Tool};
```

## Provider selection policy

- Application code should select providers via the unified enum and client:

```rust
let client = ai_lib::AiClient::new(ai_lib::Provider::OpenAI)?;
```

- Adapter modules exist but are considered internal for OSS usage. They are present primarily for tests and advanced scenarios. Favor `Provider` + `AiClient` over importing adapter types directly.

## Public module tree (overview)

High-level layout of the public surface:

- `ai_lib::prelude` — minimal, app-focused exports
- `ai_lib::types`
  - `types::common` — message `Content`, `Message`, `Role`, choices (includes multimodal content creation methods)
  - `types::request` — `ChatCompletionRequest`
  - `types::response` — `ChatCompletionResponse`, `Usage`, `UsageStatus`
  - `types::error` — `AiLibError`
  - `types::function_call` — function calling types
- `ai_lib::api` — streaming types like `ChatCompletionChunk`
- `ai_lib::client` — `AiClient`, `AiClientBuilder`, `Provider`
- `ai_lib::config` — `ConnectionOptions` and related settings
- `ai_lib::metrics` — metrics traits and helpers
- `ai_lib::transport` — HTTP and streaming transport abstractions
- `ai_lib::utils` — file utilities and helpers
- `ai_lib::provider` — provider adapters and configuration
  - `provider::config` — provider configuration types
  - `provider::configs` — provider configurations
  - `provider::pricing` — pricing information
  - `provider::classification` — provider classification
  - `provider::models` — model management (feature-gated)
  - `provider::utils` — **internal multimodal functionality** (file processing, content conversion, upload handling)
- `ai_lib::circuit_breaker` — circuit breaker functionality
- `ai_lib::rate_limiter` — rate limiting functionality
- `ai_lib::error_handling` — error handling and recovery

## Do and Don't

- Do: prefer `ai_lib::prelude::*` in applications for clarity and stability.
- Do: import from `ai_lib::{...}` (crate root) for common items when you want explicit names.
- Do: use `Content::from_image_file()` and `Content::from_audio_file()` for multimodal content creation.
- Don't: combine wildcard imports across domains (e.g. `types::{common::*, response::*}`) as it may create name collisions.
- Don't: directly import from `ai_lib::provider::utils` — use the public `Content` methods instead.

## Migration: `Usage` and `UsageStatus`

- Old location: `ai_lib::types::common::{Usage, UsageStatus}`
- New location: `ai_lib::types::response::{Usage, UsageStatus}`
- Also re-exported at the crate root: `ai_lib::{Usage, UsageStatus}`
- Deprecation policy: legacy aliases remain for a transition period and will be removed before 1.0. Please migrate to the new paths.

## IDE discoverability tips

- Search for types at the crate root first (`ai_lib::Usage`, `ai_lib::ChatCompletionRequest`).
- Response usage types are tagged with documentation aliases such as "token_usage" to improve search.

## Quick import cheat sheet

| Goal | Import |
|------|--------|
| Build a client and send a request | `use ai_lib::prelude::*;` |
| Explicit types without prelude | `use ai_lib::{AiClient, Provider, ChatCompletionRequest};` |
| Multimodal content creation | `use ai_lib::{Content, Message, Role};` (includes `Content::from_image_file()`, `Content::from_audio_file()`) |
| Access response usage metadata | `use ai_lib::{Usage, UsageStatus};` or `use ai_lib::types::response::{Usage, UsageStatus};` |
| Library/advanced: domain modules | `use ai_lib::types::{request::*, response::*, common::*, error::AiLibError};` |
| Streaming events | `use ai_lib::api::ChatCompletionChunk;` |
| Configuration overrides | `use ai_lib::config::ConnectionOptions;` |
