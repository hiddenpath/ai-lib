# ai-lib (manifest-first) — 需求规格与差距补全文档

版本：1.0  
作者（角色）：项目总监 / 首席工程师（建议）  
目标受众：产品/架构/工程团队、社区贡献者、企业客户评估者

简介
----
你已决定从头启动一个以 manifest 为起点的新 ai-lib 项目，把“参数文档（manifest）”作为单一真源（source of truth）并以此驱动 runtime、provider 适配器、治理和生态。本文档把当前 ai-lib 已有或应有的功能需求完整列出、合理化、并指出潜在漏洞与补充项，形成新项目的正式需求文档草案，供立项、排期与开发使用。

目标（Why）
----
- 将 ai-lib 打造为“数据驱动的单一接口大模型执行引擎”：用户通过统一的 StandardRequest/manifest 即可接入任意大模型。
- 将厂商/模型差异通过 manifest 中的映射与格式化器消化，避免将多厂商知识硬编码进库。
- 实现 OSS + PRO 分层：开源核心（manifest schema、loader、runtime reference impl），企业/PRO 提供治理、registry、UI、codegen 等增值功能。
- 降低新增模型/厂商的工程成本并提高社区参与度。

范围（Scope）
----
包含：
- manifest 规范设计（YAML/JSON + JSON Schema）
- manifest loader、validator 与 CLI
- mapping 引擎（parameter mapping、path/nesting、payload_format）
- PayloadBuilder 插件（openai, google, anthropic, generic）
- 标准请求/响应类型（StandardRequest、UnifiedResponse）
- AiClient runtime（auth、endpoint templating、retry、streaming 解析）
- 工具/函数调用（ToolDefinition / ToolCall）统一与映射
- 多模态支持（text/image/audio/…）的统一载荷抽象
- 测试矩阵、golden tests、CI 校验流程
- 文档、示例、迁移指南与 SDK（首版 Rust，后续 Python/TS）
- Manifest registry（repo or service）与治理流程设计（基础）

不包含（初期可选/PRO）
- 托管 Registry SaaS（可作为 PRO）
- 完整 UI（registry 管理面板，PRO）
- 高性能 codegen（可选进阶）
- 企业专用审计/计费器（PRO）

总体功能列表（Detailed Functional Requirements）
----
注：按功能模块列出，带必要的行为说明、输入输出与验收条件。

1. Manifest 与 Schema（核心）
- 要求：
  - 定义 ai-lib-manifest v1（YAML/JSON）格式及 JSON Schema。
  - 必须包含：version、meta、standard_schema（通用参数定义）、providers（模板/映射）、models（实例化）、capabilities、connection_vars、parameter_mapping、payload_format、extensions/extra。
  - 支持 mapping 类型：identity、rename、nesting（dot.path）、elevation（顶层提升）、template（字符串模版）、conditional mapping（可选）。
  - 支持 model-level overrides、default values、capabilities 标注（vision, audio, tools, streaming, json_schema, function_calls 等）。
- 验收：
  - JSON Schema 可以校验示例 manifest（OpenAI/Gemini/Anthropic）。
  - CI 中有 manifest 验证步骤。

2. Manifest Loader & CLI
- 要求：
  - 在运行时/编译时加载 manifest。
  - CLI 工具：validate-manifest、preview-payload、generate-mapping-report。
  - 支持从本地 repo、远程 registry（URL）、环境变量注入连接变量（但不包含 secrets）。
- 验收：
  - CLI 能对 sample manifests 进行校验并生成 mapping 预览。

3. StandardRequest / Standard Types（SDK 层）
- 要求：
  - 定义统一的 StandardRequest（messages、inference_params、tools、multimodal parts、streaming flag）。
  - InferenceParams 包含主要字段（temperature, top_p, top_k, max_tokens, stop_sequences, logit_bias, seed, stream 等）并允许扩展 extra: HashMap<String, Value>。
  - Message role 标准化：system, user, assistant, tool。
- 验收：
  - 示例能用 StandardRequest 同时表达 OpenAI 与 Google 请求语义。

4. Mapping 引擎与 PayloadBuilder（核心运行时）
- 要求：
  - 实现 mapping 引擎：将 StandardRequest + InferenceParams 按 manifest 的 parameter_mapping 与 payload_format 生成 provider-specific JSON/HTTP body。
  - PayloadBuilder trait 与多种实现（openai_json, google_native, anthropic_messages, custom）。
  - 支持嵌套路径创建（生成 generationConfig: { temperature: ... }）。
  - 支持 headers、query 参数、endpoint path template 填充。
- 验收：
  - 给定 StandardRequest 与 manifest，输出与厂商预期一致的 JSON（golden test）。

