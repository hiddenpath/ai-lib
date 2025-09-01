# AI-lib: Rust 统一 AI SDK

> **Rust 生态中最全面的统一 AI SDK** 🦀✨

## 🎯 概述

**ai-lib** 是一个统一的 Rust AI SDK，为多个大语言模型提供商提供单一、一致的接口。采用混合架构设计，平衡了开发者体验和提供商特定功能，提供从简单使用到高级定制的渐进式配置选项，以及构建自定义模型管理器和负载均衡数组的强大工具。

**核心亮点：**
- 🚀 **17+ AI 提供商** 支持统一接口
- ⚡ **混合架构** - 配置驱动 + 独立适配器
- 🔧 **渐进式配置** - 从简单到企业级
- 🌊 **通用流式支持** - 所有提供商实时响应
- 🛡️ **企业级可靠性** - 重试、错误处理、代理支持
- 📊 **高级功能** - 多模态、函数调用、批处理
- 🎛️ **系统配置管理** - 环境变量 + 显式覆盖

## 🏗️ 核心架构

### 混合设计哲学
ai-lib 使用**混合架构**，结合了两种方式的优势：

- **配置驱动适配器**：OpenAI 兼容 API 的最小布线（Groq、DeepSeek、Anthropic 等）
- **独立适配器**：独特 API 的完全控制（OpenAI、Gemini、Mistral、Cohere）
- **四层设计**：客户端 → 适配器 → 传输 → 通用类型
- **优势**：代码重用、可扩展性、自动功能继承

### 渐进式配置系统
四个配置复杂度级别，满足您的需求：

```rust
// 级别 1：简单使用，自动检测
let client = AiClient::new(Provider::Groq)?;

// 级别 2：自定义 base URL
let client = AiClientBuilder::new(Provider::Groq)
    .with_base_url("https://custom.groq.com")
    .build()?;

// 级别 3：添加代理支持
let client = AiClientBuilder::new(Provider::Groq)
    .with_base_url("https://custom.groq.com")
    .with_proxy(Some("http://proxy.example.com:8080"))
    .build()?;

// 级别 4：高级配置
let client = AiClientBuilder::new(Provider::Groq)
    .with_base_url("https://custom.groq.com")
    .with_proxy(Some("http://proxy.example.com:8080"))
    .with_timeout(Duration::from_secs(60))
    .with_pool_config(32, Duration::from_secs(90))
    .build()?;
```

## 🚀 核心功能

### 🔄 **统一提供商切换**
用一行代码在 AI 提供商之间切换：

```rust
let groq_client = AiClient::new(Provider::Groq)?;
let gemini_client = AiClient::new(Provider::Gemini)?;
let claude_client = AiClient::new(Provider::Anthropic)?;
```

### 🌊 **通用流式支持**
所有提供商的实时流式响应，支持 SSE 解析和回退模拟：

```rust
use futures::StreamExt;

let mut stream = client.chat_completion_stream(request).await?;
while let Some(item) = stream.next().await {
    let chunk = item?;
    if let Some(content) = chunk.choices.get(0).and_then(|c| c.delta.content.clone()) {
        print!("{}", content); // 实时输出
    }
}
```

### 🛡️ **企业级可靠性**
- **自动重试**：指数退避重试
- **智能错误分类**：可重试 vs 永久性错误
- **代理支持**：带身份验证的代理
- **超时管理**：优雅降级

```rust
match client.chat_completion(request).await {
    Ok(response) => println!("成功: {}", response.choices[0].message.content.as_text()),
    Err(e) => {
        if e.is_retryable() {
            println!("可重试错误，等待 {}ms", e.retry_delay_ms());
            // 实现重试逻辑
        } else {
            println!("永久性错误: {}", e);
        }
    }
}
```

### 🎛️ **系统配置管理**
全面的配置系统，支持环境变量和显式覆盖：

#### 环境变量支持
```bash
# API 密钥
export GROQ_API_KEY=your_groq_api_key
export OPENAI_API_KEY=your_openai_api_key
export DEEPSEEK_API_KEY=your_deepseek_api_key

# 代理配置
export AI_PROXY_URL=http://proxy.example.com:8080

# 提供商特定的 Base URLs
export GROQ_BASE_URL=https://custom.groq.com
export DEEPSEEK_BASE_URL=https://custom.deepseek.com
```

