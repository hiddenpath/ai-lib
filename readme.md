# AI-lib: Unified AI SDK for Rust

> **A unified Rust SDK that provides a single interface to multiple AI providers using a hybrid architecture**

## Overview

**ai-lib** is a unified AI SDK for Rust that offers a single, consistent interface for interacting with multiple large language model providers. It uses a hybrid architecture that balances developer ergonomics with provider-specific features, providing progressive configuration options from simple usage to advanced customization, along with powerful tools for building custom model managers and load-balanced arrays.

**Note**: Upgrade guides and PR notes have been moved to the `docs/` directory. See `docs/UPGRADE_0.2.0.md` and `docs/PR_0.2.0.md` for migration and PR details.

## Supported AI Providers

- ✅ **Groq** (config-driven) — llama3, mixtral models
- ✅ **xAI Grok** (config-driven) — grok models
- ✅ **DeepSeek** (config-driven) — deepseek-chat, deepseek-reasoner
- ✅ **Anthropic Claude** (config-driven) — claude-3.5-sonnet
- ✅ **Google Gemini** (independent adapter) — gemini-1.5-pro, gemini-1.5-flash
- ✅ **OpenAI** (independent adapter) — gpt-3.5-turbo, gpt-4
- ✅ **Qwen / Tongyi Qianwen** (config-driven) — Qwen family (OpenAI-compatible)
- ✅ **Cohere** (independent adapter) — command/generate models
- ✅ **Baidu Wenxin (ERNIE)** (config-driven) — ernie-3.5, ernie-4.0
- ✅ **Tencent Hunyuan** (config-driven) — hunyuan family
- ✅ **iFlytek Spark** (config-driven) — spark models (voice+text friendly)
- ✅ **Moonshot / Kimi** (config-driven) — kimi series (long-text scenarios)
- ✅ **Mistral** (independent adapter) — mistral models
- ✅ **Hugging Face Inference** (config-driven) — hub-hosted models
- ✅ **TogetherAI** (config-driven) — together.ai hosted models
- ✅ **Azure OpenAI** (config-driven) — Azure-hosted OpenAI endpoints
- ✅ **Ollama** (config-driven/local) — local Ollama instances

## Core Features

### 🚀 **Unified Interface & Provider Switching**
Switch between AI providers with a single line of code:

```rust
let groq_client = AiClient::new(Provider::Groq)?;
let gemini_client = AiClient::new(Provider::Gemini)?;
let claude_client = AiClient::new(Provider::Anthropic)?;
```

### 🎯 **Progressive Configuration**
Build AI clients with progressive customization levels:

```rust
// Level 1: Simple usage with auto-detection
let client = AiClient::new(Provider::Groq)?;

// Level 2: Custom base URL
let client = AiClientBuilder::new(Provider::Groq)
    .with_base_url("https://custom.groq.com")
    .build()?;

// Level 3: Add proxy support
let client = AiClientBuilder::new(Provider::Groq)
    .with_base_url("https://custom.groq.com")
    .with_proxy(Some("http://proxy.example.com:8080"))
    .build()?;

// Level 4: Advanced configuration
let client = AiClientBuilder::new(Provider::Groq)
    .with_base_url("https://custom.groq.com")
    .with_proxy(Some("http://proxy.example.com:8080"))
    .with_timeout(Duration::from_secs(60))
    .with_pool_config(32, Duration::from_secs(90))
    .build()?;
```

### 🌊 **Universal Streaming Support**
Real-time streaming responses for all providers with SSE parsing and fallback emulation:

```rust
use futures::StreamExt;

let mut stream = client.chat_completion_stream(request).await?;
while let Some(item) = stream.next().await {
    let chunk = item?;
    if let Some(content) = chunk.choices.get(0).and_then(|c| c.delta.content.clone()) {
        print!("{}", content); // real-time output
    }
}
```

### 🔄 **Enterprise-Grade Reliability**
- **Automatic retries** with exponential backoff
- **Smart error classification** (retryable vs. permanent)
- **Proxy support** with authentication
- **Timeout management** and graceful degradation

### 🌐 **Flexible Proxy Configuration**
The library provides flexible proxy configuration options to avoid automatic environment variable reading:

```rust
// Default: No proxy, no environment variable reading
let client = AiClientBuilder::new(Provider::Groq).build()?;

// Explicitly disable proxy
let client = AiClientBuilder::new(Provider::Groq)
    .without_proxy()
    .build()?;

// Use specific proxy URL
let client = AiClientBuilder::new(Provider::Groq)
    .with_proxy(Some("http://proxy.example.com:8080"))
    .build()?;

// Use AI_PROXY_URL environment variable
let client = AiClientBuilder::new(Provider::Groq)
    .with_proxy(None)
    .build()?;
```

```rust
match client.chat_completion(request).await {
    Ok(response) => println!("Success: {}", response.choices[0].message.content.as_text()),
    Err(e) => {
        if e.is_retryable() {
            println!("Retryable error, sleeping {}ms", e.retry_delay_ms());
            // implement retry logic
        } else {
            println!("Permanent error: {}", e);
        }
    }
}
```

### ⚡ **Hybrid Architecture**
- **Config-driven adapters**: Minimal wiring for OpenAI-compatible APIs
- **Independent adapters**: Full control for unique APIs
- **Four-layer design**: Client → Adapter → Transport → Common types
- **Benefits**: Code reuse, extensibility, automatic feature inheritance

### 🏗️ **Custom Model Management**
Build sophisticated model management systems:

