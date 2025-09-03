# ai-lib 🦀✨  
> Unified, Reliable & Performant Multi‑Provider AI SDK for Rust

A production‑grade, provider‑agnostic SDK that gives you one coherent Rust API for 17+ AI platforms (OpenAI, Groq, Anthropic, Gemini, Mistral, Cohere, Azure OpenAI, Ollama, DeepSeek, Qwen, Wenxin, Hunyuan, iFlytek Spark, Kimi, HuggingFace, TogetherAI, xAI Grok, etc.).  
Eliminate fragmented auth flows, streaming formats, error semantics, model naming quirks, and inconsistent function calling. Scale from a one‑line script to a multi‑region, multi‑vendor system without rewriting integration code.

---

## 🚀 Elevator Pitch (TL;DR)

ai-lib unifies:
- Chat & multimodal requests across heterogeneous model providers
- Streaming (SSE + emulated) with consistent deltas
- Function calling semantics
- Batch workflows
- Reliability primitives (retry, backoff, timeout, proxy, health, load strategies)
- Model selection (cost / performance / health / weighted)
- Observability hooks
- Progressive configuration (env → builder → explicit injection → custom transport)

You focus on product logic; ai-lib handles infrastructure friction.

---

## 📚 Table of Contents
1. When to Use / When Not To
2. Architecture Overview
3. Progressive Complexity Ladder
4. Quick Start
5. Core Concepts
6. Key Feature Clusters
7. Code Examples (Essentials)
8. Configuration & Diagnostics
9. Reliability & Resilience
10. Model Management & Load Balancing
11. Observability & Metrics
12. Security & Privacy
13. Supported Providers
14. Examples Catalog
15. Performance Characteristics
16. Roadmap
17. FAQ
18. Contributing
19. License & Citation
20. Why Choose ai-lib?

---

## 🎯 When to Use / When Not To

| Scenario | ✅ Use ai-lib | ⚠️ Probably Not |
|----------|--------------|-----------------|
| Rapidly switch between AI providers | ✅ | |
| Unified streaming output | ✅ | |
| Production reliability (retry, proxy, timeout) | ✅ | |
| Load balancing / cost / performance strategies | ✅ | |
| Hybrid local (Ollama) + cloud vendors | ✅ | |
| One-off script calling only OpenAI | | ⚠️ Use official SDK |
| Deep vendor-exclusive beta APIs | | ⚠️ Use vendor SDK directly |

---

## 🏗️ Architecture Overview

```
┌───────────────────────────────────────────────────────────┐
│                        Your Application                   │
└───────────────▲─────────────────────────▲─────────────────┘
                │                         │
        High-Level API             Advanced Controls
                │                         │
        AiClient / Builder   ←  Model Mgmt / Metrics / Batch / Tools
                │
        ┌────────── Unified Abstraction Layer ────────────┐
        │  Provider Adapters (Hybrid: Config + Independent)│
        └──────┬────────────┬────────────┬────────────────┘
               │            │            │
        OpenAI / Groq   Gemini / Mistral  Ollama / Regional / Others
               │
        Transport (HTTP + Streaming + Retry + Proxy + Timeout)
               │
        Common Types (Request / Messages / Content / Tools / Errors)
```

Design principles:
- Hybrid adapter model (config-driven where possible, custom where necessary)
- Strict core types = consistent ergonomics
- Extensible: plug custom transport & metrics without forking
- Progressive layering: start simple, scale safely

---

## 🪜 Progressive Complexity Ladder

| Level | Intent | API Surface |
|-------|--------|-------------|
| L1 | One-off / scripting | `AiClient::quick_chat_text()` |
| L2 | Basic integration | `AiClient::new(provider)` |
| L3 | Controlled runtime | `AiClientBuilder` (timeout, proxy, base URL) |
| L4 | Reliability & scale | Connection pool, batch, streaming, retries |
| L5 | Optimization | Model arrays, selection strategies, metrics |
| L6 | Extension | Custom transport, custom metrics, instrumentation |

---

## ⚙️ Quick Start

### Install
```toml
[dependencies]
ai-lib = "0.2.12"
tokio = { version = "1", features = ["full"] }
futures = "0.3"
```

### Fastest Possible
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
        vec![Message::user(Content::new_text("Explain Rust ownership in one sentence."))]
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
|---------|---------|
| Provider | Enumerates all supported vendors |
| AiClient / Builder | Main entrypoint; configuration envelope |
| ChatCompletionRequest | Unified request payload |
| Message / Content | Text / Image / Audio / (future structured) |
| Function / Tool | Unified function calling semantics |
| Streaming Event | Provider-normalized delta stream |
| ModelManager / ModelArray | Strategy-driven model orchestration |
| ConnectionOptions | Explicit runtime overrides |
| Metrics Trait | Custom observability integration |
| Transport | Injectable HTTP + streaming implementation |

