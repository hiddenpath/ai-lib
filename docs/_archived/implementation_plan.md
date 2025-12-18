# Phase 3 Implementation Plan: Architecture Cleanup (Comprehensive - Option B)

## Design Philosophy

基于用户需求优化的方案B，专注于：
1. **Provider扩展性**：添加新provider只需修改2-3个文件
2. **开发者友好**：清晰的模块结构，完善的文档和示例
3. **可维护性**：单一职责原则，低耦合高内聚

## Current State Analysis

### `client_impl.rs` Issues (1574 lines, 66KB)
- ❌ **God Object**: 一个文件承担太多职责
- ❌ **Provider Coupling**: Provider创建逻辑与请求执行逻辑混在一起
- ❌ **Low Cohesion**: Builder、执行、辅助函数全部混在一起
- ❌ **Hard to Test**: 单元测试需要加载整个巨大的文件

### Provider Addition Current Workflow
添加新provider当前需要修改的地方：
1. `src/client/provider.rs`: 添加Provider枚举变体 (2行)
2. `src/client/provider.rs`: 添加default_chat_model匹配 (1行)  
3. `src/client/client_impl.rs`: 添加adapter创建逻辑 (~20行，多处match)
4. `src/provider/configs.rs`: 添加provider配置 (~30行)
5. `src/provider/mod.rs`: 可能需要导出新adapter

**目标**：减少到只需修改1-2个文件

## Proposed Architecture (Option B Enhanced)

```
src/client/
├── mod.rs                  # 模块组织和公共导出
├── client_impl.rs          # AiClient核心定义 (~150行)
├── builder.rs              # AiClientBuilder实现 (~600行)
├── request.rs              # 请求处理 (chat_completion) (~200行)
├── stream.rs               # 流式处理 (streaming requests) (~200行)
├── batch.rs                # 批处理 (batch processing) (~100行)
├── failover.rs             # 故障转移逻辑 (~150行)
├── helpers.rs              # 便捷方法 (~150行)
├── provider_factory.rs     # NEW: Provider适配器工厂 (~100行)
├── provider.rs             # Provider枚举 (保持不变)
└── model_options.rs        # ModelOptions (保持不变)
```

### Key Innovation: `provider_factory.rs`

**目标**：将所有provider创建逻辑集中到一个文件，添加新provider只需修改此文件。

```rust
// src/client/provider_factory.rs
use crate::api::ChatApi;
use crate::provider::*;
use crate::types::AiLibError;

pub struct ProviderFactory;

impl ProviderFactory {
    /// 创建provider适配器的统一入口
    /// 添加新provider只需在这里添加一个match分支
    pub fn create_adapter(
        provider: Provider,
        api_key: Option<String>,
        base_url: Option<String>,
        transport: Option<DynHttpTransportRef>,
    ) -> Result<Box<dyn ChatApi>, AiLibError> {
        match provider {
            // Config-driven providers (使用GenericAdapter)
            Provider::Groq => create_generic(
                ProviderConfigs::groq(), api_key, base_url, transport
            ),
            Provider::XaiGrok => create_generic(
                ProviderConfigs::xai_grok(), api_key, base_url, transport
            ),
            // ... 其他config-driven providers
            
            // Independent adapters (专用adapter)
            Provider::OpenAI => Ok(Box::new(
                OpenAiAdapter::new(api_key, base_url, transport)?
            )),
            Provider::Gemini => Ok(Box::new(
                GeminiAdapter::new(api_key, base_url, transport)?
            )),
            // ... 其他独立adapters
        }
    }
    
    /// 获取provider默认模型（从provider.rs委托）
    pub fn default_model(provider: Provider) -> &'static str {
        provider.default_chat_model()
    }
}

// 辅助函数
fn create_generic(
    config: ProviderConfig,
    api_key: Option<String>,
    base_url: Option<String>,
    transport: Option<DynHttpTransportRef>,
) -> Result<Box<dyn ChatApi>, AiLibError> {
    let mut adapter = if let Some(key) = api_key {
        GenericAdapter::new_with_api_key(config, Some(key))?
    } else {
        GenericAdapter::new(config)?
    };
    if let Some(url) = base_url {
        adapter = adapter.with_base_url(url);
    }
    if let Some(t) = transport {
        adapter = adapter.with_transport(t);
    }
    Ok(Box::new(adapter))
}
```

**优势**：
- ✅ 添加新provider只需在`provider_factory.rs`添加一个match分支
- ✅ 所有provider创建逻辑集中，易于维护
- ✅ 清晰的职责划分

## Module Decomposition Details

### 1. [KEEP] `src/client/client_impl.rs` (~150 lines)

**职责**：AiClient核心定义和基本配置

