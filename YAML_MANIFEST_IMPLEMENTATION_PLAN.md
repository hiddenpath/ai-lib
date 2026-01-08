# YAML Manifest实施计划：彻底重构ai-lib配置系统

**实施日期**: 2025-01-XX  
**项目状态**: **激进重构模式** - 无需向后兼容  
**目标**: 完全废弃JSON配置，实现YAML Manifest革命性设计

---

## 执行摘要

**机会难得**：当前代码未发布，可以**完全废弃JSON配置系统**，进行彻底重构。

**核心策略**：
- ✅ **零兼容性顾虑** - 直接废弃所有现有配置代码
- ✅ **革命性重构** - 全面实现YAML Manifest设计
- ✅ **2025年领先** - 构建最先进的AI配置系统
- ✅ **企业级就绪** - 完整支持治理和扩展需求

**实施目标**：**让ai-lib成为Rust生态最先进的AI统一SDK**

---

## 一、废弃清单：完全清理现有配置系统

### 1.1 待废弃的文件和模块

**配置相关**:
- ❌ `src/defaults/models.json` - 废弃JSON格式
- ❌ `src/config/mod.rs` - 现有配置模块
- ❌ `src/config/embedded.rs` - 嵌入式配置
- ❌ `src/config/file.rs` - 文件配置
- ❌ `src/config/provider_trait.rs` - 现有trait
- ❌ `src/config/converter.rs` - 转换逻辑

**注册表相关**:
- ❌ `src/registry/mod.rs` - 现有注册表实现
- ❌ `src/registry/model.rs` - 模型定义
- ❌ `src/registry/watcher.rs` - 配置热重载

**提供商配置**:
- ❌ `src/provider/config.rs` - ProviderConfig
- ❌ `src/provider/configs.rs` - 配置工厂
- ❌ `src/provider/classification.rs` - 分类逻辑

### 1.2 保留但重构的模块

**保留但完全重写**:
- 🔄 `src/provider/generic.rs` - 改为ConfigDrivenAdapter
- 🔄 `src/client/builder.rs` - 集成新的Manifest系统
- 🔄 `src/types/function_call.rs` - 扩展工具调用支持

---

## 二、YAML Manifest核心设计

### 2.1 完整三层架构