5. AiClient（运行时客户端）
- 要求：
  - 提供统一的 AiClient：初始化时绑定 model_id 或 provider manifest；提供 chat(), embed(), generate_image(), tool_call() 等方法（基于模型 capabilities 自动启用/禁用）。
  - Auth 抽象（bearer, api_key, aws_sigv4, google_adc）。secrets 通过 env/vault 注入而非 manifest。
  - Endpoint templating、timeout、retry/backoff、proxy 支持。
  - Streaming 支持：SSE、chunked、gRPC-stream（如适用）解析器，并输出统一的 streaming 事件语义。
  - 支持 passthrough/escape-hatch：允许将 extra 参数直接注入请求。
- 验收：
  - AiClient 能对至少 3 个 manifest（OpenAI/Gemini/Anthropic）产生正确请求并解析模拟响应（mock/sandbox）。

6. Tools / Function Calls（工具执行与解析）
- 要求：
  - 标准化 ToolDefinition：id, name, description, input_schema, output_schema, invocation_style。
  - 支持厂商函数调用映射（OpenAI function_calls、Anthropic 的 tooling 等）。
  - 支持 tool_call 路由与统一 ToolCall 结果（包含 raw_response）。
- 验收：
  - StandardRequest 的 tool 指定能被映射为厂商对应格式，并能将响应解析为统一 ToolCall。

7. Multimodal 支持
- 要求：
  - 标准化 ContentPart（text, image{url/base64}, audio{url/base64,format}, video）。
  - 对厂商间差异做映射（例如：image as URL vs base64）。
  - 支持附件上传策略（multipart/form-data, base64 embedded）与 manifest 中说明。
- 验收：
  - 多模态请求能正确转换为至少两个厂商的 payload。

8. UnifiedResponse 与 Telemetry
- 要求：
  - 统一的响应结构 UnifiedResponse（id, content, usage, tool_calls, raw_response, meta{http_status, latency, provider, model, manifest_version}）。
  - Telemetry 元数据注入（request_id, model_id, provider, manifest_version）。
- 验收：
  - AiClient 返回的 response 包含 meta 且可用于监控指标计算。

9. Error Mapping 与 Retry 策略
- 要求：
  - 统一错误枚举（RateLimit, AuthError, MappingError, ProviderError, Timeout, ParsingError）。
  - 将厂商错误码映射为统一错误；支持 provider 定制 retry 策略。
- 验收：
  - 在模拟不同厂商错误时，错误被正确映射并触发合适 retry/backoff。

10. Security & Secrets 管理
- 要求：
  - Manifest 不包含 secrets；secrets 通过外部机制注入（env, vault）。
  - 提供安全审计点（who changed manifest, manifest_version）。
  - 对 manifest inputs 做 schema 校验以避免 header injection 等风险。
- 验收：
  - secret 注入工作流文档化并且 CI 不允许 secrets 进入 manifest。

11. Testing / CI / Golden Tests
- 要求：
  - 每个 manifest PR 触发验证：JSON Schema 校验 + payload snapshot/golden tests + optional mock request smoke test。
  - 提供工具自动生成 payload snapshot（preview-payload）。
- 验收：
  - 所有 manifest 变更必须通过 CI。

12. Manifest Registry & Governance（基础）
- 要求：
  - 有清晰的 registry policy：owner、reviewers、schema version gate、PR 流程。
  - 支持发布版本（manifest semantic versioning）与审计记录。
  - 可选：托管 registry 服务（PRO）。
- 验收：
  - registry 中的 manifest 有 owner 字段，CI 检查 owner 是否存在。

13. Documentation & DX
- 要求：
  - Manifest 规范文档、示例 manifests、如何新增 provider/model、迁移指南、常见坑和 edge cases。
  - 示例 SDK（Rust 首发，后续 Python/TS）。
  - 教程 notebook、end-to-end demo（训练→部署→调用）。
- 验收：
  - 文档覆盖主要用例且提供可运行示例。

14. SDKs 与语言支持
- 要求：
  - 首期实现 Rust SDK（manifest 驱动 runtime、PayloadBuilder trait、AiClient）。
  - 后续提供 Python、TypeScript SDK（与 Rust runtime 兼容或独立实现 manifest 解析）。
  - 提供 HTTP-forwarding 简单客户端（对接前端/other services）。
- 验收：
  - Rust SDK 具备完整功能且发布至 crates.io（alpha），包含示例。

15. Performance 与 Codegen（进阶）
- 要求：
  - 动态 mapping 需考虑高 QPS 性能：提供 codegen（把 manifest 编译为高效序列化代码）或预编译模板路径。
  - 支持缓存 precompiled payload templates（减少 runtime 序列化）。
- 验收：
  - 通过基准测试证明 codegen/预编译在高并发场景比 naive runtime mapping 有明显优势。

16. Migration 与 兼容策略
- 要求：
  - 提供迁移工具（从旧 provider config -> manifest）。
  - 在新项目启动期提供兼容 shim（旧 API 到 manifest 引擎）。
- 验收：
  - 现有常用 provider 配置可自动转为 manifest（脚本或工具）。