#### 显式配置覆盖
```rust
use ai_lib::{AiClient, Provider, ConnectionOptions};
use std::time::Duration;

let opts = ConnectionOptions {
    base_url: Some("https://custom.groq.com".into()),
    proxy: Some("http://proxy.example.com:8080".into()),
    api_key: Some("explicit-api-key".into()),
    timeout: Some(Duration::from_secs(45)),
    disable_proxy: false,
};
let client = AiClient::with_options(Provider::Groq, opts)?;
```

#### 配置验证工具
```bash
# 内置配置检查工具
cargo run --example check_config

# 网络诊断工具
cargo run --example network_diagnosis

# 代理配置测试
cargo run --example proxy_example
```

### 📦 **批处理**
高效的批处理，支持多种策略：

```rust
// 并发批处理，带并发限制
let responses = client.chat_completion_batch(requests, Some(5)).await?;

// 智能批处理（自动选择策略）
let responses = client.chat_completion_batch_smart(requests).await?;

// 顺序批处理
let responses = client.chat_completion_batch(requests, None).await?;
```

### 🎨 **多模态支持**
统一的文本、图像、音频和结构化数据内容类型：

```rust
use ai_lib::types::common::Content;

let message = Message {
    role: Role::User,
    content: Content::Image {
        url: Some("https://example.com/image.jpg".into()),
        mime: Some("image/jpeg".into()),
        name: None,
    },
    function_call: None,
};
```

### 🛠️ **函数调用**
所有提供商的统一函数调用：

```rust
let tool = Tool {
    name: "get_weather".to_string(),
    description: Some("获取天气信息".to_string()),
    parameters: serde_json::json!({
        "type": "object",
        "properties": {
            "location": {"type": "string"}
        }
    }),
};

let request = ChatCompletionRequest::new(model, messages)
    .with_functions(vec![tool])
    .with_function_call(FunctionCallPolicy::Auto);
```

### 📊 **可观测性与指标**
全面的指标和可观测性支持：

```rust
use ai_lib::metrics::{Metrics, NoopMetrics};

// 自定义指标实现
struct CustomMetrics;

#[async_trait::async_trait]
impl Metrics for CustomMetrics {
    async fn incr_counter(&self, name: &str, value: u64) {
        // 记录到您的指标系统
    }
    
    async fn start_timer(&self, name: &str) -> Option<Box<dyn Timer + Send>> {
        // 开始计时操作
    }
}

let client = AiClient::new_with_metrics(Provider::Groq, Arc::new(CustomMetrics))?;
```

### 🏗️ **自定义模型管理**
复杂的模型管理和负载均衡：

```rust
// 基于性能的模型选择
let mut manager = CustomModelManager::new("groq")
    .with_strategy(ModelSelectionStrategy::PerformanceBased);

// 负载均衡模型数组
let mut array = ModelArray::new("production")
    .with_strategy(LoadBalancingStrategy::RoundRobin);

array.add_endpoint(ModelEndpoint {
    name: "us-east-1".to_string(),
    url: "https://api-east.groq.com".to_string(),
    weight: 1.0,
    healthy: true,
});
```

### 🔧 **灵活传输层**
自定义传输注入，用于测试和特殊需求：

```rust
// 测试用自定义传输
let mock_transport = Arc::new(MockTransport::new());
let adapter = GenericAdapter::with_transport_ref(config, mock_transport)?;

// 自定义 HTTP 客户端配置
let transport = HttpTransport::with_custom_client(custom_client)?;
```

### ⚡ **性能优化**
企业级性能，最小开销：

- **内存高效**：<2MB 内存占用
- **低延迟**：<1ms 请求开销
- **快速流式**：<10ms 流式延迟
- **连接池**：可配置连接重用
- **异步/等待**：完整的 tokio 异步支持

### 🛡️ **安全与隐私**
企业环境的内置安全功能：

- **API 密钥管理**：安全的环境变量处理
- **代理支持**：企业代理集成
- **TLS/SSL**：完整的 HTTPS 支持，证书验证
- **无数据日志**：默认不记录请求/响应
- **审计跟踪**：合规的可选指标

### 🔄 **上下文控制与内存管理**
高级对话管理，带上下文控制：