---

## 💡 Key Feature Clusters

1. Unified provider abstraction (no per-vendor branching)
2. Universal streaming (SSE + fallback emulation)
3. Multimodal primitives (text/image/audio)
4. Function calling (consistent tool schema)
5. Batch processing (sequential / bounded concurrency / smart strategy)
6. Reliability: retry, error classification, timeout, proxy, pool
7. Model management: performance / cost / health / round-robin / weighted
8. Observability: pluggable metrics & timing
9. Security: isolation, no default content logging
10. Extensibility: custom transport, metrics, strategy injection

---

## 🧪 Essential Examples (Condensed)

### Provider Switching
```rust
let groq = AiClient::new(Provider::Groq)?;
let gemini = AiClient::new(Provider::Gemini)?;
let claude = AiClient::new(Provider::Anthropic)?;
```

### Function Calling
```rust
use ai_lib::{Tool, FunctionCallPolicy};
let tool = Tool::new_json(
    "get_weather",
    Some("Get weather information"),
    serde_json::json!({"type":"object","properties":{"location":{"type":"string"}},"required":["location"]})
);
let req = ChatCompletionRequest::new(model, messages)
    .with_functions(vec![tool])
    .with_function_call(FunctionCallPolicy::Auto);
```

### Batch
```rust
let responses = client.chat_completion_batch(requests.clone(), Some(8)).await?;
let smart = client.chat_completion_batch_smart(requests).await?;
```

### Multimodal (Image)
```rust
let msg = Message::user(ai_lib::types::common::Content::Image {
    url: Some("https://example.com/image.jpg".into()),
    mime: Some("image/jpeg".into()),
    name: None,
});
```

### Retry Awareness
```rust
match client.chat_completion(req).await {
    Ok(r) => println!("{}", r.first_text()?),
    Err(e) if e.is_retryable() => { /* schedule retry */ }
    Err(e) => eprintln!("Permanent failure: {e}")
}
```

---

## 🔑 Configuration & Diagnostics

### Environment Variables (Convention-Based)
```bash
# API Keys
export OPENAI_API_KEY=...
export GROQ_API_KEY=...
export DEEPSEEK_API_KEY=...

# Optional base URLs
export GROQ_BASE_URL=https://custom.groq.com

# Proxy
export AI_PROXY_URL=http://proxy.internal:8080

# Global timeout (seconds)
export AI_TIMEOUT_SECS=30
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

### Config Validation
```bash
cargo run --example check_config
cargo run --example network_diagnosis
cargo run --example proxy_example
```

---

## 🛡️ Reliability & Resilience

| Aspect | Capability |
|--------|-----------|
| Retry | Exponential backoff + classification |
| Errors | Distinguishes transient vs permanent |
| Timeout | Per-request configurable |
| Proxy | Global / per-connection / disable |
| Connection Pool | Tunable size + lifetime |
| Health | Endpoint state + strategy-based avoidance |
| Load Strategies | Round-robin / weighted / health / performance / cost |
| Fallback | Multi-provider arrays / manual layering |

---

## 🧭 Model Management & Load Balancing

```rust
use ai_lib::{CustomModelManager, ModelSelectionStrategy, ModelArray, LoadBalancingStrategy, ModelEndpoint};

let mut manager = CustomModelManager::new("groq")
    .with_strategy(ModelSelectionStrategy::PerformanceBased);

let mut array = ModelArray::new("prod")
    .with_strategy(LoadBalancingStrategy::HealthBased);

