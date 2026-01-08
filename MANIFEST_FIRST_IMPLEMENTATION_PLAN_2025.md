# ai-lib Manifest-First 实施计划：2025年LLM API革命就绪

**版本**: 2.0 - 吸收2025年新趋势  
**日期**: 2025-01-XX  
**决策者**: 架构委员会  
**执行者**: 项目总监 & 首席工程师  
**状态**: 🟢 **准备实施**

---

## 执行摘要

**核心决策**: 采纳manifest-first架构，全面拥抱2025年LLM API新趋势

**关键创新**:
- ✅ **Responses API原生支持** - OpenAI Responses风格payload_format
- ✅ **Agentic工具链** - 并行tools、server-side tools、built-in工具链
- ✅ **多模态深化** - video/audio/document/citations支持
- ✅ **Streaming事件模型** - thinking deltas、partial tool_calls
- ✅ **企业级治理** - prompt caching、reasoning tokens、service tiers

**实施策略**: 5阶段渐进交付，总计22周，3-4人并行开发

**成功标准**: 22周后发布ai-lib-manifest 1.0，支持主流6家provider，完整agentic loop

---

## 核心架构决策

### 1. Manifest-First 原则

**单一真源**: manifest作为所有行为的权威来源
- Provider差异 → manifest映射
- 能力检测 → manifest capabilities
- 错误处理 → manifest error_mapping
- 默认值 → manifest defaults

**2025年就绪**:
- Responses API风格支持
- Agentic loop原生能力
- 高级streaming事件模型
- 企业级治理hooks

### 2. OSS + PRO 分层

**OSS核心** (ai-lib-manifest):
- manifest schema & loader
- PayloadBuilder & mapping引擎
- AiClient runtime
- 基础registry

**PRO增值** (ai-lib-pro):
- 企业registry服务
- UI管理面板
- 高级governance
- Codegen优化

### 3. 兼容性策略

**API兼容**: 保持现有ai-lib API表面兼容
**渐进迁移**: 3个月过渡期，双轨运行
**向下兼容**: 旧config自动转换为manifest

---

## 五阶段实施路线图

### Phase 0: 基础架构与2025年Schema (Week 1-2)

**目标**: 建立manifest v1.1规范，支持2025年LLM API新趋势

**核心交付物**:
1. **Manifest Schema v1.1** - 包含所有2025年扩展字段
2. **Rust核心类型** - StandardRequest、UnifiedResponse、StreamingEvent
3. **CLI工具** - validate-manifest、preview-payload
4. **基础loader** - YAML解析、验证、错误处理

**2025年关键扩展**:

#### 新增Manifest字段
```yaml
version: "1.1"
standard_schema:
  # ... existing fields ...

  # 🆕 2025年扩展
  agentic_loop:
    max_iterations: 10
    stop_conditions: ["tool_result", "final_answer"]
    reasoning_effort: "auto"

  streaming_events:
    supported_events: ["PartialContentDelta", "ThinkingDelta", "PartialToolCall"]
    thinking_blocks: true
    citations_enabled: true

providers:
  openai:
    # 🆕 Responses API支持
    response_strategy: "responses_api"
    payload_format: "openai_responses"

    # 🆕 工具链映射
    tools_mapping:
      standard_tool:
        provider_name: "functions"
        schema_path: "functions[].parameters"
        parallel: true
        invoke_style: "parallel"

    # 🆕 高级特性
    experimental_tools: ["builtin_search", "code_execution"]
    prompt_caching:
      enabled: true
      ttl: 3600
    service_tier:
      priority: "high"
      batch_supported: true

models:
  gpt-4o:
    # 🆕 Agentic能力
    agentic_capabilities:
      reasoning_effort: "high"
      thinking_blocks: true
      parallel_tools: true
```

