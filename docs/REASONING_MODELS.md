# 推理模型支持指南

## 概述

ai-lib 通过现有的 API 能力完美支持推理模型，无需额外的接口抽象。本指南展示了如何与 Groq 等厂商的推理模型进行交互的最佳实践。

## 支持的推理模型

### Groq 推理模型
- **qwen-qwq-32b**: Qwen 推理模型，支持结构化推理
- **deepseek-r1-distill-llama-70b**: DeepSeek R1 推理模型
- **openai/gpt-oss-20b**: OpenAI OSS 推理模型
- **openai/gpt-oss-120b**: OpenAI OSS 大型推理模型

## 推理模式

### 1. 结构化推理
使用函数调用进行步骤化推理，适合需要结构化输出的场景。

```rust
use ai_lib::{AiClient, Provider, ChatCompletionRequest, Message, Role, Content};
use ai_lib::types::function_call::{Tool, FunctionCallPolicy};
use serde_json::json;

let client = AiClient::new(Provider::Groq)?;

let reasoning_tool = Tool {
    name: "step_by_step_reasoning".to_string(),
    description: Some("执行步骤化推理解决复杂问题".to_string()),
    parameters: Some(json!({
        "type": "object",
        "properties": {
            "problem": {"type": "string", "description": "要解决的问题"},
            "steps": {
                "type": "array",
                "items": {
                    "type": "object",
                    "properties": {
                        "step_number": {"type": "integer"},
                        "description": {"type": "string"},
                        "reasoning": {"type": "string"},
                        "conclusion": {"type": "string"}
                    }
                }
            },
            "final_answer": {"type": "string"}
        }
    })),
};

let request = ChatCompletionRequest::new(
    "qwen-qwq-32b".to_string(),
    vec![Message {
        role: Role::User,
        content: Content::Text("解决这个数学问题：2x + 3 = 11".to_string()),
        function_call: None,
    }],
)
.with_functions(vec![reasoning_tool])
.with_function_call(FunctionCallPolicy::Auto("auto".to_string()));

let response = client.chat_completion(request).await?;
```

### 2. 流式推理
观察推理过程的实时输出，适合需要展示推理步骤的场景。

```rust
let request = ChatCompletionRequest::new(
    "qwen-qwq-32b".to_string(),
    vec![Message {
        role: Role::User,
        content: Content::Text("解释光合作用的工作原理".to_string()),
        function_call: None,
    }],
);

let mut stream = client.chat_completion_stream(request).await?;

while let Some(chunk) = stream.next().await {
    match chunk {
        Ok(chunk) => {
            if let Some(choice) = chunk.choices.first() {
                if let Some(content) = &choice.delta.content {
                    print!("{}", content);
                    std::io::stdout().flush().unwrap();
                }
            }
        }
        Err(e) => {
            println!("流式错误: {}", e);
            break;
        }
    }
}
```

### 3. JSON 格式推理
获取结构化的推理结果，适合需要程序化处理的场景。

```rust
let request = ChatCompletionRequest::new(
    "qwen-qwq-32b".to_string(),
    vec![Message {
        role: Role::System,
        content: Content::Text("你是一个推理助手。请始终以有效的JSON格式回复，包含你的推理过程和最终答案。".to_string()),
        function_call: None,
    }, Message {
        role: Role::User,
        content: Content::Text("法国的首都是什么？为什么它很重要？".to_string()),
        function_call: None,
    }],
);

let response = client.chat_completion(request).await?;

// 解析 JSON 推理结果
for choice in response.choices {
    let content = choice.message.content.as_text();
    if let Ok(reasoning_json) = serde_json::from_str::<serde_json::Value>(&content) {
        println!("结构化推理结果: {}", serde_json::to_string_pretty(&reasoning_json)?);
    }
}
```

### 4. 推理配置
使用逃生通道传递厂商特定的推理参数。

```rust
let mut request = ChatCompletionRequest::new(
    "qwen-qwq-32b".to_string(),
    vec![Message {
        role: Role::User,
        content: Content::Text("解决这个复杂问题：x² + 5x + 6 = 0".to_string()),
        function_call: None,
    }],
);

// 添加 Groq 特定的推理参数
request = request
    .with_provider_specific("reasoning_format", serde_json::Value::String("parsed".to_string()))
    .with_provider_specific("reasoning_effort", serde_json::Value::String("high".to_string()))
    .with_provider_specific("parallel_tool_calls", serde_json::Value::Bool(true))
    .with_provider_specific("service_tier", serde_json::Value::String("flex".to_string()));

let response = client.chat_completion(request).await?;
```