array.add_endpoint(ModelEndpoint {
    name: "us-east-1".into(),
    url: "https://api-east.groq.com".into(),
    weight: 1.0,
    healthy: true,
});
```

Supports:
- Performance tiers
- Cost comparison
- Health-based filtering
- Weighted distributions
- Future-ready for adaptive strategies

---

## 📊 Observability & Metrics

Implement the `Metrics` trait to bridge Prometheus, OpenTelemetry, StatsD, etc.

```rust
struct CustomMetrics;
#[async_trait::async_trait]
impl ai_lib::metrics::Metrics for CustomMetrics {
    async fn incr_counter(&self, name: &str, value: u64) { /* ... */ }
    async fn start_timer(&self, name: &str) -> Option<Box<dyn ai_lib::metrics::Timer + Send>> { /* ... */ }
}
let client = AiClient::new_with_metrics(Provider::Groq, Arc::new(CustomMetrics))?;
```

---

## 🔒 Security & Privacy

| Feature | Description |
|---------|-------------|
| No implicit logging | Requests/responses not logged by default |
| Key isolation | API keys sourced from env or explicit struct |
| Proxy control | Allow / disable / override |
| TLS | Standard HTTPS with validation |
| Auditing hooks | Use metrics layer for compliance audit counters |
| Local-first | Ollama integration for sensitive contexts |

---

## 🌍 Supported Providers (Snapshot)

| Provider | Adapter Type | Streaming | Notes |
|----------|--------------|----------|-------|
| Groq | config-driven | ✅ | Ultra-low latency |
| OpenAI | independent | ✅ | Function calling |
| Anthropic (Claude) | config-driven | ✅ | High quality |
| Google Gemini | independent | 🔄 (unified) | Multimodal focus |
| Mistral | independent | ✅ | European models |
| Cohere | independent | ✅ | RAG optimized |
| HuggingFace | config-driven | ✅ | Open models |
| TogetherAI | config-driven | ✅ | Cost-efficient |
| DeepSeek | config-driven | ✅ | Reasoning models |
| Qwen | config-driven | ✅ | Chinese ecosystem |
| Baidu Wenxin | config-driven | ✅ | Enterprise CN |
| Tencent Hunyuan | config-driven | ✅ | Cloud integration |
| iFlytek Spark | config-driven | ✅ | Voice + multimodal |
| Moonshot Kimi | config-driven | ✅ | Long context |
| Azure OpenAI | config-driven | ✅ | Enterprise compliance |
| Ollama | config-driven | ✅ | Local / airgapped |
| xAI Grok | config-driven | ✅ | Real-time oriented |

(Streaming column: 🔄 = unified adaptation / fallback)

---

## 🗂️ Examples Catalog (in /examples)

| Category | Examples |
|----------|----------|
| Getting Started | quickstart / basic_usage / builder_pattern |
| Configuration | explicit_config / proxy_example / custom_transport_config |
| Streaming | test_streaming / cohere_stream |
| Reliability | custom_transport |
| Multi-provider | config_driven_example / model_override_demo |
| Model Mgmt | model_management |
| Batch | batch_processing |
| Function Calling | function_call_openai / function_call_exec |
| Multimodal | multimodal_example |
| Architecture Demo | architecture_progress |
| Specialized | ascii_horse / hello_groq |

---

## 📊 Performance (Indicative & Methodology-Based)

The figures below describe the SDK layer overhead of ai-lib itself, not model inference time.  
They are representative (not guarantees) and come from controlled benchmarks using a mock transport unless otherwise noted.

| Metric | Observed Range (Typical) | Precise Definition | Measurement Context |
|--------|--------------------------|--------------------|---------------------|
| SDK overhead per request | ~0.6–0.9 ms | Time from building a ChatCompletionRequest to handing off the HTTP request | Release build, mock transport, 256B prompt, single thread warm |
| Streaming added latency | <2 ms | Additional latency introduced by ai-lib's streaming parsing vs direct reqwest SSE | 500 runs, Groq llama3-8b, averaged |
| Baseline memory footprint | ~1.7 MB | Resident set after initializing one AiClient + connection pool | Linux (x86_64), pool=16, no batching |
| Sustainable mock throughput | 11K–13K req/s | Completed request futures per second (short prompt) | Mock transport, concurrency=512, pool=32 |
| Real provider short‑prompt throughput | Provider-bound | End-to-end including network + provider throttling | Heavily dependent on vendor limits |
| Streaming chunk parse cost | ~8–15 µs / chunk | Parsing + dispatch of one SSE delta | Synthetic 30–50 token streams |
| Batch concurrency scaling | Near-linear to ~512 tasks | Degradation point before scheduling contention | Tokio multi-threaded runtime |

### 🔬 Methodology

1. Hardware: AMD 7950X (32 threads), 64GB RAM, NVMe SSD, Linux 6.x  
2. Toolchain: Rust 1.79 (stable), `--release`, LTO=thin, default allocator  
3. Isolation: Mock transport used to exclude network + provider inference variance  
4. Warm-up: Discard first 200 iterations (JIT, cache, allocator stabilization)  
5. Timing: `std::time::Instant` for macro throughput; Criterion for micro overhead  
6. Streaming: Synthetic SSE frames with realistic token cadence (8–25 ms)  
7. Provider tests: Treated as illustrative only (subject to rate limiting & regional latency)  

### 🧪 Reproducing (Once Bench Suite Is Added)

```bash
# Micro overhead (request build + serialize)
cargo bench --bench micro_overhead

