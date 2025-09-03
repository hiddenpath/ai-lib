# ai-lib 🦀✨  
> 统一、可靠、高性能的多提供商 AI SDK for Rust

一个生产级的、提供商无关的 SDK，为您提供 17+ AI 平台的统一 Rust API（OpenAI、Groq、Anthropic、Gemini、Mistral、Cohere、Azure OpenAI、Ollama、DeepSeek、Qwen、文心、混元、讯飞星火、Kimi、HuggingFace、TogetherAI、xAI Grok 等）。  
消除分散的认证流程、流式格式、错误语义、模型命名差异和不一致的函数调用。从一行脚本扩展到多区域、多供应商系统，无需重写集成代码。

---

## 🚀 核心价值（TL;DR）

ai-lib 统一了：
- 跨异构模型提供商的聊天和多模态请求
- 流式传输（SSE + 模拟）与一致的增量
- 函数调用语义
- 批处理工作流
- 可靠性原语（重试、退避、超时、代理、健康、负载策略）
- 模型选择（成本/性能/健康/加权）
- 可观测性钩子
- 渐进式配置（环境变量 → 构建器 → 显式注入 → 自定义传输）

您专注于产品逻辑；ai-lib 处理基础设施摩擦。

---

## 📚 目录
1. 何时使用/何时不使用
2. 架构概述
3. 渐进式复杂度阶梯
4. 快速开始
5. 核心概念
6. 关键功能集群
7. 代码示例（精华）
8. 配置与诊断
9. 可靠性与弹性
10. 模型管理与负载均衡
11. 可观测性与指标
12. 安全与隐私
13. 支持的提供商
14. 示例目录
15. 性能特征
16. 路线图
17. 常见问题
18. 贡献指南
19. 许可证与引用
20. 为什么选择 ai-lib？

---

## 🎯 何时使用/何时不使用

| 场景 | ✅ 使用 ai-lib | ⚠️ 可能不适合 |
|------|---------------|---------------|
| 快速切换 AI 提供商 | ✅ | |
| 统一的流式输出 | ✅ | |
| 生产可靠性（重试、代理、超时） | ✅ | |
| 负载均衡/成本/性能策略 | ✅ | |
| 混合本地（Ollama）+ 云供应商 | ✅ | |
| 仅调用 OpenAI 的一次性脚本 | | ⚠️ 使用官方 SDK |
| 深度供应商专属测试版 API | | ⚠️ 直接使用供应商 SDK |

---

## 🏗️ 架构概述

```
┌───────────────────────────────────────────────────────────┐
│                        您的应用程序                       │
└───────────────▲─────────────────────────▲─────────────────┘
                │                         │
        高级 API                    高级控制
                │                         │
        AiClient / Builder   ←  模型管理 / 指标 / 批处理 / 工具
                │
        ┌────────── 统一抽象层 ────────────┐
        │  提供商适配器（混合：配置 + 独立）│
        └──────┬────────────┬────────────┬────────────────┘
               │            │            │
        OpenAI / Groq   Gemini / Mistral  Ollama / 区域 / 其他
               │
        传输层（HTTP + 流式 + 重试 + 代理 + 超时）
               │
        通用类型（请求 / 消息 / 内容 / 工具 / 错误）
```

设计原则：
- 混合适配器模型（尽可能配置驱动，必要时自定义）
- 严格的核心类型 = 一致的易用性
- 可扩展：插入自定义传输和指标而无需分叉
- 渐进式分层：从简单开始，安全扩展

---

## 🪜 渐进式复杂度阶梯

| 级别 | 意图 | API 表面 |
|------|------|----------|
| L1 | 一次性/脚本 | `AiClient::quick_chat_text()` |
| L2 | 基本集成 | `AiClient::new(provider)` |
| L3 | 受控运行时 | `AiClientBuilder`（超时、代理、基础 URL） |
| L4 | 可靠性与扩展 | 连接池、批处理、流式、重试 |
| L5 | 优化 | 模型数组、选择策略、指标 |
| L6 | 扩展 | 自定义传输、自定义指标、工具化 |

