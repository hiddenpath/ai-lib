# ai-lib 🦀✨  

> A unified, reliable, high-performance multi-provider AI SDK for Rust

A production-grade, provider-agnostic SDK that provides a unified Rust API for 20+ AI platforms and growing (OpenAI, Groq, Anthropic, Gemini, Mistral, Cohere, Azure OpenAI, Ollama, DeepSeek, Qwen, Baidu ERNIE, Tencent Hunyuan, iFlytek Spark, Kimi, HuggingFace, TogetherAI, xAI Grok, OpenRouter, Replicate, Perplexity, AI21, ZhipuAI, MiniMax, and more).  
Eliminates fragmented authentication flows, streaming formats, error semantics, model naming differences, and inconsistent function calling. Scale from one-liner scripts to production systems without rewriting integration code.

---
[Official Website](https://www.ailib.info/)

## 🚀 Core Value

ai-lib unifies AI provider complexity into a single, ergonomic Rust interface:

- **Universal API**: Chat, multimodal, and function calling across all providers
- **Multimodal Content**: Easy image and audio content creation with `Content::from_image_file()` and `Content::from_audio_file()`
- **Unified Streaming**: Consistent SSE/JSONL parsing with real-time deltas
- **Reliability**: Built-in retry, timeout, circuit breaker, and error classification
- **Flexible Configuration**: Environment variables, builder pattern, or explicit overrides
- **Production Ready**: Connection pooling, proxy support, observability hooks

**Result**: Focus on your product logic while ai-lib handles provider integration friction.

> Import guidance: In application code, prefer `use ai_lib::prelude::*;` for a minimal set of common items. Library authors may use explicit imports by domain. See the module tree and import patterns guide: `docs/MODULE_TREE_AND_IMPORTS.md`.

## ⚙️ Quick Start

### Installation

Basic installation (core features):
```toml
[dependencies]
ai-lib = "0.4.0"
tokio = { version = "1", features = ["full"] }
futures = "0.3"
```

For streaming support, enable the `streaming` feature:
```toml
[dependencies]
ai-lib = { version = "0.4.0", features = ["streaming"] }
tokio = { version = "1", features = ["full"] }
futures = "0.3"
```

For full functionality (streaming, resilience, routing):
```toml
[dependencies]
ai-lib = { version = "0.4.0", features = ["all"] }
tokio = { version = "1", features = ["full"] }
futures = "0.3"
```

### Simple Usage
```rust
use ai_lib::prelude::*;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = AiClient::new(Provider::Groq)?;
    let req = ChatCompletionRequest::new(
        "llama3-8b-8192".to_string(),
        vec![Message::user("Hello!")]
    );
    let reply = client.chat_completion(req).await?;
    println!("Reply: {}", reply.first_text().unwrap_or_default());
    Ok(())
}
```

### Standard Usage
```rust
// Application code can also use the prelude for minimal imports
use ai_lib::prelude::*;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = AiClient::new(Provider::OpenAI)?;
    let req = ChatCompletionRequest::new(
        "gpt-3.5-turbo".to_string(), // Explicit model or use client.default_chat_model()
        vec![Message {
            role: Role::User,
            content: Content::Text("Explain Rust ownership in one sentence.".to_string()),
            function_call: None,
        }],
    );
    // .with_extension("parallel_tool_calls", serde_json::json!(true)); // Optional extensions

    let resp = client.chat_completion(req).await?;
    println!("Answer: {}", resp.choices[0].message.content.as_text());
    Ok(())
}
```

### Streaming

> **Note:** Streaming requires the `streaming` feature (or `all` feature) to be enabled.

```rust
use ai_lib::prelude::*;
use futures::StreamExt;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = AiClient::new(Provider::OpenAI)?;
    let req = ChatCompletionRequest::new(
        "gpt-3.5-turbo".to_string(),
        vec![Message::user("Tell me a short story")]
    );

    let mut stream = client.chat_completion_stream(req).await?;
    while let Some(chunk) = stream.next().await {
        let c = chunk?;
        if let Some(delta) = c.choices.get(0).and_then(|ch| ch.delta.content.clone()) {
            print!("{delta}");
        }
    }
    Ok(())
}
```

---

## 🧠 Core Concepts

| Concept | Purpose |
|--------|---------|
| **Provider** | Enumerates all supported AI providers |
| **AiClient** | Main entry point with unified interface |
| **ChatCompletionRequest** | Standardized request payload |
| **Message / Content** | Text, image, audio content types |
| **Streaming Event** | Provider-standardized delta streams |
| **ConnectionOptions** | Runtime configuration overrides |
| **Metrics Trait** | Custom observability integration |
| **Transport** | Injectable HTTP + streaming layer |
| **Usage / UsageStatus** | Response-level usage metadata (tokens + status). Import from `ai_lib::Usage` or `ai_lib::types::response::Usage` |

---

## 💡 Key Features

### Core Capabilities
- **Unified Provider Abstraction**: Single API across all providers
- **Universal Streaming**: Consistent SSE/JSONL parsing with real-time deltas
- **Multimodal Support**: Text, image, and audio content handling
- **Function Calling**: Consistent tool patterns and OpenAI compatibility
- **Batch Processing**: Sequential and concurrent processing strategies

### Reliability & Production
- **Built-in Resilience**: Retry with exponential backoff, circuit breakers
- **Strategy Builders**: `AiClientBuilder::with_round_robin_chain` / `with_failover_chain` compose routing before runtime
- **Error Classification**: Distinguish transient vs permanent failures
- **Connection Management**: Pooling, timeouts, proxy support
- **Observability**: Pluggable metrics and tracing integration
- **Security**: No sensitive content logging by default

---

## 📈 Upgrading from 0.3.x to 0.4.0

**Major changes in 0.4.0**: Trait Shift 1.0 Evolution - moved from enum-based to trait-driven architecture.

- `AiClient::new()` now returns `Result<AiClient, AiLibError>` (no more panics)
- Routing: `with_failover(vec![...])` → `AiClientBuilder::with_failover_chain(vec![...])`
- Removed sentinel model `"__route__"` - use strategy builders instead
- New `ChatProvider` trait unifies all provider implementations

**📖 [Complete Upgrade Guide](docs/UPGRADE_0.4.0_USER_GUIDE.md) | [中文升级指南](docs/UPGRADE_0.4.0_USER_GUIDE_CN.md)**

---

## 🌍 Supported Providers

*17+ providers and growing* - We continuously add new AI platforms to support the evolving ecosystem.

| Provider | Streaming | Highlights |
|----------|-----------|------------|
| **Groq** | ✅ | Ultra-low latency inference |
| **OpenAI** | ✅ | GPT models, function calling |
| **Anthropic** | ✅ | Claude models, high quality |
| **Google Gemini** | ✅ | Multimodal capabilities |
| **Mistral** | ✅ | European models |
| **Cohere** | ✅ | RAG-optimized |
| **HuggingFace** | ✅ | Open source models |
| **TogetherAI** | ✅ | Cost-effective inference |
| **OpenRouter** | ✅ | Unified gateway, multi-provider model routing |
| **Replicate** | ✅ | Hosted OSS models gateway |
| **DeepSeek** | ✅ | Reasoning models |
| **Qwen** | ✅ | Chinese ecosystem |
| **Baidu ERNIE** | ✅ | Enterprise China |
| **Tencent Hunyuan** | ✅ | Cloud integration |
| **iFlytek Spark** | ✅ | Voice + multimodal |
| **Moonshot Kimi** | ✅ | Long context |
| **Azure OpenAI** | ✅ | Enterprise compliance |
| **Ollama** | ✅ | Local/air-gapped |
| **xAI Grok** | ✅ | Real-time oriented |
| **Perplexity** | ✅ | Search-augmented chat |
| **AI21** | ✅ | Jurassic models |
| **ZhipuAI (GLM)** | ✅ | China GLM series |
| **MiniMax** | ✅ | China multimodal |

*See [examples/](examples/) for provider-specific usage patterns.*

### Gateway Providers
ai-lib supports gateway providers like OpenRouter and Replicate that provide unified access to multiple AI models. Gateway platforms use `provider/model` format for model naming (e.g., `openai/gpt-4o`), while direct providers use original model names (e.g., `gpt-4o`).

---

## 🔑 Configuration

### Environment Variables
```bash
# API Keys (convention-based)
export OPENAI_API_KEY=...
export GROQ_API_KEY=...
export GEMINI_API_KEY=...
export ANTHROPIC_API_KEY=...
export OPENROUTER_API_KEY=...
export REPLICATE_API_TOKEN=...
export PERPLEXITY_API_KEY=...
export AI21_API_KEY=...
export ZHIPU_API_KEY=...
export MINIMAX_API_KEY=...

# Optional: Custom endpoints
export GROQ_BASE_URL=https://custom.groq.com

# Optional: Proxy and timeouts
export AI_PROXY_URL=http://proxy.internal:8080
export AI_TIMEOUT_SECS=30

# Optional: Connection pooling (enabled by default)
export AI_HTTP_POOL_MAX_IDLE_PER_HOST=32
export AI_HTTP_POOL_IDLE_TIMEOUT_MS=90000

# Optional: Override default models per provider
export GROQ_MODEL=llama-3.1-8b-instant
export MISTRAL_MODEL=mistral-small-latest
export DEFAULT_AI_MODEL=gpt-4o-mini
```

### Model Selection & Fallbacks

- **Auto defaults**: Pass `"auto"` (case-insensitive) or an empty string as the model when
  constructing `ChatCompletionRequest` and ai-lib will inject the provider’s preferred
  model (or your `AiClientBuilder::with_default_chat_model` override).
- **Env overrides**: Set `*_MODEL` environment variables (e.g. `GROQ_MODEL`, `OPENAI_MODEL`)
  to change defaults without touching code. These overrides feed the new `ModelResolver`
  and apply across builders, streaming, and batch flows.
- **Invalid-model recovery**: When a provider rejects a request with
  `invalid_model`/`model_not_found`, ai-lib now retries with documented fallbacks and
  surfaces actionable hints (including links such as [Groq Models](https://console.groq.com/docs/models))
  in the returned `AiLibError::ModelNotFound`.
- **Per-provider context**: Call `client.default_chat_model()` to inspect the effective
  model string that will be used for the next request—handy when composing multi-provider
  strategies like failover chains.

### Programmatic Configuration
```rust
use ai_lib::{AiClient, Provider, ConnectionOptions};
use std::time::Duration;

let client = AiClient::with_options(
    Provider::Groq,
    ConnectionOptions {
        base_url: Some("https://custom.groq.com".into()),
        proxy: Some("http://proxy.internal:8080".into()),
        api_key: Some("override-key".into()),
        timeout: Some(Duration::from_secs(45)),
        disable_proxy: false,
    }
)?;
```

## 🔌 Bring Your Own Provider

Use `CustomProviderBuilder` + `AiClientBuilder::with_strategy` to plug in
OpenAI-compatible gateways (self-hosted or vendor previews) without editing the
`Provider` enum. See `examples/custom_provider_injection.rs` for a full demo.

```rust
use ai_lib::{
    client::{AiClientBuilder, Provider},
    provider::builders::CustomProviderBuilder,
    types::{ChatCompletionRequest, Message, Role, Content},
};

let labs_gateway = CustomProviderBuilder::new("labs-gateway")
    .with_base_url("https://labs.example.com/v1")
    .with_api_key_env("LABS_GATEWAY_TOKEN")
    .with_default_chat_model("labs-gpt-35")
    .build_provider()?;

let client = AiClientBuilder::new(Provider::OpenAI) // Enum ignored when strategy is provided
    .with_strategy(labs_gateway)
    .build()?;

let resp = client
    .chat_completion(ChatCompletionRequest::new(
        "labs-gpt-35".to_string(),
        vec![Message {
            role: Role::User,
            content: Content::Text("Hello labs!".to_string()),
            function_call: None,
        }],
    ))
    .await?;
println!("labs> {}", resp.first_text().unwrap_or_default());
```

### Per-provider Builders

Prefer builder wrappers (e.g., `GroqBuilder`, `OpenAiBuilder`) when you want clearer configuration or when composing strategies.

```rust
use ai_lib::provider::GroqBuilder;

let client = GroqBuilder::new()
    .with_base_url("https://api.groq.com")
    .with_proxy(Some("http://proxy.internal:8080"))
    .build()?; // returns AiClient
```

### Concurrency Control
```rust
use ai_lib::{AiClientBuilder, Provider};

let client = AiClientBuilder::new(Provider::Groq)
    .with_max_concurrency(64)
    .for_production()
    .build()?;
```

---

## 🔁 Routing & Failover (OSS)

Use `with_failover_chain` or `with_round_robin_chain` to wire strategies before the client sends requests.

```rust
use ai_lib::{client::AiClientBuilder, Provider};

let client = AiClientBuilder::new(Provider::OpenAI)
    .with_failover_chain(vec![Provider::Anthropic, Provider::Groq])?
    .build()?;
```

Combine with `with_round_robin_chain` or `RoutingStrategyBuilder` for weighted/round-robin routing. Strategy composition now happens during client construction—no sentinel models or runtime branching required.

## 🛡️ Reliability & Resilience

| Feature | Description |
|---------|-------------|
| **Retry Logic** | Exponential backoff with intelligent error classification |
| **Error Handling** | Distinguish transient vs permanent failures |
| **Timeouts** | Configurable per-request and global timeouts |
| **Proxy Support** | Global, per-connection, or disabled proxy handling |
| **Connection Pooling** | Tunable pool size and connection lifecycle |
| **Health Checks** | Endpoint monitoring and policy-based routing |
| **Fallback Strategies** | Multi-provider arrays and manual failover |

---

## 📊 Observability & Metrics

### Custom Metrics Integration
```rust
struct CustomMetrics;
#[async_trait::async_trait]
impl ai_lib::metrics::Metrics for CustomMetrics {
    async fn incr_counter(&self, name: &str, value: u64) { /* ... */ }
    async fn start_timer(&self, name: &str) -> Option<Box<dyn ai_lib::metrics::Timer + Send>> { /* ... */ }
}
let client = AiClient::new_with_metrics(Provider::Groq, Arc::new(CustomMetrics))?;
```

### Usage Tracking
```rust
match response.usage_status {
    UsageStatus::Finalized => println!("Accurate token counts: {:?}", response.usage),
    UsageStatus::Estimated => println!("Estimated tokens: {:?}", response.usage),
    UsageStatus::Pending => println!("Usage data not yet available"),
    UsageStatus::Unsupported => println!("Provider doesn't support usage tracking"),
}
```
Migration: `Usage`/`UsageStatus` are defined in `ai_lib::types::response` and re-exported at the root. Old imports from `types::common` are deprecated and will be removed before 1.0.

### Optional Features

By default, ai-lib ships with a minimal feature set. Enable features as needed:

| Feature | Description | Alias |
|---------|-------------|-------|
| `unified_sse` | Common SSE parser for streaming | `streaming` |
| `interceptors` | Retry, timeout, circuit breaker pipeline | `resilience` |
| `unified_transport` | Shared HTTP client factory | `transport` |
| `config_hot_reload` | Config hot-reload traits | `hot_reload` |
| `cost_metrics` | Basic cost accounting via environment variables | - |
| `routing_mvp` | Model selection and routing capabilities | - |
| `observability` | Tracer and AuditSink interfaces | - |
| `all` | Enable all features above | - |

**Recommended for most applications:**
```toml
ai-lib = { version = "0.4.0", features = ["streaming", "resilience"] }
```

---

## 🗂️ Examples

| Category | Examples |
|----------|----------|
| **Getting Started** | `quickstart`, `basic_usage`, `builder_pattern` |
| **Configuration** | `explicit_config`, `proxy_example`, `custom_transport_config` |
| **Streaming** | `test_streaming`, `cohere_stream` |
| **Reliability** | `custom_transport`, `resilience_example` |
| **Multi-Provider** | `config_driven_example`, `model_override_demo`, `custom_provider_injection`, `routing_modelarray` |
| **Model Management** | `model_management`, `routing_modelarray` |
| **Batch Processing** | `batch_processing` |
| **Function Calling** | `function_call_openai`, `function_call_exec` |
| **Multimodal** | `multimodal_example` |
| **Advanced** | `architecture_progress`, `reasoning_best_practices` |

---

## 📄 License

Dual-licensed under MIT or Apache License 2.0 - choose what works best for your project.

---

## 🤝 Contributing

1. Fork & clone repository
2. Create feature branch: `git checkout -b feature/your-feature`
3. Run tests: `cargo test`
4. Add examples for new features
5. Follow adapter patterns (prefer config-driven over custom)
6. Open PR with rationale + benchmarks (if performance impact)

**We value**: clarity, test coverage, minimal surface area, incremental composability.

---

## 📚 Citation

```bibtex
@software{ai-lib,
    title = {ai-lib: A Unified AI SDK for Rust},
    author = {Luqiang Wang},
    url = {https://github.com/hiddenpath/ai-lib},
    year = {2025}
}
```

---

<div align="center">
  <strong>ai-lib: Build resilient, fast, multi-provider AI systems in Rust—without glue code fatigue.</strong><br/><br/>
  ⭐ If this saves you time, give it a star and share feedback in Issues/Discussions!
</div>