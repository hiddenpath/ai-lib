# 应用引入方式

本指南解释了 ai-lib 应用程序和库的推荐导入模式，帮助您为您的用例选择最合适的方法。

## 快速参考

| 使用场景 | 推荐导入方式 | 示例 |
|----------|-------------|------|
| **应用开发** | `use ai_lib::prelude::*;` | 获取最小常用集 |
| **显式控制** | `use ai_lib::{AiClient, Provider, ...};` | 顶层重导出 |
| **库开发** | 领域特定导入 | `use ai_lib::types::response::Usage;` |
| **多模态内容** | `use ai_lib::{Content, Message, Role};` | 内容创建方法 |

## 导入策略

### 1. Prelude（推荐用于应用程序）

对于大多数应用程序代码，`prelude` 提供了导入常用类型和特征的最便捷方式：

```rust
use ai_lib::prelude::*;

#[tokio::main]
async fn main() -> Result<(), AiLibError> {
    let client = AiClient::new(Provider::OpenAI)?;
    
    let request = ChatCompletionRequest::new(
        "gpt-4".to_string(),
        vec![Message {
            role: Role::User,
            content: Content::new_text("Hello, world!"),
            function_call: None,
        }],
    );
    
    let response = client.chat_completion(request).await?;
    println!("Response: {}", response.choices[0].message.content.as_text());
    Ok(())
}
```

**prelude 包含的内容：**
- `AiClient`, `AiClientBuilder`, `Provider`
- `ChatCompletionRequest`, `ChatCompletionResponse`, `Choice`
- `Content`, `Message`, `Role`
- `Usage`, `UsageStatus`
- `AiLibError`

### 2. 顶层重导出（显式控制）

当您想要显式控制导入但仍避免深层模块路径时：

```rust
use ai_lib::{AiClient, AiClientBuilder, Provider};
use ai_lib::{ChatCompletionRequest, ChatCompletionResponse};
use ai_lib::{Content, Message, Role};
use ai_lib::{Usage, UsageStatus, AiLibError};

#[tokio::main]
async fn main() -> Result<(), AiLibError> {
    let client = AiClient::new(Provider::Groq)?;
    // ... 其余代码
    Ok(())
}
```

### 3. 领域特定导入（库开发）

对于库作者或当您需要细粒度控制时：

```rust
use ai_lib::types::request::ChatCompletionRequest;
use ai_lib::types::response::{ChatCompletionResponse, Usage, UsageStatus};
use ai_lib::types::common::{Content, Message, Role};
use ai_lib::types::error::AiLibError;
use ai_lib::client::{AiClient, Provider};
```

## 多模态内容创建

### 图像内容

```rust
use ai_lib::prelude::*;

// 从文件路径（自动处理）
let image_content = Content::from_image_file("path/to/image.png");

// 从 URL
let image_content = Content::new_image(
    Some("https://example.com/image.png".to_string()),
    Some("image/png".to_string()),
    Some("image.png".to_string()),
);

// 从数据 URL
let image_content = Content::from_data_url(
    "data:image/png;base64,iVBORw0KGgo...".to_string(),
    Some("image/png".to_string()),
    Some("image.png".to_string()),
);
```

### 音频内容

```rust
use ai_lib::prelude::*;

// 从文件路径（自动处理）
let audio_content = Content::from_audio_file("path/to/audio.mp3");

// 从 URL
let audio_content = Content::new_audio(
    Some("https://example.com/audio.mp3".to_string()),
    Some("audio/mpeg".to_string()),
);
```

### 混合内容消息

```rust
use ai_lib::prelude::*;

let messages = vec![
    Message {
        role: Role::User,
        content: Content::new_text("请分析这张图片"),
        function_call: None,
    },
    Message {
        role: Role::User,
        content: Content::from_image_file("path/to/image.png"),
        function_call: None,
    },
];
```

## 提供商选择

### 基本提供商选择