```rust
// 忽略之前的消息，保留系统指令
let request = ChatCompletionRequest::new(model, messages)
    .ignore_previous();

// 上下文窗口管理
let request = ChatCompletionRequest::new(model, messages)
    .with_max_tokens(1000)
    .with_temperature(0.7);
```

### 📁 **文件上传与多模态处理**
自动文件处理，支持上传和内联：

```rust
// 本地文件上传，自动大小检测
let message = Message {
    role: Role::User,
    content: Content::Image {
        url: None,
        mime: Some("image/jpeg".into()),
        name: Some("./local_image.jpg".into()),
    },
    function_call: None,
};

// 远程文件引用
let message = Message {
    role: Role::User,
    content: Content::Image {
        url: Some("https://example.com/image.jpg".into()),
        mime: Some("image/jpeg".into()),
        name: None,
    },
    function_call: None,
};
```

## 🌍 支持的 AI 提供商

| 提供商 | 架构 | 流式 | 模型 | 特殊功能 |
|--------|------|------|------|----------|
| **Groq** | 配置驱动 | ✅ | llama3-8b/70b, mixtral-8x7b | 快速推理，低延迟 |
| **DeepSeek** | 配置驱动 | ✅ | deepseek-chat, deepseek-reasoner | 中国专注，成本效益 |
| **Anthropic** | 配置驱动 | ✅ | claude-3.5-sonnet | 自定义认证，高质量 |
| **Google Gemini** | 独立 | 🔄 | gemini-1.5-pro/flash | URL 认证，多模态 |
| **OpenAI** | 独立 | ✅ | gpt-3.5-turbo, gpt-4 | 代理支持，函数调用 |
| **Qwen** | 配置驱动 | ✅ | Qwen 系列 | OpenAI 兼容，阿里云 |
| **百度文心** | 配置驱动 | ✅ | ernie-3.5, ernie-4.0 | 千帆平台，中文模型 |
| **腾讯混元** | 配置驱动 | ✅ | 混元系列 | 云端点，企业级 |
| **科大讯飞星火** | 配置驱动 | ✅ | 星火系列 | 语音+文本友好，多模态 |
| **月之暗面 Kimi** | 配置驱动 | ✅ | kimi 系列 | 长文本场景，上下文感知 |
| **Mistral** | 独立 | ✅ | mistral 模型 | 欧洲专注，开源权重 |
| **Cohere** | 独立 | ✅ | command/generate | 命令模型，RAG 优化 |
| **HuggingFace** | 配置驱动 | ✅ | hub 模型 | 开源，社区模型 |
| **TogetherAI** | 配置驱动 | ✅ | together 模型 | 成本效益，GPU 访问 |
| **Azure OpenAI** | 配置驱动 | ✅ | Azure 模型 | 企业级，合规 |
| **Ollama** | 配置驱动 | ✅ | 本地模型 | 自托管，隐私优先 |
| **xAI Grok** | 配置驱动 | ✅ | grok 模型 | xAI 平台，实时数据 |

## 🚀 快速开始

### 安装
```toml
[dependencies]
ai-lib = "0.2.11"
tokio = { version = "1.0", features = ["full"] }
futures = "0.3"
```

### 基本使用
```rust
use ai_lib::{AiClient, Provider, ChatCompletionRequest, Message, Role, Content};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 创建客户端，自动配置检测
    let client = AiClient::new(Provider::Groq)?;
    
    // 准备请求
    let request = ChatCompletionRequest::new(
        "llama3-8b-8192".to_string(),
        vec![Message {
            role: Role::User,
            content: Content::new_text("来自 ai-lib 的问候！"),
            function_call: None,
        }],
    );
    
    // 发送请求
    let response = client.chat_completion(request).await?;
    println!("响应: {}", response.choices[0].message.content.as_text());
    
    Ok(())
}
```

### 生产环境最佳实践
```rust
use ai_lib::{AiClientBuilder, Provider, CustomModelManager, ModelSelectionStrategy};
use std::time::Duration;

// 1. 使用构建器模式进行高级配置
let client = AiClientBuilder::new(Provider::Groq)
    .with_timeout(Duration::from_secs(30))
    .with_pool_config(16, Duration::from_secs(60))
    .build()?;

// 2. 实现模型管理
let mut manager = CustomModelManager::new("groq")
    .with_strategy(ModelSelectionStrategy::CostBased);

// 3. 添加健康检查和监控
let mut array = ModelArray::new("production")
    .with_strategy(LoadBalancingStrategy::HealthBased);
```

