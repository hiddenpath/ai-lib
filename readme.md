# ai-lib 🦀✨  

> A unified, reliable, high-performance multi-provider AI SDK for Rust

A production-grade, provider-agnostic SDK that provides a unified Rust API for 17+ AI platforms and growing (OpenAI, Groq, Anthropic, Gemini, Mistral, Cohere, Azure OpenAI, Ollama, DeepSeek, Qwen, Baidu ERNIE, Tencent Hunyuan, iFlytek Spark, Kimi, HuggingFace, TogetherAI, xAI Grok, and more).  
Eliminates fragmented authentication flows, streaming formats, error semantics, model naming differences, and inconsistent function calling. Scale from one-liner scripts to production systems without rewriting integration code.

---
[Official Website](https://www.ailib.info/)

## 🚀 Core Value

ai-lib unifies AI provider complexity into a single, ergonomic Rust interface:

- **Universal API**: Chat, multimodal, and function calling across all providers
- **Unified Streaming**: Consistent SSE/JSONL parsing with real-time deltas
- **Reliability**: Built-in retry, timeout, circuit breaker, and error classification
- **Flexible Configuration**: Environment variables, builder pattern, or explicit overrides
- **Production Ready**: Connection pooling, proxy support, observability hooks

**Result**: Focus on your product logic while ai-lib handles provider integration friction.

## ⚙️ Quick Start

### Installation
```toml
[dependencies]
ai-lib = "0.3.2"
tokio = { version = "1", features = ["full"] }
futures = "0.3"
```

### One-liner Chat
```rust
use ai_lib::Provider;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let reply = ai_lib::AiClient::quick_chat_text(Provider::Groq, "Hello!").await?;
    println!("Reply: {reply}");
    Ok(())
}
```

### Standard Usage
```rust
use ai_lib::{AiClient, Provider, Message, Role, Content, ChatCompletionRequest};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = AiClient::new(Provider::OpenAI)?;
    let req = ChatCompletionRequest::new(
        client.default_chat_model(),
        vec![Message {
            role: Role::User,
            content: Content::new_text("Explain Rust ownership in one sentence."),
            function_call: None,
        }]
    );
    let resp = client.chat_completion(req).await?;
    println!("Answer: {}", resp.first_text()?);
    Ok(())
}
```

### Streaming
```rust
use futures::StreamExt;
let mut stream = client.chat_completion_stream(req).await?;
while let Some(chunk) = stream.next().await {
    let c = chunk?;
    if let Some(delta) = c.choices[0].delta.content.clone() {
        print!("{delta}");
    }
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
- **Error Classification**: Distinguish transient vs permanent failures
- **Connection Management**: Pooling, timeouts, proxy support
- **Observability**: Pluggable metrics and tracing integration
- **Security**: No sensitive content logging by default

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
| **DeepSeek** | ✅ | Reasoning models |
| **Qwen** | ✅ | Chinese ecosystem |
| **Baidu ERNIE** | ✅ | Enterprise China |
| **Tencent Hunyuan** | ✅ | Cloud integration |
| **iFlytek Spark** | ✅ | Voice + multimodal |
| **Moonshot Kimi** | ✅ | Long context |
| **Azure OpenAI** | ✅ | Enterprise compliance |
| **Ollama** | ✅ | Local/air-gapped |
| **xAI Grok** | ✅ | Real-time oriented |

*See [examples/](examples/) for provider-specific usage patterns.*

---

## 🔑 Configuration

### Environment Variables
```bash
# API Keys (convention-based)
export OPENAI_API_KEY=...
export GROQ_API_KEY=...
export GEMINI_API_KEY=...
export ANTHROPIC_API_KEY=...

# Optional: Custom endpoints
export GROQ_BASE_URL=https://custom.groq.com

# Optional: Proxy and timeouts
export AI_PROXY_URL=http://proxy.internal:8080
export AI_TIMEOUT_SECS=30

# Optional: Connection pooling (enabled by default)
export AI_HTTP_POOL_MAX_IDLE_PER_HOST=32
export AI_HTTP_POOL_IDLE_TIMEOUT_MS=90000
```

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

### Concurrency Control
```rust
use ai_lib::{AiClientBuilder, Provider};

let client = AiClientBuilder::new(Provider::Groq)
    .with_max_concurrency(64)
    .for_production()
    .build()?;
```

---

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

### Optional Features
- `interceptors`: Retry, timeout, circuit breaker pipeline
- `unified_sse`: Common SSE parser for all providers
- `unified_transport`: Shared HTTP client factory
- `cost_metrics`: Basic cost accounting via environment variables
- `routing_mvp`: Model selection and routing capabilities

---

## 🗂️ Examples

| Category | Examples |
|----------|----------|
| **Getting Started** | `quickstart`, `basic_usage`, `builder_pattern` |
| **Configuration** | `explicit_config`, `proxy_example`, `custom_transport_config` |
| **Streaming** | `test_streaming`, `cohere_stream` |
| **Reliability** | `custom_transport`, `resilience_example` |
| **Multi-Provider** | `config_driven_example`, `model_override_demo` |
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