# Upgrade Guide: ai-lib v0.4.0 → v0.5.0

This document outlines the changes in ai-lib v0.5.0 and how to migrate your code.

## Overview

v0.5.0 introduces a **Manifest-First Data-Driven Architecture** where all model and provider configurations are loaded from YAML manifest files. This enables:

- **Zero-code provider support**: Add new AI providers with only YAML configuration
- **Operator-based streaming**: Unified event handling without provider-specific code branches
- **Runtime hot-reload**: Update configurations while your application runs
- **2025-ready features**: Tool calling, multimodal, agentic loops, streaming events

## Core Architecture Change

```
Before (v0.4.0):  Provider enum → Hardcoded adapter → API
After (v0.5.0):   YAML Manifest → ConfigDrivenAdapter → API
```

The manifest defines three layers:
1. **`standard_schema`**: Unified parameter definitions (temperature, max_tokens, tools, etc.)
2. **`providers`**: Provider-specific mappings and streaming configurations
3. **`models`**: Concrete model instances with capabilities and pricing

## Breaking Changes

**None for typical usage.** v0.5.0 maintains full backward compatibility with v0.4.0 code.

## New Features

### 1. Manifest-Driven Client Creation

```rust
use ai_lib::client::ManifestClient;
use ai_lib::manifest::ManifestLoader;
use std::sync::Arc;

// Load manifest
let manifest = Arc::new(ManifestLoader::load_from_file("aimanifest.yaml")?);

// Create client
let mut client = ManifestClient::new(manifest);
client.select_model("gpt-4o")?;

// Use unified API
let response = client.chat(request).await?;
```

### 2. Model-Driven Builder (Recommended)

```rust
// v0.4.0 style (still works)
let client = AiClient::new(Provider::OpenAI)?;

// v0.5.0 style (recommended)
let client = AiClientBuilder::new(Provider::OpenAI)
    .with_model("gpt-4o")
    .build()?;
```

### 3. Operator-Based Streaming

Streaming events are now parsed using manifest-defined operators:

```yaml
# aimanifest.yaml - OpenAI streaming config
providers:
  openai:
    streaming:
      event_map:
        - match: "exists($.choices[*].delta.content)"
          emit: "PartialContentDelta"
          fields:
            content: "$.choices[*].delta.content"
        - match: "exists($.choices[*].delta.tool_calls)"
          emit: "PartialToolCall"
          fields:
            arguments: "$.choices[*].delta.tool_calls[*].function.arguments"
```

```rust
// Unified streaming events - no provider-specific handling needed
while let Some(event) = stream.next().await {
    match event? {
        StreamingEvent::PartialContentDelta(delta) => print!("{}", delta.delta),
        StreamingEvent::PartialToolCall(tool) => handle_tool(tool),
        StreamingEvent::ThinkingDelta(thinking) => log_thinking(thinking),
        _ => {}
    }
}
```

### 4. Hot Reload (Feature-Gated)

```toml
# Cargo.toml
[dependencies]
ai-lib = { version = "0.5", features = ["config_hot_reload"] }
```

```rust
use ai_lib::registry::watcher::FileWatcher;

// Watch manifest file for changes
let watcher = FileWatcher::new("aimanifest.yaml".into())?;
watcher.spawn(); // Updates GLOBAL_REGISTRY automatically
```

### 5. Capability Pre-flight Validation

```rust
// Automatic validation before API calls
let result = client.chat(request_with_tools).await;
// Returns Err(UnsupportedFeature) if model doesn't support tools
```

## Deprecation Notices

### GenericAdapter

`GenericAdapter` is deprecated in favor of `ConfigDrivenAdapter`:

```rust
// Deprecated (still works but shows warning)
use ai_lib::provider::GenericAdapter;

// Recommended
use ai_lib::adapter::dynamic::ConfigDrivenAdapter;
```

### JSON Configuration

The `models.json` configuration format is deprecated. Use `aimanifest.yaml` instead:

```yaml
# aimanifest.yaml structure
version: "1.1"
standard_schema:
  parameters:
    temperature: { type: float, range: [0.0, 2.0] }
    max_tokens: { type: integer, max: 32768 }
providers:
  openai:
    base_url: "https://api.openai.com/v1"
    auth: { type: bearer, token_env: "OPENAI_API_KEY" }
    # ... streaming, event_map, etc.
models:
  gpt-4o:
    provider: openai
    context_window: 128000
    capabilities: [chat, vision, tools, streaming]
```

## Migration Checklist

- [ ] Update `Cargo.toml` to v0.5.0
- [ ] (Optional) Create custom `aimanifest.yaml` for additional providers
- [ ] (Optional) Migrate to `ManifestClient` or `with_model()` pattern
- [ ] (Optional) Enable `config_hot_reload` for dynamic updates
- [ ] Test that existing code works without changes

## FAQ

**Q: Will my v0.4.0 code break?**
A: No. All existing APIs remain compatible.

**Q: Where is the default manifest?**
A: `aimanifest.yaml` is embedded at compile time. For runtime overrides, use `ManifestLoader::load_from_file()`.

**Q: How do I add a custom provider?**
A: Add a new entry under `providers:` in your YAML manifest with the required fields (base_url, auth, streaming, etc.). No Rust code changes needed.

**Q: What about Groq, DeepSeek, and other OpenAI-compatible services?**
A: They use `payload_format: openai_style` and inherit OpenAI's streaming behavior automatically. Just specify the correct `base_url` and `token_env`.