## 📚 示例

### 入门指南
- **快速开始**: `cargo run --example quickstart` - 简单使用指南
- **基本使用**: `cargo run --example basic_usage` - 核心功能
- **构建器模式**: `cargo run --example builder_pattern` - 配置示例

### 高级功能
- **模型管理**: `cargo run --example model_management` - 自定义管理器和负载均衡
- **批处理**: `cargo run --example batch_processing` - 高效批处理操作
- **函数调用**: `cargo run --example function_call_openai` - 函数调用示例
- **多模态**: `cargo run --example multimodal_example` - 图像和音频支持

### 配置与测试
- **配置检查**: `cargo run --example check_config` - 验证您的设置
- **网络诊断**: `cargo run --example network_diagnosis` - 故障排除连接
- **代理测试**: `cargo run --example proxy_example` - 代理配置
- **显式配置**: `cargo run --example explicit_config` - 运行时配置

### 核心功能
- **架构**: `cargo run --example test_hybrid_architecture` - 混合设计演示
- **流式**: `cargo run --example test_streaming_improved` - 实时流式
- **重试**: `cargo run --example test_retry_mechanism` - 错误处理
- **提供商**: `cargo run --example test_all_providers` - 多提供商测试

## 💼 使用场景与最佳实践

### 🏢 企业应用
```rust
// 多提供商负载均衡，高可用性
let mut array = ModelArray::new("production")
    .with_strategy(LoadBalancingStrategy::HealthBased);

array.add_endpoint(ModelEndpoint {
    name: "groq-primary".to_string(),
    url: "https://api.groq.com".to_string(),
    weight: 0.7,
    healthy: true,
});

array.add_endpoint(ModelEndpoint {
    name: "openai-fallback".to_string(),
    url: "https://api.openai.com".to_string(),
    weight: 0.3,
    healthy: true,
});
```

### 🔬 研发环境
```rust
// 轻松进行提供商比较研究
let providers = vec![Provider::Groq, Provider::OpenAI, Provider::Anthropic];

for provider in providers {
    let client = AiClient::new(provider)?;
    let response = client.chat_completion(request.clone()).await?;
    println!("{}: {}", provider, response.choices[0].message.content.as_text());
}
```

### 🚀 生产部署
```rust
// 生产就绪配置，带监控
let client = AiClientBuilder::new(Provider::Groq)
    .with_timeout(Duration::from_secs(30))
    .with_pool_config(16, Duration::from_secs(60))
    .with_metrics(Arc::new(CustomMetrics))
    .build()?;
```

### 🔒 隐私优先应用
```rust
// 自托管 Ollama，用于隐私敏感应用
let client = AiClientBuilder::new(Provider::Ollama)
    .with_base_url("http://localhost:11434")
    .without_proxy() // 确保无外部连接
    .build()?;
```

## 🎛️ 配置管理

### 环境变量
```bash
# 必需：API 密钥
export GROQ_API_KEY=your_groq_api_key
export OPENAI_API_KEY=your_openai_api_key
export DEEPSEEK_API_KEY=your_deepseek_api_key

# 可选：代理配置
export AI_PROXY_URL=http://proxy.example.com:8080

# 可选：提供商特定的 Base URLs
export GROQ_BASE_URL=https://custom.groq.com
export DEEPSEEK_BASE_URL=https://custom.deepseek.com
export OLLAMA_BASE_URL=http://localhost:11434

# 可选：超时配置
export AI_TIMEOUT_SECS=30
```

### 配置验证
ai-lib 提供内置工具来验证您的配置：

```bash
# 检查所有配置设置
cargo run --example check_config

# 诊断网络连接
cargo run --example network_diagnosis

# 测试代理配置
cargo run --example proxy_example
```

### 显式配置
需要显式配置注入的场景：

```rust
use ai_lib::{AiClient, Provider, ConnectionOptions};

let opts = ConnectionOptions {
    base_url: Some("https://custom.groq.com".into()),
    proxy: Some("http://proxy.example.com:8080".into()),
    api_key: Some("explicit-key".into()),
    timeout: Some(Duration::from_secs(45)),
    disable_proxy: false,
};

let client = AiClient::with_options(Provider::Groq, opts)?;
```

