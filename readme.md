# AI-lib: Unified AI SDK for Rust

> **A unified Rust SDK that provides a single interface to multiple AI providers using a hybrid architecture**

## Overview

**ai-lib** is a unified AI SDK for Rust that offers a single, consistent interface for interacting with multiple large language model providers. It uses a hybrid architecture that balances developer ergonomics with provider-specific features.

**Note**: upgrade guides and PR notes have been moved to the `docs/` directory to keep the repository root clean. See `docs/UPGRADE_0.2.0.md` and `docs/PR_0.2.0.md` for migration and PR details.

## Supported AI Providers

- ✅ **Groq** (config-driven) — supports llama3, mixtral models
- ✅ **xAI Grok** (config-driven) — supports grok models
- ✅ **DeepSeek** (config-driven) — supports deepseek-chat, deepseek-reasoner
- ✅ **Anthropic Claude** (config-driven) — supports claude-3.5-sonnet
- ✅ **Google Gemini** (independent adapter) — supports gemini-1.5-pro, gemini-1.5-flash
- ✅ **OpenAI** (independent adapter) — supports gpt-3.5-turbo, gpt-4 (may require a proxy in some regions)
- ✅ **Qwen / Tongyi Qianwen (Alibaba Cloud)** (config-driven) — supports Qwen family (OpenAI-compatible)
- ✅ **Cohere** (independent adapter) — supports command/generate models (SSE streaming + fallback)
- ✅ **Baidu Wenxin (Baidu ERNIE)** (config-driven) — supports ernie-3.5, ernie-4.0 (OpenAI-compatible endpoints via the Qianfan platform; may require AK/SK and OAuth)
- ✅ **Tencent Hunyuan** (config-driven) — supports the hunyuan family (OpenAI-compatible endpoints; cloud account and keys required)
- ✅ **iFlytek Spark** (config-driven) — supports spark models (OpenAI-compatible, good for mixed voice+text scenarios)
- ✅ **Moonshot / Kimi** (config-driven) — supports kimi series (OpenAI-compatible, suitable for long-text scenarios)
- ✅ **Mistral** (independent adapter) — supports mistral models
- ✅ **Hugging Face Inference** (config-driven) — supports hub-hosted models
- ✅ **TogetherAI** (config-driven) — supports together.ai hosted models
- ✅ **Azure OpenAI** (config-driven) — supports Azure-hosted OpenAI endpoints
- ✅ **Ollama** (config-driven / local) — supports local Ollama instances

## Core features

### 🚀 Zero-cost provider switching
Switch between AI providers with a single line of code — the unified API ensures a seamless experience:

```rust
// Instant provider switching — same API, different backends
let groq_client = AiClient::new(Provider::Groq)?;
let gemini_client = AiClient::new(Provider::Gemini)?;
let claude_client = AiClient::new(Provider::Anthropic)?;
```

Runtime selection is supported (for example via environment variables or other logic).

### 🌊 Universal streaming support
Provides realtime streaming responses for all providers; SSE parsing and fallback emulation ensure consistent behavior:

```rust
use futures::StreamExt;

let mut stream = client.chat_completion_stream(request).await?;
print!("streamed output: ");
while let Some(item) = stream.next().await {
    let chunk = item?;
    if let Some(content) = chunk.choices.get(0).and_then(|c| c.delta.content.clone()) {
        print!("{}", content); // realtime output
    }
}
```

Includes a cancel handle (`CancelHandle`) and a planned backpressure API, suitable for low-latency UI applications.

### 🔄 Enterprise-grade reliability and error handling
- **Automatic retries with exponential backoff**: intelligently retry transient failures (e.g., timeouts, rate limits).
- **Smart error classification**: distinguish retryable errors (network issues) from permanent errors (authentication failures) and provide recovery guidance.
- **Proxy support**: HTTP/HTTPS proxies with auth for enterprise environments.
- **Timeout management**: configurable timeouts and graceful degradation to ensure production stability.

Example error handling:

```rust
match client.chat_completion(request).await {
    Ok(response) => println!("success: {}", response.choices[0].message.content.as_text()),
    Err(e) => {
        if e.is_retryable() {
            println!("retryable error, sleeping {}ms", e.retry_delay_ms());
            tokio::time::sleep(Duration::from_millis(e.retry_delay_ms())).await;
            // implement retry logic
        } else {
            println!("permanent error: {}", e);
        }
    }
}
```

### ⚡ Hybrid architecture
- **Config-driven adapters**: for OpenAI-compatible APIs; require minimal wiring (≈15 lines) and inherit SSE streaming, proxy, and upload behaviors.
- **Independent adapters**: full control for providers with unique APIs, including custom auth and response parsing.
- **Four-layer design**: unified client layer, adapter layer, transport layer (HttpTransport with proxy and retry), and common types — ensuring type safety with no extra runtime dependencies.
- **Benefits**: major code reuse, flexible extensibility, and automatic feature inheritance.

### 📊 Metrics & observability
A minimal metrics surface (the `Metrics` and `Timer` traits) with a default `NoopMetrics` implementation. Adapters include request counters and duration timers and accept injected metrics implementations for testing or production monitoring.

### 📁 Multimodal & file support
- Supports text, JSON, image, and audio content types.
- File upload / inline helpers with size checks and upload fallbacks.
- Function-calling / tool support: unified `Tool` and `FunctionCall` types with cross-provider parsing and execution.

Minimal tool-calling example:

```rust
let mut req = ChatCompletionRequest::new("gpt-4".to_string(), vec![]);
req.functions = Some(vec![Tool { /* ... */ }]);
req.function_call = Some(FunctionCallPolicy::Auto("auto".to_string()));
```