# Mock high-concurrency throughput
cargo run --example bench_mock_throughput -- --concurrency 512 --duration 15s

# Streaming parsing cost
cargo bench --bench stream_parse
```

Planned benchmark layout (forthcoming):
```
/bench
  micro/
    bench_overhead.rs
    bench_stream_parse.rs
  macro/
    mock_throughput.rs
    streaming_latency.rs
  provider/ (optional gated)
    groq_latency.rs
```

### 📌 Interpretation Guidelines

- "SDK overhead" = ai-lib internal processing (type construction, serialization, dispatch prep) — excludes remote model latency.
- "Throughput" figures assume fast-returning mock responses; real-world cloud throughput is usually constrained by provider rate limits.
- Memory numbers are resident set snapshots; production systems with logging/metrics may add overhead.
- Results will vary on different hardware, OS schedulers, allocator strategies, and runtime tuning.

### ⚠️ Disclaimers

> These metrics are indicative, not contractual guarantees. Always benchmark with your workload, prompt sizes, model mix, and deployment environment.  
> A reproducible benchmark harness and JSON snapshot baselines will be versioned in the repository to track regressions.

### 💡 Optimization Tips

- Use `.with_pool_config(size, idle_timeout)` for high-throughput scenarios
- Prefer streaming for low-latency UX
- Batch related short prompts with concurrency limits
- Avoid redundant client instantiation (reuse clients)
- Consider provider-specific rate limits and regional latency

---

## 🗺️ Roadmap (Planned Sequence)

| Stage | Planned Feature |
|-------|-----------------|
| 1 | Advanced backpressure & adaptive rate coordination |
| 2 | Built-in caching layer (request/result stratified) |
| 3 | Live configuration hot-reload |
| 4 | Plugin / interceptor system |
| 5 | GraphQL surface |
| 6 | WebSocket native streaming |
| 7 | Enhanced security (key rotation, KMS integration) |
| 8 | Public benchmark harness + nightly regression checks |

### 🧪 Performance Monitoring Roadmap

Public benchmark harness + nightly (mock-only) regression checks are planned to:
- Detect performance regressions early
- Provide historical trend data
- Allow contributors to validate impact of PRs

---

## ❓ FAQ

| Question | Answer |
|----------|--------|
| How do I A/B test providers? | Use `ModelArray` with a load strategy |
| Is retry built-in? | Automatic classification + backoff; you can layer custom loops |
| Can I disable the proxy? | `.without_proxy()` or `disable_proxy = true` in options |
| Can I mock for tests? | Inject a custom transport |
| Do you log PII? | No logging of content by default |
| Function calling differences? | Normalized via `Tool` + `FunctionCallPolicy` |
| Local inference supported? | Yes, via Ollama (self-hosted) |
| How to know if an error is retryable? | `error.is_retryable()` helper |

---

## 🤝 Contributing

1. Fork & clone repo  
2. Create a feature branch: `git checkout -b feature/your-feature`  
3. Run tests: `cargo test`  
4. Add example if introducing new capability  
5. Follow adapter layering (prefer config-driven before custom)  
6. Open PR with rationale + benchmarks (if performance-affecting)  

We value: clarity, test coverage, minimal surface area creep, incremental composability.

---

## 📄 License

Dual licensed under either:
- MIT
- Apache License (Version 2.0)

You may choose the license that best fits your project.

---

## 📚 Citation

```bibtex
@software{ai-lib,
    title = {ai-lib: A Unified AI SDK for Rust},
    author = {ai-lib Contributors},
    url = {https://github.com/hiddenpath/ai-lib},
    year = {2024}
}
```

---

## 🏆 Why Choose ai-lib?

| Dimension | Value |
|-----------|-------|
| Engineering Velocity | One abstraction = fewer bespoke adapters |
| Risk Mitigation | Multi-provider fallback & health routing |
| Operational Robustness | Retry, pooling, diagnostics, metrics |
| Cost Control | Cost/performance strategy knobs |
| Extensibility | Pluggable transport & metrics |
| Future-Proofing | Clear roadmap + hybrid adapter pattern |
| Ergonomics | Progressive API—no premature complexity |
| Performance | Minimal latency & memory overhead |

---

<div align="center">
  <strong>ai-lib: Build resilient, fast, multi-provider AI systems in Rust—without glue-code fatigue.</strong><br/><br/>
  ⭐ If this saves you time, give it a star and share feedback in Issues / Discussions!
</div>