#### 新增Rust类型
```rust
// 🆕 2025年Streaming事件模型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StreamingEvent {
    PartialContentDelta { content: String, model: String },
    ThinkingDelta { thinking: String, effort: ReasoningEffort },
    PartialToolCall { tool_id: String, args: serde_json::Value },
    ToolCallStarted { tool_id: String, name: String },
    ToolCallEnded { tool_id: String, result: ToolResult },
    CitationChunk { source: String, locator: String, snippet: String },
    FinalCandidate { content: String, usage: Usage },
}

// 🆕 Agentic Loop配置
#[derive(Debug, Clone, Deserialize)]
pub struct AgenticConfig {
    pub max_iterations: usize,
    pub stop_conditions: Vec<String>,
    pub reasoning_effort: ReasoningEffort,
    pub thinking_blocks: bool,
}

// 🆕 工具映射配置
#[derive(Debug, Clone, Deserialize)]
pub struct ToolsMapping {
    pub provider_name: String,
    pub schema_path: String,
    pub parallel: bool,
    pub invoke_style: ToolInvokeStyle,
}

// 🆕 多模态扩展
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ContentPart {
    Text(String),
    Image { url: Option<String>, base64: Option<String>, mime: String },
    Audio { url: Option<String>, base64: Option<String>, format: String },
    Video { url: Option<String>, base64: Option<String>, format: String },
    Document { url: Option<String>, base64: Option<String>, mime: String, pages: Option<Vec<u32>> },
}
```

**验收标准**:
- ✅ Manifest v1.1 JSON Schema定稿
- ✅ 包含所有2025年扩展字段
- ✅ CLI工具能验证manifest语法
- ✅ Rust类型编译通过

**风险**: Schema设计不完整 → **缓解**: 参考OpenAI Responses API、Anthropic工具文档

### Phase 1: 核心运行时实现 (Week 3-6)

**目标**: 实现PayloadBuilder、mapping引擎、AiClient基础

**核心交付物**:
1. **Mapping引擎** - 支持复杂path mapping、模板替换
2. **PayloadBuilder** - Responses API、标准JSON、Anthropic风格
3. **AiClient** - 基础chat()、streaming支持
4. **Streaming Parser** - 统一事件模型

**关键实现**:

#### Mapping引擎
```rust
pub struct MappingEngine {
    manifest: Arc<Manifest>,
}

impl MappingEngine {
    // 🆕 支持复杂path mapping
    pub fn map_parameter(
        &self,
        standard_param: &str,
        value: &serde_json::Value,
        provider_id: &str,
    ) -> Result<serde_json::Value, MappingError> {
        let mapping = self.get_mapping(provider_id, standard_param)?;

        match mapping {
            MappingRule::Direct(path) => set_json_path(serde_json::json!({}), path, value.clone()),
            MappingRule::Template(template) => self.apply_template(template, value),
            MappingRule::Conditional(conditions) => self.apply_conditional(conditions, value),
            MappingRule::Nested(path_map) => self.apply_nested_mapping(path_map, value),
        }
    }

    // 🆕 模板替换 (mustache-like)
    fn apply_template(&self, template: &str, value: &serde_json::Value) -> Result<serde_json::Value, MappingError> {
        // {{value}} -> actual value
        // {{config.api_key}} -> from manifest
        // 支持嵌套和条件
    }
}
```

#### PayloadBuilder trait
```rust
#[async_trait]
pub trait PayloadBuilder: Send + Sync {
    async fn build_payload(
        &self,
        request: &StandardRequest,
        manifest: &Manifest,
        provider_id: &str,
        model_id: &str,
    ) -> Result<serde_json::Value, PayloadError>;

    // 🆕 Responses API支持
    async fn build_responses_payload(
        &self,
        request: &StandardRequest,
        manifest: &Manifest,
        provider_id: &str,
    ) -> Result<serde_json::Value, PayloadError>;

    // 🆕 工具调用payload
    async fn build_tools_payload(
        &self,
        tools: &[ToolDefinition],
        manifest: &Manifest,
        provider_id: &str,
    ) -> Result<serde_json::Value, PayloadError>;
}
```

#### AiClient实现
```rust
pub struct AiClient {
    manifest: Arc<Manifest>,
    provider_id: String,
    model_id: String,
    payload_builder: Box<dyn PayloadBuilder>,
    transport: DynHttpTransportRef,
    auth_resolver: Box<dyn AuthResolver>,
}

impl AiClient {
    // 🆕 基础chat方法
    pub async fn chat(&self, request: StandardRequest) -> Result<UnifiedResponse, AiLibError> {
        // 1. 能力预检
        self.validate_capabilities(&request)?;

        // 2. 构建payload
        let payload = self.payload_builder.build_payload(
            &request, &self.manifest, &self.provider_id, &self.model_id
        ).await?;

        // 3. 发送请求
        let response = self.send_request(payload).await?;

        // 4. 解析响应
        self.parse_response(response).await
    }

    // 🆕 Streaming支持
    pub async fn chat_stream(
        &self,
        request: StandardRequest
    ) -> Result<Box<dyn Stream<Item = Result<StreamingEvent, AiLibError>> + Send>, AiLibError> {
        // 实现streaming逻辑
    }
}
```