## 推理工具库

ai-lib 提供了 `ReasoningUtils` 工具库，简化推理模型的使用：

```rust
use examples::reasoning_utils::*;

// 创建推理助手
let client = AiClient::new(Provider::Groq)?;
let assistant = ReasoningAssistant::new(client, "qwen-qwq-32b".to_string());

// 数学推理
let math_result = assistant.solve_math_problem("计算 2^10 + 3^5 的值").await?;

// 逻辑推理
let logic_result = assistant.solve_logic_problem("如果所有的鸟都会飞，企鹅是鸟，那么企鹅会飞吗？").await?;

// 科学推理
let science_result = assistant.solve_science_problem("解释为什么天空是蓝色的").await?;

// 带配置的推理
let config = ReasoningConfig {
    format: ReasoningFormat::Structured,
    effort: ReasoningEffort::High,
    parallel_tool_calls: Some(true),
    service_tier: Some(ServiceTier::Flex),
};
let config_result = assistant.solve_with_config("复杂问题", config).await?;
```

## 推理参数说明

### Groq 推理参数

| 参数 | 类型 | 说明 | 可选值 |
|------|------|------|--------|
| `reasoning_format` | string | 推理格式 | `parsed`, `raw`, `hidden` |
| `reasoning_effort` | string | 推理努力级别 | `low`, `medium`, `high`, `none`, `default` |
| `parallel_tool_calls` | boolean | 并行工具调用 | `true`, `false` |
| `service_tier` | string | 服务层级 | `on_demand`, `flex`, `auto` |

### 推理格式说明

- **parsed**: 解析后的推理过程，适合人类阅读
- **raw**: 原始推理过程，包含内部思考
- **hidden**: 隐藏推理过程，只显示最终结果

### 推理努力级别

- **low**: 低努力级别，快速推理
- **medium**: 中等努力级别，平衡速度和深度
- **high**: 高努力级别，深度推理
- **none**: 不进行推理
- **default**: 默认努力级别

## 最佳实践

### 1. 选择合适的推理模式

- **结构化推理**: 需要程序化处理结果时
- **流式推理**: 需要实时展示推理过程时
- **JSON 格式**: 需要结构化数据时
- **配置推理**: 需要精细控制推理行为时

### 2. 优化推理性能

```rust
// 使用高努力级别获得更好的推理质量
let config = ReasoningConfig {
    format: ReasoningFormat::Structured,
    effort: ReasoningEffort::High,
    parallel_tool_calls: Some(true),
    service_tier: Some(ServiceTier::Flex),
};
```

### 3. 错误处理

```rust
match client.chat_completion(request).await {
    Ok(response) => {
        // 处理成功响应
        for choice in response.choices {
            println!("推理结果: {}", choice.message.content.as_text());
        }
    }
    Err(e) => {
        match e {
            ai_lib::AiLibError::UnsupportedFeature(msg) => {
                println!("模型不支持推理功能: {}", msg);
            }
            ai_lib::AiLibError::ProviderError(msg) => {
                println!("推理请求失败: {}", msg);
            }
            _ => {
                println!("其他错误: {}", e);
            }
        }
    }
}
```

### 4. 推理结果验证

```rust
use examples::reasoning_utils::ReasoningUtils;

let result = ReasoningUtils::parse_reasoning_result(&content);
match result {
    Ok(reasoning_result) => {
        let validation = ReasoningUtils::validate_reasoning_result(&reasoning_result);
        match validation {
            ValidationResult::Valid => println!("推理结果有效"),
            ValidationResult::Invalid(msg) => println!("推理结果无效: {}", msg),
        }
    }
    Err(e) => println!("解析推理结果失败: {}", e),
}
```

## 示例代码

完整的示例代码请参考：

- `examples/reasoning_best_practices.rs` - 推理模型最佳实践示例
- `examples/reasoning_utils.rs` - 推理工具库

运行示例：

```bash
# 设置环境变量
export GROQ_API_KEY=your_api_key_here

# 运行推理示例
cargo run --example reasoning_best_practices

# 运行推理工具库示例
cargo run --example reasoning_utils
```

## 总结

ai-lib 通过现有的 API 能力完美支持推理模型，无需额外的接口抽象。开发者可以根据需求选择合适的推理模式，通过函数调用、流式处理、JSON 格式等方式与推理模型进行交互。推理工具库提供了便捷的辅助函数，简化了推理模型的使用。

这种设计既保持了 ai-lib 的简洁性，又为开发者提供了强大的推理能力支持。