### 🔧 Dependency injection & testability
- An object-safe transport abstraction (`DynHttpTransportRef`) allows injecting mock transports for unit tests.
- Adapter constructors support custom transport injection.

Example:

```rust
let transport: DynHttpTransportRef = my_test_transport.into();
let adapter = GenericAdapter::with_transport_ref(config, transport)?;
```

### 🚀 Performance & scalability
- **Benchmarks**: memory <2MB, client overhead <1ms, streaming chunk latency <10ms.
- **Connection pooling**: automatic reuse with tunable `reqwest::Client` options (max idle connections, idle timeout).
- **Custom configuration**: timeouts, proxy, and pool parameters via `HttpTransportConfig`.

Custom pool example:

```rust
let reqwest_client = Client::builder()
    .pool_max_idle_per_host(32)
    .build()?;
let transport = HttpTransport::with_client(reqwest_client, Duration::from_secs(30));
```

## Quickstart

### Installation
Add to your `Cargo.toml`:

```toml
[dependencies]
ai-lib = "0.2.0"
tokio = { version = "1.0", features = ["full"] }
futures = "0.3"
```

### One-minute tryout (no API key required)
Construct a client and request without making network calls:

```rust
use ai_lib::{AiClient, Provider, ChatCompletionRequest, Message, Role, Content};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = AiClient::new(Provider::Groq)?;
    let req = ChatCompletionRequest::new(
        "test-model".to_string(),
        vec![Message { role: Role::User, content: Content::new_text("Hello from ai-lib"), function_call: None }]
    );
    Ok(())
}
```

### Real requests
Set API keys and proxy:

```bash
export GROQ_API_KEY=your_groq_api_key
export AI_PROXY_URL=https://proxy.example.com:8080
cargo run --example basic_usage
```

## Environment variables

- **API keys**: e.g. `GROQ_API_KEY`, `OPENAI_API_KEY`, etc.
- **Proxy**: `AI_PROXY_URL` supports HTTP/HTTPS and auth.

## Examples & tests

- Hybrid architecture: `cargo run --example test_hybrid_architecture`
- Streaming: `cargo run --example test_streaming_improved`
- Retry behavior: `cargo run --example test_retry_mechanism`
- Provider tests: `cargo run --example test_groq_generic`, etc.

## Provider details

| Provider | Status | Architecture | Streaming | Models | Notes |
|--------|------|------|----------|------|------|
| **Groq** | ✅ production-ready | config-driven | ✅ | llama3-8b/70b, mixtral-8x7b | fast inference, proxy support |
| **DeepSeek** | ✅ production-ready | config-driven | ✅ | deepseek-chat, deepseek-reasoner | China-focused, direct access |
| **Anthropic** | ✅ production-ready | config-driven | ✅ | claude-3.5-sonnet | custom auth required |
| **Google Gemini** | ✅ production-ready | independent adapter | 🔄 | gemini-1.5-pro/flash | URL parameter auth |
| **OpenAI** | ✅ production-ready | independent adapter | ✅ | gpt-3.5-turbo, gpt-4 | may require proxy in some regions |
| **Qwen** | ✅ production-ready | config-driven | ✅ | Qwen family | uses DASHSCOPE_API_KEY |
| **Baidu Wenxin (ERNIE)** | ✅ production-ready | config-driven | ✅ | ernie-3.5, ernie-4.0 | OpenAI-compatible via Qianfan; may require AK/SK and OAuth — see Baidu Cloud console |
| **Tencent Hunyuan** | ✅ production-ready | config-driven | ✅ | hunyuan family | Tencent Cloud offers OpenAI-compatible endpoints (cloud account & keys required) — see Tencent docs |
| **iFlytek Spark** | ✅ production-ready | config-driven | ✅ | spark family | voice+text friendly, OpenAI-compatible endpoints — see iFlytek docs |
| **Moonshot Kimi** | ✅ production-ready | config-driven | ✅ | kimi family | OpenAI-compatible endpoints; suitable for long-text scenarios — see Moonshot platform |

## Roadmap

### Implemented
- Hybrid architecture and universal streaming support.
- Enterprise-grade error handling, retry, and proxy support.
- Multimodal primitives, function-calling, and metrics scaffold.
- Transport injection and upload tests.

### Planned
- Advanced backpressure API and benchmark CI.
- Connection pool tuning and plugin system.
- Built-in caching and load balancing.

## Contributing

Contributions are welcome (new providers, performance work, docs).

1. Clone: `git clone https://github.com/hiddenpath/ai-lib.git`
2. Create a branch: `git checkout -b feature/new-feature`
3. Test: `cargo test`
4. Open a PR.

## Community & support

- 📖 Docs: [docs.rs/ai-lib](https://docs.rs/ai-lib)
- 🐛 Issues: [GitHub Issues](https://github.com/hiddenpath/ai-lib/issues)
- 💬 Discussions: [GitHub Discussions](https://github.com/hiddenpath/ai-lib/discussions)

## Acknowledgements & license

Thanks to the AI providers and the Rust community. Dual licensed: MIT or Apache 2.0.

Citation:

```bibtex
@software{ai-lib,
    title = {ai-lib: A Unified AI SDK for Rust},
    author = {AI-lib Contributors},
    url = {https://github.com/hiddenpath/ai-lib},
    year = {2024}
}
```

---

<div align="center">
  ai-lib: the most comprehensive unified AI SDK in the Rust ecosystem. 🦀✨
</div>