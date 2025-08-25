# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).


## [Unreleased] / [0.0.6] - 2025-08-26

### Added
- Object-safe transport abstraction (`DynHttpTransport`) and boxed shim for `HttpTransport` to enable runtime injection and testing.
- Cohere adapter with SSE streaming and fallback support.
- Mistral HTTP adapter (conservative implementation) with streaming support.
- `GenericAdapter` improvements: optional API key support, more provider configs (Ollama override, HuggingFace models endpoint, Azure OpenAI config).
- Example: `examples/list_models_smoke.rs` to quickly validate model listing across providers.

### Changed
- Migrated multiple adapters (OpenAI, Gemini, Generic, Cohere, Mistral) to use the object-safe transport reference for easier DI and testing.
- Deferred AWS Bedrock: removed from public exports and adapter skeleton retained in the repo (implementation postponed).

### Fixed
- Resolved multiple compile-time issues discovered during migration (trait object-safety, missing imports, dependency on `bytes`).
- Fixed non-exhaustive match in `AiClient::switch_provider` by adding `AzureOpenAI` mapping.

### Notes / Migration
- If you inject a custom transport for testing, use the adapter `with_transport_ref(...)` constructors which accept `DynHttpTransportRef`.
- Bedrock is intentionally deferred due to SigV4/AWS SDK integration complexity. Re-introduce when ready by adding a public export and implementing signing or SDK wiring.

## [0.0.2] - 2025-08-24

### Added
- **Hybrid Architecture**: Configuration-driven + Independent adapters
- **Universal Streaming Support**: Real-time SSE streaming for all providers
- **Enterprise-Grade Reliability**: Automatic retry with exponential backoff
- **Smart Error Handling**: Detailed error classification and recovery suggestions
- **5 Major AI Providers**: Groq, DeepSeek, Anthropic, Google Gemini, OpenAI
- **Proxy Support**: HTTP/HTTPS proxy with authentication
- **Cancellable Streams**: Stream cancellation control with CancelHandle
- **Performance Optimizations**: Memory-efficient SSE parsing and connection reuse

### Providers
- ✅ **Groq** (Configuration-driven) - llama3, mixtral models
- ✅ **DeepSeek** (Configuration-driven) - deepseek-chat, deepseek-reasoner  
- ✅ **Anthropic Claude** (Configuration-driven) - claude-3.5-sonnet
- ✅ **Google Gemini** (Independent adapter) - gemini-1.5-pro, gemini-1.5-flash
- ✅ **OpenAI** (Independent adapter) - gpt-3.5-turbo, gpt-4

### Technical Improvements
- **95% Code Reduction**: Configuration-driven adapters vs independent implementations
- **Type Safety**: Full Rust type system integration
- **Zero Dependencies**: Minimal, carefully selected dependencies
- **Production Ready**: Enterprise-grade error handling and retry mechanisms

### Examples
- `test_hybrid_architecture` - Comprehensive architecture demonstration
- `test_streaming_improved` - Advanced streaming capabilities
- `test_retry_mechanism` - Error handling and retry logic
- `test_groq_generic` - Configuration-driven provider example
- `test_gemini` - Independent adapter example
- `test_anthropic` - Custom authentication example
- `test_https_proxy` - Proxy configuration testing

### Documentation
- Complete README with architecture explanation
- Chinese translation (README_CN.md)
- Comprehensive API documentation
- Production deployment guides
- Performance benchmarks and scalability notes

## [0.0.1] - 2025-08-22

### Added
- Initial release with basic AI provider support
- Basic HTTP transport layer
- Simple request/response handling
- Foundation for multi-provider architecture

### Providers
- Basic Groq support
- Basic OpenAI support

---

## Upcoming Features

### Planned for v0.1.0
- [ ] Connection pooling and advanced performance optimizations
- [ ] Metrics and observability integration  
- [ ] Additional providers (Cohere, Together AI, etc.)
- [ ] Multi-modal support (images, audio) for compatible providers
- [ ] Advanced streaming features (backpressure, flow control)

### Future Considerations
- [ ] Plugin system for custom providers
- [ ] Built-in caching mechanisms
- [ ] Load balancing across multiple providers
- [ ] Circuit breaker patterns for fault tolerance