## 🏗️ 模型管理工具

### 主要功能
- **选择策略**：轮询、加权、基于性能、基于成本
- **负载均衡**：健康检查、连接跟踪、多端点
- **成本分析**：计算不同 token 数量的成本
- **性能指标**：速度和质量层级，响应时间跟踪

### 使用示例
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
    pricing: PricingInfo::new(0.05, 0.10), // $0.05/1K 输入, $0.10/1K 输出
    performance: PerformanceMetrics::new()
        .with_speed(SpeedTier::Fast)
        .with_quality(QualityTier::Good),
};

manager.add_model(model);
```

## 📊 性能与基准测试

### 🚀 性能特征
- **内存占用**：基本使用 <2MB
- **请求开销**：<1ms 每请求
- **流式延迟**：<10ms 首块
- **并发请求**：1000+ 并发连接
- **吞吐量**：现代硬件上 10,000+ 请求/秒

### 🔧 性能优化技巧
```rust
// 高吞吐量应用使用连接池
let client = AiClientBuilder::new(Provider::Groq)
    .with_pool_config(32, Duration::from_secs(90))
    .build()?;

// 多个请求的批处理
let responses = client.chat_completion_batch(requests, Some(10)).await?;

// 实时应用的流式处理
let mut stream = client.chat_completion_stream(request).await?;
```

### 📈 可扩展性功能
- **水平扩展**：多个客户端实例
- **负载均衡**：内置提供商负载均衡
- **健康检查**：自动端点健康监控
- **断路器**：自动故障检测
- **速率限制**：可配置请求节流

## 🚧 路线图

### ✅ 已实现
- 混合架构和通用流式
- 企业级错误处理和重试
- 多模态原语和函数调用
- 渐进式客户端配置
- 自定义模型管理工具
- 负载均衡和健康检查
- 系统配置管理
- 批处理能力
- 全面的指标和可观测性
- 性能优化
- 安全功能

### 🚧 计划中
- 高级背压 API
- 连接池调优
- 插件系统
- 内置缓存
- 配置热重载
- 高级安全功能
- GraphQL 支持
- WebSocket 流式

## 🤝 贡献

1. 克隆: `git clone https://github.com/hiddenpath/ai-lib.git`
2. 分支: `git checkout -b feature/new-feature`
3. 测试: `cargo test`
4. PR: 开启拉取请求

## 📖 社区与支持

- 📖 **文档**: [docs.rs/ai-lib](https://docs.rs/ai-lib)
- 🐛 **问题**: [GitHub Issues](https://github.com/hiddenpath/ai-lib/issues)
- 💬 **讨论**: [GitHub Discussions](https://github.com/hiddenpath/ai-lib/discussions)

## 📄 许可证

双重许可：MIT 或 Apache 2.0

## 📚 引用

```bibtex
@software{ai-lib,
    title = {ai-lib: A Unified AI SDK for Rust},
    author = {AI-lib Contributors},
    url = {https://github.com/hiddenpath/ai-lib},
    year = {2024}
}
```

## 🏆 为什么选择 ai-lib？

### 🎯 **统一体验**
- **单一 API**：学习一次，到处使用
- **提供商无关**：切换提供商无需代码更改
- **一致接口**：所有提供商使用相同模式

### ⚡ **性能优先**
- **最小开销**：<1ms 请求开销
- **高吞吐量**：10,000+ 请求/秒
- **低内存**：<2MB 占用
- **快速流式**：<10ms 首块

### 🛡️ **企业就绪**
- **生产级**：为规模和可靠性而构建
- **安全专注**：无数据日志，代理支持
- **监控就绪**：全面的指标和可观测性
- **合规友好**：审计跟踪和隐私控制

### 🔧 **开发者友好**
- **渐进式配置**：从简单到高级
- **丰富示例**：30+ 示例覆盖所有功能
- **全面文档**：详细的文档和指南
- **活跃社区**：开源，积极开发

### 🌍 **全球支持**
- **17+ 提供商**：覆盖所有主要 AI 平台
- **多区域**：支持全球部署
- **本地选项**：自托管 Ollama 支持
- **中国专注**：与中国提供商深度集成

---

<div align="center">
  ai-lib: Rust 生态中最全面的统一 AI SDK。🦀✨
  
  **准备构建 AI 应用的未来？** 🚀
</div>