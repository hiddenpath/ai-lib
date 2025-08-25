# ai-lib Copilot 编码代理说明
# Copilot Coding Agent Instructions for ai-lib

## Project Overview
- **ai-lib** is a unified AI SDK for Rust, providing a single interface for multiple AI model providers (Groq, DeepSeek, Anthropic, Gemini, OpenAI).
- The architecture is hybrid: some providers use configuration-driven adapters, others use custom adapters for API compatibility.
- All provider logic is abstracted behind the `ChatApi` trait and accessed via the `AiClient`.

## Key Components
- `src/client.rs`: Main entry point, implements `AiClient` and provider selection logic.
- `src/provider/`: Contains provider adapters and configuration logic. Use `GenericAdapter` for most providers; custom adapters for OpenAI/Gemini.
- `src/types/`: Unified data structures (`ChatCompletionRequest`, `ChatCompletionResponse`, `Message`, `Role`, etc.).
- `src/api/`: API traits and streaming logic.
- `src/transport/`: HTTP transport abstraction, proxy support, error handling.

## Patterns & Conventions
- **Provider Switching**: Use `AiClient::new(Provider::X)` to switch backend with unified API.
- **Adapters**: For Groq, DeepSeek, Anthropic use `GenericAdapter::new(ProviderConfigs::X())`. For OpenAI/Gemini use their respective adapters.
- **Error Handling**: All errors use the `AiLibError` enum in `src/types/error.rs`.
- **Streaming**: All providers support streaming via `chat_completion_stream`.
- **Proxy Support**: Set `AI_PROXY_URL` env variable for HTTP/HTTPS proxy (with/without auth).

## Developer Workflow
- **Build**: `cargo build` or `cargo check`.
- **Test**: Example-based testing in `examples/` (run with `cargo run --example <name>`). No standard Rust tests in `tests/` yet.
- **Debug**: Use example files for provider/network debugging.
- **Publish**: `cargo publish` for crates.io; force push for GitHub if needed.

## Integration Points
- No external SDK dependencies for providers; all HTTP APIs are called directly.
- Proxy and API keys are configured via environment variables.

## Example Usage
```rust
let client = AiClient::new(Provider::Groq)?;
let request = ChatCompletionRequest::new("llama3-8b-8192".to_string(), vec![Message { role: Role::User, content: "Hello".to_string() }]);
let response = client.chat_completion(request).await?;
```

## References
- See `README.md` and `README_CN.md` for more usage and architecture details.
- Key files: `src/client.rs`, `src/provider/`, `src/types/`, `src/api/`, `src/transport/`, `examples/`

---
If any section is unclear or missing, please provide feedback for further refinement.