```yaml
# ai-lib-manifest.yaml
version: "1.0"
metadata:
  description: "AI-Lib Provider Manifest"
  last_updated: "2025-01-XX"

# 第一层：标准接口定义（开发者统一接口）
standard_schema:
  # 基础参数
  parameters:
    temperature:
      type: float
      range: [0.0, 2.0]
      default: 1.0
    max_tokens:
      type: integer
      min: 1
      max: 32768
    stream:
      type: boolean
      default: false

  # 工具调用（2025年核心）
  tools:
    schema: "standard_tool_definition"
    choice_policy: ["auto", "none", "required", "specific"]
    strict_mode: boolean
    parallel_calls: boolean

  # 响应格式
  response_format:
    types: ["text", "json", "structured"]
    schema_validation: boolean

  # 多模态内容
  multimodal:
    image:
      formats: ["png", "jpeg", "gif", "webp"]
      max_size: "10MB"
    audio:
      formats: ["mp3", "wav", "ogg", "m4a", "flac"]
      max_size: "25MB"
    video:
      formats: ["mp4", "avi", "mov"]
      max_size: "100MB"

# 第二层：提供商异构映射（核心转换逻辑）
providers:
  openai:
    version: "v1"
    base_url: "https://api.openai.com/v1"
    auth:
      type: bearer
      token_env: "OPENAI_API_KEY"

    # 请求体映射
    payload_format: "openai_style"
    parameter_mappings:
      temperature: "temperature"
      max_tokens: "max_tokens"
      stream: "stream"
      tools: "tools"
      tool_choice: "tool_choice"
      tool_choice_required: "tool_choice=function"

    # 特殊处理
    special_handling:
      system_message: "messages[0]"  # 系统消息位置

    # 响应格式映射
    response_format: "openai_style"
    response_paths:
      content: "choices[0].message.content"
      tool_calls: "choices[0].message.tool_calls"
      usage: "usage"
      finish_reason: "choices[0].finish_reason"

    # 流式响应处理
    streaming:
      event_format: "data_lines"
      content_path: "choices[0].delta.content"
      tool_call_path: "choices[0].delta.tool_calls"

    # 实验性特性
    experimental_features:
      - "strict_tools"
      - "parallel_tool_calls"

  anthropic:
    version: "v1"
    base_url: "https://api.anthropic.com/v1"
    auth:
      type: bearer
      token_env: "ANTHROPIC_API_KEY"
      extra_headers:
        - name: "anthropic-version"
          value: "2023-06-01"
        - name: "anthropic-beta"
          value: "tools-2024-05-16"

    payload_format: "anthropic_style"
    parameter_mappings:
      temperature: "temperature"
      max_tokens: "max_tokens"
      stream: "stream"
      tools: "tools"
      tool_choice: "tool_choice"
      system_message: "system"  # 顶级字段

    special_handling:
      system_prompt: "system顶层字段"
      tool_result: "tool_result格式"

    response_format: "anthropic_style"
    response_paths:
      content: "content[0].text"  # content_block结构
      tool_calls: "content[0].tool_calls"
      usage: "usage"
      stop_reason: "stop_reason"

    streaming:
      event_format: "anthropic_sse"
      content_path: "delta.text"
      tool_call_path: "delta.tool_calls"

    experimental_features:
      - "mcp"
      - "advanced-tool-use-2025"

  gemini:
    version: "v1beta"
    base_url: "https://generativelanguage.googleapis.com/v1beta"
    auth:
      type: query_param
      param_name: "key"
      token_env: "GEMINI_API_KEY"

    payload_format: "gemini_style"
    parameter_mappings:
      temperature: "generationConfig.temperature"
      max_tokens: "generationConfig.maxOutputTokens"
      stream: null  # Gemini不支持流式
      tools: "tools"
      tool_choice: "toolConfig"

    special_handling:
      message_structure: "contents数组"
      inline_data: "inlineData格式"

    response_format: "gemini_style"
    response_paths:
      content: "candidates[0].content.parts[0].text"
      tool_calls: "candidates[0].content.parts[0].functionCall"
      finish_reason: "candidates[0].finishReason"

# 第三层：模型实例配置（具体覆盖）
models:
  gpt-4o:
    provider: openai
    model_id: "gpt-4o"
    display_name: "GPT-4o"
    context_window: 128000
    capabilities:
      - vision
      - tools
      - json_mode
      - audio
    pricing:
      input_per_token: 0.000005
      output_per_token: 0.000015
    overrides: {}  # 继承provider配置

  claude-3-5-sonnet:
    provider: anthropic
    model_id: "claude-3-5-sonnet-20241022"
    display_name: "Claude 3.5 Sonnet"
    context_window: 200000
    capabilities:
      - vision
      - tools
      - json_mode
    pricing:
      input_per_token: 0.000003
      output_per_token: 0.000015
    overrides:
      max_tokens: 4096  # 覆盖默认值

  gemini-pro-vision:
    provider: gemini
    model_id: "gemini-pro-vision"
    display_name: "Gemini Pro Vision"
    context_window: 16384
    capabilities:
      - vision
      - tools
    pricing:
      input_per_token: 0.00000025
      output_per_token: 0.0000005
    overrides:
      temperature: "generationConfig.temperature"  # 路径覆盖
```

### 2.2 核心Rust架构设计

```rust
// 新架构：完全基于Manifest的动态适配器

pub mod manifest {
    pub mod loader;     // YAML加载和验证
    pub mod schema;     // 结构化Schema定义
    pub mod validator;  // 配置验证器
}

pub mod adapter {
    pub mod dynamic;    // 基于配置的动态适配器
    pub mod payload;    // 请求体构建器
    pub mod response;   // 响应解析器
    pub mod streaming;  // 流式处理器
}

// 核心类型定义
#[derive(Debug, Clone, Deserialize)]
pub struct Manifest {
    pub version: String,
    pub standard_schema: StandardSchema,
    pub providers: HashMap<String, ProviderDefinition>,
    pub models: HashMap<String, ModelDefinition>,
}

#[derive(Debug, Clone)]
pub struct ConfigDrivenAdapter {
    manifest: Arc<Manifest>,
    provider_def: ProviderDefinition,
    model_def: ModelDefinition,
    transport: DynHttpTransportRef,
    auth_resolver: Box<dyn AuthResolver>,
}

impl ChatProvider for ConfigDrivenAdapter {
    async fn chat_completion(
        &self,
        request: ChatCompletionRequest
    ) -> Result<ChatCompletionResponse, AiLibError> {
        // 1. 能力检查
        self.validate_capabilities(&request)?;

        // 2. 使用配置的映射规则构建请求体
        let payload = self.build_payload(&request)?;

        // 3. 发送HTTP请求
        let response = self.send_request(payload).await?;

        // 4. 使用配置的解析规则处理响应
        let parsed = self.parse_response(response)?;

        Ok(parsed)
    }
}
```

