# AI-lib: Rust统一AI SDK

> **为多个AI提供商提供统一接口的Rust SDK，采用混合架构设计**

## 概览

**ai-lib** 是一个为Rust设计的统一AI SDK，提供与多个大型语言模型提供商交互的单一、一致性接口。采用先进的混合架构，兼顾开发效率与功能性。

**注意**：辅助升级和PR说明已移至 `docs/` 目录，以保持仓库根目录的简洁。有关迁移和PR详情，请参阅 `docs/UPGRADE_0.2.0.md` 和 `docs/PR_0.2.0.md`。

## 支持的AI提供商

- ✅ **Groq**（配置驱动） - 支持 llama3、mixtral 模型
- ✅ **xAI Grok**（配置驱动） - 支持 grok 模型
- ✅ **DeepSeek**（配置驱动） - 支持 deepseek-chat、deepseek-reasoner
- ✅ **Anthropic Claude**（配置驱动） - 支持 claude-3.5-sonnet
- ✅ **Google Gemini**（独立适配器） - 支持 gemini-1.5-pro、gemini-1.5-flash
- ✅ **OpenAI**（独立适配器） - 支持 gpt-3.5-turbo、gpt-4（部分地区需代理）
- ✅ **Qwen / 通义千问（阿里云）**（配置驱动） - 支持 Qwen 系列（兼容 OpenAI）
- ✅ **Cohere**（独立适配器） - 支持 command/generate 模型（SSE流式传输 + 回退）
- ✅ **百度文心一言（Baidu ERNIE）**（配置驱动） - 支持 ernie-3.5、ernie-4.0（通过千帆平台的 OpenAI 兼容接口，需 AK/SK 与 OAuth）
- ✅ **腾讯混元（Hunyuan）**（配置驱动） - 支持 hunyuan 系列（OpenAI 兼容接口，需腾讯云账号与密钥）
- ✅ **讯飞星火（iFlytek Spark）**（配置驱动） - 支持 spark 系列（OpenAI 兼容接口，语音+文本场景友好）
- ✅ **月之暗面 Kimi（Moonshot AI）**（配置驱动） - 支持 kimi 系列（OpenAI 兼容接口，适合长文本场景）
- ✅ **Mistral**（独立适配器） - 支持 mistral 系列
- ✅ **Hugging Face Inference**（配置驱动） - 支持 hub 托管模型
- ✅ **TogetherAI**（配置驱动） - 支持 together.ai 托管模型
- ✅ **Azure OpenAI**（配置驱动） - 支持 Azure 托管的 OpenAI 端点
- ✅ **Ollama**（配置驱动/本地） - 支持本地 Ollama 实例

## 核心特性

### 🚀 零成本提供商切换
只需一行代码即可在不同AI提供商之间切换，统一接口确保无缝体验：

```rust
// 即时切换提供商 - 统一接口，不同后端
let groq_client = AiClient::new(Provider::Groq)?;
let gemini_client = AiClient::new(Provider::Gemini)?;
let claude_client = AiClient::new(Provider::Anthropic)?;
```

运行时动态切换支持基于环境变量或其他逻辑选择提供商。

### 🌊 通用流式响应支持
为所有提供商提供实时流式响应，SSE解析和模拟回退确保一致性：

```rust
use futures::StreamExt;

let mut stream = client.chat_completion_stream(request).await?;
print!("流式输出: ");
while let Some(item) = stream.next().await {
    let chunk = item?;
    if let Some(content) = chunk.choices.get(0).and_then(|c| c.delta.content.clone()) {
        print!("{}", content); // 实时输出
    }
}
```

包括取消句柄（CancelHandle）和背压机制（计划中），适用于低延迟UI应用。

### 🔄 企业级可靠性和错误处理
- **自动重试与指数退避**：针对瞬时失败（如网络超时、速率限制）智能重试。
- **智能错误分类**：区分可重试错误（如网络问题）和永久错误（如认证失败），提供恢复建议。
- **代理支持**：HTTP/HTTPS代理及认证，适用于企业环境。
- **超时管理**：可配置超时和优雅降级，确保生产稳定性。

示例错误处理：

```rust
match client.chat_completion(request).await {
    Ok(response) => println!("成功: {}", response.choices[0].message.content.as_text()),
    Err(e) => {
        if e.is_retryable() {
            println!("可重试错误，等待 {}ms", e.retry_delay_ms());
            tokio::time::sleep(Duration::from_millis(e.retry_delay_ms())).await;
            // 实现重试逻辑
        } else {
            println!("永久性错误: {}", e);
        }
    }
}
```

### ⚡ 混合架构设计
- **配置驱动适配器**：适用于兼容OpenAI的API，仅需少量配置代码（约15行），自动继承SSE流式、代理等功能。
- **独立适配器**：为独特API提供完全控制，包括自定义认证和响应解析。
- **四层结构**：统一客户端层、适配器层、传输层（HttpTransport，支持代理和重试）、公共类型层，确保类型安全和零额外依赖。
- **优势**：95%代码减少、灵活扩展、自动功能继承。

### 📊 指标与可观测性
最小化指标框架，包括`Metrics`和`Timer` trait，默认`NoopMetrics`实现。适配器内置请求计数器和时长计时器，支持注入自定义指标用于测试或生产监控。

### 📁 多模态与文件支持
- 支持文本、JSON、图像、音频内容类型。
- 文件上传/内联辅助函数，边界检查和失败回退。
- 函数调用/工具支持：统一`Tool`和`FunctionCall`类型，跨提供商解析和执行。