保留内容：
- `AiClient` struct定义
- 基本构造函数 (`new`, `new_with_metrics`)
- 配置方法 (`with_metrics`, `with_failover_chain`, `with_round_robin_chain`)
- `current_provider()`
- `default_chat_model()` (委托给ProviderFactory)

移除内容：
- Builder实现 → `builder.rs`
- 请求执行 → `request.rs`, `stream.rs`, `batch.rs`
- Failover逻辑 → `failover.rs`
- 辅助方法 → `helpers.rs`

---

### 2. [NEW] `src/client/provider_factory.rs` (~100 lines)

**职责**：统一的Provider适配器创建工厂

内容：
- `ProviderFactory::create_adapter()` - 创建适配器的唯一入口
- `create_generic()` - 辅助函数，创建通用适配器
- Provider特定逻辑封装

**添加新provider示例**：
```rust
// 只需在create_adapter中添加：
Provider::NewProvider => create_generic(
    ProviderConfigs::new_provider(),  // 在configs.rs中定义
    api_key, base_url, transport
),
```

---

### 3. [NEW] `src/client/request.rs` (~200 lines)

**职责**：处理单个同步请求

移动内容：
- `chat_completion()` 方法
- Request preprocessing逻辑
- Routing逻辑 (如果有)
- Interceptor调用

依赖：
- `ProviderFactory` (创建adapter)
- `FailoverHandler` (故障转移)

---

### 4. [NEW] `src/client/stream.rs` (~200 lines)

**职责**：处理流式请求

移动内容：
- `chat_completion_stream()`
- `chat_completion_stream_with_cancel()`
- Stream wrapper实现
- Streaming-specific逻辑

---

### 5. [NEW] `src/client/batch.rs` (~100 lines)

**职责**：批处理请求

移动内容：
- `chat_completion_batch()`
- `chat_completion_batch_smart()`
- Batch processing策略

---

### 6. [NEW] `src/client/failover.rs` (~150 lines)

> **更新**：故障转移现在由 `AiClientBuilder::with_failover_chain`/`with_round_robin_chain`
> 直接在构建阶段注入 `FailoverProvider`/`RoundRobinProvider`。原计划中的
> `FailoverHandler` 留作设计记录，实际实现已经交由策略提供者完成，因此无需新增 `failover.rs`。

---

### 7. [MOVE] `src/client/builder.rs` (~600 lines)

**职责**：AiClientBuilder实现

移动内容：
- `AiClientBuilder` struct定义
- 所有builder方法 (`with_*`, `enable_*`)
- `build()` 方法 (使用`ProviderFactory`)

更新：
```rust
impl AiClientBuilder {
    pub fn build(self) -> Result<AiClient, AiLibError> {
        // 使用ProviderFactory创建adapter
        let adapter = ProviderFactory::create_adapter(
            self.provider,
            self.api_key,
            self.base_url,
            self.transport,
        )?;
        
        // 构建AiClient
        Ok(AiClient {
            adapter,
            // ... 其他字段
        })
    }
}
```

---

### 8. [NEW] `src/client/helpers.rs` (~150 lines)

**职责**：便捷辅助方法

移动内容：
- `list_models()`
- `switch_provider()`
- `build_simple_request()`
- `build_simple_request_with_model()`

---

### 9. [UPDATE] `src/client/mod.rs`

```rust
// 模块声明
mod client_impl;
mod builder;
mod provider_factory;
mod request;
mod stream;
mod batch;
mod helpers;
mod metadata;
mod model_options;
mod provider;

// 公共导出
pub use client_impl::AiClient;
pub use builder::AiClientBuilder;
pub use provider::Provider;
pub use model_options::ModelOptions;

// 内部模块 (不公开)
pub(crate) use metadata::{metadata_from_provider, ClientMetadata};
pub(crate) use provider_factory::ProviderFactory;
```

---

## Enhanced Error Handling

### [MODIFY] `src/types/error.rs`

#### 1. Add Error Severity
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorSeverity {
    /// Transient errors - should retry
    Transient,
    /// Client errors - bad request, invalid config
    Client,
    /// Server errors - provider issues  
    Server,
    /// Fatal errors - auth failures, unsupported
    Fatal,
}

