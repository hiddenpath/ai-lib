# 响应结构化解析指南（通用版）

本文介绍 ai-lib 提供的通用结构化解析能力（feature `response_parser`），用于从大模型输出中提取结构化信息。

## 能力概览
- **JSON 提取**：解析裸 JSON 或代码块中的 JSON。
- **Markdown 分节**：按 `## ` 标题拆分为段落。
- **代码块收集**：提取代码块 (lang, code)。
- **兜底文本**：找不到结构化信息时返回原文本。

## 核心类型
- `AutoParser`：自动按“JSON → 分节 → 代码块 → 文本”顺序解析，输出 `ParsedResponse`。
- `MarkdownSectionParser`：提取 `HashMap<section, content>`。
- `JsonResponseParser<T>`：解析裸 JSON 或代码块 JSON 到 `T`（支持 `serde::Deserialize`）。
- `ParsedResponse`：
  - `Json(serde_json::Value)`
  - `Sections(HashMap<String, String>)`
  - `CodeBlocks(Vec<(String, String)>)`
  - `Text(String)`

## 快速示例
```rust
use ai_lib::response_parser::{AutoParser, ParsedResponse, ResponseParser};

let parser = AutoParser::default();
let result = parser.parse(response_text).await?;
match result {
    ParsedResponse::Json(v) => { /* 处理 JSON */ }
    ParsedResponse::Sections(map) => { /* 处理分节 */ }
    ParsedResponse::CodeBlocks(blocks) => { /* 处理代码块 */ }
    ParsedResponse::Text(raw) => { /* 兜底文本 */ }
}
```

解析 JSON 代码块：
```rust
use ai_lib::response_parser::{JsonResponseParser, extract_json_from_text};

#[derive(serde::Deserialize)]
struct MySchema { answer: String }

let parser = JsonResponseParser::<MySchema>::new();
if let Some(json_str) = extract_json_from_text(response_text) {
    if let Ok(parsed) = parser.parse(&json_str).await {
        // 使用 parsed
    }
}
```

## 使用建议
- **提示词约束**：在 Prompt 中要求模型使用 `##` 标题或返回 JSON 代码块，提升解析成功率。
- **回退策略**：优先 JSON，再分节；无法解析时保留原文交给上层处理。
- **可插拔**：不内置业务结构，开发者可在应用侧基于分节/JSON映射到自定义类型。

## 启用方式
在依赖的项目 `Cargo.toml` 中启用 feature：
```toml
ai-lib = { version = "...", features = ["response_parser"] }
```

## 示例
参见 `examples/response_parsing.rs`，演示：
- 自动解析
- 提取代码块 JSON 并反序列化
- 分节信息探查

