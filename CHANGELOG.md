# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.1] - 2024-12-19

### Added
- **Client Builder Pattern**: New `AiClientBuilder` for flexible client configuration with progressive customization levels
- **Automatic Environment Variable Detection**: Auto-detects `GROQ_BASE_URL`, `AI_PROXY_URL` and other provider-specific environment variables
- **Model Management Tools**: Comprehensive tools for building custom model managers and arrays
  - `CustomModelManager` with multiple selection strategies (Round-robin, Weighted, Performance-based, Cost-based)
  - `ModelArray` for load balancing and A/B testing with health checks
  - `ModelInfo`, `ModelCapabilities`, `PricingInfo`, and `PerformanceMetrics` structures
- **Enhanced Provider Configuration**: Each provider now specifies default chat and optional multimodal models
- **Progressive Customization**: Four levels of client configuration from simple to advanced

### Enhanced
- **ProviderConfig**: Added `chat_model` and `multimodal_model` fields for better model management
- **ProviderConfigs**: Updated all provider configurations with sensible default models
- **Documentation**: Comprehensive English documentation and examples for all new features
- **Examples**: Added new examples demonstrating builder pattern and model management tools

### Changed
- **Backward Compatibility**: All existing code continues to work without modification
- **Method Signatures**: Updated `ProviderConfig::openai_compatible()` to include model parameters
- **Configuration Validation**: Enhanced validation to include new model fields

### Examples
- **Quickstart**: `cargo run --example quickstart` - Simple usage guide
- **Builder Pattern**: `cargo run --example builder_pattern` - Complete builder pattern demonstration
- **Model Management**: `cargo run --example model_management` - Custom model managers and arrays

### Key Benefits
- **Developer Experience**: Faster time-to-first-AI-app with sensible defaults
- **Flexibility**: Easy customization for advanced use cases
- **Production Ready**: Load balancing, health checks, and sophisticated model selection
- **Extensible**: Build custom model managers for any provider or use case

## [0.2.0] - 2024-12-18

### Added
- Hybrid architecture with universal streaming support
- Enterprise-grade error handling, retry, and proxy support
- Multimodal primitives, function-calling, and metrics scaffold
- Transport injection and upload tests
- Support for multiple AI providers including Groq, OpenAI, Anthropic, and more

### Changed
- Improved error handling and retry mechanisms
- Enhanced streaming capabilities
- Better proxy support and configuration

## [0.1.0] - 2024-12-17

### Added
- Initial release of ai-lib
- Basic AI provider support
- Core chat completion functionality
- Streaming support
- Basic error handling

---

## Versioning

- **Major version (0.x.0)**: Breaking changes or major new features
- **Minor version (0.2.x)**: New features, enhancements, or improvements
- **Patch version (0.2.x)**: Bug fixes and minor improvements

## Migration Guide

### From 0.2.0 to 0.2.1

No breaking changes. All existing code continues to work:

```rust
// This still works exactly the same
let client = AiClient::new(Provider::Groq)?;

// New optional features available
let client = AiClientBuilder::new(Provider::Groq)
    .with_base_url("https://custom.groq.com")
    .with_proxy("http://proxy.example.com:8080")
    .build()?;
```

### New Features Usage

```rust
// Custom model manager
let mut manager = CustomModelManager::new("groq")
    .with_strategy(ModelSelectionStrategy::PerformanceBased);

// Model array for load balancing
let mut array = ModelArray::new("production")
    .with_strategy(LoadBalancingStrategy::RoundRobin);
```

## Contributing

Please read [CONTRIBUTING.md](CONTRIBUTING.md) for details on our code of conduct and the process for submitting pull requests.

## License

This project is licensed under the MIT License - see the [LICENSE-MIT](LICENSE-MIT) file for details.
