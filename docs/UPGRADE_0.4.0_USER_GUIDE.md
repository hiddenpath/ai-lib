---
title: Upgrade to 0.4.0
group: Guide
order: 5
description: Complete migration guide from ai-lib 0.3.x to 0.4.0
---

# Upgrade to ai-lib 0.4.0

This guide helps you migrate from ai-lib 0.3.x to 0.4.0. The major change is the **Trait Shift 1.0 Evolution**: moving from enum-based dispatch to a trait-driven architecture.

## Breaking Changes Overview

| Area | 0.3.x | 0.4.0 |
|------|-------|-------|
| Client creation | `AiClient::new(Provider::X)` could panic | Returns `Result<AiClient, AiLibError>` |
| Internal dispatch | Enum matching | `Box<dyn ChatProvider>` dynamic dispatch |
| Routing | `with_failover(vec![...])` + sentinel model | `with_failover_chain(vec![...])` strategy builders |

## Quick Migration Steps

### 1. Update Cargo.toml

```toml
[dependencies]
ai-lib = "0.4.0"
```

### 2. Handle Client Creation Errors

```rust
// ⛔️ Old (0.3.x) - Could panic
let client = AiClient::new(Provider::Groq);

// ✅ New (0.4.0) - Explicit error handling
let client = AiClient::new(Provider::Groq)?;
```

### 3. Update Routing Code

```rust
// ⛔️ Old (0.3.x) - Sentinel-based failover
let client = AiClient::new(Provider::OpenAI)
    .with_failover(vec![Provider::Anthropic, Provider::Groq]);

// ✅ New (0.4.0) - Strategy builders
let client = AiClientBuilder::new(Provider::OpenAI)
    .with_failover_chain(vec![Provider::Anthropic, Provider::Groq])?
    .build()?;
```

## Detailed Migration Guide

### Client Creation

**Before (0.3.x):**
```rust
use ai_lib::{AiClient, Provider};

let client = AiClient::new(Provider::Groq); // Could panic!
```

**After (0.4.0):**
```rust
use ai_lib::{AiClient, Provider};

let client = AiClient::new(Provider::Groq)?;
// or
let client = AiClient::new(Provider::Groq).expect("Failed to create client");
```

### Routing and Failover

**Before (0.3.x):**
```rust
// Sentinel-based routing
let client = AiClient::new(Provider::OpenAI)
    .with_failover(vec![Provider::Anthropic, Provider::Groq]);

let request = ChatCompletionRequest::new("__route__".to_string(), messages);
```

**After (0.4.0):**
```rust
// Strategy builders - Pre-compose at runtime
let client = AiClientBuilder::new(Provider::OpenAI)
    .with_failover_chain(vec![Provider::Anthropic, Provider::Groq])?
    .build()?;

let request = ChatCompletionRequest::new("gpt-4".to_string(), messages);
```

### Round-Robin Load Balancing

**New in 0.4.0:**
```rust
let client = AiClientBuilder::new(Provider::OpenAI)
    .with_round_robin_chain(vec![Provider::Groq, Provider::Mistral])?
    .build()?;
```

### Custom Provider Injection

**New in 0.4.0:**
```rust
use ai_lib::provider::builders::CustomProviderBuilder;

let custom = CustomProviderBuilder::new("my-gateway")
    .with_base_url("https://gateway.example.com/v1")
    .with_api_key_env("MY_GATEWAY_KEY")
    .with_default_chat_model("gpt-4")
    .build_provider()?;

let client = AiClientBuilder::new(Provider::OpenAI)
    .with_strategy(custom)
    .build()?;
```

### Provider-Specific Parameters

**New in 0.4.0:**
```rust
let request = ChatCompletionRequest::new(model, messages)
    .with_extension("reasoning_effort", serde_json::json!("high"))
    .with_extension("temperature", serde_json::json!(0.7));
```

## Import Changes

No major import changes. The prelude remains the recommended approach:

```rust
// ✅ Still works (recommended)
use ai_lib::prelude::*;

// ✅ Still works (direct imports)
use ai_lib::{AiClient, Provider, ChatCompletionRequest};
```

## New Features in 0.4.0

### ChatProvider Trait

All providers now implement a unified `ChatProvider` trait:
- `chat()` - Single completion
- `stream()` - Streaming completion
- `batch()` - Batch processing
- `list_models()` - Available models

### Enhanced Error Handling

All client operations now return `Result<T, AiLibError>` for better error handling.

### Improved Streaming

Unified SSE/JSONL parsing logic across all providers.

## Testing Your Migration

After updating your code:

```bash
# Build
cargo build

# Run tests
cargo test

# Check for any remaining issues
cargo check
```

## Common Issues & Solutions

### "Method not found" errors
- **Issue**: Old routing methods no longer exist
- **Solution**: Use `AiClientBuilder::with_failover_chain()` or `with_round_robin_chain()`

### Panics on client creation
- **Issue**: `AiClient::new()` now returns Result
- **Solution**: Add `?` or `.unwrap()` / `.expect()`

### Sentinel model errors
- **Issue**: `"__route__"` model no longer works
- **Solution**: Use strategy builders instead of sentinel models

## Need Help?

- [API Reference](https://docs.rs/ai-lib/0.4.0) - Complete API documentation
- [Examples](https://github.com/hiddenpath/ai-lib/tree/main/examples) - Updated examples for 0.4.0
- [GitHub Issues](https://github.com/hiddenpath/ai-lib/issues) - Report migration issues
- [Discord Community](https://discord.gg/ai-lib) - Get help from the community

## Verification Checklist

- [ ] Updated `Cargo.toml` to ai-lib = "0.4.0"
- [ ] Added `?` to `AiClient::new()` calls
- [ ] Replaced `with_failover()` with `with_failover_chain()`
- [ ] Removed `"__route__"` sentinel models
- [ ] Updated imports if needed
- [ ] Tested compilation with `cargo build`
- [ ] Ran tests with `cargo test`
