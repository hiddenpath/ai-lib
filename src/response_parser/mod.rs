//! 结构化输出解析（通用版）
//!
//! 提供与业务无关的通用解析能力，用于从大模型响应中提取可结构化处理的数据：
//! - JSON / 代码块中的 JSON
//! - Markdown `##` 分节
//! - 代码块收集
//! - 兜底纯文本
//!
//! 不内置特定场景结构，开发者可在应用侧基于这些原子能力二次映射。

use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::collections::HashMap;

/// 核心解析 trait
#[async_trait::async_trait]
pub trait ResponseParser {
    type Output;
    type Error: std::fmt::Display + Send + Sync;

    async fn parse(&self, response: &str) -> Result<Self::Output, Self::Error>;
    fn can_parse(&self, response: &str) -> bool;
}

/// Markdown 分节解析（基于 `## ` 标题）
pub struct MarkdownSectionParser;

impl MarkdownSectionParser {
    pub fn new() -> Self {
        Self
    }

    pub fn extract_sections(&self, text: &str) -> HashMap<String, String> {
        let mut sections = HashMap::new();
        let lines: Vec<&str> = text.lines().collect();

        let mut current_section = String::new();
        let mut current_content = String::new();
        let mut in_section = false;

        for line in lines {
            if line.starts_with("## ") {
                if in_section && !current_section.is_empty() {
                    sections.insert(current_section.clone(), current_content.trim().to_string());
                }
                current_section = line.trim_start_matches("## ").to_string();
                current_content = String::new();
                in_section = true;
            } else if in_section {
                current_content.push_str(line);
                current_content.push('\n');
            }
        }

        if in_section && !current_section.is_empty() {
            sections.insert(current_section, current_content.trim().to_string());
        }

        sections
    }
}

#[async_trait::async_trait]
impl ResponseParser for MarkdownSectionParser {
    type Output = HashMap<String, String>;
    type Error = ParseError;

    async fn parse(&self, response: &str) -> Result<Self::Output, Self::Error> {
        let sections = self.extract_sections(response);
        Ok(sections)
    }

    fn can_parse(&self, response: &str) -> bool {
        response.contains("## ")
    }
}

/// JSON 解析器（支持裸 JSON 与代码块 JSON）
pub struct JsonResponseParser<T> {
    _phantom: std::marker::PhantomData<T>,
}

impl<T> JsonResponseParser<T> {
    pub fn new() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }
}

#[async_trait::async_trait]
impl<T> ResponseParser for JsonResponseParser<T>
where
    T: for<'de> Deserialize<'de> + Send + Sync,
{
    type Output = T;
    type Error = ParseError;

    async fn parse(&self, response: &str) -> Result<Self::Output, Self::Error> {
        if let Ok(result) = serde_json::from_str::<T>(response) {
            return Ok(result);
        }

        if let Some(json_str) = extract_json_from_text(response) {
            if let Ok(result) = serde_json::from_str::<T>(&json_str) {
                return Ok(result);
            }
        }

        Err(ParseError::InvalidJson(
            "Could not parse JSON from response".to_string(),
        ))
    }

    fn can_parse(&self, response: &str) -> bool {
        serde_json::from_str::<JsonValue>(response).is_ok()
            || response.contains("```json")
            || (response.contains('{') && response.contains('}'))
    }
}

/// 自动解析器：按通用顺序返回结构化结果
pub struct AutoParser {
    pub json_first: bool,
}

impl Default for AutoParser {
    fn default() -> Self {
        Self { json_first: true }
    }
}

#[async_trait::async_trait]
impl ResponseParser for AutoParser {
    type Output = ParsedResponse;
    type Error = ParseError;

