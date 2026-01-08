# ⚠️ 重要公告：本仓库已停止更新

> **本仓库（ai-lib）已停止维护，不再接受新的 PR、Issue 或功能更新。**

## 🚀 迁移到 ai-lib-rust

**ai-lib 项目已完全转向 [ai-lib-rust](https://github.com/hiddenpath/ai-lib-rust)**，这是一个全新的、基于 **manifest-first（协议优先）** 和 **数据驱动** 架构的通用 AI 接口运行时。

### 为什么迁移？

- ✅ **协议驱动架构**: 所有逻辑由 YAML 协议文件驱动，无需硬编码 provider 逻辑
- ✅ **统一标准**: 基于 [AI-Protocol](https://github.com/hiddenpath/ai-protocol) 规范，确保跨运行时一致性
- ✅ **更简洁的 API**: 开发者友好的接口，避免复杂混乱的用户界面
- ✅ **更好的可维护性**: 模块化设计，清晰的架构分层
- ✅ **生产就绪**: 完整的测试覆盖、CI/CD 集成、协议验证

### 如何迁移？

1. **查看新项目**: [ai-lib-rust](https://github.com/hiddenpath/ai-lib-rust)
2. **查看协议规范**: [AI-Protocol](https://github.com/hiddenpath/ai-protocol)
3. **迁移指南**: 请参考 ai-lib-rust 的 README 和示例代码，或查看 [MIGRATION_GUIDE.md](./MIGRATION_GUIDE.md)

### 获取帮助

- **新项目 Issues**: [ai-lib-rust Issues](https://github.com/hiddenpath/ai-lib-rust/issues)
- **协议规范**: [AI-Protocol](https://github.com/hiddenpath/ai-protocol)
- **讨论**: 请在 ai-lib-rust 仓库中提出问题和建议

---

# ai-lib 🦀✨ (已停止维护)

> 面向 Rust 的统一、可靠、高性能多厂商 AI SDK

**⚠️ 注意**: 本仓库已停止更新。请迁移到 [ai-lib-rust](https://github.com/hiddenpath/ai-lib-rust)。

一个生产级、厂商无关的 SDK，提供面向 20+ 家且持续增加 的 AI 平台的统一 Rust API（OpenAI、Groq、Anthropic、Gemini、Mistral、Cohere、Azure OpenAI、Ollama、DeepSeek、Qwen、百度文心、腾讯混元、讯飞星火、Kimi、HuggingFace、TogetherAI、xAI Grok、OpenRouter、Replicate、Perplexity、AI21、智谱AI、MiniMax 等）。  
它消除了分散的认证流程、流式格式、错误语义、模型命名差异和不一致的函数调用。无需重写集成代码，即可从一行脚本扩展到生产系统。

---
[官方网站](https://www.ailib.info/)

## 🚀 核心价值

ai-lib 将多家 AI 厂商的复杂性统一为一个简洁的人体工学 Rust 接口：

- **通用 API**：在所有厂商上统一的聊天、多模态与函数调用
- **多模态内容**：便捷的图像和音频内容创建，支持 `Content::from_image_file()` 和 `Content::from_audio_file()`
- **统一流式**：一致的 SSE/JSONL 解析与实时增量
- **可靠性**：内置重试、超时、熔断与错误分类
- **灵活配置**：环境变量、Builder 模式或显式覆盖
- **生产就绪**：连接池、代理支持、可观测性钩子

**结果**：你专注产品逻辑，ai-lib 处理供应商集成的繁琐工作。

> 导入建议：应用层优先使用 `use ai_lib::prelude::*;` 获取最小常用集；库作者建议按领域显式导入。参见模块树与导入模式指南：`docs/MODULE_TREE_AND_IMPORTS.md`。

## ⚙️ 快速开始

### 安装

基础安装（核心功能）：
```toml
[dependencies]
ai-lib = "0.4.0"
tokio = { version = "1", features = ["full"] }
futures = "0.3"
```

启用流式支持：
```toml
[dependencies]
ai-lib = { version = "0.4.0", features = ["streaming"] }
tokio = { version = "1", features = ["full"] }
futures = "0.3"
```

完整功能（流式、弹性、路由）：
```toml
[dependencies]
ai-lib = { version = "0.4.0", features = ["all"] }
tokio = { version = "1", features = ["full"] }
futures = "0.3"
```

### 简单用法
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

### 标准用法
```rust
// 应用层可以使用 prelude 来最小化导入
use ai_lib::prelude::*;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = AiClient::new(Provider::OpenAI)?;
    let req = ChatCompletionRequest::new(
        "gpt-3.5-turbo".to_string(), // 显式模型或使用 client.default_chat_model()
        vec![Message {
            role: Role::User,
            content: Content::Text("Explain Rust ownership in one sentence.".to_string()),
            function_call: None,
        }],
    );
    // .with_extension("parallel_tool_calls", serde_json::json!(true)); // 可选扩展

    let resp = client.chat_completion(req).await?;
    println!("Answer: {}", resp.choices[0].message.content.as_text());
    Ok(())
}
```

### 流式聊天

> **注意：** 流式功能需要启用 `streaming` 特性（或 `all` 特性）。

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

## 🧠 核心概念

| 概念 | 目的 |
|--------|---------|
| **Provider** | 枚举所有支持的 AI 厂商 |
| **AiClient** | 统一接口的主入口 |
| **ChatCompletionRequest** | 标准化的请求载荷 |
| **Message / Content** | 文本、图像、音频等内容类型 |
| **Streaming Event** | 厂商标准化的增量流 |
| **ConnectionOptions** | 运行时配置覆盖 |
| **Metrics Trait** | 自定义可观测性集成 |
| **Transport** | 可注入的 HTTP + 流式层 |
| **Usage / UsageStatus** | 响应级使用量元数据（令牌 + 状态）。从 `ai_lib::Usage` 或 `ai_lib::types::response::Usage` 导入 |

---

## 💡 关键特性

### 核心能力
- **统一厂商抽象**：跨所有厂商的单一 API
- **统一流式传输**：一致的 SSE/JSONL 解析与实时增量
- **多模态支持**：文本、图像、音频内容处理
- **函数调用**：一致的工具模式，兼容 OpenAI
- **批处理**：顺序和并发处理策略

### 可靠性与生产
- **内置弹性**：指数退避重试、熔断器
- **策略构建器**：`AiClientBuilder::with_round_robin_chain` / `with_failover_chain` 在运行前组合路由策略
- **错误分类**：区分瞬态与永久失败
- **连接管理**：池化、超时、代理支持
- **可观测性**：可插拔指标与追踪集成
- **安全**：默认不记录敏感内容

---

## 🌍 支持的厂商

*17+ 家且持续增加* —— 我们持续新增平台以适配演进中的生态。

| 厂商 | 流式 | 特点 |
|----------|-----------|------------|
| **Groq** | ✅ | 超低延迟推理 |
| **OpenAI** | ✅ | GPT 模型，函数调用 |
| **Anthropic** | ✅ | Claude 模型，高质量 |
| **Google Gemini** | ✅ | 多模态能力 |
| **Mistral** | ✅ | 欧洲模型 |
| **Cohere** | ✅ | RAG 优化 |
| **HuggingFace** | ✅ | 开源模型 |
| **TogetherAI** | ✅ | 成本效益推理 |
| **OpenRouter** | ✅ | 统一网关，多厂商模型路由 |
| **Replicate** | ✅ | 托管开源模型 |
| **DeepSeek** | ✅ | 推理导向模型 |
| **Qwen** | ✅ | 中文生态 |
| **百度文心** | ✅ | 企业级中国市场 |
| **腾讯混元** | ✅ | 云集成 |
| **讯飞星火** | ✅ | 语音 + 多模态 |
| **月之暗面Kimi** | ✅ | 长上下文模型 |
| **Azure OpenAI** | ✅ | 企业合规 |
| **Ollama** | ✅ | 本地/隔离部署 |
| **xAI Grok** | ✅ | 实时导向 |
| **Perplexity** | ✅ | 搜索增强对话 |
| **AI21** | ✅ | Jurassic 模型 |
| **智谱AI (GLM)** | ✅ | 中国 GLM 系列 |
| **MiniMax** | ✅ | 中国多模态 |

*参见 [examples/](examples/) 获取厂商特定使用模式。*

### 网关型提供商
ai-lib 支持 OpenRouter、Replicate 等网关型提供商，通过统一接口访问多个 AI 模型。网关平台使用 `provider/model` 格式的模型命名（如 `openai/gpt-4o`），而直接提供商使用原始模型名（如 `gpt-4o`）。

---

## 🔑 配置

### 环境变量
```bash
# API Keys（约定）
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

# 可选：自定义 Endpoint
export GROQ_BASE_URL=https://custom.groq.com

# 可选：代理与超时
export AI_PROXY_URL=http://proxy.internal:8080
export AI_TIMEOUT_SECS=30

# 可选：连接池（默认启用）
export AI_HTTP_POOL_MAX_IDLE_PER_HOST=32
export AI_HTTP_POOL_IDLE_TIMEOUT_MS=90000

# 可选：按厂商覆盖默认模型
export GROQ_MODEL=llama-3.1-8b-instant
export MISTRAL_MODEL=mistral-small-latest
export DEFAULT_AI_MODEL=gpt-4o-mini
```

### 模型选择与兜底

- **自动默认值**：构造 `ChatCompletionRequest` 时将 `model` 设为 `"auto"`（大小写不敏感）
  或空字符串，ai-lib 会自动注入该 Provider 的推荐模型，或采用
  `AiClientBuilder::with_default_chat_model` 的自定义值。
- **环境变量覆盖**：通过 `*_MODEL` 环境变量（如 `GROQ_MODEL`、`OPENAI_MODEL`）即可
  在不改代码的前提下切换默认模型。这些变量由新的 `ModelResolver` 统一读取，
  对普通调用、流式和批处理均生效。
- **无效模型恢复**：当后端返回 `invalid_model/model_not_found` 时，ai-lib 会自动
  尝试配置中的备选模型，并在最终的 `AiLibError::ModelNotFound` 中附带可操作提示
  与文档链接（例如 [Groq 模型列表](https://console.groq.com/docs/models)）。
- **运行时可见性**：调用 `client.default_chat_model()` 可以查询当前实际使用的模型，
  便于调试多 Provider failover/round-robin 的场景。

### 程序化配置
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

## 🔌 自定义提供商

使用 `CustomProviderBuilder` + `AiClientBuilder::with_strategy` 可以在不修改 `Provider` 枚举的情况下接入 OpenAI 兼容的自建网关或厂商预览版。完整示例参见 `examples/custom_provider_injection.rs`。

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

let client = AiClientBuilder::new(Provider::OpenAI) // 策略提供时枚举被忽略
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

### 厂商专属 Builder

每个 Provider 都对应一个专属 Builder（例如 `GroqBuilder`、`OpenAiBuilder`），用于更清晰地配置参数或在组合路由策略时复用。

```rust
use ai_lib::provider::GroqBuilder;

let client = GroqBuilder::new()
    .with_base_url("https://api.groq.com")
    .with_proxy(Some("http://proxy.internal:8080"))
    .build()?; // 返回 AiClient
```

### 并发控制
```rust
use ai_lib::{AiClientBuilder, Provider};

let client = AiClientBuilder::new(Provider::Groq)
    .with_max_concurrency(64)
    .for_production()
    .build()?;
```

---

## 🔁 路由与故障转移（OSS）

使用 `with_failover_chain` 或 `with_round_robin_chain` 在发送请求前构建路由策略。

```rust
use ai_lib::{client::AiClientBuilder, Provider};

let client = AiClientBuilder::new(Provider::OpenAI)
    .with_failover_chain(vec![Provider::Anthropic, Provider::Groq])?
    .build()?;
```

结合 `with_round_robin_chain` 或 `RoutingStrategyBuilder` 实现加权/轮询路由。策略组合现在在客户端构建时完成，无需运行时分支或哨兵模型。

## 🛡️ 可靠性与弹性

| 特性 | 描述 |
|---------|-------------|
| **重试逻辑** | 指数退避 + 智能错误分类 |
| **错误处理** | 区分瞬态与永久失败 |
| **超时** | 支持按请求与全局超时 |
| **代理** | 全局/按连接/禁用 |
| **连接池** | 可调池大小与连接生命周期 |
| **健康检查** | 端点监控与策略化选择 |
| **回退策略** | 多厂商数组与手动故障切换 |

---

## 📊 可观测性与指标

### 自定义指标集成
```rust
struct CustomMetrics;
#[async_trait::async_trait]
impl ai_lib::metrics::Metrics for CustomMetrics {
    async fn incr_counter(&self, name: &str, value: u64) { /* ... */ }
    async fn start_timer(&self, name: &str) -> Option<Box<dyn ai_lib::metrics::Timer + Send>> { /* ... */ }
}
let client = AiClient::new_with_metrics(Provider::Groq, Arc::new(CustomMetrics))?;
```

### 用量跟踪
```rust
match response.usage_status {
    UsageStatus::Finalized => println!("准确令牌计数: {:?}", response.usage),
    UsageStatus::Estimated => println!("估算令牌: {:?}", response.usage),
    UsageStatus::Pending => println!("使用量数据尚未可用"),
    UsageStatus::Unsupported => println!("厂商不支持使用量跟踪"),
}
```
迁移：`Usage`/`UsageStatus` 定义在 `ai_lib::types::response` 中，作为根级别的 re-export。在 1.0 版本前，从 `types::common` 的旧导入将被移除。

### 可选特性

默认情况下，ai-lib 仅启用最小功能集。根据需要启用特性：

| 特性 | 描述 | 别名 |
|---------|-------------|-------|
| `unified_sse` | 流式传输的通用 SSE 解析器 | `streaming` |
| `interceptors` | 重试、超时、熔断器管道 | `resilience` |
| `unified_transport` | 共享 HTTP 客户端工厂 | `transport` |
| `config_hot_reload` | 配置热重载 trait | `hot_reload` |
| `cost_metrics` | 基于环境变量的基础成本核算 | - |
| `routing_mvp` | 模型选择与路由功能 | - |
| `observability` | Tracer 和 AuditSink 接口 | - |
| `all` | 启用上述所有特性 | - |

**大多数应用推荐配置：**
```toml
ai-lib = { version = "0.4.0", features = ["streaming", "resilience"] }
```

---

## 🗂️ 示例

| 类别 | 示例 |
|----------|----------|
| **入门** | `quickstart`, `basic_usage`, `builder_pattern` |
| **配置** | `explicit_config`, `proxy_example`, `custom_transport_config` |
| **流式传输** | `test_streaming`, `cohere_stream` |
| **可靠性** | `custom_transport`, `resilience_example` |
| **多厂商** | `config_driven_example`, `model_override_demo`, `custom_provider_injection`, `routing_modelarray` |
| **模型管理** | `model_management`, `routing_modelarray` |
| **批处理** | `batch_processing` |
| **函数调用** | `function_call_openai`, `function_call_exec` |
| **多模态** | `multimodal_example` |
| **高级** | `architecture_progress`, `reasoning_best_practices` |

---

## 📄 许可证

在 MIT 或 Apache License 2.0 之下双重许可——可自由选择更适合你项目的许可。

---

## 🤝 贡献

1. Fork 并克隆仓库  
2. 创建功能分支：`git checkout -b feature/your-feature`  
3. 运行测试：`cargo test`  
4. 新功能请补充示例  
5. 遵循适配器模式（优先配置驱动而非自定义）  
6. 提交 PR 时附上动机与（若有性能影响）基准数据  

**我们重视**：清晰度、测试覆盖、最小表面积、增量可组合性。

---

## 📚 引用

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
  <strong>ai-lib：用 Rust 构建弹性、快速、多厂商的 AI 系统——告别集成疲劳。</strong><br/><br/>
  ⭐ 如果它帮你节省了时间，欢迎点亮 star，并在 Issues/Discussions 留言反馈！
</div>