**验收标准**:
- ✅ 支持OpenAI Responses API格式
- ✅ 基础streaming事件解析
- ✅ 工具调用payload构建
- ✅ 性能基准 < 10ms mapping延迟

**风险**: 复杂mapping逻辑出错 → **缓解**: 严格单元测试 + golden tests

### Phase 2: 多Provider支持与工具链 (Week 7-12)

**目标**: 完整支持主流6家provider，实现agentic loop

**核心交付物**:
1. **6家Provider完整支持** - OpenAI、Anthropic、Gemini、Groq、Cohere、Ollama
2. **Agentic Loop** - 迭代工具调用、推理控制
3. **完整Multimodal** - video/audio/document支持
4. **Codegen POC** - 性能优化验证

**关键实现**:

#### Agentic Loop
```rust
pub struct AgenticLoop {
    client: AiClient,
    config: AgenticConfig,
    tool_registry: HashMap<String, Box<dyn Tool>>,
}

impl AgenticLoop {
    // 🆕 核心agentic方法
    pub async fn run_agentic(
        &self,
        initial_request: StandardRequest,
    ) -> Result<AgenticResponse, AiLibError> {
        let mut conversation = vec![initial_request];
        let mut iteration = 0;

        loop {
            if iteration >= self.config.max_iterations {
                break;
            }

            // 1. 发送当前对话到模型
            let response = self.client.chat(conversation.last().unwrap().clone()).await?;

            // 2. 检查是否需要工具调用
            if let Some(tool_calls) = &response.tool_calls {
                // 并行执行工具调用
                let tool_results = self.execute_tools_parallel(tool_calls).await?;

                // 添加工具结果到对话
                conversation.push(self.build_tool_result_message(tool_results));
            } else {
                // 检查停止条件
                if self.should_stop(&response, &conversation) {
                    break;
                }
            }

            iteration += 1;
        }

        Ok(AgenticResponse {
            final_response: response,
            iterations: iteration,
            tool_calls_made: tool_call_count,
            reasoning_tokens_used: reasoning_usage,
        })
    }

    // 🆕 并行工具执行
    async fn execute_tools_parallel(
        &self,
        tool_calls: &[ToolCall],
    ) -> Result<Vec<ToolResult>, AiLibError> {
        let futures = tool_calls.iter().map(|call| {
            let tool = self.tool_registry.get(&call.name).unwrap();
            tool.invoke(&call.arguments)
        });

        futures::future::join_all(futures).await
            .into_iter()
            .collect::<Result<Vec<_>, _>>()
    }
}
```

#### Provider支持矩阵:

| Provider | Responses API | Agentic Tools | Streaming Events | Multimodal | Priority |
|----------|---------------|---------------|------------------|------------|----------|
| OpenAI | ✅ 原生 | ✅ 并行 | ✅ 完整 | ✅ 图像 | 🔴 高 |
| Anthropic | ⚠️ 适配 | ✅ 单工具流 | ✅ thinking | ✅ 图像 | 🔴 高 |
| Gemini | ❌ | ✅ 并行 | ⚠️ 部分 | ✅ 多模态 | 🟡 中 |
| Groq | ❌ | ✅ 标准 | ✅ 基础 | ❌ | 🟡 中 |
| Cohere | ❌ | ⚠️ 自定义 | ✅ 基础 | ❌ | 🟢 低 |
| Ollama | ❌ | ⚠️ 适配 | ✅ 基础 | ⚠️ 实验性 | 🟢 低 |

#### Multimodal扩展
```rust
// 🆕 上传策略
pub enum UploadStrategy {
    Multipart,
    Base64Inline,
    UrlReference,
}

// 🆕 Citations支持
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Citation {
    pub source: String,
    pub locator: String,  // page number, timestamp, etc.
    pub snippet: Option<String>,
    pub confidence: Option<f64>,
}

impl AiClient {
    // 🆕 多模态文件上传
    pub async fn upload_multimodal(
        &self,
        content: ContentPart,
        strategy: UploadStrategy,
    ) -> Result<String, AiLibError> {
        match strategy {
            UploadStrategy::Multipart => self.upload_multipart(content).await,
            UploadStrategy::Base64Inline => self.encode_base64(content),
            UploadStrategy::UrlReference => self.get_signed_url(content).await,
        }
    }
}
```