impl AiLibError {
    pub fn severity(&self) -> ErrorSeverity {
        match self {
            AiLibError::NetworkError(_) 
            | AiLibError::TimeoutError(_) 
            | AiLibError::RateLimitExceeded(_) => ErrorSeverity::Transient,
            
            AiLibError::InvalidRequest(_)
            | AiLibError::ConfigurationError(_) 
            | AiLibError::ContextLengthExceeded(_) => ErrorSeverity::Client,
            
            AiLibError::ProviderError(_)
            | AiLibError::InvalidModelResponse(_) => ErrorSeverity::Server,
            
            AiLibError::AuthenticationError(_)
            | AiLibError::UnsupportedFeature(_) => ErrorSeverity::Fatal,
            
            _ => ErrorSeverity::Server,
        }
    }
}
```

#### 2. Add Structured Error Codes
```rust
impl AiLibError {
    pub fn error_code(&self) -> &'static str {
        match self {
            AiLibError::ProviderError(_) => "PROVIDER_ERROR",
            AiLibError::TransportError(_) => "TRANSPORT_ERROR",
            AiLibError::InvalidRequest(_) => "INVALID_REQUEST",
            AiLibError::RateLimitExceeded(_) => "RATE_LIMIT",
            AiLibError::AuthenticationError(_) => "AUTH_FAILED",
            AiLibError::ConfigurationError(_) => "CONFIG_ERROR",
            AiLibError::NetworkError(_) => "NETWORK_ERROR",
            AiLibError::TimeoutError(_) => "TIMEOUT",
            AiLibError::RetryExhausted(_) => "RETRY_EXHAUSTED",
            AiLibError::SerializationError(_) => "SERIALIZATION_ERROR",
            AiLibError::DeserializationError(_) => "DESERIALIZATION_ERROR",
            AiLibError::FileError(_) => "FILE_ERROR",
            AiLibError::UnsupportedFeature(_) => "UNSUPPORTED_FEATURE",
            AiLibError::ModelNotFound(_) => "MODEL_NOT_FOUND",
            AiLibError::InvalidModelResponse(_) => "INVALID_RESPONSE",
            AiLibError::ContextLengthExceeded(_) => "CONTEXT_TOO_LONG",
        }
    }
    
    /// Get error code with severity prefix
    /// Example: "TRANSIENT_RATE_LIMIT", "FATAL_AUTH_FAILED"
    pub fn error_code_with_severity(&self) -> String {
        format!("{:?}_{}", self.severity(), self.error_code())
            .to_uppercase()
    }
}
```

#### 3. Add Error Context Chain
```rust
impl AiLibError {
    /// Wrap error with additional context
    pub fn with_context(self, context: impl Into<String>) -> Self {
        let ctx = context.into();
        match self {
            AiLibError::ProviderError(msg) => 
                AiLibError::ProviderError(format!("{}: {}", ctx, msg)),
            AiLibError::NetworkError(msg) => 
                AiLibError::NetworkError(format!("{}: {}", ctx, msg)),
            // ... 其他变体类似
            other => other,
        }
    }
}
```

---

## Developer Experience Enhancements

### 1. [NEW] `docs/ADDING_PROVIDERS.md`

创建详细的"添加新provider"指南：

```markdown
# Adding a New Provider to ai-lib

## Quick Start

Adding a new provider requires changes to only 2-3 files:

1. **Define Provider** (`src/client/provider.rs`)
2. **Configure Provider** (`src/provider/configs.rs`)
3. **Register in Factory** (`src/client/provider_factory.rs`)

## Step-by-Step Guide

### Step 1: Add Provider Enum Variant

**File**: `src/client/provider.rs`

Add your provider to the `Provider` enum:

\`\`\`rust
pub enum Provider {
    // ... existing providers
    YourProvider,  // Add here
}
\`\`\`

Add default model:

\`\`\`rust
impl Provider {
    pub fn default_chat_model(&self) -> &'static str {
        match self {
            // ... existing
            Provider::YourProvider => "your-default-model",
        }
    }
}
\`\`\`

### Step 2: Create Provider Configuration

**File**: `src/provider/configs.rs`

\`\`\`rust
impl ProviderConfigs {
    pub fn your_provider() -> ProviderConfig {
        ProviderConfig {
            api_base: "https://api.yourprovider.com/v1".to_string(),
            api_key_env: "YOUR_PROVIDER_API_KEY".to_string(),
            default_model: "your-default-model".to_string(),
            requires_auth: true,
            // ... 其他配置
        }
    }
}
\`\`\`

### Step 3: Register in Provider Factory

**File**: `src/client/provider_factory.rs`

\`\`\`rust
impl ProviderFactory {
    pub fn create_adapter(...) -> Result<Box<dyn ChatApi>, AiLibError> {
        match provider {
            // ... existing
            Provider::YourProvider => create_generic(
                ProviderConfigs::your_provider(),
                api_key, base_url, transport
            ),
        }
    }
}
\`\`\`

### Step 4: Test Your Provider

\`\`\`rust
#[tokio::test]
async fn test_your_provider() {
    let client = AiClient::new(Provider::YourProvider).unwrap();
    // ... test code
}
\`\`\`

That's it! 🎉
\`\`\`

---

### 2. [NEW] `examples/custom_provider.rs`

创建示例代码展示如何添加自定义provider：

```rust
//! Example: Adding a custom provider to ai-lib
//!
//! This example demonstrates the minimal steps to add a new AI provider.

