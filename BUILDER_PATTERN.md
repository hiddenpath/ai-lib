# AI-lib 构建器模式 (Builder Pattern)

## 概述

AI-lib 现在支持构建器模式，为开发者提供了更灵活、层次化的客户端配置方式。这个新功能解决了原有代码中客户端创建复杂、缺乏递进自定义层次的问题。

## 核心特性

### 🎯 自动环境变量检测
- 自动检测 `GROQ_BASE_URL`、`AI_PROXY_URL` 等环境变量
- 如果没有设置，使用 ai-lib 默认配置
- 开发者无需手动配置即可快速开始

### 🔧 递进的自定义层次
- **Level 1**: 最简单的用法 - `AiClient::new(Provider::Groq)`
- **Level 2**: 自定义 base_url - `.with_base_url("https://custom.groq.com")`
- **Level 3**: 自定义代理 - `.with_proxy("http://proxy.example.com:8080")`
- **Level 4**: 高级配置 - `.with_timeout()` 和 `.with_pool_config()`

### ⚡ 向后兼容
- 现有的 `AiClient::new()` 方法完全兼容
- 自动使用新的构建器逻辑
- 无需修改现有代码

## 使用方法

### 1. 最简单的用法（推荐新手）

```rust
use ai_lib::{AiClient, Provider};

// 自动检测环境变量，使用默认配置
let client = AiClient::new(Provider::Groq)?;
```

### 2. 自定义配置

```rust
use ai_lib::{AiClientBuilder, Provider};

// 自定义 base_url 和 proxy
let client = AiClientBuilder::new(Provider::Groq)
    .with_base_url("https://custom.groq.com")
    .with_proxy("http://proxy.example.com:8080")
    .build()?;
```

### 3. 完全自定义配置

```rust
use ai_lib::{AiClientBuilder, Provider};
use std::time::Duration;

let client = AiClientBuilder::new(Provider::Groq)
    .with_base_url("https://custom.groq.com")
    .with_proxy("http://proxy.example.com:8080")
    .with_timeout(Duration::from_secs(60))
    .with_pool_config(32, Duration::from_secs(90))
    .build()?;
```

### 4. 使用便捷的 builder 方法

```rust
use ai_lib::{AiClient, Provider};

let client = AiClient::builder(Provider::Groq)
    .with_base_url("https://custom.groq.com")
    .with_proxy("http://proxy.example.com:8080")
    .build()?;
```

## 配置优先级

构建器按照以下优先级应用配置：

1. **显式设置** (通过 `with_*` 方法)
2. **环境变量**
3. **默认配置**

### 环境变量支持

| 提供商 | Base URL 环境变量 | 说明 |
|--------|------------------|------|
| Groq | `GROQ_BASE_URL` | 自定义 Groq 服务器地址 |
| DeepSeek | `DEEPSEEK_BASE_URL` | 自定义 DeepSeek 服务器地址 |
| Ollama | `OLLAMA_BASE_URL` | 自定义 Ollama 服务器地址 |
| 通用代理 | `AI_PROXY_URL` | 所有提供商的代理服务器 |

## 支持的提供商

### ✅ 支持自定义配置的提供商
- **Groq** - 支持自定义 base_url 和 proxy
- **DeepSeek** - 支持自定义 base_url 和 proxy
- **Ollama** - 支持自定义 base_url 和 proxy
- **Qwen** - 支持自定义 base_url 和 proxy
- **Baidu Wenxin** - 支持自定义 base_url 和 proxy
- **Tencent Hunyuan** - 支持自定义 base_url 和 proxy
- **iFlytek Spark** - 支持自定义 base_url 和 proxy
- **Moonshot** - 支持自定义 base_url 和 proxy
- **Anthropic** - 支持自定义 base_url 和 proxy
- **Azure OpenAI** - 支持自定义 base_url 和 proxy
- **HuggingFace** - 支持自定义 base_url 和 proxy
- **TogetherAI** - 支持自定义 base_url 和 proxy

### ⚠️ 不支持自定义配置的提供商
- **OpenAI** - 使用独立适配器
- **Gemini** - 使用独立适配器
- **Mistral** - 使用独立适配器
- **Cohere** - 使用独立适配器

