# 推理模型示例说明

本目录包含ai-lib推理模型支持的示例代码和工具库。

## 文件说明

- `reasoning_best_practices.rs` - 推理模型最佳实践示例
- `reasoning_utils.rs` - 推理工具库和助手类

## 运行示例

### 1. 设置环境变量

```bash
export GROQ_API_KEY=your_groq_api_key_here
```

### 2. 运行最佳实践示例

```bash
cargo run --example reasoning_best_practices
```

这个示例展示了：
- 结构化推理（使用函数调用）
- 流式推理（观察推理过程）
- JSON格式推理（获取结构化结果）
- 推理配置（使用厂商特定参数）
- 数学问题推理
- 逻辑推理

### 3. 运行推理工具库示例

```bash
cargo run --example reasoning_utils
```

这个示例展示了：
- 数学推理助手
- 逻辑推理助手
- 科学推理助手
- 推理结果解析和验证

## 支持的推理模型

- **qwen-qwq-32b**: Qwen推理模型
- **deepseek-r1-distill-llama-70b**: DeepSeek R1推理模型
- **openai/gpt-oss-20b**: OpenAI OSS推理模型
- **openai/gpt-oss-120b**: OpenAI OSS大型推理模型

## 推理模式

1. **结构化推理**: 使用函数调用进行步骤化推理
2. **流式推理**: 观察推理过程的实时输出
3. **JSON格式推理**: 获取结构化的推理结果
4. **配置推理**: 使用逃生通道传递厂商特定参数

## 更多信息

详细的推理模型支持指南请参考：`docs/REASONING_MODELS.md`
