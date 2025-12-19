---
title: 升级到 0.4.0
group: 指南
order: 5
description: 从 ai-lib 0.3.x 到 0.4.0 的完整迁移指南
---

# 升级到 ai-lib 0.4.0

本指南帮助您从 ai-lib 0.3.x 迁移到 0.4.0。主要变更在于 **Trait Shift 1.0 演进**：从基于枚举的分发转向基于 trait 的架构。

## 破坏性变更概述

| 领域 | 0.3.x | 0.4.0 |
|------|-------|-------|
| 客户端创建 | `AiClient::new(Provider::X)` 可能 panic | 返回 `Result<AiClient, AiLibError>` |
| 内部分发 | 枚举匹配 | `Box<dyn ChatProvider>` 动态分发 |
| 路由 | `with_failover(vec![...])` + 哨兵模型 | `with_failover_chain(vec![...])` 策略构建器 |

## 快速迁移步骤

### 1. 更新 Cargo.toml

```toml
[dependencies]
ai-lib = "0.4.0"
```

### 2. 处理客户端创建错误

```rust
// ⛔️ 旧版（0.3.x）- 可能 panic
let client = AiClient::new(Provider::Groq);

// ✅ 新版（0.4.0）- 显式错误处理
let client = AiClient::new(Provider::Groq)?;
```

### 3. 更新路由代码

```rust
// ⛔️ 旧版（0.3.x）- 基于哨兵的故障转移
let client = AiClient::new(Provider::OpenAI)
    .with_failover(vec![Provider::Anthropic, Provider::Groq]);

// ✅ 新版（0.4.0）- 策略构建器
let client = AiClientBuilder::new(Provider::OpenAI)
    .with_failover_chain(vec![Provider::Anthropic, Provider::Groq])?
    .build()?;
```

## 详细迁移指南

### 客户端创建

**迁移前 (0.3.x)：**
```rust
use ai_lib::{AiClient, Provider};

let client = AiClient::new(Provider::Groq); // 可能 panic！
```

**迁移后 (0.4.0)：**
```rust
use ai_lib::{AiClient, Provider};

let client = AiClient::new(Provider::Groq)?;
// 或
let client = AiClient::new(Provider::Groq).expect("创建客户端失败");
```

### 路由和故障转移

**迁移前 (0.3.x)：**
```rust
// 基于哨兵的路由
let client = AiClient::new(Provider::OpenAI)
    .with_failover(vec![Provider::Anthropic, Provider::Groq]);

let request = ChatCompletionRequest::new("__route__".to_string(), messages);
```

**迁移后 (0.4.0)：**
```rust
// 策略构建器 - 在运行时预组合
let client = AiClientBuilder::new(Provider::OpenAI)
    .with_failover_chain(vec![Provider::Anthropic, Provider::Groq])?
    .build()?;

let request = ChatCompletionRequest::new("gpt-4".to_string(), messages);
```

### 轮询负载均衡

**0.4.0 中的新功能：**
```rust
let client = AiClientBuilder::new(Provider::OpenAI)
    .with_round_robin_chain(vec![Provider::Groq, Provider::Mistral])?
    .build()?;
```

### 自定义提供商注入

**0.4.0 中的新功能：**
```rust
use ai_lib::provider::builders::CustomProviderBuilder;

let custom = CustomProviderBuilder::new("my-gateway")
    .with_base_url("https://gateway.example.com/v1")
    .with_api_key_env("MY_GATEWAY_KEY")
    .with_default_chat_model("gpt-4")
    .build_provider()?;

let client = AiClientBuilder::new(Provider::OpenAI)
    .with_strategy(custom)
    .build()?;
```

### 提供商特定参数

**0.4.0 中的新功能：**
```rust
let request = ChatCompletionRequest::new(model, messages)
    .with_extension("reasoning_effort", serde_json::json!("high"))
    .with_extension("temperature", serde_json::json!(0.7));
```

## 导入变更

导入路径没有重大变更。prelude 仍然是推荐的方法：

```rust
// ✅ 仍然有效（推荐）
use ai_lib::prelude::*;

// ✅ 仍然有效（直接导入）
use ai_lib::{AiClient, Provider, ChatCompletionRequest};
```

## 0.4.0 中的新功能

### ChatProvider Trait

所有提供商现在都实现了统一的 `ChatProvider` trait：
- `chat()` - 单次完成
- `stream()` - 流式完成
- `batch()` - 批处理
- `list_models()` - 可用模型

### 增强的错误处理

所有客户端操作现在都返回 `Result<T, AiLibError>` 以提供更好的错误处理。

### 改进的流式处理

跨所有提供商统一的 SSE/JSONL 解析逻辑。

## 测试迁移

更新代码后：

```bash
# 构建
cargo build

# 运行测试
cargo test

# 检查剩余问题
cargo check
```

## 常见问题及解决方案

### "方法未找到"错误
- **问题**：旧的路由方法不再存在
- **解决方案**：使用 `AiClientBuilder::with_failover_chain()` 或 `with_round_robin_chain()`

### 客户端创建时 panic
- **问题**：`AiClient::new()` 现在返回 Result
- **解决方案**：添加 `?` 或 `.unwrap()` / `.expect()`

### 哨兵模型错误
- **问题**：`"__route__"` 模型不再有效
- **解决方案**：使用策略构建器而非哨兵模型

## 需要帮助？

- [API 参考](https://docs.rs/ai-lib/0.4.0) - 完整的 API 文档
- [示例](https://github.com/hiddenpath/ai-lib/tree/main/examples) - 0.4.0 的更新示例
- [GitHub Issues](https://github.com/hiddenpath/ai-lib/issues) - 报告迁移问题
- [Discord 社区](https://discord.gg/ai-lib) - 从社区获取帮助

## 验证清单

- [ ] 将 `Cargo.toml` 中的 ai-lib 更新为 "0.4.0"
- [ ] 为 `AiClient::new()` 调用添加 `?`
- [ ] 将 `with_failover()` 替换为 `with_failover_chain()`
- [ ] 移除 `"__route__"` 哨兵模型
- [ ] 根据需要更新导入
- [ ] 使用 `cargo build` 测试编译
- [ ] 使用 `cargo test` 运行测试