## 实际应用场景

### 场景1：开发环境
```rust
// 使用本地 Ollama 服务器
let client = AiClientBuilder::new(Provider::Ollama)
    .with_base_url("http://localhost:11434")
    .build()?;
```

### 场景2：企业环境
```rust
// 使用企业代理和自定义服务器
let client = AiClientBuilder::new(Provider::Groq)
    .with_base_url("https://ai.internal.company.com")
    .with_proxy("http://proxy.company.com:8080")
    .with_timeout(Duration::from_secs(120))
    .with_pool_config(64, Duration::from_secs(300))
    .build()?;
```

### 场景3：多区域部署
```rust
// 根据环境变量选择不同区域
let base_url = match std::env::var("AI_REGION") {
    Ok(region) => format!("https://ai.{}.company.com", region),
    Err(_) => "https://ai.default.company.com".to_string(),
};

let client = AiClientBuilder::new(Provider::Groq)
    .with_base_url(&base_url)
    .build()?;
```

## 错误处理

构建器会正确处理各种配置错误：

```rust
// 为不支持的提供商设置自定义配置会返回错误
match AiClientBuilder::new(Provider::OpenAI)
    .with_base_url("https://custom.openai.com")
    .build()
{
    Ok(_) => println!("这不应该成功"),
    Err(e) => println!("正确捕获错误: {}", e),
}
```

## 性能优化

### 连接池配置
```rust
let client = AiClientBuilder::new(Provider::Groq)
    .with_pool_config(32, Duration::from_secs(90))
    .build()?;
```

### 超时配置
```rust
let client = AiClientBuilder::new(Provider::Groq)
    .with_timeout(Duration::from_secs(60))
    .build()?;
```

## 迁移指南

### 从旧版本迁移

**旧代码：**
```rust
let client = AiClient::new(Provider::Groq)?;
```

**新代码（无需修改）：**
```rust
let client = AiClient::new(Provider::Groq)?;  // 完全兼容
```

**新代码（使用构建器）：**
```rust
let client = AiClientBuilder::new(Provider::Groq).build()?;
```

### 添加自定义配置

**旧方式（需要手动创建配置）：**
```rust
// 需要手动创建 HttpTransportConfig 和 ProviderConfig
let transport_config = HttpTransportConfig { /* ... */ };
let transport = HttpTransport::new_with_config(transport_config)?;
let provider_config = ProviderConfigs::groq();
let adapter = GenericAdapter::with_transport(provider_config, transport)?;
```

**新方式（使用构建器）：**
```rust
let client = AiClientBuilder::new(Provider::Groq)
    .with_base_url("https://custom.groq.com")
    .with_proxy("http://proxy.example.com:8080")
    .build()?;
```

## 最佳实践

### 1. 开发阶段
```rust
// 使用默认配置，快速开始
let client = AiClient::new(Provider::Groq)?;
```

### 2. 测试阶段
```rust
// 使用环境变量配置
export GROQ_BASE_URL=https://test.groq.com
export AI_PROXY_URL=http://test.proxy.com:8080

let client = AiClientBuilder::new(Provider::Groq).build()?;
```

### 3. 生产阶段
```rust
// 显式配置，确保稳定性
let client = AiClientBuilder::new(Provider::Groq)
    .with_base_url("https://prod.groq.com")
    .with_proxy("http://prod.proxy.com:8080")
    .with_timeout(Duration::from_secs(60))
    .with_pool_config(32, Duration::from_secs(90))
    .build()?;
```

## 总结

AI-lib 的构建器模式为开发者提供了：

1. **🚀 快速开始** - 一行代码创建客户端，自动检测环境变量
2. **🔧 灵活配置** - 支持递进的自定义配置层次
3. **🔄 向后兼容** - 现有代码无需修改
4. **⚡ 性能优化** - 支持连接池和超时配置
5. **🌍 企业友好** - 支持代理、自定义服务器等企业需求

这个新功能让开发者能够以最快的速度写出第一个AI应用程序，同时保持代码的清晰性和可维护性。