---

## ⚙️ 快速开始

### 安装
```toml
[dependencies]
ai-lib = "0.2.12"
tokio = { version = "1", features = ["full"] }
futures = "0.3"
```

### 最快方式
```rust
use ai_lib::Provider;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let reply = ai_lib::AiClient::quick_chat_text(Provider::Groq, "Ping?").await?;
    println!("回复: {reply}");
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
        vec![Message::user(Content::new_text("用一句话解释 Rust 所有权。"))]
    );
    let resp = client.chat_completion(req).await?;
    println!("答案: {}", resp.first_text()?);
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
| Provider | 枚举所有支持的供应商 |
| AiClient / Builder | 主入口点；配置封装 |
| ChatCompletionRequest | 统一请求负载 |
| Message / Content | 文本/图像/音频/（未来结构化） |
| Function / Tool | 统一函数调用语义 |
| Streaming Event | 提供商标准化增量流 |
| ModelManager / ModelArray | 策略驱动的模型编排 |
| ConnectionOptions | 显式运行时覆盖 |
| Metrics Trait | 自定义可观测性集成 |
| Transport | 可注入的 HTTP + 流式实现 |

---

## 💡 关键功能集群

1. 统一提供商抽象（无按供应商分支）
2. 通用流式传输（SSE + 回退模拟）
3. 多模态原语（文本/图像/音频）
4. 函数调用（一致的工具模式）
5. 批处理（顺序/有界并发/智能策略）
6. 可靠性：重试、错误分类、超时、代理、池
7. 模型管理：性能/成本/健康/轮询/加权
8. 可观测性：可插拔指标和计时
9. 安全性：隔离，默认不记录内容
10. 可扩展性：自定义传输、指标、策略注入

---

## 🧪 精华示例（浓缩版）

### 提供商切换
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
let msg = Message::user(ai_lib::types::common::Content::Image {
    url: Some("https://example.com/image.jpg".into()),
    mime: Some("image/jpeg".into()),
    name: None,
});
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
# API 密钥
export OPENAI_API_KEY=...
export GROQ_API_KEY=...
export DEEPSEEK_API_KEY=...

# 可选的基础 URL
export GROQ_BASE_URL=https://custom.groq.com

# 代理
export AI_PROXY_URL=http://proxy.internal:8080

# 全局超时（秒）
export AI_TIMEOUT_SECS=30
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

---

## 🛡️ 可靠性与弹性

| 方面 | 能力 |
|------|------|
| 重试 | 指数退避 + 分类 |
| 错误 | 区分瞬态与永久 |
| 超时 | 每请求可配置 |
| 代理 | 全局/每连接/禁用 |
| 连接池 | 可调大小 + 生命周期 |
| 健康 | 端点状态 + 基于策略的避免 |
| 负载策略 | 轮询/加权/健康/性能/成本 |
| 回退 | 多提供商数组/手动分层 |

---

## 🧭 模型管理与负载均衡

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

支持：
- 性能层级
- 成本比较
- 基于健康的过滤
- 加权分布
- 为自适应策略做好准备

---

## 📊 可观测性与指标

实现 `Metrics` trait 以桥接 Prometheus、OpenTelemetry、StatsD 等。

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

## 🔒 安全与隐私

| 功能 | 描述 |
|------|------|
| 无隐式记录 | 默认不记录请求/响应 |
| 密钥隔离 | API 密钥来自环境变量或显式结构 |
| 代理控制 | 允许/禁用/覆盖 |
| TLS | 标准 HTTPS 与验证 |
| 审计钩子 | 使用指标层进行合规审计计数 |
| 本地优先 | 敏感上下文的 Ollama 集成 |

---

## 🌍 支持的提供商（快照）

| 提供商 | 适配器类型 | 流式传输 | 备注 |
|--------|------------|----------|------|
| Groq | config-driven | ✅ | 超低延迟 |
| OpenAI | independent | ✅ | 函数调用 |
| Anthropic (Claude) | config-driven | ✅ | 高质量 |
| Google Gemini | independent | 🔄 (统一) | 多模态重点 |
| Mistral | independent | ✅ | 欧洲模型 |
| Cohere | independent | ✅ | RAG 优化 |
| HuggingFace | config-driven | ✅ | 开源模型 |
| TogetherAI | config-driven | ✅ | 成本效益 |
| DeepSeek | config-driven | ✅ | 推理模型 |
| Qwen | config-driven | ✅ | 中文生态 |
| 百度文心 | config-driven | ✅ | 企业级中文 |
| 腾讯混元 | config-driven | ✅ | 云集成 |
| 讯飞星火 | config-driven | ✅ | 语音 + 多模态 |
| Moonshot Kimi | config-driven | ✅ | 长上下文 |
| Azure OpenAI | config-driven | ✅ | 企业合规 |
| Ollama | config-driven | ✅ | 本地/隔离 |
| xAI Grok | config-driven | ✅ | 实时导向 |

（流式传输列：🔄 = 统一适配/回退）

---

## 🗂️ 示例目录（在 /examples 中）

| 类别 | 示例 |
|------|------|
| 入门 | quickstart / basic_usage / builder_pattern |
| 配置 | explicit_config / proxy_example / custom_transport_config |
| 流式传输 | test_streaming / cohere_stream |
| 可靠性 | custom_transport |
| 多提供商 | config_driven_example / model_override_demo |
| 模型管理 | model_management |
| 批处理 | batch_processing |
| 函数调用 | function_call_openai / function_call_exec |
| 多模态 | multimodal_example |
| 架构演示 | architecture_progress |
| 专业 | ascii_horse / hello_groq |

---

## 📊 性能（指示性 & 基于方法论）

以下数字描述 ai-lib 本身的 SDK 层开销，不包括模型推理时间。  
它们是代表性的（非保证），来自使用模拟传输的受控基准测试，除非另有说明。

| 指标 | 观察范围（典型） | 精确定义 | 测量上下文 |
|------|------------------|----------|------------|
| 每请求 SDK 开销 | ~0.6–0.9 ms | 从构建 ChatCompletionRequest 到移交 HTTP 请求的时间 | 发布构建，模拟传输，256B 提示，单线程预热 |
| 流式传输增加延迟 | <2 ms | ai-lib 流式解析相对于直接 reqwest SSE 引入的额外延迟 | 500 次运行，Groq llama3-8b，平均 |
| 基线内存占用 | ~1.7 MB | 初始化一个 AiClient + 连接池后的常驻集 | Linux (x86_64)，池=16，无批处理 |
| 可持续模拟吞吐量 | 11K–13K req/s | 每秒完成的请求未来（短提示） | 模拟传输，并发=512，池=32 |
| 真实提供商短提示吞吐量 | 提供商限制 | 端到端包括网络 + 提供商限制 | 严重依赖供应商限制 |
| 流式块解析成本 | ~8–15 µs / 块 | 解析 + 分发一个 SSE 增量 | 合成 30–50 令牌流 |
| 批处理并发扩展 | 近线性到 ~512 任务 | 调度争用前的降级点 | Tokio 多线程运行时 |

### 🔬 方法论

1. 硬件：AMD 7950X（32 线程），64GB RAM，NVMe SSD，Linux 6.x  
2. 工具链：Rust 1.79（稳定版），`--release`，LTO=thin，默认分配器  
3. 隔离：使用模拟传输排除网络 + 提供商推理差异  
4. 预热：丢弃前 200 次迭代（JIT、缓存、分配器稳定）  
5. 计时：`std::time::Instant` 用于宏吞吐量；Criterion 用于微开销  
6. 流式传输：具有真实令牌节奏的合成 SSE 帧（8–25 ms）  
7. 提供商测试：仅作为说明性（受速率限制和区域延迟影响）  

### 🧪 重现（一旦添加基准套件）

```bash
# 微开销（请求构建 + 序列化）
cargo bench --bench micro_overhead

