# AI-Lib Manifest验证方式设计

## 概述

基于对讨论内容的深入分析，AI-Lib采用**Code-First（代码优先）**的验证方式，将Rust struct定义作为唯一的真理来源，通过Serde进行结构验证，Validator trait进行逻辑验证，并通过schemars自动生成JSON Schema提供编辑器支持。

## 核心设计原则

### 1. Code-First 验证方式

```rust
// Rust struct是唯一的真理来源
#[derive(Debug, Clone, Deserialize, Serialize, Validate)]
pub struct Manifest {
    pub version: String,
    pub standard_schema: StandardSchema,
    pub providers: HashMap<String, ProviderDefinition>,
    pub models: HashMap<String, ModelDefinition>,
}
```

**优势**:
- **类型安全**: 编译时保证结构正确性
- **性能**: 零成本抽象的反序列化
- **维护性**: 代码即文档，无需同步多个Schema文件
- **开发体验**: IDE提供完美的自动补全

### 2. Tri-Brid 验证架构

```
Rust Struct (真理来源)
    ↓ Serde (结构验证)
YAML/JSON解析
    ↓ Validator (逻辑验证)
业务规则检查
    ↓ Schemars (Schema生成)
JSON Schema (编辑器支持)
```

### 3. 验证时机

- **编译时**: 类型检查，防止结构错误
- **运行时**: 结构验证 + 逻辑验证的双重保障
- **开发时**: JSON Schema提供编辑器智能提示

## 实现细节

### 结构验证 (Serde)

```rust
pub fn load_manifest(content: &str) -> ManifestResult<Manifest> {
    // 1. 结构验证 - Serde保证YAML结构匹配Rust类型
    let manifest: Manifest = serde_yaml::from_str(content)?;

    // 2. 逻辑验证 - Validator检查业务规则
    manifest.validate()?;

    // 3. 额外验证 - Manifest特定规则
    ManifestValidator::validate_manifest(&manifest)?;

    Ok(manifest)
}
```

### 逻辑验证 (Validator)

```rust
#[derive(Validate)]
pub struct ModelDefinition {
    pub provider: String,
    #[validate(length(min = 1))]
    pub model_id: String,
    #[validate(range(min = 1, max = 1000000))]
    pub context_window: Option<u32>,
}
```

### Schema生成 (Schemars)

```rust
// 导出JSON Schema用于编辑器支持
pub fn export_json_schema() -> String {
    let schema = schemars::schema_for!(Manifest);
    serde_json::to_string_pretty(&schema).unwrap()
}
```

## CLI工具支持

### 验证命令
```bash
# 验证manifest文件
cargo run --bin manifest_cli -- validate --file aimenifest.yaml --verbose

# 输出结果:
# ✅ Manifest验证成功！
# 📊 版本: 1.1
# 🏢 提供商数量: 4
# 🤖 模型数量: 4
```

### Schema导出
```bash
# 导出JSON Schema到stdout
cargo run --bin manifest_cli -- export-schema

# 导出到文件
cargo run --bin manifest_cli -- export-schema --output schema.json
```

### 编辑器集成

在YAML文件顶部添加：
```yaml
# $schema: ./schema.json
version: "1.1"
# 现在VS Code会提供完整的自动补全和验证！
```

## 与传统JSON Schema对比

| 特性 | 传统JSON Schema | AI-Lib Code-First |
| --- | --- | --- |
| 定义位置 | 单独的.schema.json文件 | Rust代码中 |
| 维护成本 | 需要同步代码和Schema | 代码即Schema |
| 类型安全 | 弱，运行时才发现错误 | 强，编译时保证 |
| 开发体验 | 有限的编辑器支持 | 完整的IDE支持 |
| 性能 | 额外解析开销 | 零成本抽象 |
| 扩展性 | 需要手动维护 | 自动生成 |

## 错误处理

### 分层错误信息

1. **结构错误**: Serde提供精确的字段路径和错误原因
2. **逻辑错误**: Validator提供业务规则验证信息
3. **业务错误**: ManifestValidator提供领域特定验证

### 错误示例

```rust
// 结构错误: 字段类型不匹配
Error: missing field `standard_schema` at line 5 column 1

// 逻辑错误: 违反业务规则
Error: validation error: context_window must be between 1 and 1000000

// 业务错误: 配置不一致
Error: Model 'gpt-4' does not belong to provider 'anthropic'
```

## 最佳实践

### 1. 结构体设计

- 使用枚举处理多态类型（ProviderEnum）
- 利用serde tag处理复杂嵌套结构
- 添加合适的validator约束

### 2. 错误信息

- 提供清晰的错误消息
- 包含修复建议
- 支持详细模式输出

### 3. 向后兼容

- 版本化schema
- 渐进式验证规则
- 迁移工具支持

## Phase 3 规划

### 当前状态
- ✅ 基础Code-First验证架构
- ✅ Serde + Validator双重验证
- ✅ 基础JSON Schema导出
- ❌ 完整的自动Schema生成

### Phase 3 目标
- [ ] 完善所有结构体的JsonSchema derive
- [ ] 实现完整的自动Schema生成
- [ ] 支持多版本Schema
- [ ] 增强错误信息和修复建议
- [ ] 集成更多编辑器支持
- [ ] 添加验证规则测试套件

## 总结

AI-Lib的Code-First验证方式完美体现了现代Rust开发的最佳实践：

1. **类型安全优先**: Rust的类型系统提供编译时保证
2. **性能优化**: 零成本抽象的反序列化
3. **开发体验**: 完整的IDE支持和自动补全
4. **维护效率**: 代码即文档，无需额外Schema文件

这种设计不仅提高了代码质量和开发效率，还为AI-Lib提供了强大的配置验证能力，支持复杂的企业级应用场景。
