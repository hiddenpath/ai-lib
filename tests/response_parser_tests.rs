//! Tests for generic response parsing (no business-specific structures)

#[cfg(feature = "response_parser")]
use ai_lib::response_parser::{
    extract_code_blocks, extract_json_from_text, AutoParser, JsonResponseParser,
    MarkdownSectionParser, ParsedResponse, ResponseParser,
};

#[cfg(all(test, feature = "response_parser"))]
mod tests {
    #[cfg(feature = "response_parser")]
    use super::*;

    #[test]
    fn test_markdown_section_extraction() {
        let parser = MarkdownSectionParser::new();

        let text = r#"# Title

Intro

## Section A
Line A1
Line A2

## Section B
Line B1
Line B2
"#;

        let sections = parser.extract_sections(text);
        assert_eq!(sections.len(), 2);
        assert!(sections["Section A"].contains("Line A1"));
        assert!(sections["Section B"].contains("Line B1"));
    }

    #[test]
    fn test_extract_code_blocks() {
        let text = r#"Here is code:

```rust
fn main() {}
```

```json
{"a":1}
```
"#;
        let blocks = extract_code_blocks(text);
        assert_eq!(blocks.len(), 2);
        assert_eq!(blocks[0].0, "rust");
        assert!(blocks[0].1.contains("fn main"));
        assert_eq!(blocks[1].0, "json");
        assert!(blocks[1].1.contains("\"a\""));
    }

    #[test]
    fn test_extract_json_from_text() {
        let text = r#"Some text
```json
{"answer":42}
```
More text"#;
        let json = extract_json_from_text(text).unwrap();
        assert!(json.contains("\"answer\""));
    }

    #[test]
    fn test_json_response_parser() {
        #[derive(serde::Deserialize)]
        struct TestStruct {
            name: String,
            value: i32,
        }

        let parser = JsonResponseParser::<TestStruct>::new();
        let json_str = r#"{"name":"foo","value":7}"#;
        let parsed = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(async { parser.parse(json_str).await });
        assert!(parsed.is_ok());
        let parsed = parsed.unwrap();
        assert_eq!(parsed.name, "foo");
        assert_eq!(parsed.value, 7);
    }

    #[test]
    fn test_auto_parser_json_first() {
        let parser = AutoParser::default();
        let input = r#"{"x":1,"y":2}"#;
        let parsed = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(async { parser.parse(input).await });
        match parsed.unwrap() {
            ParsedResponse::Json(v) => assert_eq!(v["x"], 1),
            _ => panic!("expected Json"),
        }
    }

    #[test]
    fn test_auto_parser_sections() {
        let parser = AutoParser::default();
        let input = r#"## Reasoning
Step-by-step

## Answer
42
"#;
        let parsed = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(async { parser.parse(input).await });
        match parsed.unwrap() {
            ParsedResponse::Sections(map) => {
                assert!(map.contains_key("Reasoning"));
                assert!(map.contains_key("Answer"));
            }
            _ => panic!("expected Sections"),
        }
    }
}