# 模拟高并发吞吐量
cargo run --example bench_mock_throughput -- --concurrency 512 --duration 15s

# 流式解析成本
cargo bench --bench stream_parse
```

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

- "SDK 开销" = ai-lib 内部处理（类型构造、序列化、分发准备）— 排除远程模型延迟。
- "吞吐量" 数字假设快速返回的模拟响应；真实世界云吞吐量通常受提供商速率限制约束。
- 内存数字是常驻集快照；具有日志/指标的生产系统可能增加开销。
- 结果将在不同硬件、OS 调度器、分配器策略和运行时调优中变化。

### ⚠️ 免责声明

> 这些指标是指示性的，不是合同保证。始终使用您的工作负载、提示大小、模型组合和部署环境进行基准测试。  
> 可重现的基准测试工具和 JSON 快照基线将在存储库中版本化以跟踪回归。

### 💡 优化技巧

- 在高吞吐量场景中使用 `.with_pool_config(size, idle_timeout)`
- 为低延迟 UX 优先使用流式传输
- 使用并发限制批处理相关短提示
- 避免冗余客户端实例化（重用客户端）
- 考虑提供商特定的速率限制和区域延迟

---

## 🗺️ 路线图（计划序列）

| 阶段 | 计划功能 |
|------|----------|
| 1 | 高级背压和自适应速率协调 |
| 2 | 内置缓存层（请求/结果分层） |
| 3 | 实时配置热重载 |
| 4 | 插件/拦截器系统 |
| 5 | GraphQL 表面 |
| 6 | WebSocket 原生流式传输 |
| 7 | 增强安全性（密钥轮换、KMS 集成） |
| 8 | 公共基准测试工具 + 夜间回归检查 |

### 🧪 性能监控路线图

计划中的公共基准测试工具 + 夜间（仅模拟）回归检查将：
- 早期检测性能回归
- 提供历史趋势数据
- 允许贡献者验证 PR 的影响

---

## ❓ 常见问题

| 问题 | 答案 |
|------|------|
| 如何 A/B 测试提供商？ | 使用带有负载策略的 `ModelArray` |
| 重试是内置的吗？ | 自动分类 + 退避；您可以分层自定义循环 |
| 可以禁用代理吗？ | `.without_proxy()` 或选项中的 `disable_proxy = true` |
| 可以模拟测试吗？ | 注入自定义传输 |
| 您记录 PII 吗？ | 默认不记录内容 |
| 函数调用差异？ | 通过 `Tool` + `FunctionCallPolicy` 标准化 |
| 支持本地推理吗？ | 是的，通过 Ollama（自托管） |
| 如何知道错误是否可重试？ | `error.is_retryable()` 助手 |

---

## 🤝 贡献

1. Fork 并克隆仓库  
2. 创建功能分支：`git checkout -b feature/your-feature`  
3. 运行测试：`cargo test`  
4. 如果引入新功能则添加示例  
5. 遵循适配器分层（优先配置驱动而非自定义）  
6. 打开 PR 并说明理由 + 基准测试（如果影响性能）  

我们重视：清晰度、测试覆盖率、最小表面区域增长、增量可组合性。

---

## 📄 许可证

双重许可，可选择：
- MIT
- Apache License (Version 2.0)

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

## 🏆 为什么选择 ai-lib？

| 维度 | 价值 |
|------|------|
| 工程速度 | 一个抽象 = 更少的定制适配器 |
| 风险缓解 | 多提供商回退和健康路由 |
| 运营稳健性 | 重试、池化、诊断、指标 |
| 成本控制 | 成本/性能策略旋钮 |
| 可扩展性 | 可插拔传输和指标 |
| 未来保障 | 清晰的路线图 + 混合适配器模式 |
| 易用性 | 渐进式 API—无过早复杂性 |
| 性能 | 最小延迟和内存开销 |

---

<div align="center">
  <strong>ai-lib：在 Rust 中构建弹性、快速、多提供商 AI 系统——无需胶水代码疲劳。</strong><br/><br/>
  ⭐ 如果这为您节省了时间，请给它一个星标并在 Issues / Discussions 中分享反馈！
</div>