use ai_lib::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Step 1: Use your new provider
    let client = AiClient::new(Provider::YourProvider)?;
    
    // Step 2: Make a request
    let request = ChatCompletionRequest::new(
        "your-model".to_string(),
        vec![Message {
            role: Role::User,
            content: Content::Text("Hello!".to_string()),
            function_call: None,
        }],
    );
    
    let response = client.chat_completion(request).await?;
    println!("Response: {:?}", response);
    
    Ok(())
}
```

---

### 3. Module Documentation

每个新模块都添加详细的module-level文档：

```rust
//! Request execution module.
//!
//! This module handles single synchronous chat completion requests.
//! It coordinates with:
//! - `ProviderFactory`: Creates provider adapters
//! - Strategy providers (`RoundRobinProvider`, `FailoverProvider`)
//! - `InterceptorPipeline`: Applies interceptors
//!
//! # Example
//!
//! ```rust
//! // Internal usage - typically called via AiClient
//! let response = request::execute_chat_completion(
//!     &adapter, &request, &interceptor_pipeline, &metrics
//! ).await?;
//! ```
```

---

## Implementation Steps

### Phase 1: Setup (Create Empty Modules)
1. ✅ Create `src/client/provider_factory.rs` (empty)
2. ✅ Create `src/client/request.rs` (empty)
3. ✅ Create `src/client/stream.rs` (empty)
4. ✅ Create `src/client/batch.rs` (empty)
5. ✅ Create `src/client/failover.rs` (empty)
6. ✅ Create `src/client/helpers.rs` (empty)
7. ✅ Update `src/client/mod.rs` with module declarations

### Phase 2: Provider Factory (Critical Path)
1. ✅ Implement `ProviderFactory::create_adapter()`
2. ✅ Move provider creation logic from `client_impl.rs`
3. ✅ Test compilation

### Phase 3: Move Builder
1. ✅ Create `src/client/builder.rs`
2. ✅ Move `AiClientBuilder` from `client_impl.rs`
3. ✅ Update builder to use `ProviderFactory`
4. ✅ Test compilation

### Phase 4: Move Execution Logic
1. ✅ Move `chat_completion` to `request.rs`
2. ✅ Move streaming methods to `stream.rs`
3. ✅ Move batch methods to `batch.rs`
4. ✅ Move failover logic to `failover.rs`
5. ✅ Test compilation after each move

### Phase 5: Move Helpers
1. ✅ Move helper methods to `helpers.rs`
2. ✅ Final cleanup of `client_impl.rs`
3. ✅ Test compilation

### Phase 6: Error Handling
1. ✅ Add `severity()` method
2. ✅ Add `error_code()` methods
3. ✅ Add `with_context()` method
4. ✅ Test error handling

> **Status update (2025-11-27)**  
> Implemented in `src/types/error.rs` with the new `ErrorSeverity` enum, structured `error_code()` helpers, and `with_context()` propagation. Request/stream failover paths now consult severity and annotate returned errors, and helper utilities (e.g. file uploads) wrap upstream failures with contextual strings for easier debugging.

### Phase 7: Documentation
1. ✅ Create `docs/ADDING_PROVIDERS.md`
2. ✅ Create `examples/custom_provider.rs`
3. ✅ Add module-level docs to all new modules
4. ✅ Update README if needed

## Phase 4: 1.0 Evolution (The Trait Shift)
### Goal
Shift from Enum-based to Trait-based architecture for true openness.

### Step 1: Core Architecture Refactoring
1. [ ] Rename `ChatApi` to `ChatProvider` (or alias it)
2. [ ] Update `AiClient` to hold `Box<dyn ChatProvider>` instead of `Provider` enum
3. [ ] Downgrade `Provider` enum to a factory helper
4. [ ] Implement `FailoverProvider` struct (implementing `ChatProvider`)
5. [ ] Implement `RoundRobinProvider` struct (implementing `ChatProvider`)

### Step 2: Routing Logic Migration
1. [ ] Remove `__route__` magic string logic from `client_impl.rs` / `stream.rs`
2. [ ] Update `AiClientBuilder` to support strategy composition
3. [ ] Verify routing via `FailoverProvider`

### Step 3: API Completion & Cleanup
1. [x] Ensure `extensions` field in `ChatCompletionRequest` works as `provider_specific`
2. [ ] Remove deprecated aliases in `types::common`
3. [ ] Standardize `AiLibError` for failover triggers

### Step 4: Developer Experience
1. [ ] Create `OpenAiBuilder`, `GroqBuilder` etc.
2. [ ] Update documentation (UPGRADE_1.0.0.md)

### Step 5: Quality & Release
1. [ ] CI MSRV 1.70 check
2. [ ] Wiremock tests
3. [ ] Release 0.4.0 (Trait Shift) -> 1.0.0 (Final)
