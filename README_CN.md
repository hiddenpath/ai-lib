# ai-lib 🦀✨  

> 统一、可靠、高性能的多厂商AI SDK for Rust

一个生产级、厂商无关的SDK，为17+个AI平台（OpenAI、Groq、Anthropic、Gemini、Mistral、Cohere、Azure OpenAI、Ollama、DeepSeek、Qwen、文心一言、混元、讯飞星火、Kimi、HuggingFace、TogetherAI、xAI Grok等）提供统一的Rust API。  
消除分散的认证流程、流式格式、错误语义、模型命名差异和不一致的函数调用。从一行脚本扩展到多区域、多厂商系统，无需重写集成代码。

---
[官方网站](https://www.ailib.info/)

## 🚀 核心价值（TL;DR）

ai-lib统一了：
- 跨异构模型厂商的聊天和多模态请求
- 统一流式（统一SSE解析器 + JSONL 协议）与一致的增量
- 函数调用语义（含 OpenAI 风格 tool_calls 对齐）
- 推理模型支持（结构化、流式、JSON格式）
- 批处理工作流
- 可靠性原语（重试、退避、超时、代理、健康检查、负载策略）
- 模型选择（成本/性能/健康/加权）
- 可观测性钩子
- 渐进式配置（环境变量 → 构建器 → 显式注入 → 自定义传输）

您专注于产品逻辑；ai-lib处理基础设施摩擦。

---

## 📚 目录
1. 适用场景/不适用场景
2. 架构概述
3. 渐进式复杂度阶梯
4. 快速开始
5. 核心概念
6. 关键功能集群
7. 代码示例（要点）
8. 配置与诊断
9. 可靠性与弹性
10. 模型管理与负载均衡
11. 可观测性与指标
12. 安全与隐私
13. 支持的厂商
14. 示例目录
15. 性能特征
16. 路线图
17. 常见问题
18. 贡献指南
19. 许可证与引用
20. 为什么选择ai-lib？

---

## 🎯 适用场景/不适用场景

| 场景 | ✅ 使用ai-lib | ⚠️ 可能不适合 |
|------|--------------|-----------------|
| 快速切换AI厂商 | ✅ | |
| 统一流式输出 | ✅ | |
| 生产可靠性（重试、代理、超时） | ✅ | |
| 负载均衡/成本/性能策略 | ✅ | |
| 混合本地（Ollama）+ 云厂商 | ✅ | |
| 一次性脚本仅调用OpenAI | | ⚠️ 使用官方SDK |
| 深度厂商专属测试版API | | ⚠️ 直接使用厂商SDK |

---

## 🏗️ 架构概述

```
┌───────────────────────────────────────────────────────────┐
│                        您的应用程序                        │
└───────────────▲─────────────────────────▲─────────────────┘
                │                         │
        高级API                    高级控制
                │                         │
        AiClient / Builder   ←  模型管理/指标/批处理/工具
                │
        ┌────────── 统一抽象层 ────────────┐
        │  厂商适配器（混合：配置+独立）    │
        └──────┬────────────┬────────────┬────────────────┘
               │            │            │
        OpenAI / Groq   Gemini / Mistral  Ollama / 区域/其他
               │
        传输层（HTTP + 流式 + 重试 + 代理 + 超时）
               │
        通用类型（请求/消息/内容/工具/错误）
```

设计原则：
- 混合适配器模型（尽可能配置驱动，必要时自定义）
- 严格的核心类型 = 一致的人机工程学
- 可扩展：插入自定义传输和指标而无需分叉
- 渐进式分层：从简单开始，安全扩展

---

## 🪜 渐进式复杂度阶梯

| 级别 | 意图 | API表面 |
|------|------|---------|
| L1 | 一次性/脚本 | `AiClient::quick_chat_text()` |
| L2 | 基本集成 | `AiClient::new(provider)` |
| L3 | 受控运行时 | `AiClientBuilder`（超时、代理、基础URL） |
| L4 | 可靠性和规模 | 连接池、批处理、流式、重试 |
| L5 | 优化 | 模型数组、选择策略、指标 |
| L6 | 扩展 | 自定义传输、自定义指标、仪表化 |

---

## ⚙️ 快速开始

### 安装
```toml
[dependencies]
ai-lib = "0.3.1"
tokio = { version = "1", features = ["full"] }
futures = "0.3"
```

### 最快方式
```rust
use ai_lib::Provider;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let reply = ai_lib::AiClient::quick_chat_text(Provider::Groq, "Ping?").await?;
    println!("Reply: {reply}");
    Ok(())
}
```

### 标准聊天
```rust
use ai_lib::{AiClient, Provider, Message, Role, Content, ChatCompletionRequest};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = AiClient::new(Provider::OpenAI)?;
    let req = ChatCompletionRequest::new(
        client.default_chat_model(),
        vec![Message {
            role: Role::User,
            content: Content::new_text("用一句话解释Rust所有权。"),
            function_call: None,
        }]
    );
    let resp = client.chat_completion(req).await?;
    println!("Answer: {}", resp.first_text()?);
    Ok(())
}
```

### 流式传输
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
|------|------|
| Provider | 枚举所有支持的厂商 |
| AiClient / Builder | 主入口点；配置封装 |
| ChatCompletionRequest | 统一请求负载 |
| Message / Content | 文本/图像/音频/（未来结构化） |
| Function / Tool | 统一函数调用语义 |
| Streaming Event | 厂商标准化增量流 |
| ModelManager / ModelArray | 策略驱动的模型编排 |
| ConnectionOptions | 显式运行时覆盖 |
| Metrics Trait | 自定义可观测性集成 |
| Transport | 可注入的HTTP + 流式实现 |

---

## 💡 关键功能集群

1. 统一厂商抽象（无每厂商分支）
2. 通用流式传输（统一SSE解析器 + JSONL；带回退模拟）
3. 多模态原语（文本/图像/音频）
4. 函数调用（一致的工具模式；tool_calls 兼容）
5. 推理模型支持（结构化、流式、JSON格式）
6. 批处理（顺序/有界并发/智能策略）
7. 可靠性：重试、错误分类、超时、代理、池、拦截器管线（特性）
8. 模型管理：性能/成本/健康/轮询/加权
9. 可观测性：可插拔指标和计时
10. 安全性：隔离、无默认内容日志
11. 可扩展性：自定义传输、指标、策略注入

---

## 🧪 要点示例（精简）

### 厂商切换
```rust
let groq = AiClient::new(Provider::Groq)?;
let gemini = AiClient::new(Provider::Gemini)?;
let claude = AiClient::new(Provider::Anthropic)?;
```

### 函数调用
```rust
use ai_lib::{Tool, FunctionCallPolicy};
let tool = Tool::new_json(
    "get_weather",
    Some("获取天气信息"),
    serde_json::json!({"type":"object","properties":{"location":{"type":"string"}},"required":["location"]})
);
let req = ChatCompletionRequest::new(model, messages)
    .with_functions(vec![tool])
    .with_function_call(FunctionCallPolicy::Auto);
```

### 批处理
```rust
let responses = client.chat_completion_batch(requests.clone(), Some(8)).await?;
let smart = client.chat_completion_batch_smart(requests).await?;
```

### 多模态（图像）
```rust
let msg = Message {
    role: Role::User,
    content: ai_lib::types::common::Content::Image {
    url: Some("https://example.com/image.jpg".into()),
    mime: Some("image/jpeg".into()),
    name: None,
    },
    function_call: None,
};
```

### 推理模型
```rust
// 结构化推理与函数调用
let reasoning_tool = Tool::new_json(
    "step_by_step_reasoning",
    Some("执行步骤化推理"),
    serde_json::json!({
        "type": "object",
        "properties": {
            "problem": {"type": "string"},
            "steps": {"type": "array", "items": {"type": "object"}},
            "final_answer": {"type": "string"}
        }
    })
);

let request = ChatCompletionRequest::new(model, messages)
    .with_functions(vec![reasoning_tool])
    .with_function_call(FunctionCallPolicy::Auto);

// 流式推理
let mut stream = client.chat_completion_stream(request).await?;
while let Some(chunk) = stream.next().await {
    if let Some(content) = &chunk?.choices[0].delta.content {
        print!("{}", content);
    }
}

// 厂商特定推理配置
let request = ChatCompletionRequest::new(model, messages)
    .with_provider_specific("reasoning_format", serde_json::Value::String("parsed".to_string()))
    .with_provider_specific("reasoning_effort", serde_json::Value::String("high".to_string()));
```

### 重试感知
```rust
match client.chat_completion(req).await {
    Ok(r) => println!("{}", r.first_text()?),
    Err(e) if e.is_retryable() => { /* 安排重试 */ }
    Err(e) => eprintln!("永久失败: {e}")
}
```

---

## 🔑 配置与诊断

### 环境变量（基于约定）
```bash
# API密钥
export OPENAI_API_KEY=...
export GROQ_API_KEY=...
export GEMINI_API_KEY=...
export ANTHROPIC_API_KEY=...
export DEEPSEEK_API_KEY=...

# 可选基础URL
export GROQ_BASE_URL=https://custom.groq.com

# 代理
export AI_PROXY_URL=http://proxy.internal:8080

# 全局超时（秒）
export AI_TIMEOUT_SECS=30

# 可选：成本指标（启用 `cost_metrics` 特性时生效）
export COST_INPUT_PER_1K=0.5
export COST_OUTPUT_PER_1K=1.5

# 可选：HTTP 连接池参数（默认已启用连接池）
# 每主机最大空闲连接数
export AI_HTTP_POOL_MAX_IDLE_PER_HOST=32
# 空闲连接超时（毫秒）
export AI_HTTP_POOL_IDLE_TIMEOUT_MS=90000
```

### 显式覆盖
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

### 配置验证
```bash
cargo run --example check_config
cargo run --example network_diagnosis
cargo run --example proxy_example
```

### ℹ️ 指示性定价查询（可选）

优先使用环境变量（启用 `cost_metrics`）：`COST_INPUT_PER_1K`、`COST_OUTPUT_PER_1K`。
未设置时，可选择性查询一个“指示性”的默认表：

```rust
// 首选 env；如果未设置，可使用指示性估算
let usd = ai_lib::metrics::cost::estimate_usd(1000, 2000); // 若设置则使用 env

// 可选：指示性查询（仅 OSS，非合同价）
if let Some(p) = ai_lib::provider::pricing::get_pricing(ai_lib::Provider::DeepSeek, "deepseek-chat") {
    let approx = p.calculate_cost(1000, 2000);
    println!("指示性成本 ≈ ${:.4}", approx);
}
```

说明：
- 数值仅为代表性参考；请以供应商/合同价目为准。
- PRO 部署建议使用集中价目目录与热更新，而非静态查表。

---

## 🛡️ 可靠性与弹性

| 方面 | 能力 |
|------|------|
| 重试 | 指数退避 + 分类 |
| 错误 | 区分瞬态与永久 |
| 超时 | 每请求可配置 |
| 代理 | 全局/每连接/禁用 |
| 连接池 | 可调大小 + 生命周期 |
| 健康检查 | 端点状态 + 基于策略的避免 |
| 负载策略 | 轮询/加权/健康/性能/成本 |
| 回退 | 多厂商数组/手动分层 |

---

### ❗ 错误与重试语义

ai-lib 将厂商与 HTTP 失败统一映射为结构化错误，便于一致处理：

- 认证：401/403 → `AuthenticationError`
- 限流：429/409/425 → `RateLimitExceeded`
- 超时：显式超时或 408 → `TimeoutError`
- 服务器瞬态：5xx → `NetworkError`（可重试）
- 传输启发式：连接/超时 → `NetworkError`/`TimeoutError`
- JSON：`DeserializationError`；无效URL/配置：`ConfigurationError`

辅助方法：

```rust
if err.is_retryable() {
    tokio::time::sleep(Duration::from_millis(err.retry_delay_ms())).await;
    // 重试...
}
```

厂商说明（仅供了解——均已由 ai-lib 统一处理）：
- Gemini：通过 `x-goog-api-key` 认证，SSE 流式。ai-lib 已自动设置请求头并标准化事件，无需编写厂商特定代码。参见 `https://ai.google.dev/api`。
- Anthropic：使用 `x-api-key` 与版本头。ai-lib 已自动设置并标准化增量，无需编写厂商特定代码。参见 `https://docs.anthropic.com/en/api/overview`。

## 🧭 模型管理与负载均衡

```rust
use ai_lib::{AiClientBuilder, ChatCompletionRequest, Message, Provider, Role};
use ai_lib::types::common::Content;
use ai_lib::provider::models::{ModelArray, ModelEndpoint, LoadBalancingStrategy};

// 构建 ModelArray 并通过 builder 挂载（需启用 feature: routing_mvp）
let mut array = ModelArray::new("prod").with_strategy(LoadBalancingStrategy::RoundRobin);
array.add_endpoint(ModelEndpoint {
    name: "groq-70b".to_string(),
    model_name: "llama-3.3-70b-versatile".to_string(),
    url: "https://api.groq.com".to_string(),
    weight: 1.0,
    healthy: true,
    connection_count: 0,
});
array.add_endpoint(ModelEndpoint {
    name: "groq-8b".to_string(),
    model_name: "llama-3.1-8b-instant".to_string(),
    url: "https://api.groq.com".to_string(),
    weight: 1.0,
    healthy: true,
    connection_count: 0,
});

let client = AiClientBuilder::new(Provider::Groq)
    .with_routing_array(array)
    .build()?;

// 使用占位模型 "__route__" 触发路由
let req = ChatCompletionRequest::new(
    "__route__".to_string(),
    vec![Message { role: Role::User, content: Content::new_text("打个招呼"), function_call: None }]
);
let resp = client.chat_completion(req).await?;
println!("已选择模型: {}", resp.model);
# Ok::<(), ai_lib::AiLibError>(())
```

- 最小健康检查：选择端点时，客户端会在使用前探测 `{base_url}`（或 OpenAI 兼容路径 `{base_url}/models`）。
- 指标（`routing_mvp` 特性下）：
  - `routing_mvp.request`
  - `routing_mvp.selected`
  - `routing_mvp.health_fail`
  - `routing_mvp.fallback_default`
  - `routing_mvp.no_endpoint`
  - `routing_mvp.missing_array`

---

## 📊 可观测性与指标

实现`Metrics`特征以桥接Prometheus、OpenTelemetry、StatsD等。

```rust
struct CustomMetrics;
#[async_trait::async_trait]
impl ai_lib::metrics::Metrics for CustomMetrics {
    async fn incr_counter(&self, name: &str, value: u64) { /* ... */ }
    async fn start_timer(&self, name: &str) -> Option<Box<dyn ai_lib::metrics::Timer + Send>> { /* ... */ }
}
let client = AiClient::new_with_metrics(Provider::Groq, Arc::new(CustomMetrics))?;
```

#### 采集 routing_mvp 指标

启用 `routing_mvp` 后，客户端在路由过程中会触发以下计数器：

```rust
// 可能出现的指标键：
// routing_mvp.request, routing_mvp.selected, routing_mvp.health_fail,
// routing_mvp.fallback_default, routing_mvp.no_endpoint, routing_mvp.missing_array

use std::sync::Arc;
use ai_lib::{AiClientBuilder, Provider};

struct PrintMetrics;
#[async_trait::async_trait]
impl ai_lib::metrics::Metrics for PrintMetrics {
    async fn incr_counter(&self, name: &str, value: u64) { println!("cnt {} += {}", name, value); }
    async fn record_gauge(&self, name: &str, value: f64) { println!("gauge {} = {}", name, value); }
    async fn start_timer(&self, _name: &str) -> Option<Box<dyn ai_lib::metrics::Timer + Send>> { None }
    async fn record_histogram(&self, name: &str, value: f64) { println!("hist {} = {}", name, value); }
    async fn record_histogram_with_tags(&self, name: &str, value: f64, tags: &[(&str, &str)]) { println!("hist {} = {} tags={:?}", name, value, tags); }
    async fn incr_counter_with_tags(&self, name: &str, value: u64, tags: &[(&str, &str)]) { println!("cnt {} += {} tags={:?}", name, value, tags); }
    async fn record_gauge_with_tags(&self, name: &str, value: f64, tags: &[(&str, &str)]) { println!("gauge {} = {} tags={:?}", name, value, tags); }
    async fn record_error(&self, name: &str, error_type: &str) { println!("error {} type={}", name, error_type); }
    async fn record_success(&self, name: &str, success: bool) { println!("success {} = {}", name, success); }
}

let metrics = Arc::new(PrintMetrics);
let client = AiClientBuilder::new(Provider::Groq)
    .with_metrics(metrics)
    .build()?;
```

### 特性开关（可选）

- `interceptors`：拦截器 trait 与管线（示例：interceptors_pipeline）
- `unified_sse`：通用 SSE 解析器（`GenericAdapter` 已可接入）
- `unified_transport`：共享 reqwest 客户端工厂
- `cost_metrics`：基于环境变量的最小成本核算（见上方 COST_* 配置）
- `routing_mvp`：启用 `ModelArray` 路由；将请求的 model 设为 "__route__" 触发路由

这些功能通过环境变量进行配置，适合大多数使用场景。

### 企业级功能

对于高级企业级功能，请考虑 [ai-lib-pro]：

- **高级路由**: 策略驱动路由、健康监控、自动故障转移
- **企业可观测性**: 结构化日志、指标、分布式追踪
- **成本管理**: 集中定价表和预算跟踪
- **配额管理**: 租户/组织配额和速率限制
- **审计与合规**: 带脱敏的综合审计跟踪
- **安全性**: 信封加密和密钥管理
- **配置**: 热重载配置管理

ai-lib-pro在开源ai-lib基础上构建，无破坏性更改，为企业用户提供无缝升级路径。

### 分层：OSS 与 PRO

- **OSS（本仓库）**：统一接口、流式、重试/超时/代理、可配置连接池、轻量限流与背压、批处理并发控制。偏向环境变量驱动，零外部中台依赖，开箱即可用。
- **PRO**：多租户配额与优先级、自适应并发/限流、策略驱动路由、集中配置与热更新、深度可观测性与导出、审计/合规、集中价目与预算护栏。无需改业务代码即可平滑升级。

#### 本地验证矩阵
```bash
# 代码规范（将警告视为错误）
cargo clippy --all-features -- -D warnings

# 默认测试集
cargo test

# 特性测试集
cargo test --features unified_sse
cargo test --features "cost_metrics routing_mvp"

# 构建所有示例
cargo build --examples

# 关键示例快速运行
cargo run --example quickstart
cargo run --example proxy_example
cargo run --features interceptors --example interceptors_pipeline
cargo run --features "interceptors unified_sse" --example mistral_features
```

---

## 🔒 安全与隐私

| 功能 | 描述 |
|------|------|
| 无隐式日志 | 默认不记录请求/响应 |
| 密钥隔离 | API密钥来自环境或显式结构 |
| 代理控制 | 允许/禁用/覆盖 |
| TLS | 标准HTTPS与验证 |
| 审计钩子 | 使用指标层进行合规审计计数器 |
| 本地优先 | Ollama集成用于敏感上下文 |

---

## 🌍 支持的厂商（快照）

| 厂商 | 适配器类型 | 流式 | 备注 |
|------|------------|------|------|
| Groq | 配置驱动 | ✅ | 超低延迟 |
| OpenAI | 独立 | ✅ | 函数调用 |
| Anthropic (Claude) | 配置驱动 | ✅ | 高质量 |
| Google Gemini | 独立 | ✅ | 使用 `x-goog-api-key` 头；SSE 走 `streamGenerateContent` |
| Mistral | 独立 | ✅ | 欧洲模型 |
| Cohere | 独立 | ✅ | RAG优化 |
| HuggingFace | 配置驱动 | ✅ | 开放模型 |
| TogetherAI | 配置驱动 | ✅ | 成本效益 |
| DeepSeek | 配置驱动 | ✅ | 推理模型 |
| Qwen | 配置驱动 | ✅ | 中文生态 |
| 百度文心一言 | 配置驱动 | ✅ | 企业CN |
| 腾讯混元 | 配置驱动 | ✅ | 云集成 |
| 讯飞星火 | 配置驱动 | ✅ | 语音+多模态 |
| Moonshot Kimi | 配置驱动 | ✅ | 长上下文 |
| Azure OpenAI | 配置驱动 | ✅ | 企业合规 |
| Ollama | 配置驱动 | ✅ | 本地/气隙 |
| xAI Grok | 配置驱动 | ✅ | 实时导向 |

（流式列：🔄 = 统一适配/回退）

---

## 🗂️ 示例目录（在/examples中）

| 类别 | 示例 |
|------|------|
| 入门 | quickstart / basic_usage / builder_pattern |
| 配置 | explicit_config / proxy_example / custom_transport_config |
| 流式 | test_streaming / cohere_stream |
| 可靠性 | custom_transport / concurrency_best_practices |
| 多厂商 | config_driven_example / model_override_demo |
| 模型管理 | model_management |
| 批处理 | batch_processing |
| 函数调用 | function_call_openai / function_call_exec |
| 多模态 | multimodal_example |
| 架构演示 | architecture_progress |
| 专业 | ascii_horse / hello_groq |

补充（流式）：gemini_streaming / anthropic_streaming / mistral_streaming / deepseek_streaming

### 故障排查（Gemini 404）

- 现象：v1beta `generateContent` 调用 `models/gemini-pro` 返回 NOT_FOUND
- 解决：使用 `gemini-1.5-flash`（当前 v1beta 支持）或先列出模型确认
- 示例：`cargo run --example gemini_streaming`

### 流式快速运行

```bash
# Gemini（设置密钥后运行）
$env:GEMINI_API_KEY="your_key"; cargo run --example gemini_streaming

# Anthropic（设置密钥后运行）
$env:ANTHROPIC_API_KEY="your_key"; cargo run --example anthropic_streaming
```

### 请求级覆盖（代理/超时/API Key）

```rust
use ai_lib::{AiClient, Provider, ConnectionOptions};
let client = AiClient::with_options(
    Provider::Groq,
    ConnectionOptions { proxy: Some("http://localhost:8080".into()), timeout: Some(Duration::from_secs(45)), ..Default::default() }
)?;
```

---

## 📊 性能（指示性和方法论基础）

下面的数字描述了ai-lib本身的SDK层开销，而不是模型推理时间。  
它们是代表性的（不是保证），来自使用模拟传输的受控基准测试，除非另有说明。

| 指标 | 观察范围（典型） | 精确定义 | 测量上下文 |
|------|------------------|----------|------------|
| 每请求SDK开销 | ~0.6–0.9 ms | 从构建ChatCompletionRequest到移交HTTP请求的时间 | 发布构建，模拟传输，256B提示，单线程预热 |
| 流式传输增加延迟 | <2 ms | ai-lib的流式解析与直接reqwest SSE相比引入的额外延迟 | 500次运行，Groq llama3-8b，平均 |
| 基线内存占用 | ~1.7 MB | 初始化一个AiClient + 连接池后的常驻集 | Linux (x86_64)，池=16，无批处理 |
| 可持续模拟吞吐量 | 11K–13K req/s | 每秒完成的请求期货（短提示） | 模拟传输，并发=512，池=32 |
| 真实厂商短提示吞吐量 | 厂商限制 | 端到端包括网络+厂商限制 | 严重依赖供应商限制 |
| 流式块解析成本 | ~8–15 µs / 块 | 解析+分发一个SSE增量 | 合成30–50令牌流 |
| 批处理并发扩展 | 近线性到~512任务 | 调度争用前的降级点 | Tokio多线程运行时 |

### 🔬 方法论

1. 硬件：AMD 7950X（32线程），64GB RAM，NVMe SSD，Linux 6.x  
2. 工具链：Rust 1.79（稳定），`--release`，LTO=thin，默认分配器  
3. 隔离：使用模拟传输排除网络+厂商推理方差  
4. 预热：丢弃前200次迭代（JIT、缓存、分配器稳定）  
5. 计时：`std::time::Instant`用于宏吞吐量；Criterion用于微开销  
6. 流式：具有真实令牌节奏的合成SSE帧（8–25 ms）  
7. 厂商测试：仅作为说明性（受速率限制和区域延迟影响）  

### 🧪 重现（一旦添加基准套件）

```bash
# 微开销（请求构建+序列化）
cargo bench --bench micro_overhead

# 模拟高并发吞吐量
cargo run --example bench_mock_throughput -- --concurrency 512 --duration 15s

# 流式解析成本
cargo bench --bench stream_parse
```

### 背压与并发上限（可选）

- 简单做法：在批量接口上设置并发上限 `concurrency_limit`
- 全局做法：使用 Builder 提供的最大并发门闸（信号量）

```rust
use ai_lib::{AiClientBuilder, Provider};

// 为当前客户端设置全局最大并发（例如 64）
let client = AiClientBuilder::new(Provider::Groq)
    .with_max_concurrency(64)
    .for_production() // 可选：加载生产预设（包含保守的限流/熔断/背压）
    .build()?;
```

说明：
- 该门闸在 `chat_completion` 与流式接口中获取许可，直到调用完成/流结束自动释放。
- 若无可用许可，将返回 `RateLimitExceeded`，可配合重试/排队策略使用。

计划的基准布局（即将推出）：
```
/bench
  micro/
    bench_overhead.rs
    bench_stream_parse.rs
  macro/
    mock_throughput.rs
    streaming_latency.rs
  provider/ (可选门控)
    groq_latency.rs
```

### 📌 解释指南

- "SDK开销" = ai-lib内部处理（类型构造、序列化、分发准备）— 排除远程模型延迟。
- "吞吐量"数字假设快速返回的模拟响应；真实世界云吞吐量通常受厂商速率限制约束。
- 内存数字是常驻集快照；具有日志/指标的生产系统可能增加开销。
- 结果将在不同硬件、OS调度器、分配器策略和运行时调优上变化。

### ⚠️ 免责声明

> 这些指标是指示性的，不是合同保证。始终使用您的工作负载、提示大小、模型组合和部署环境进行基准测试。  
> 可重现的基准测试工具和JSON快照基线将在存储库中版本化以跟踪回归。

### 💡 优化技巧

- 在高吞吐量场景中使用`.with_pool_config(size, idle_timeout)`
- 为低延迟UX优先使用流式传输
- 使用并发限制批处理相关短提示
- 避免冗余客户端实例化（重用客户端）
- 考虑厂商特定速率限制和区域延迟

---

## 🗺️ 路线图（计划序列）

| 阶段 | 计划功能 |
|------|----------|
| 1 | 高级背压和自适应速率协调 |
| 2 | 内置缓存层（请求/结果分层） |
| 3 | 实时配置热重载 |
| 4 | 插件/拦截器系统 |
| 5 | GraphQL表面 |
| 6 | WebSocket原生流式传输 |
| 7 | 增强安全性（密钥轮换、KMS集成） |
| 8 | 公共基准测试工具+夜间回归检查 |

### 🧪 性能监控路线图

计划公共基准测试工具+夜间（仅模拟）回归检查以：
- 早期检测性能回归
- 提供历史趋势数据
- 允许贡献者验证PR的影响

---

## ❓ 常见问题

| 问题 | 答案 |
|------|------|
| 如何A/B测试厂商？ | 使用带有负载策略的`ModelArray` |
| 重试是内置的吗？ | 自动分类+退避；您可以分层自定义循环 |
| 我可以禁用代理吗？ | `.without_proxy()`或选项中的`disable_proxy = true` |
| 我可以为测试模拟吗？ | 注入自定义传输 |
| 您记录PII吗？ | 默认不记录内容 |
| 函数调用差异？ | 通过`Tool` + `FunctionCallPolicy`标准化 |
| 支持本地推理吗？ | 是的，通过Ollama（自托管） |
| 如何知道错误是否可重试？ | `error.is_retryable()`助手 |

---

## 🤝 贡献指南

1. Fork & 克隆仓库  
2. 创建功能分支：`git checkout -b feature/your-feature`  
3. 运行测试：`cargo test`  
4. 如果引入新功能则添加示例  
5. 遵循适配器分层（在自定义之前优先配置驱动）  
6. 打开PR并说明理由+基准测试（如果影响性能）  

我们重视：清晰度、测试覆盖率、最小表面区域蔓延、增量可组合性。

---

## 📄 许可证

双重许可：
- MIT
- Apache许可证（版本2.0）

您可以选择最适合您项目的许可证。

---

## 📚 引用

```bibtex
@software{ai-lib,
    title = {ai-lib: A Unified AI SDK for Rust},
    author = {ai-lib Contributors},
    url = {https://github.com/hiddenpath/ai-lib},
    year = {2024}
}
```

---

## 🏆 为什么选择ai-lib？

| 维度 | 价值 |
|------|------|
| 工程速度 | 一个抽象=更少的定制适配器 |
| 风险缓解 | 多厂商回退和健康路由 |
| 运营稳健性 | 重试、池化、诊断、指标 |
| 成本控制 | 成本/性能策略旋钮 |
| 可扩展性 | 可插拔传输和指标 |
| 面向未来 | 清晰的路线图+混合适配器模式 |
| 人机工程学 | 渐进式API—无过早复杂性 |
| 性能 | 最小延迟和内存开销 |

---

<div align="center">
  <strong>ai-lib：在Rust中构建弹性、快速、多厂商AI系统—无胶水代码疲劳。</strong><br/><br/>
  ⭐ 如果这为您节省了时间，请给它一个星标并在Issues/Discussions中分享反馈！
</div>