    async fn parse(&self, response: &str) -> Result<Self::Output, Self::Error> {
        // 1) JSON 优先
        if self.json_first {
            if let Ok(json) = serde_json::from_str::<JsonValue>(response) {
                return Ok(ParsedResponse::Json(json));
            }
            if let Some(json_str) = extract_json_from_text(response) {
                if let Ok(json) = serde_json::from_str::<JsonValue>(&json_str) {
                    return Ok(ParsedResponse::Json(json));
                }
            }
        }

        // 2) Markdown 分节
        let section_parser = MarkdownSectionParser::new();
        let sections = section_parser.extract_sections(response);
        if !sections.is_empty() {
            return Ok(ParsedResponse::Sections(sections));
        }

        // 3) JSON Lines (if strictly multiple lines of JSON)
        let json_lines_parser = JsonLinesResponseParser::<JsonValue>::new();
        if json_lines_parser.can_parse(response) {
            // Re-use logic to avoid double parse cost (or just parse)
            if let Ok(lines) = json_lines_parser.parse(response).await {
                // Heuristic: Logic check - is it really lists of JSON?
                if lines.len() > 1 || (lines.len() == 1 && !response.trim().starts_with('[')) {
                    return Ok(ParsedResponse::JsonLines(lines));
                }
            }
        }

        // 4) Code Blocks
        let blocks = extract_code_blocks(response);
        if !blocks.is_empty() {
            return Ok(ParsedResponse::CodeBlocks(blocks));
        }

        // 4) 兜底文本
        Ok(ParsedResponse::Text(response.to_string()))
    }

    fn can_parse(&self, _response: &str) -> bool {
        true
    }
}

/// 解析错误
#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("Invalid JSON: {0}")]
    InvalidJson(String),

    #[error("Invalid format: {0}")]
    InvalidFormat(String),
}

/// 自动解析结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ParsedResponse {
    Json(JsonValue),
    JsonLines(Vec<JsonValue>),
    Sections(HashMap<String, String>),
    CodeBlocks(Vec<(String, String)>),
    Text(String),
}

/// JSON Lines 解析器
pub struct JsonLinesResponseParser<T> {
    _phantom: std::marker::PhantomData<T>,
}

impl<T> JsonLinesResponseParser<T> {
    pub fn new() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }
}

#[async_trait::async_trait]
impl<T> ResponseParser for JsonLinesResponseParser<T>
where
    T: for<'de> Deserialize<'de> + Send + Sync,
{
    type Output = Vec<T>;
    type Error = ParseError;

    async fn parse(&self, response: &str) -> Result<Self::Output, Self::Error> {
        let mut results = Vec::new();
        for line in response.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            // Skip code block markers if present loosely
            if line.starts_with("```") {
                continue;
            }

            if let Ok(val) = serde_json::from_str::<T>(line) {
                results.push(val);
            }
        }

        if results.is_empty() {
            return Err(ParseError::InvalidFormat(
                "No valid JSON lines found".to_string(),
            ));
        }
        Ok(results)
    }

    fn can_parse(&self, response: &str) -> bool {
        let lines: Vec<&str> = response
            .lines()
            .map(|l| l.trim())
            .filter(|l| !l.is_empty() && !l.starts_with("```"))
            .collect();
        if lines.is_empty() {
            return false;
        }

        // At least one line must be valid JSON object
        lines
            .iter()
            .any(|l| serde_json::from_str::<JsonValue>(l).is_ok())
    }
}

/// 从文本中提取 JSON（代码块或裸 JSON）
pub fn extract_json_from_text(text: &str) -> Option<String> {
    // ```json ... ```
    if let Some(start) = text.find("```json") {
        if let Some(end) = text[start + 7..].find("```") {
            let json_content = &text[start + 7..start + 7 + end];
            return Some(json_content.trim().to_string());
        }
    }

    // 任意代码块
    if let Some(start) = text.find("```") {
        if let Some(end) = text[start + 3..].find("```") {
            let content = &text[start + 3..start + 3 + end];
            if content.trim().starts_with('{') || content.trim().starts_with('[') {
                return Some(content.trim().to_string());
            }
        }
    }

    None
}

/// 提取代码块 (lang, code)
pub fn extract_code_blocks(text: &str) -> Vec<(String, String)> {
    let mut blocks = Vec::new();
    let mut lines = text.lines().peekable();

    while let Some(line) = lines.next() {
        if line.starts_with("```") {
            let lang = line.trim_start_matches("```").trim().to_string();
            let mut code = String::new();

            while let Some(code_line) = lines.next() {
                if code_line.starts_with("```") {
                    break;
                }
                code.push_str(code_line);
                code.push('\n');
            }

            blocks.push((lang, code.trim().to_string()));
        }
    }

    blocks
}
