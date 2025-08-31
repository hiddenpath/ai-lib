# AI-lib: Rust 统一 AI SDK

> **一个使用混合架构为多个 AI 提供商提供单一接口的统一 Rust SDK**

## 概述

**ai-lib** 是一个统一的 Rust AI SDK，为多个大语言模型提供商提供单一、一致的接口。它使用混合架构，平衡了开发者体验和提供商特定功能，提供从简单使用到高级定制的渐进式配置选项，以及构建自定义模型管理器和负载均衡数组的强大工具。

**注意**：升级指南和 PR 说明已移至 `docs/` 目录。请参阅 `docs/UPGRADE_0.2.0.md` 和 `docs/PR_0.2.0.md` 了解迁移和 PR 详情。

## 支持的 AI 提供商

- ✅ **Groq** (配置驱动) — llama3, mixtral 模型
- ✅ **xAI Grok** (配置驱动) — grok 模型
- ✅ **DeepSeek** (配置驱动) — deepseek-chat, deepseek-reasoner
- ✅ **Anthropic Claude** (配置驱动) — claude-3.5-sonnet
- ✅ **Google Gemini** (独立适配器) — gemini-1.5-pro, gemini-1.5-flash
- ✅ **OpenAI** (独立适配器) — gpt-3.5-turbo, gpt-4
- ✅ **Qwen / 通义千问** (配置驱动) — Qwen 系列 (OpenAI 兼容)
- ✅ **Cohere** (独立适配器) — command/generate 模型
- ✅ **百度文心 (ERNIE)** (配置驱动) — ernie-3.5, ernie-4.0
- ✅ **腾讯混元** (配置驱动) — 混元系列
- ✅ **科大讯飞星火** (配置驱动) — 星火模型 (语音+文本友好)
- ✅ **月之暗面 / Kimi** (配置驱动) — kimi 系列 (长文本场景)
- ✅ **Mistral** (独立适配器) — mistral 模型
- ✅ **Hugging Face Inference** (配置驱动) — hub 托管模型
- ✅ **TogetherAI** (配置驱动) — together.ai 托管模型
- ✅ **Azure OpenAI** (配置驱动) — Azure 托管的 OpenAI 端点
- ✅ **Ollama** (配置驱动/本地) — 本地 Ollama 实例

## 核心功能

### 🚀 **统一接口和提供商切换**
用一行代码在 AI 提供商之间切换：

```rust
let groq_client = AiClient::new(Provider::Groq)?;
let gemini_client = AiClient::new(Provider::Gemini)?;
let claude_client = AiClient::new(Provider::Anthropic)?;
```

### 🎯 **渐进式配置**
构建具有渐进式定制级别的 AI 客户端：

```rust
// 第1级：自动检测的简单使用
let client = AiClient::new(Provider::Groq)?;

// 第2级：自定义基础 URL
let client = AiClientBuilder::new(Provider::Groq)
    .with_base_url("https://custom.groq.com")
    .build()?;

// 第3级：添加代理支持
let client = AiClientBuilder::new(Provider::Groq)
    .with_base_url("https://custom.groq.com")
    .with_proxy("http://proxy.example.com:8080")
    .build()?;

// 第4级：高级配置
let client = AiClientBuilder::new(Provider::Groq)
    .with_base_url("https://custom.groq.com")
    .with_proxy("http://proxy.example.com:8080")
    .with_timeout(Duration::from_secs(60))
    .with_pool_config(32, Duration::from_secs(90))
    .build()?;
```

### 🌊 **通用流式支持**
为所有提供商提供实时流式响应，支持 SSE 解析和回退模拟：

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

### 🔄 **企业级可靠性**
- **自动重试**：指数退避重试机制
- **智能错误分类**：区分可重试和永久性错误
- **代理支持**：支持身份验证的 HTTP/HTTPS 代理
- **超时管理**：可配置超时和优雅降级

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

### ⚡ **混合架构**
- **配置驱动适配器**：OpenAI 兼容 API 的最小布线
- **独立适配器**：独特 API 的完全控制
- **四层设计**：客户端 → 适配器 → 传输 → 通用类型
- **优势**：代码重用、可扩展性、自动功能继承

### 🏗️ **自定义模型管理**
构建复杂的模型管理系统：

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

