# ai-lib → ai-lib-rust 迁移指南

**日期**: 2026-01-06  
**状态**: ai-lib 已停止维护，请迁移到 ai-lib-rust

---

## 📢 重要公告

**ai-lib 项目已停止维护**，所有新功能和修复都在 [ai-lib-rust](https://github.com/hiddenpath/ai-lib-rust) 中进行。

## 🎯 为什么迁移？

### ai-lib-rust 的优势

1. **协议驱动架构 (Manifest-First)**
   - 所有逻辑由 YAML 协议文件驱动
   - 无需硬编码 provider 逻辑
   - 添加新 provider 只需添加协议文件，无需修改代码

2. **统一标准 (AI-Protocol)**
   - 基于 [AI-Protocol](https://github.com/hiddenpath/ai-protocol) 规范
   - 确保跨运行时一致性
   - 标准化的错误分类、重试策略、流式处理

3. **更简洁的 API**
   - 开发者友好的接口
   - 避免复杂混乱的用户界面
   - 清晰的模块划分

4. **更好的可维护性**
   - 模块化设计
   - 清晰的架构分层
   - 完整的测试覆盖

5. **生产就绪**
   - CI/CD 集成
   - 协议验证
   - 完整的文档

## 📦 快速开始

### 安装 ai-lib-rust

```toml
[dependencies]
ai-lib-rust = "0.2"
tokio = { version = "1.0", features = ["full"] }
futures = "0.3"
```

### 基本使用

```rust
use ai_lib_rust::prelude::*;
use futures::StreamExt;

#[tokio::main]
async fn main() -> ai_lib_rust::Result<()> {
    // 创建客户端（使用 Provider facade）
    let client = Provider::Anthropic
        .model("claude-3-5-sonnet")
        .build_client()
        .await?;

    // 创建请求
    let messages = vec![Message::user("Hello!")];
    let req = ChatCompletionRequest::new(messages)
        .temperature(0.7)
        .stream();

    // 流式响应
    let mut stream = client.chat_completion_stream(req).await?;
    while let Some(event) = stream.next().await {
        match event? {
            StreamingEvent::PartialContentDelta { content, .. } => {
                print!("{content}");
            }
            StreamingEvent::StreamEnd { .. } => break,
            _ => {}
        }
    }

    Ok(())
}
```

## 🔄 API 对比

### 客户端创建

**ai-lib (旧)**:
```rust
use ai_lib::prelude::*;

let client = AiClient::new(Provider::Groq)?;
```

**ai-lib-rust (新)**:
```rust
use ai_lib_rust::prelude::*;

// 方式 1: 使用 Provider facade
let client = Provider::Groq
    .model("llama3-70b-8192")
    .build_client()
    .await?;

// 方式 2: 直接使用模型 ID
let client = AiClient::new("groq/llama3-70b-8192").await?;
```

### 请求创建

**ai-lib (旧)**:
```rust
let req = ChatCompletionRequest::new(
    "gpt-3.5-turbo".to_string(),
    vec![Message::user("Hello!")]
);
```

**ai-lib-rust (新)**:
```rust
let messages = vec![Message::user("Hello!")];
let req = ChatCompletionRequest::new(messages)
    .temperature(0.7)
    .max_tokens(100);
```

### 流式处理

**ai-lib (旧)**:
```rust
let mut stream = client.chat_completion_stream(req).await?;
while let Some(chunk) = stream.next().await {
    let c = chunk?;
    if let Some(delta) = c.choices.get(0).and_then(|ch| ch.delta.content.clone()) {
        print!("{delta}");
    }
}
```

**ai-lib-rust (新)**:
```rust
let mut stream = client.chat_completion_stream(req).await?;
while let Some(event) = stream.next().await {
    match event? {
        StreamingEvent::PartialContentDelta { content, .. } => {
            print!("{content}");
        }
        StreamingEvent::StreamEnd { .. } => break,
        _ => {}
    }
}
```

## 🆕 新特性

### 1. 协议驱动架构

所有 provider 配置都在协议文件中，无需修改代码：

```yaml
# v1/providers/openai.yaml
id: openai
protocol_version: "1.5"
base_url: "https://api.openai.com/v1"
# ... 完整的协议配置
```

### 2. 统一的事件系统

所有 provider 使用统一的事件类型：

```rust
enum StreamingEvent {
    PartialContentDelta { content: String, .. },
    ToolCallDelta { .. },
    StreamEnd { finish_reason: String, .. },
    // ...
}
```

### 3. 协议验证

自动验证协议文件是否符合规范：

```bash
cargo run --bin validate_protocols
```

### 4. 多模态支持

```rust
let blocks = vec![
    ContentBlock::text("Describe this image."),
    ContentBlock::image_from_file("image.jpg")?,
];
let message = Message::with_content(MessageRole::User, MessageContent::blocks(blocks));
```

## 📚 资源

- **新项目**: [ai-lib-rust](https://github.com/hiddenpath/ai-lib-rust)
- **协议规范**: [AI-Protocol](https://github.com/hiddenpath/ai-protocol)
- **示例代码**: [ai-lib-rust/examples](https://github.com/hiddenpath/ai-lib-rust/tree/main/examples)
- **文档**: [ai-lib-rust/README.md](https://github.com/hiddenpath/ai-lib-rust/blob/main/README.md)

## ❓ 常见问题

### Q: ai-lib 还会更新吗？

A: 不会。所有新功能和修复都在 ai-lib-rust 中进行。

### Q: 现有代码还能用吗？

A: ai-lib 的最后一个版本（0.4.0）仍然可用，但建议尽快迁移到 ai-lib-rust。

### Q: 迁移需要多长时间？

A: 取决于项目复杂度。简单的项目可能只需要几小时，复杂的项目可能需要几天。

### Q: 有自动迁移工具吗？

A: 目前没有，但 API 设计相似，迁移相对简单。可以参考本指南和示例代码。

## 🤝 获取帮助

- **Issues**: [ai-lib-rust Issues](https://github.com/hiddenpath/ai-lib-rust/issues)
- **讨论**: 在 ai-lib-rust 仓库中提出问题和建议

---

**最后更新**: 2026-01-06  
**维护者**: AI-Protocol Team