非功能性需求（NFR）
----
- 可用性（Availability）：目标 SLO 99.9%（取决部署与 PRO SLA）。
- 可扩展性：支持并行高并发请求（通过 HTTP client 池、streaming 优化）。
- 安全性：manifest 不含 secret；提供审计与 ACL（PRO）。
- 延迟：尽量保持与原生 provider SDK 相当，针对高 QPS 提供 codegen。
- 可维护性：manifest-first 使新增 provider 成本降到最小，CI 强制校验保证质量。
- 国际化/本地化：文档和错误信息支持英文优先，后续考虑多语言。

漏洞补全与常见边界场景（必须在新项目中解决）
----
- 系统默认值不一致：不同 provider 对默认 stop、max_tokens 等行为不一致，manifest 必须允许明确声明 provider-specific default，以及 SDK 应在 mapping 时保留旧默认以保证兼容。
- streaming 事件边界：SSE/event chunking 的边界语义不同，需设计统一的 streaming event model（包括 partial delta、final、tool_call_started/ended）。
- 授权与多租户场景：registry 或托管服务需支持多租户隔离、审计和 RBAC（PRO）。
- 模态能力检测：manifest 中 capabilities 字段必须用于 preflight 校验（禁止把不支持的 multimodal 输入发给模型）。
- JSON Schema 强化：对 tools 的 input_schema 与 output_schema 应支持 OpenAPI/JSON Schema 子集，并能用于运行时的输入校验。
- Error semantics drift：厂商错误码与 HTTP 状态码不完全对应，必须把 error mapping 作为 manifest 的可选字段。

实施建议（高层 Roadmap 与里程碑）
----
建议采取新仓（ai-lib-manifest）+ 渐进交付策略：

阶段 0 — 需求确认与 Schema 草案（1–2 周）
- 产物：manifest v1 草案 + JSON Schema 草案 + 项目 README

阶段 1 — PoC（2–4 周）
- 产物：manifest loader、validator、CLI（validate-preview）、Rust StandardRequest 与简单 PayloadBuilder（openai）、AiClient prototype（mocked HTTP）

阶段 2 — 多 provider 支持 + 测试（4–8 周）
- 产物：google/anthropic mapping、streaming 支持、golden tests、CI 校验集成

阶段 3 — SDK 完整化 + Docs（4–6 周）
- 产物：Rust SDK 发布 alpha、示例 notebooks、迁移工具脚本、示例 manifests

阶段 4 — Registry 与治理（6–12 周，可并行）
- 产物：manifest registry（repo 或 service）、PR 流程、owner 审核、基础 governance

阶段 5 — 企业特性与 codegen（可选，长期）
- 产物：UI/Registry 服务、codegen module、PRO 功能（审计、ACL、SLA）

资源估算（粗略）
----
- PoC（阶段 0–1）：1–2 人月（1–2 eng）
- 到生产就绪（阶段 0–3）：5–10 人月（2–3 eng 并行）
- Registry + PRO（阶段 4–5）：额外 4–8 人月（含产品/PM/安全/运维）

验收标准（Acceptance Criteria）
----
- manifest v1 Schema 定稿并在 CI 中生效；
- Rust SDK（alpha）能以 manifest 为输入对至少 3 个厂商（OpenAI, Google, Anthropic）生成正确 payload 并解析 mock responses；
- CI 中包含 manifest 校验与 payload snapshot 检查；
- 提供迁移示例与兼容 shim 以确保现有用户迁移成本可控；
- 文档覆盖如何新增 manifest、如何 debug mapping、常见差异说明。

风险与缓解（Summary）
----
- 风险：manifest 格式误用导致生产问题。缓解：CI 强制校验 + sandbox 测试 + 不允许 secrets 入 manifest。
- 风险：性能回退。缓解：引入 codegen、预编译模板、缓存。
- 风险：社区接受度/迁移阻力。缓解：保持旧 API 兼容、提供迁移工具与清晰文档。
- 风险：维护双轨成本。缓解：定义明确迁移期、弃用窗口与自动化迁移工具。

下一步（建议的立即行动）
----
1. 确认目标与范围：团队、时间容忍度（是否并行保留旧库）与商业目标（OSS-first vs PRO-优先）。
2. 立项并创建新仓 ai-lib-manifest，提交 manifest v1 草案与 JSON Schema。
3. 启动 PoC：实现 manifest loader、openai PayloadBuilder、Rust StandardRequest、AiClient prototype（mock）与 CLI validate-preview。
4. 在 PoC 基础上评估 performance 与兼容性，决定是否并行拆分 registry 服务与 codegen 模块。
5. 我可以立即为你生成：
   - ai-lib-manifest.v1.yaml 样例（包含 OpenAI/Gemini/Anthropic）与对应 JSON Schema；
   - Rust 草稿 types（ModelConfig, InferenceParams, PayloadBuilder trait, 简单 mapping 引擎示例）；
   - PoC Roadmap（按周细化的 12 周计划与人员分配建议）。

你想先要哪一项交付物？（我建议先给出 manifest YAML + JSON Schema 与 Rust types 草稿，便于直接启动 PoC。）