---

## 三、实施路线图：激进重构计划

### Phase 1: 核心架构 (Week 1-2) - 5天

**目标**: 建立YAML Manifest的基础架构

**任务**:
1. **设计Schema类型** (2天)
   - [ ] 定义完整的Rust结构体 (StandardSchema, ProviderDefinition, ModelDefinition)
   - [ ] 实现serde反序列化支持
   - [ ] 添加配置验证逻辑

2. **实现Manifest加载器** (2天)
   - [ ] YAML文件解析
   - [ ] 配置验证和错误处理
   - [ ] 热重载支持 (可选)

3. **基础测试** (1天)
   - [ ] 单元测试配置加载
   - [ ] 验证基本YAML解析

**输出**: 完整的配置加载和验证系统

### Phase 2: 动态适配器核心 (Week 3-4) - 7天

**目标**: 实现基于配置的动态请求/响应处理

**任务**:
1. **Payload构建器** (3天)
   - [ ] 实现参数映射系统
   - [ ] 支持嵌套路径 (generationConfig.temperature)
   - [ ] 特殊处理逻辑 (system消息位置等)

2. **Response解析器** (2天)
   - [ ] 实现路径解析 (choices[0].message.content)
   - [ ] 支持不同响应格式 (OpenAI/Anthropic/Gemini)
   - [ ] 流式响应处理

3. **工具调用映射** (2天)
   - [ ] 扩展FunctionCall支持
   - [ ] 实现不同provider的工具格式转换
   - [ ] 严格模式和并行调用支持

**输出**: 可以处理所有主要provider的动态适配器

### Phase 3: 高级特性 (Week 5-6) - 6天

**目标**: 实现2025年AI特性支持

**任务**:
1. **认证系统扩展** (2天)
   - [ ] OAuth2支持
   - [ ] Google ADC支持
   - [ ] 自定义headers

2. **多模态处理** (2天)
   - [ ] 文件上传逻辑
   - [ ] 内容类型检测
   - [ ] 大小限制验证

3. **能力检查系统** (2天)
   - [ ] 请求前能力验证
   - [ ] 错误消息生成
   - [ ] 降级策略

**输出**: 完整的2025年AI特性支持

### Phase 4: 集成与测试 (Week 7-8) - 8天

**目标**: 与现有系统集成，完整测试

**任务**:
1. **Builder集成** (2天)
   - [ ] 修改AiClientBuilder使用新系统
   - [ ] 提供向后兼容的简单API
   - [ ] 错误处理和日志

2. **完整测试覆盖** (4天)
   - [ ] 所有provider的请求/响应测试
   - [ ] 工具调用测试
   - [ ] 流式处理测试
   - [ ] 多模态测试

3. **性能优化** (2天)
   - [ ] 配置预编译
   - [ ] 缓存优化
   - [ ] 基准测试

**输出**: 生产就绪的完整系统

---

## 四、技术实现细节

### 4.1 YAML Schema的Rust表达