最小工具调用示例：

```rust
let mut req = ChatCompletionRequest::new("gpt-4".to_string(), vec![]);
req.functions = Some(vec![Tool { /* ... */ }]);
req.function_call = Some(FunctionCallPolicy::Auto("auto".to_string()));
```

### 🔧 依赖注入与测试友好
- 对象安全传输层（`DynHttpTransportRef`），便于注入模拟传输进行单元测试。
- 适配器构造函数支持自定义传输注入。

示例：

```rust
let transport: DynHttpTransportRef = my_test_transport.into();
let adapter = GenericAdapter::with_transport_ref(config, transport)?;
```

### 🚀 性能与可扩展性
- **基准**：内存 <2MB，客户端开销 <1ms，流式块延迟 <10ms。
- **连接池**：自动重用，支持自定义`reqwest::Client`调优（最大空闲连接、空闲超时）。
- **自定义配置**：通过`HttpTransportConfig`设置超时、代理和池参数。

自定义池示例：

```rust
let reqwest_client = Client::builder()
    .pool_max_idle_per_host(32)
    .build()?;
let transport = HttpTransport::with_client(reqwest_client, Duration::from_secs(30));
```

## 快速入门

### 安装
在 `Cargo.toml` 中添加：

```toml
[dependencies]
ai-lib = "0.2.0"
tokio = { version = "1.0", features = ["full"] }
futures = "0.3"
```

### 一分钟体验（无需API密钥）
构造客户端和请求，无网络调用：

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

### 真实请求设置
设置API密钥和代理：

```bash
export GROQ_API_KEY=your_groq_api_key
export AI_PROXY_URL=https://proxy.example.com:8080
cargo run --example basic_usage
```

## 环境变量

- **API密钥**：如`GROQ_API_KEY`、`OPENAI_API_KEY`等。
- **代理**：`AI_PROXY_URL`支持HTTP/HTTPS和认证。

## 示例与测试

- 混合架构：`cargo run --example test_hybrid_architecture`
- 流式响应：`cargo run --example test_streaming_improved`
- 重试机制：`cargo run --example test_retry_mechanism`
- 提供商测试：`cargo run --example test_groq_generic` 等。

## 提供商详情

| 提供商 | 状态 | 架构 | 流式支持 | 模型 | 备注 |
|--------|------|------|----------|------|------|
| **Groq** | ✅ 生产就绪 | 配置驱动 | ✅ | llama3-8b/70b, mixtral-8x7b | 快速推理，支持代理 |
| **DeepSeek** | ✅ 生产就绪 | 配置驱动 | ✅ | deepseek-chat, deepseek-reasoner | 中国AI，直接连接 |
| **Anthropic** | ✅ 生产就绪 | 配置驱动 | ✅ | claude-3.5-sonnet | 自定义认证 |
| **Google Gemini** | ✅ 生产就绪 | 独立适配器 | 🔄 | gemini-1.5-pro/flash | URL参数认证 |
| **OpenAI** | ✅ 生产就绪 | 独立适配器 | ✅ | gpt-3.5-turbo, gpt-4 | 需代理（部分地区） |
| **Qwen** | ✅ 生产就绪 | 配置驱动 | ✅ | Qwen系列 | 使用DASHSCOPE_API_KEY |
| **Baidu 文心一言 (ERNIE)** | ✅ 生产就绪 | 配置驱动 | ✅ | ernie-3.5, ernie-4.0 | 通过百度千帆平台的 OpenAI 兼容接口（需 AK/SK 与 OAuth），请参考百度智能云控制台 |
| **Tencent 混元 (Hunyuan)** | ✅ 生产就绪 | 配置驱动 | ✅ | hunyuan 系列 | 腾讯云提供 OpenAI 兼容端点（需云账号与密钥），详见腾讯云文档 |
| **讯飞 星火 (iFlytek Spark)** | ✅ 生产就绪 | 配置驱动 | ✅ | spark 系列 | 支持语音+文本混合场景，提供 OpenAI 兼容接口，详见讯飞开放平台 |
| **月之暗面 Kimi (Moonshot AI)** | ✅ 生产就绪 | 配置驱动 | ✅ | kimi 系列 | OpenAI 兼容接口，适合长文本处理，详见 Moonshot 平台 |

## 路线图

### 已实现特性
- 混合架构与通用流式支持。
- 企业级错误处理、重试和代理。
- 多模态基础、函数调用和指标框架。
- 传输注入与上传测试。

### 计划特性
- 高级背压API和性能基准CI。
- 连接池调优和插件系统。
- 内置缓存与负载均衡。

## 贡献指南

欢迎贡献新提供商、性能优化和文档改进。

1. 克隆仓库：`git clone https://github.com/hiddenpath/ai-lib.git`
2. 创建分支：`git checkout -b feature/new-feature`
3. 测试：`cargo test`
4. 提交PR。

## 社区与支持

- 📖 文档：[docs.rs/ai-lib](https://docs.rs/ai-lib)
- 🐛 问题：[GitHub Issues](https://github.com/hiddenpath/ai-lib/issues)
- 💬 讨论：[GitHub Discussions](https://github.com/hiddenpath/ai-lib/discussions)

## 致谢与许可证

感谢AI提供商和Rust社区。双许可证：MIT 或 Apache 2.0。

引用：

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
**ai-lib**：Rust生态中最全面的统一AI SDK。🦀✨
</div>