**验收标准**:
- ✅ 6家provider完整支持
- ✅ Agentic loop端到端工作
- ✅ 所有multimodal格式支持
- ✅ Codegen性能提升 > 50%

**风险**: Provider API差异大 → **缓解**: 抽象层设计 + 扩展manifest字段

### Phase 3: 测试与质量保证 (Week 13-16)

**目标**: 建立完整的测试矩阵和CI/CD

**核心交付物**:
1. **测试矩阵** - Payload snapshots、streaming tests、E2E tests
2. **CI/CD Pipeline** - 自动验证、性能基准
3. **性能优化** - Codegen、缓存、多线程
4. **文档** - API文档、迁移指南、示例

**测试矩阵**:

#### Payload Snapshot Tests
```rust
#[cfg(test)]
mod payload_snapshots {
    // 🆕 针对每个provider的golden tests
    #[test]
    fn openai_responses_payload_snapshot() {
        let request = create_standard_request_with_tools();
        let manifest = load_test_manifest();

        let payload = PayloadBuilder::build_for_provider(
            &request, &manifest, "openai", "gpt-4o"
        ).await.unwrap();

        // 与golden file比较
        assert_payload_matches_golden(&payload, "openai_responses_golden.json");
    }
}
```

#### Streaming Tests
```rust
#[cfg(test)]
mod streaming_tests {
    #[tokio::test]
    async fn anthropic_thinking_deltas() {
        let client = create_test_client("anthropic", "claude-3-5-sonnet");
        let request = create_agentic_request();

        let events = collect_streaming_events(client.chat_stream(request).await).await;

        // 验证thinking deltas顺序
        assert_thinking_deltas_sequence(&events);
        // 验证tool calls完整性
        assert_tool_calls_completeness(&events);
    }
}
```

#### Performance Benchmarks
```rust
#[cfg(test)]
mod benchmarks {
    use criterion::{criterion_group, criterion_main, Criterion};

    fn payload_mapping_benchmark(c: &mut Criterion) {
        let manifest = load_large_manifest();
        let request = create_complex_request();

        c.bench_function("complex_payload_mapping", |b| {
            b.iter(|| {
                let payload = black_box(mapping_engine.map_request(&request, &manifest, "openai"));
                black_box(payload);
            });
        });
    }

    // 🆕 Codegen性能对比
    fn codegen_vs_runtime_benchmark(c: &mut Criterion) {
        // 对比codegen生成的代码 vs 运行时mapping
    }
}
```

**验收标准**:
- ✅ 测试覆盖率 > 90%
- ✅ CI通过所有golden tests
- ✅ 性能基准稳定
- ✅ 文档覆盖完整

**风险**: 测试维护成本高 → **缓解**: 自动化golden test更新

### Phase 4: 生态与PRO功能 (Week 17-22)

**目标**: 建立生态系统，企业级PRO功能

**核心交付物**:
1. **Manifest Registry** - 社区贡献和治理
2. **PRO功能** - 企业治理、UI、审计
3. **SDK生态** - Python/TS绑定
4. **企业集成** - 审计、SLA、RBAC

**Registry设计**:
```rust
// 🆕 Registry服务
pub struct ManifestRegistry {
    storage: Arc<dyn RegistryStorage>,
    validator: ManifestValidator,
    auditor: Option<RegistryAuditor>,
}

impl ManifestRegistry {
    // 提交新manifest
    pub async fn submit_manifest(
        &self,
        manifest: Manifest,
        submitter: &str,
    ) -> Result<ManifestId, RegistryError> {
        // 验证
        self.validator.validate(&manifest)?;

        // 审计
        if let Some(auditor) = &self.auditor {
            auditor.record_submission(&manifest, submitter).await?;
        }

        // 存储
        let id = self.storage.store(manifest).await?;

        Ok(id)
    }

    // 搜索manifest
    pub async fn search_manifests(
        &self,
        query: SearchQuery,
    ) -> Result<Vec<ManifestSummary>, RegistryError> {
        self.storage.search(query).await
    }
}
```

**PRO功能**:
```rust
// 🆕 企业治理
#[cfg(feature = "enterprise")]
pub struct EnterpriseClient {
    base_client: AiClient,
    auditor: Arc<dyn Auditor>,
    rate_limiter: Arc<dyn RateLimiter>,
    cost_tracker: Arc<dyn CostTracker>,
}

#[cfg(feature = "enterprise")]
impl EnterpriseClient {
    // 审计所有请求
    pub async fn chat_with_audit(
        &self,
        request: StandardRequest,
        user_context: &UserContext,
    ) -> Result<UnifiedResponse, AiLibError> {
        // 权限检查
        self.check_permissions(user_context, &request).await?;

        // 记录审计日志
        self.auditor.record_request(&request, user_context).await?;

        // 执行请求
        let response = self.base_client.chat(request).await?;

        // 记录响应和成本
        self.auditor.record_response(&response, user_context).await?;
        self.cost_tracker.record_usage(&response.usage, user_context).await?;

        Ok(response)
    }
}
```