```rust
// 标准schema定义
#[derive(Debug, Clone, Deserialize)]
pub struct StandardSchema {
    pub parameters: HashMap<String, ParameterDefinition>,
    pub tools: ToolSchema,
    pub response_format: ResponseFormatSchema,
    pub multimodal: MultimodalSchema,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ParameterDefinition {
    pub param_type: ParameterType,
    #[serde(flatten)]
    pub constraints: ParameterConstraints,
}

// 提供商定义
#[derive(Debug, Clone, Deserialize)]
pub struct ProviderDefinition {
    pub version: String,
    pub base_url: String,
    pub auth: AuthConfig,
    pub payload_format: PayloadFormat,
    pub parameter_mappings: HashMap<String, MappingRule>,
    pub special_handling: HashMap<String, SpecialHandling>,
    pub response_format: ResponseFormat,
    pub response_paths: HashMap<String, JsonPath>,
    pub streaming: StreamingConfig,
    pub experimental_features: Vec<String>,
}

// 模型定义
#[derive(Debug, Clone, Deserialize)]
pub struct ModelDefinition {
    pub provider: String,
    pub model_id: String,
    pub display_name: Option<String>,
    pub context_window: usize,
    pub capabilities: Vec<Capability>,
    pub pricing: PricingInfo,
    pub overrides: HashMap<String, serde_json::Value>,
}
```

### 4.2 动态映射系统的实现

```rust
// 参数映射引擎
pub struct ParameterMapper {
    mappings: HashMap<String, MappingRule>,
}

impl ParameterMapper {
    pub fn map_parameter(
        &self,
        standard_param: &str,
        value: &serde_json::Value,
        target: &mut serde_json::Value
    ) -> Result<(), MappingError> {
        let rule = self.mappings.get(standard_param)
            .ok_or(MappingError::NoMapping)?;

        match rule {
            MappingRule::Direct(path) => {
                set_json_path(target, path, value.clone())?;
            }
            MappingRule::Transform(transform) => {
                let transformed = transform.apply(value)?;
                set_json_path(target, &transform.target_path, transformed)?;
            }
            MappingRule::Conditional(conditions) => {
                for condition in conditions {
                    if condition.matches(value) {
                        set_json_path(target, &condition.target_path, value.clone())?;
                        break;
                    }
                }
            }
        }
        Ok(())
    }
}
```

### 4.3 响应解析系统的实现

```rust
// 响应解析引擎
pub struct ResponseParser {
    paths: HashMap<String, JsonPath>,
    format: ResponseFormat,
}

impl ResponseParser {
    pub fn parse_response(
        &self,
        response: serde_json::Value
    ) -> Result<ChatCompletionResponse, ParseError> {
        match self.format {
            ResponseFormat::OpenAI => self.parse_openai_response(response),
            ResponseFormat::Anthropic => self.parse_anthropic_response(response),
            ResponseFormat::Gemini => self.parse_gemini_response(response),
        }
    }

    fn parse_openai_response(&self, response: serde_json::Value) -> Result<ChatCompletionResponse, ParseError> {
        let content_path = self.paths.get("content").unwrap();
        let content = get_json_path(&response, content_path)?;

        let tool_calls_path = self.paths.get("tool_calls");
        let tool_calls = if let Some(path) = tool_calls_path {
            get_json_path(&response, path)?
        } else {
            Value::Null
        };

        // 构建标准响应...
        Ok(ChatCompletionResponse { /* ... */ })
    }
}
```

---

## 五、测试策略与质量保证

### 5.1 单元测试

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_openai_parameter_mapping() {
        let manifest = load_test_manifest();
        let mapper = ParameterMapper::from_provider(&manifest.providers["openai"]);

        let mut target = json!({});
        mapper.map_parameter("temperature", &json!(0.7), &mut target)?;

        assert_eq!(target["temperature"], json!(0.7));
    }

    #[test]
    fn test_anthropic_system_message() {
        let manifest = load_test_manifest();
        let mapper = ParameterMapper::from_provider(&manifest.providers["anthropic"]);

        let request = ChatCompletionRequest::new("claude-3".to_string(), vec![
            Message::system("You are helpful".to_string()),
            Message::user("Hello".to_string()),
        ]);

        let payload = mapper.build_payload(&request)?;
        assert_eq!(payload["system"], json!("You are helpful"));
    }
}
```

### 5.2 集成测试

```rust
#[cfg(test)]
mod integration_tests {
    use super::*;
    use wiremock::MockServer;

