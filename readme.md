# ai-lib 🦀✨  

> A unified, reliable, high-performance multi-provider AI SDK for Rust

A production-grade, provider-agnostic SDK that provides a unified Rust API for 17+ AI platforms (OpenAI, Groq, Anthropic, Gemini, Mistral, Cohere, Azure OpenAI, Ollama, DeepSeek, Qwen, Baidu ERNIE, Tencent Hunyuan, iFlytek Spark, Kimi, HuggingFace, TogetherAI, xAI Grok, etc.).  
Eliminates fragmented authentication flows, streaming formats, error semantics, model naming differences, and inconsistent function calling. Scale from one-liner scripts to multi-region, multi-provider systems without rewriting integration code.

---
[Official Website](https://www.ailib.info/)

## 🚀 Core Value (TL;DR)

ai-lib unifies:
- Chat and multimodal requests across heterogeneous model providers
- Unified streaming (unified SSE parser + JSONL protocol) with consistent deltas
- Function calling semantics (including OpenAI-style tool_calls alignment)
- Reasoning model support (structured, streaming, JSON formats)
- Batch processing workflows
- Reliability primitives (retry, backoff, timeout, proxy, health checks, load strategies)
- Model selection (cost/performance/health/weighted)
- Observability hooks
- Progressive configuration (env vars → builder → explicit injection → custom transport)

You focus on product logic; ai-lib handles infrastructure friction.

## ⚙️ Quick Start

### Installation
```toml
[dependencies]
ai-lib = "0.3.2"
tokio = { version = "1", features = ["full"] }
futures = "0.3"
```

### Fastest Way
```rust
use ai_lib::Provider;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let reply = ai_lib::AiClient::quick_chat_text(Provider::Groq, "Ping?").await?;
    println!("Reply: {reply}");
    Ok(())
}
```

### Standard Chat
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
| Provider | Enumerates all supported providers |
| AiClient / Builder | Main entry point; configuration encapsulation |
| ChatCompletionRequest | Unified request payload |
| Message / Content | Text/image/audio/(future structured) |
| Function / Tool | Unified function calling semantics |
| Streaming Event | Provider-standardized delta streams |
| ModelManager / ModelArray | Strategy-driven model orchestration |
| ConnectionOptions | Explicit runtime overrides |
| Metrics Trait | Custom observability integration |
| Transport | Injectable HTTP + streaming implementation |

---

## 💡 Key Feature Clusters

1. Unified provider abstraction (no per-provider branching)
2. Universal streaming (unified SSE parser + JSONL; with fallback simulation)
3. Multimodal primitives (text/image/audio)
4. Function calling (consistent tool patterns; tool_calls compatibility)
5. Reasoning model support (structured, streaming, JSON formats)
6. Batch processing (sequential/bounded concurrency/smart strategies)
7. Reliability: retry, error classification, timeout, proxy, pooling, interceptor pipeline (features)
8. Model management: performance/cost/health/round-robin/weighted
9. Observability: pluggable metrics and timing
10. Security: isolation, no default content logging
11. Extensibility: custom transport, metrics, strategy injection

---

## 🌍 Supported Providers (Snapshot)

| Provider | Adapter Type | Streaming | Notes |
|----------|--------------|-----------|-------|
| Groq | Config-driven | ✅ | Ultra-low latency |
| OpenAI | Independent | ✅ | Function calling |
| Anthropic (Claude) | Config-driven | ✅ | High quality |
| Google Gemini | Independent | ✅ | Uses `x-goog-api-key` header |
| Mistral | Independent | ✅ | European models |
| Cohere | Independent | ✅ | RAG-optimized |
| HuggingFace | Config-driven | ✅ | Open models |
| TogetherAI | Config-driven | ✅ | Cost-effective |
| DeepSeek | Config-driven | ✅ | Reasoning models |
| Qwen | Config-driven | ✅ | Chinese ecosystem |
| Baidu ERNIE | Config-driven | ✅ | Enterprise CN |
| Tencent Hunyuan | Config-driven | ✅ | Cloud integration |
| iFlytek Spark | Config-driven | ✅ | Voice + multimodal |
| Moonshot Kimi | Config-driven | ✅ | Long context |
| Azure OpenAI | Config-driven | ✅ | Enterprise compliance |
| Ollama | Config-driven | ✅ | Local/air-gapped |
| xAI Grok | Config-driven | ✅ | Real-time oriented |

---

## 🔑 Configuration & Diagnostics

### Environment Variables (Convention-based)
```bash
# API Keys
export OPENAI_API_KEY=...
export GROQ_API_KEY=...
export GEMINI_API_KEY=...
export ANTHROPIC_API_KEY=...
export DEEPSEEK_API_KEY=...

# Optional Base URLs
export GROQ_BASE_URL=https://custom.groq.com

# Proxy
export AI_PROXY_URL=http://proxy.internal:8080

# Global Timeout (seconds)
export AI_TIMEOUT_SECS=30

# Optional: HTTP connection pool (enabled by default)
export AI_HTTP_POOL_MAX_IDLE_PER_HOST=32
export AI_HTTP_POOL_IDLE_TIMEOUT_MS=90000

# Optional: Cost Metrics (when `cost_metrics` feature enabled)
export COST_INPUT_PER_1K=0.5
export COST_OUTPUT_PER_1K=1.5
```

### Explicit Overrides
```rust
use ai_lib::{AiClient, Provider, ConnectionOptions};
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

### Backpressure & Concurrency Cap (Optional)

- Simple: use `concurrency_limit` in batch APIs
- Global: set a max concurrency gate via Builder

```rust
use ai_lib::{AiClientBuilder, Provider};

let client = AiClientBuilder::new(Provider::Groq)
    .with_max_concurrency(64)
    .for_production()
    .build()?;
```

Notes:
- The gate acquires a permit for `chat_completion` and streaming calls and releases it when finished.
- If no permits are available, `RateLimitExceeded` is returned; combine with retry/queueing if needed.

---

## 🛡️ Reliability & Resilience

| Aspect | Capability |
|--------|------------|
| Retry | Exponential backoff + classification |
| Errors | Distinguish transient vs permanent |
| Timeout | Per-request configurable |
| Proxy | Global/per-connection/disable |
| Connection Pool | Tunable size + lifecycle |
| Health Checks | Endpoint status + policy-based avoidance |
| Load Strategies | Round-robin/weighted/health/performance/cost |
| Fallback | Multi-provider arrays/manual layering |

---

## 📊 Observability & Metrics

Implement `Metrics` trait to bridge Prometheus, OpenTelemetry, StatsD, etc.

```rust
struct CustomMetrics;
#[async_trait::async_trait]
impl ai_lib::metrics::Metrics for CustomMetrics {
    async fn incr_counter(&self, name: &str, value: u64) { /* ... */ }
    async fn start_timer(&self, name: &str) -> Option<Box<dyn ai_lib::metrics::Timer + Send>> { /* ... */ }
}
let client = AiClient::new_with_metrics(Provider::Groq, Arc::new(CustomMetrics))?;
```

### Feature Flags (Optional)

- `interceptors`: Interceptor trait & pipeline
- `unified_sse`: Common SSE parser
- `unified_transport`: Shared reqwest client factory
- `cost_metrics`: Minimal cost accounting via env vars
- `routing_mvp`: Enable `ModelArray` routing

### Enterprise Features

For advanced enterprise capabilities, consider [ai-lib-pro]:

- **Advanced Routing**: Policy-driven routing, health monitoring, automatic failover
- **Enterprise Observability**: Structured logging, metrics, distributed tracing
- **Cost Management**: Centralized pricing tables and budget tracking
- **Quota Management**: Tenant/organization quotas and rate limiting
- **Audit & Compliance**: Comprehensive audit trails with redaction
- **Security**: Envelope encryption and key management
- **Configuration**: Hot-reload configuration management

ai-lib-pro layers on top of the open-source ai-lib without breaking changes, providing a seamless upgrade path for enterprise users.

### Tiering: OSS vs PRO

- **OSS (this crate)**: unified API, streaming, retries/timeouts/proxy, configurable pool, lightweight rate limiting and backpressure, batch concurrency controls. Simple env-driven setup; zero external services required.
- **PRO**: multi-tenant quotas & priorities, adaptive concurrency/limits, policy-driven routing, centralized config and hot-reload, deep observability/exporters, audit/compliance, cost catalog and budget guardrails. Drop-in upgrade without code changes.

---

## 🗂️ Examples Directory (in /examples)

| Category | Examples |
|----------|----------|
| Getting Started | quickstart / basic_usage / builder_pattern |
| Configuration | explicit_config / proxy_example / custom_transport_config |
| Streaming | test_streaming / cohere_stream |
| Reliability | custom_transport |
| Multi-Provider | config_driven_example / model_override_demo |
| Model Management | model_management |
| Batch Processing | batch_processing |
| Function Calling | function_call_openai / function_call_exec |
| Multimodal | multimodal_example |
| Architecture Demo | architecture_progress |
| Professional | ascii_horse / hello_groq |

---

## 📄 License

Dual-licensed:
- MIT
- Apache License (Version 2.0)

You may choose the license that best fits your project.

---

## 🤝 Contributing Guide

1. Fork & clone repository  
2. Create feature branch: `git checkout -b feature/your-feature`  
3. Run tests: `cargo test`  
4. Add examples if introducing new features  
5. Follow adapter layering (prefer config-driven before custom)  
6. Open PR with rationale + benchmarks (if performance impact)  

We value: clarity, test coverage, minimal surface area creep, incremental composability.

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