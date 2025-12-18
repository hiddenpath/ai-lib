# Upgrade Guide: 0.3.x → 0.4.0

This guide helps you migrate projects from the 0.3.x line to 0.4.0. The main changes center on the **Trait Shift 1.0 Evolution**: a move from enum-based dispatch to a trait-driven architecture.

## Breaking Changes Summary

| Area | 0.3.x | 0.4.0 |
|------|-------|-------|
| Client creation | `AiClient::new(Provider::X)` could panic | Returns `Result<AiClient, AiLibError>` |
| Internal dispatch | Enum matching | `Box<dyn ChatProvider>` dynamic dispatch |
| Routing | `with_failover(vec![...])` + sentinel model | `with_failover_chain(vec![...])` strategy builders |

## Migration Steps

### 1. Handle `AiClient::new` Result

```rust
// ⛔️ Old (0.3.x) - Could panic or required unwrap
let client = AiClient::new(Provider::Groq);

// ✅ New (0.4.0) - Explicit error handling
let client = AiClient::new(Provider::Groq)?;
// or
let client = AiClient::new(Provider::Groq).expect("Failed to create client");
```

### 2. Update Routing/Failover Code

If you used `with_failover`, migrate to strategy builders:

```rust
// ⛔️ Old (0.3.x) - Sentinel-based failover
let client = AiClient::new(Provider::OpenAI)
    .with_failover(vec![Provider::Anthropic, Provider::Groq]);

// ✅ New (0.4.0) - Strategy builders
let client = AiClientBuilder::new(Provider::OpenAI)
    .with_failover_chain(vec![Provider::Anthropic, Provider::Groq])?
    .build()?;
```

For round-robin routing:

```rust
// ✅ New (0.4.0)
let client = AiClientBuilder::new(Provider::OpenAI)
    .with_round_robin_chain(vec![Provider::Groq, Provider::Mistral])?
    .build()?;
```

### 3. Import Path Changes

The prelude remains the recommended approach. Core types are unchanged:

```rust
// ✅ Recommended (unchanged)
use ai_lib::prelude::*;

// ✅ Direct imports still work
use ai_lib::{AiClient, Provider, ChatCompletionRequest};
```

### 4. Provider-Specific Parameters

Use `with_extension()` for provider-specific settings:

```rust
let request = ChatCompletionRequest::new(model, messages)
    .with_extension("reasoning_effort", serde_json::json!("high"));
```

## New Features in 0.4.0

### ChatProvider Trait

All providers now implement a unified `ChatProvider` trait:
- `chat()` - Single completion
- `stream()` - Streaming completion
- `batch()` - Batch processing
- `list_models()` - Available models

### Strategy Builders

Pre-runtime routing composition:
- `with_failover_chain(providers)` - Ordered failover
- `with_round_robin_chain(providers)` - Load distribution

### Custom Providers

Inject OpenAI-compatible endpoints without modifying the `Provider` enum:

```rust
let custom = CustomProviderBuilder::new("my-gateway")
    .with_base_url("https://gateway.example.com/v1")
    .with_api_key_env("MY_GATEWAY_KEY")
    .build_provider()?;

let client = AiClientBuilder::new(Provider::OpenAI)
    .with_strategy(custom)
    .build()?;
```

## Verification

After migration, verify your code compiles and runs:

```bash
cargo build
cargo test
```

## Need Help?

- [API Reference](https://docs.rs/ai-lib/0.4.0)
- [Examples](https://github.com/hiddenpath/ai-lib/tree/main/examples)
- [GitHub Issues](https://github.com/hiddenpath/ai-lib/issues)