**验收标准**:
- ✅ Registry服务稳定运行
- ✅ PRO功能完整实现
- ✅ SDK生态有Python/TS绑定
- ✅ 企业集成通过安全审计

---

## 实施资源与时间估算

### 团队配置

**核心团队** (3-4人):
- **架构师/首席工程师** (1人): 总体设计、代码审查、性能优化
- **资深工程师** (1-2人): 核心实现、provider适配、测试
- **工具链工程师** (1人): CLI、CI/CD、codegen、registry

**外部资源**:
- **产品经理**: 需求澄清、优先级排序
- **安全专家**: 安全审查、企业功能设计
- **DevOps**: 基础设施、监控、部署

### 时间分配

| Phase | 时间 | 工程师分配 | 关键里程碑 |
|-------|------|-----------|-----------|
| **Phase 0** | 2周 | 2人 | Manifest v1.1定稿、核心类型实现 |
| **Phase 1** | 4周 | 3人 | 核心运行时完成、基础streaming |
| **Phase 2** | 6周 | 4人 | 6家provider支持、agentic loop |
| **Phase 3** | 4周 | 3人 | 测试矩阵完成、性能优化 |
| **Phase 4** | 6周 | 4人 | Registry上线、PRO功能就绪 |

**总计**: 22周，约5-6个月

### 风险管理

#### 高风险项目

1. **2025年API变化快**
   - **缓解**: 模块化设计、manifest热重载、版本管理

2. **性能要求高**
   - **缓解**: 性能基准测试、codegen优化、缓存策略

3. **企业安全要求**
   - **缓解**: 安全专家参与、安全审计、零信任设计

#### 技术债务管理

1. **保持manifest向后兼容**
2. **API设计稳定后冻结**
3. **定期重构技术债务**

---

## 成功度量标准

### 技术指标

- ✅ **功能完整性**: 支持6家主流provider + 2025年特性
- ✅ **性能表现**: Payload mapping < 5ms，streaming延迟 < 100ms
- ✅ **测试覆盖**: 单元测试 > 90%，集成测试100%通过
- ✅ **兼容性**: 现有ai-lib用户零代码修改

### 业务指标

- ✅ **社区采用**: 100+ manifest贡献，1000+ GitHub stars
- ✅ **企业客户**: 5+企业客户验证，SLA 99.9%
- ✅ **生态健康**: Python/TS SDK发布，活跃社区

### 时间里程碑

- **Week 2**: Manifest v1.1发布，PoC演示
- **Week 6**: 核心运行时完成，OpenAI+Anthropic完整支持
- **Week 12**: Agentic loop发布，6家provider就绪
- **Week 16**: 生产就绪，完整测试通过
- **Week 22**: 1.0版本发布，企业PRO功能上线

---

## 立即行动计划

### Week 1-2 (Phase 0)

1. **创建新仓库** `ai-lib-manifest`
2. **实现Manifest Schema v1.1**
   - 包含所有2025年扩展字段
   - JSON Schema验证
   - 示例manifests
3. **核心Rust类型**
   - StandardRequest/Response
   - StreamingEvent模型
   - AgenticConfig/ToolMapping
4. **CLI工具基础**
   - validate-manifest
   - preview-payload

### 关键决策点

1. **Schema冻结**: Phase 0结束时manifest schema定稿
2. **API稳定**: Phase 1结束时Rust API稳定
3. **兼容策略**: Phase 2开始时确认迁移计划

---

## 结论

这个实施计划将ai-lib转变为**2025年最先进的LLM统一SDK**，完全拥抱新趋势：

- **Responses API原生支持**
- **Agentic工具链革命**
- **多模态深度集成**
- **企业级治理能力**

通过22周的精心实施，我们将交付一个**真正manifest-first、production-ready、future-proof**的ai-lib新版本。

**开始执行Phase 0！**

---

**文档版本历史**:
- v1.0: 初始需求规格
- v2.0: 吸收2025年新趋势，完整实施计划