### 📊 **高级功能**
- **多模态支持**：文本、JSON、图像和音频内容
- **函数调用**：统一的 `Tool` 和 `FunctionCall` 类型
- **指标和可观测性**：请求计数器和持续时间计时器
- **依赖注入**：用于测试的模拟传输
- **性能**：<2MB 内存、<1ms 开销、<10ms 流式延迟

## 快速开始

### 安装
```toml
[dependencies]
ai-lib = "0.2.1"
tokio = { version = "1.0", features = ["full"] }
futures = "0.3"
```

### 基本使用
```rust
use ai_lib::{AiClient, Provider, ChatCompletionRequest, Message, Role, Content};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = AiClient::new(Provider::Groq)?;
    let req = ChatCompletionRequest::new(
        "test-model".to_string(),
        vec![Message { role: Role::User, content: Content::new_text("来自 ai-lib 的问候"), function_call: None }]
    );
    Ok(())
}
```

### 生产环境最佳实践
```rust
use ai_lib::{AiClientBuilder, Provider, CustomModelManager, ModelSelectionStrategy};

// 1. 使用环境变量进行配置
let client = AiClientBuilder::new(Provider::Groq)
    .with_timeout(Duration::from_secs(30))
    .build()?;

// 2. 实现模型管理
let mut manager = CustomModelManager::new("groq")
    .with_strategy(ModelSelectionStrategy::CostBased);

// 3. 添加健康检查和监控
let mut array = ModelArray::new("production")
    .with_strategy(LoadBalancingStrategy::HealthBased);
```

### 环境变量
```bash
export GROQ_API_KEY=your_groq_api_key
export AI_PROXY_URL=https://proxy.example.com:8080
```

## 示例

### 入门指南
- **快速开始**: `cargo run --example quickstart` - 简单使用指南
- **构建器模式**: `cargo run --example builder_pattern` - 配置示例

### 高级功能
- **模型管理**: `cargo run --example model_management` - 自定义管理器和负载均衡

### 核心功能
- **架构**: `cargo run --example test_hybrid_architecture`
- **流式**: `cargo run --example test_streaming_improved`
- **重试**: `cargo run --example test_retry_mechanism`
- **提供商**: `cargo run --example test_groq_generic`

## 提供商详情

| 提供商 | 架构 | 流式 | 模型 | 说明 |
|--------|------|------|------|------|
| **Groq** | 配置驱动 | ✅ | llama3-8b/70b, mixtral-8x7b | 快速推理 |
| **DeepSeek** | 配置驱动 | ✅ | deepseek-chat, deepseek-reasoner | 中国专注 |
| **Anthropic** | 配置驱动 | ✅ | claude-3.5-sonnet | 自定义认证 |
| **Google Gemini** | 独立 | 🔄 | gemini-1.5-pro/flash | URL 认证 |
| **OpenAI** | 独立 | ✅ | gpt-3.5-turbo, gpt-4 | 可能需要代理 |
| **Qwen** | 配置驱动 | ✅ | Qwen 系列 | OpenAI 兼容 |
| **百度文心** | 配置驱动 | ✅ | ernie-3.5, ernie-4.0 | 千帆平台 |
| **腾讯混元** | 配置驱动 | ✅ | 混元系列 | 云端点 |
| **科大讯飞星火** | 配置驱动 | ✅ | 星火系列 | 语音+文本友好 |
| **月之暗面 Kimi** | 配置驱动 | ✅ | kimi 系列 | 长文本场景 |

## 模型管理工具

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

## 路线图

### ✅ 已实现
- 混合架构和通用流式支持
- 企业级错误处理和重试
- 多模态原语和函数调用
- 渐进式客户端配置
- 自定义模型管理工具
- 负载均衡和健康检查

### 🚧 计划中
- 高级背压 API
- 连接池调优
- 插件系统
- 内置缓存

## 贡献

1. 克隆: `git clone https://github.com/hiddenpath/ai-lib.git`
2. 分支: `git checkout -b feature/new-feature`
3. 测试: `cargo test`
4. PR: 开启拉取请求

## 社区和支持

- 📖 **文档**: [docs.rs/ai-lib](https://docs.rs/ai-lib)
- 🐛 **问题**: [GitHub Issues](https://github.com/hiddenpath/ai-lib/issues)
- 💬 **讨论**: [GitHub Discussions](https://github.com/hiddenpath/ai-lib/discussions)

## 许可证

双重许可：MIT 或 Apache 2.0

## 引用

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
  ai-lib: Rust 生态系统中最全面的统一 AI SDK。🦀✨
</div>