    #[tokio::test]
    async fn test_openai_chat_completion() {
        let mock_server = MockServer::start().await;

        // Mock OpenAI API response
        mock_server.register(Mock::given(method("POST"))
            .and(path("/v1/chat/completions"))
            .respond_with(ResponseTemplate::new(200)
                .set_body_json(json!({
                    "choices": [{
                        "message": {"content": "Hello from OpenAI"}
                    }]
                }))));

        let manifest = create_test_manifest();
        let adapter = ConfigDrivenAdapter::new(
            Arc::new(manifest),
            "openai",
            &mock_server.uri()
        );

        let request = ChatCompletionRequest::new(
            "gpt-4".to_string(),
            vec![Message::user("Hello")]
        );

        let response = adapter.chat_completion(request).await?;
        assert_eq!(response.choices[0].message.content, "Hello from OpenAI");
    }
}
```

### 5.3 性能测试

```rust
#[cfg(test)]
mod benches {
    use criterion::{criterion_group, criterion_main, Criterion};

    fn bench_parameter_mapping(c: &mut Criterion) {
        let manifest = load_test_manifest();
        let mapper = ParameterMapper::from_provider(&manifest.providers["openai"]);

        c.bench_function("openai_parameter_mapping", |b| {
            b.iter(|| {
                let mut target = json!({});
                mapper.map_parameter("temperature", &json!(0.7), &mut target).unwrap();
                mapper.map_parameter("max_tokens", &json!(1000), &mut target).unwrap();
            });
        });
    }

    criterion_group!(benches, bench_parameter_mapping);
    criterion_main!(benches);
}
```

---

## 六、成功标准与验收条件

### 6.1 功能验收

- ✅ **OpenAI兼容**: 完整支持GPT系列模型
- ✅ **Anthropic支持**: Claude模型全功能
- ✅ **Gemini支持**: Google Gemini多模态
- ✅ **工具调用**: 所有provider的统一工具调用
- ✅ **流式处理**: 完整SSE/JSONL支持
- ✅ **多模态**: 图像、音频、视频处理

### 6.2 性能验收

- ✅ **冷启动**: < 100ms配置加载
- ✅ **热请求**: < 10ms参数映射
- ✅ **内存**: < 50MB基线内存使用
- ✅ **并发**: 支持1000并发请求

### 6.3 扩展性验收

- ✅ **新provider**: 纯YAML配置添加
- ✅ **新特性**: 无需Rust代码修改
- ✅ **向后兼容**: 优雅降级策略

---

## 七、风险与缓解

### 7.1 技术风险

**YAML复杂度管理**:
- **缓解**: 分模块加载，逐步验证
- **测试**: 完整的schema验证测试

**动态映射性能**:
- **缓解**: 预编译映射规则，缓存结果
- **监控**: 详细性能基准测试

### 7.2 实施风险

**激进重构范围**:
- **缓解**: 分阶段实施，每阶段可独立验证
- **回滚**: 保留git历史，支持快速回滚

**测试覆盖不足**:
- **缓解**: TDD模式，先写测试再实现功能
- **目标**: 目标测试覆盖率 > 90%

---

## 八、实施开始

### 8.1 第一步：环境准备

```bash
# 1. 创建新分支
git checkout -b feature/yaml-manifest-revolution

# 2. 安装依赖
cargo add serde_yaml
cargo add serde_json
cargo add jsonpath-rust  # JSON路径解析

# 3. 创建目录结构
mkdir -p src/manifest
mkdir -p src/adapter
mkdir -p benches
```

### 8.2 第一天：核心类型定义

开始实现`src/manifest/schema.rs`，定义完整的YAML Schema对应的Rust结构体。

**目标**: Day 1结束时，有完整的类型定义和基本的serde支持。

---

## 结论

这个YAML Manifest革命性设计将让ai-lib成为**Rust生态最先进的AI统一SDK**：

1. **零代码扩展** - 新AI提供商只需YAML配置
2. **2025年领先** - 完整支持现代AI特性
3. **企业级治理** - 能力检查、审计、合规
4. **性能卓越** - 动态映射不牺牲性能
5. **未来proof** - DSL设计支持长期演进

**让我们开始这场革命！**

---

**实施计划版本**: 1.0  
**创建日期**: 2025-01-XX  
**负责人**: 项目总监 & 资深工程师  
**状态**: **准备实施**
