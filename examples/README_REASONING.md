# Reasoning Models Examples

This directory contains example code and utility libraries for ai-lib's reasoning model support.

## Files

- `reasoning_best_practices.rs` - Reasoning models best practices example
- `reasoning_utils.rs` - Reasoning utility library and helper classes

## Running Examples

### 1. Set Environment Variables

```bash
export GROQ_API_KEY=your_groq_api_key_here
```

### 2. Run Best Practices Example

```bash
cargo run --example reasoning_best_practices
```

This example demonstrates:
- Structured reasoning (using function calls)
- Streaming reasoning (observing reasoning process)
- JSON format reasoning (getting structured results)
- Reasoning configuration (using provider-specific parameters)
- Mathematical problem reasoning
- Logical reasoning

### 3. Run Reasoning Utils Example

```bash
cargo run --example reasoning_utils
```

This example demonstrates:
- Mathematical reasoning assistant
- Logical reasoning assistant
- Scientific reasoning assistant
- Reasoning result parsing and validation

## Supported Reasoning Models

- **qwen-qwq-32b**: Qwen reasoning model
- **deepseek-r1-distill-llama-70b**: DeepSeek R1 reasoning model
- **openai/gpt-oss-20b**: OpenAI OSS reasoning model
- **openai/gpt-oss-120b**: OpenAI OSS large reasoning model

## Reasoning Modes

1. **Structured Reasoning**: Step-by-step reasoning using function calls
2. **Streaming Reasoning**: Real-time output observation of reasoning process
3. **JSON Format Reasoning**: Getting structured reasoning results
4. **Configuration Reasoning**: Using escape hatches to pass provider-specific parameters

## More Information

For detailed reasoning model support guide, please refer to: `docs/REASONING_MODELS.md`