```rust
// Performance-based model selection
let mut manager = CustomModelManager::new("groq")
    .with_strategy(ModelSelectionStrategy::PerformanceBased);

// Load-balanced model arrays
let mut array = ModelArray::new("production")
    .with_strategy(LoadBalancingStrategy::RoundRobin);

array.add_endpoint(ModelEndpoint {
    name: "us-east-1".to_string(),
    url: "https://api-east.groq.com".to_string(),
    weight: 1.0,
    healthy: true,
});
```

### 📊 **Advanced Capabilities**
- **Multimodal support**: Text, JSON, image, and audio content
- **Function calling**: Unified `Tool` and `FunctionCall` types
- **Metrics & observability**: Request counters and duration timers
- **Dependency injection**: Mock transports for testing
- **Performance**: <2MB memory, <1ms overhead, <10ms streaming latency

## Quickstart

### Installation
```toml
[dependencies]
ai-lib = "0.2.1"
tokio = { version = "1.0", features = ["full"] }
futures = "0.3"
```

### Basic Usage
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

### Production Best Practices
```rust
use ai_lib::{AiClientBuilder, Provider, CustomModelManager, ModelSelectionStrategy};

// 1. Use environment variables for configuration
let client = AiClientBuilder::new(Provider::Groq)
    .with_timeout(Duration::from_secs(30))
    .build()?;

// 2. Implement model management
let mut manager = CustomModelManager::new("groq")
    .with_strategy(ModelSelectionStrategy::CostBased);

// 3. Add health checks and monitoring
let mut array = ModelArray::new("production")
    .with_strategy(LoadBalancingStrategy::HealthBased);
```

### Environment Variables
```bash
export GROQ_API_KEY=your_groq_api_key
export AI_PROXY_URL=https://proxy.example.com:8080
```

## Examples

### Getting Started
- **Quickstart**: `cargo run --example quickstart` - Simple usage guide
- **Builder Pattern**: `cargo run --example builder_pattern` - Configuration examples

### Advanced Features
- **Model Management**: `cargo run --example model_management` - Custom managers and load balancing

### Core Functionality
- **Architecture**: `cargo run --example test_hybrid_architecture`
- **Streaming**: `cargo run --example test_streaming_improved`
- **Retry**: `cargo run --example test_retry_mechanism`
- **Providers**: `cargo run --example test_groq_generic`

## Provider Details

| Provider | Architecture | Streaming | Models | Notes |
|----------|--------------|-----------|--------|-------|
| **Groq** | config-driven | ✅ | llama3-8b/70b, mixtral-8x7b | Fast inference |
| **DeepSeek** | config-driven | ✅ | deepseek-chat, deepseek-reasoner | China-focused |
| **Anthropic** | config-driven | ✅ | claude-3.5-sonnet | Custom auth |
| **Google Gemini** | independent | 🔄 | gemini-1.5-pro/flash | URL auth |
| **OpenAI** | independent | ✅ | gpt-3.5-turbo, gpt-4 | Proxy may be needed |
| **Qwen** | config-driven | ✅ | Qwen family | OpenAI-compatible |
| **Baidu Wenxin** | config-driven | ✅ | ernie-3.5, ernie-4.0 | Qianfan platform |
| **Tencent Hunyuan** | config-driven | ✅ | hunyuan family | Cloud endpoints |
| **iFlytek Spark** | config-driven | ✅ | spark family | Voice+text friendly |
| **Moonshot Kimi** | config-driven | ✅ | kimi family | Long-text scenarios |

## Model Management Tools

### Key Features
- **Selection strategies**: Round-robin, weighted, performance-based, cost-based
- **Load balancing**: Health checks, connection tracking, multiple endpoints
- **Cost analysis**: Calculate costs for different token counts
- **Performance metrics**: Speed and quality tiers with response time tracking

### Example Usage
```rust
use ai_lib::{CustomModelManager, ModelSelectionStrategy, ModelInfo, ModelCapabilities, PricingInfo, PerformanceMetrics};

let mut manager = CustomModelManager::new("groq")
    .with_strategy(ModelSelectionStrategy::PerformanceBased);

let model = ModelInfo {
    name: "llama3-8b-8192".to_string(),
    display_name: "Llama 3 8B".to_string(),
    capabilities: ModelCapabilities::new()
        .with_chat()
        .with_code_generation()
        .with_context_window(8192),
    pricing: PricingInfo::new(0.05, 0.10), // $0.05/1K input, $0.10/1K output
    performance: PerformanceMetrics::new()
        .with_speed(SpeedTier::Fast)
        .with_quality(QualityTier::Good),
};

manager.add_model(model);
```

## Roadmap

### ✅ Implemented
- Hybrid architecture with universal streaming
- Enterprise-grade error handling and retry
- Multimodal primitives and function calling
- Progressive client configuration
- Custom model management tools
- Load balancing and health checks

### 🚧 Planned
- Advanced backpressure API
- Connection pool tuning
- Plugin system
- Built-in caching

## Contributing

1. Clone: `git clone https://github.com/hiddenpath/ai-lib.git`
2. Branch: `git checkout -b feature/new-feature`
3. Test: `cargo test`
4. PR: Open a pull request

## Community & Support

- 📖 **Documentation**: [docs.rs/ai-lib](https://docs.rs/ai-lib)
- 🐛 **Issues**: [GitHub Issues](https://github.com/hiddenpath/ai-lib/issues)
- 💬 **Discussions**: [GitHub Discussions](https://github.com/hiddenpath/ai-lib/discussions)

## License

Dual licensed: MIT or Apache 2.0

## Citation

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