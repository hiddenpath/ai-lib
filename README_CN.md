# ai-lib 🦀✨  

> 面向 Rust 的统一、可靠、高性能多厂商 AI SDK

一个生产级、厂商无关的 SDK，提供面向 20+ 家且持续增加 的 AI 平台的统一 Rust API（OpenAI、Groq、Anthropic、Gemini、Mistral、Cohere、Azure OpenAI、Ollama、DeepSeek、Qwen、百度文心、腾讯混元、讯飞星火、Kimi、HuggingFace、TogetherAI、xAI Grok、OpenRouter、Replicate、Perplexity、AI21、智谱AI、MiniMax 等）。  
它消除了分散的认证流程、流式格式、错误语义、模型命名差异和不一致的函数调用。无需重写集成代码，即可从一行脚本扩展到生产系统。

---
[官方网站](https://www.ailib.info/)

## 🚀 核心价值

a i-lib 将多家 AI 厂商的复杂性统一为一个简洁的人体工学 Rust 接口：

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
```toml
[dependencies]
ai-lib = "0.3.4"
tokio = { version = "1", features = ["full"] }
futures = "0.3"
```

### 一行代码聊天
```rust
use ai_lib::Provider;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let reply = ai_lib::AiClient::quick_chat_text(Provider::Groq, "Hello!").await?;
    println!("Reply: {reply}");
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
        client.default_chat_model(),
        vec![Message {
            role: Role::User,
            content: Content::Text("Explain Rust ownership in one sentence.".to_string()),
            function_call: None,
        }]
    );
    let resp = client.chat_completion(req).await?;
    println!("Answer: {}", resp.choices[0].message.content.as_text());
    Ok(())
}
```

### 流式聊天
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
| **Usage / UsageStatus** | 响应级使用量元数据（令牌 + 状态），从 `ai_lib::Usage` 或 `ai_lib::types::response::Usage` 导入 |

---

## 💡 关键特性

### 核心能力
- **统一厂商抽象**：跨厂商单一 API
- **统一流式**：一致的 SSE/JSONL 解析与实时增量
- **多模态支持**：文本、图像、音频
- **函数调用**：一致的工具模式，兼容 OpenAI
- **批处理**：顺序与并发处理策略

### 可靠性与生产
- **内置弹性**：指数退避重试、熔断器
- **基础故障转移（OSS）**：使用 `AiClient::with_failover([...])` 在可重试错误时切换厂商
- **错误分类**：区分瞬态与永久失败
- **连接管理**：池化、超时、代理支持
- **可观测性**：可插拔指标与追踪集成
- **安全**：默认不记录敏感内容

---

## 🌍 支持的厂商

*17+ 家且持续增加* —— 我们持续新增平台以适配演进中的生态。

| 厂商 | 流式 | 特点 |
|----------|-----------|------------|
| **Groq** | ✅ | 超低延迟 |
| **OpenAI** | ✅ | GPT 系列、函数调用 |
| **Anthropic** | ✅ | Claude，高质量 |
| **Google Gemini** | ✅ | 多模态能力 |
| **Mistral** | ✅ | 欧洲模型 |
| **Cohere** | ✅ | RAG 优化 |
| **HuggingFace** | ✅ | 开源模型 |
| **TogetherAI** | ✅ | 性价比高 |
| **OpenRouter** | ✅ | 网关；支持 provider/model 路由 |
| **Replicate** | ✅ | 托管开源模型 |
| **DeepSeek** | ✅ | 推理模型 |
| **Qwen** | ✅ | 中文生态 |
| **百度文心** | ✅ | 企业级中国市场 |
| **腾讯混元** | ✅ | 云集成 |
| **讯飞星火** | ✅ | 语音 + 多模态 |
| **Kimi** | ✅ | 长上下文 |
| **Azure OpenAI** | ✅ | 企业合规 |
| **Ollama** | ✅ | 本地/气隙环境 |
| **xAI Grok** | ✅ | 实时导向 |
| **Perplexity** | ✅ | 搜索增强对话 |
| **AI21** | ✅ | Jurassic 系列 |
| **智谱AI (GLM)** | ✅ | 国产 GLM 系列 |
| **MiniMax** | ✅ | 国产多模态 |

*更多用法参见 [examples/](examples/)。*

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
```

### 代码配置
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

### 并发控制
```rust
use ai_lib::{AiClientBuilder, Provider};

let client = AiClientBuilder::new(Provider::Groq)
    .with_max_concurrency(64)
    .for_production()
    .build()?;
```

---

## 🔁 故障转移（OSS）

在网络错误、超时、限流或 5xx 等可重试错误出现时，通过 `with_failover` 定义有序的备用厂商链：

```rust
use ai_lib::{AiClient, Provider};

let client = AiClient::new(Provider::OpenAI)?
    .with_failover(vec![Provider::Anthropic, Provider::Groq]);
```

如与路由能力同时使用，模型选择会在故障转移过程中被保留。

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

### 可选特性
- `interceptors`：重试、超时、熔断管线
- `unified_sse`：统一 SSE 解析
- `unified_transport`：共享 HTTP 客户端工厂
- `cost_metrics`：基于环境变量的基础成本核算
- `routing_mvp`：模型选择与路由

---

## 🗂️ 示例

| 类别 | 示例 |
|----------|----------|
| **入门** | `quickstart`, `basic_usage`, `builder_pattern` |
| **配置** | `explicit_config`, `proxy_example`, `custom_transport_config` |
| **流式** | `test_streaming`, `cohere_stream` |
| **可靠性** | `custom_transport`, `resilience_example` |
| **多厂商** | `config_driven_example`, `model_override_demo` |
| **模型管理** | `model_management`, `routing_modelarray` |
| **批处理** | `batch_processing` |
| **函数调用** | `function_call_openai`, `function_call_exec` |
| **多模态** | `multimodal_example` |
| **进阶** | `architecture_progress`, `reasoning_best_practices` |

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
  <strong>ai-lib：用 Rust 构建弹性、快速、多厂商的 AI 系统——告别集邮式集成疲劳。</strong><br/><br/>
  ⭐ 如果它帮你节省了时间，欢迎点亮 star，并在 Issues/Discussions 留言反馈！
</div>
