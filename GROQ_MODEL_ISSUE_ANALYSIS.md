# Groq 模型问题分析报告

## 🔍 问题描述

在 `ai-lib` 中使用 Groq 时，两种不同的调用方法产生了不同的结果：

- ✅ `AiClient::new()` + 手动指定模型：**成功**
- ❌ `AiClient::quick_chat_text()`：**失败**

## 📋 代码对比

### 方法1：手动指定模型（成功）
```rust
// examples/hello_groq.rs
let client = AiClient::new(Provider::Groq)?;
let request = ChatCompletionRequest::new(
    "llama-3.1-8b-instant".to_string(), // 手动指定可用模型
    vec![Message { ... }],
);
let response = client.chat_completion(request).await?;
```

### 方法2：使用默认模型（失败）
```rust
// examples/debug_quick_groq.rs
let reply = AiClient::quick_chat_text(Provider::Groq, "Hello!").await?;
```

## 🔍 问题根源分析

### 1. 默认模型配置
在 `src/client.rs` 中：
```rust
impl Provider {
    pub fn default_chat_model(&self) -> &'static str {
        match self {
            Provider::Groq => "llama3-8b-8192", // ❌ 已停用的模型
            // ...
        }
    }
}
```

### 2. ProviderConfigs 配置
在 `src/provider/configs.rs` 中：
```rust
impl ProviderConfigs {
    pub fn groq() -> ProviderConfig {
        ProviderConfig::openai_compatible(
            "https://api.groq.com/openai/v1",
            "GROQ_API_KEY",
            "llama3-8b-8192", // ❌ 已停用的模型
            Some("llama-3.2-11b-vision"),
        )
    }
}
```

### 3. quick_chat_text 内部流程
```rust
pub async fn quick_chat_text<P: Into<String>>(
    provider: Provider,
    prompt: P,
) -> Result<String, AiLibError> {
    let client = AiClient::new(provider)?;
    let req = client.build_simple_request(prompt.into()); // 使用默认模型
    let resp = client.chat_completion(req).await?;
    resp.first_text().map(|s| s.to_string())
}

pub fn build_simple_request<S: Into<String>>(&self, prompt: S) -> ChatCompletionRequest {
    ChatCompletionRequest::new(
        self.provider.default_chat_model().to_string(), // ❌ 使用已停用的模型
        vec![...],
    )
}
```

## ❌ 错误信息
```
The model `llama3-8b-8192` has been decommissioned and is no longer supported. 
Please refer to https://console.groq.com/docs/deprecations for a recommendation 
on which model to use instead.
```

## ✅ 解决方案

### 方案1：更新默认模型配置（推荐）
更新 `src/client.rs` 中的默认模型：
```rust
Provider::Groq => "llama-3.1-8b-instant", // ✅ 使用可用模型
```

更新 `src/provider/configs.rs` 中的配置：
```rust
"llama-3.1-8b-instant", // ✅ 使用可用模型
```

### 方案2：使用手动指定模型
```rust
let client = AiClient::new(Provider::Groq)?;
let request = ChatCompletionRequest::new(
    "llama-3.1-8b-instant".to_string(),
    vec![Message { ... }],
);
```

### 方案3：使用 AiClientBuilder
```rust
let client = AiClientBuilder::new(Provider::Groq)
    .with_model("llama-3.1-8b-instant")
    .build()?;
```

## 🧪 测试样例

创建了以下调试样例来重现和分析问题：

1. `examples/debug_quick_groq.rs` - 重现错误
2. `examples/debug_groq_detailed.rs` - 详细调试分析
3. `examples/compare_groq_methods.rs` - 对比两种方法

## 📊 测试结果

- ✅ `hello_groq.rs`：使用手动指定模型，成功
- ❌ `debug_quick_groq.rs`：使用默认模型，失败
- ✅ `debug_groq_detailed.rs`：找到可用模型 `llama-3.1-8b-instant`

## 🎯 结论

问题在于 `ai-lib` 的默认模型配置没有跟上 Groq API 的变化。`llama3-8b-8192` 模型已被停用，但库中的默认配置仍然使用这个模型。需要更新默认模型配置以使用当前可用的模型。