```rust
use ai_lib::prelude::*;

// 单个提供商
let client = AiClient::new(Provider::OpenAI)?;

// 预先组合路由/故障转移
let client = ai_lib::client::AiClientBuilder::new(Provider::OpenAI)
    .with_failover_chain(vec![Provider::Anthropic, Provider::Groq])?
    .build()?;
```

### 可用提供商

```rust
// OpenAI 兼容提供商
Provider::OpenAI
Provider::AzureOpenAI
Provider::OpenRouter
Provider::Replicate
Provider::ZhipuAI
Provider::MiniMax

// 独立提供商
Provider::Anthropic
Provider::Groq
Provider::Gemini
Provider::Mistral
Provider::Cohere
Provider::Perplexity
Provider::AI21
Provider::DeepSeek
Provider::Qwen
Provider::Ollama
```

## 错误处理

```rust
use ai_lib::prelude::*;

#[tokio::main]
async fn main() -> Result<(), AiLibError> {
    let client = AiClient::new(Provider::OpenAI)?;
    
    match client.chat_completion(request).await {
        Ok(response) => {
            println!("成功: {}", response.choices[0].message.content.as_text());
        }
        Err(AiLibError::NetworkError(msg)) => {
            eprintln!("网络错误: {}", msg);
        }
        Err(AiLibError::ProviderError(msg)) => {
            eprintln!("提供商错误: {}", msg);
        }
        Err(e) => {
            eprintln!("其他错误: {}", e);
        }
    }
    
    Ok(())
}
```

## 流式处理

```rust
use ai_lib::prelude::*;
use futures::stream::StreamExt;

#[tokio::main]
async fn main() -> Result<(), AiLibError> {
    let client = AiClient::new(Provider::OpenAI)?;
    let request = ChatCompletionRequest::new(
        "gpt-4".to_string(),
        vec![Message {
            role: Role::User,
            content: Content::new_text("给我讲个故事"),
            function_call: None,
        }],
    );
    
    let mut stream = client.chat_completion_stream(request).await?;
    
    while let Some(chunk) = stream.next().await {
        match chunk {
            Ok(chunk) => {
                if let Some(delta) = chunk.choices.get(0)
                    .and_then(|c| c.delta.as_ref())
                    .and_then(|d| d.content.as_ref()) {
                    print!("{}", delta);
                }
            }
            Err(e) => eprintln!("流式错误: {}", e),
        }
    }
    
    Ok(())
}
```

## 最佳实践

### 推荐做法
- ✅ 应用开发使用 `ai_lib::prelude::*`
- ✅ 需要控制时使用显式顶层导入
- ✅ 多模态内容使用 `Content::from_image_file()` 和 `Content::from_audio_file()`
- ✅ 使用 `Provider` 枚举进行提供商选择
- ✅ 适当处理 `AiLibError` 错误

### 避免做法
- ❌ 不要跨领域组合通配符导入
- ❌ 不要直接从 `ai_lib::provider::utils` 导入（使用 `Content` 方法代替）
- ❌ 不要直接导入具体适配器类型（使用 `Provider` 枚举）
- ❌ 不要忽略错误处理

## 从之前版本迁移

### Usage 和 UsageStatus
```rust
// 旧方式（已弃用）
use ai_lib::types::common::{Usage, UsageStatus};

// 新方式（推荐）
use ai_lib::{Usage, UsageStatus};
// 或者
use ai_lib::types::response::{Usage, UsageStatus};
```

### Provider Utils
```rust
// 旧方式（不再可用）
use ai_lib::provider::utils::upload_file_with_transport;

// 新方式（使用 Content 方法）
let content = Content::from_image_file("path/to/image.png");
```

## IDE 支持

使用新的导入结构，您的 IDE 自动完成将引导您找到正确的类型：
- 输入 `ai_lib::` 将显示顶层导出
- 输入 `ai_lib::prelude::` 将显示常用项目
- 首先在 crate 根目录搜索类型

## 进一步阅读

- [模块树和导入模式](MODULE_TREE_AND_IMPORTS.md) - 详细模块结构指南
- [API 参考](https://docs.rs/ai-lib/0.4.0) - 完整 API 文档
- [示例](../examples/) - 实用用法示例


