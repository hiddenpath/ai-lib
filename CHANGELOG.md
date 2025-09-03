# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [TBD] - Provider Classification System

### Added
- **System-Level Provider Classification**: New `ProviderClassification` trait for unified provider behavior management
- **Provider Classification Constants**: System-level constants (`CONFIG_DRIVEN_PROVIDERS`, `INDEPENDENT_PROVIDERS`, `ALL_PROVIDERS`) for single source of truth
- **Unified Configuration Management**: Centralized provider configuration mapping through trait methods
- **Provider Classification Module**: New `src/provider/classification.rs` module with comprehensive provider behavior definitions

### Enhanced
- **Provider Enum**: Added `PartialEq` trait to `Provider` enum for classification support
- **Code Organization**: Eliminated duplicate provider classification logic across multiple modules
- **Type Safety**: Compile-time provider classification with automatic adapter type detection

### Changed
- **Provider Classification Logic**: Replaced hardcoded `matches!` statements with trait-based classification
- **Configuration Retrieval**: Simplified provider configuration access through unified trait methods
- **Adapter Creation**: Streamlined adapter creation logic using provider classification

### Developer Experience
- **Simplified Provider Management**: Single location for adding new providers and their classifications
- **Reduced Code Duplication**: Eliminated ~50 lines of repetitive provider classification code
- **Better Maintainability**: Centralized provider behavior definitions reduce maintenance overhead

### How to Add New Providers

#### For Config-Driven Providers (using GenericAdapter):
1. Add provider to `Provider` enum in `src/client.rs`
2. Add provider to `CONFIG_DRIVEN_PROVIDERS` array in `src/provider/classification.rs`
3. Add configuration method to `ProviderConfigs` in `src/provider/configs.rs`
4. Add configuration mapping in `ProviderClassification::get_default_config()` implementation

#### For Independent Providers (using dedicated adapters):
1. Add provider to `Provider` enum in `src/client.rs`
2. Add provider to `INDEPENDENT_PROVIDERS` array in `src/provider/classification.rs`
3. Create dedicated adapter implementation
4. Add adapter creation logic in `AiClientBuilder::build()` method

#### Example - Adding a New Config-Driven Provider:
```rust
// 1. Add to Provider enum
pub enum Provider {
    // ... existing providers
    NewProvider,  // Add here
}

// 2. Add to CONFIG_DRIVEN_PROVIDERS array
pub const CONFIG_DRIVEN_PROVIDERS: &[Provider] = &[
    // ... existing providers
    Provider::NewProvider,  // Add here
];

// 3. Add configuration method
impl ProviderConfigs {
    pub fn new_provider() -> ProviderConfig {
        ProviderConfig::openai_compatible(
            "https://api.newprovider.com/v1",
            "NEW_PROVIDER_API_KEY",
            "new-model",
            None,
        )
    }
}

// 4. Add to trait implementation
fn get_default_config(&self) -> Result<ProviderConfig, AiLibError> {
    match self {
        // ... existing mappings
        Provider::NewProvider => Ok(ProviderConfigs::new_provider()),
        // ...
    }
}
```

### Benefits
- **Single Source of Truth**: All provider classifications defined in one place
- **Type Safety**: Compile-time validation of provider behavior
- **Extensibility**: Easy addition of new providers with automatic classification
- **Maintainability**: Reduced code duplication and centralized management

## [0.2.12] 

### Added
- **System Configuration Management**: Comprehensive configuration system with environment variable support and explicit overrides
- **Configuration Validation Tools**: Built-in tools for validating configuration settings
- **Enhanced Documentation**: Complete rewrite of README with feature-focused organization
- **Context Control**: Advanced conversation management with context control features
- **File Upload & Multimodal Processing**: Automatic file handling with upload and inline support
- **Performance Optimizations**: Enterprise-grade performance with minimal overhead
- **Security & Privacy Features**: Built-in security features for enterprise environments

### Enhanced
- **README Documentation**: Restructured and enhanced both English and Chinese README files
- **Provider Support**: Updated provider information with detailed feature descriptions
- **Configuration Examples**: Added comprehensive configuration examples and best practices
- **Performance Documentation**: Added performance characteristics and optimization tips
- **Use Case Examples**: Added enterprise, research, production, and privacy-first use cases

### Changed
- **Documentation Structure**: Reorganized documentation to highlight key features and capabilities
- **Configuration Management**: Enhanced configuration management with validation tools
- **Performance Metrics**: Updated performance documentation with specific benchmarks

### Examples
- **Configuration Check**: `cargo run --example check_config` - Validate configuration settings
- **Network Diagnosis**: `cargo run --example network_diagnosis` - Troubleshoot connectivity
- **Proxy Testing**: `cargo run --example proxy_example` - Test proxy configuration
- **Explicit Config**: `cargo run --example explicit_config` - Runtime configuration

## [0.2.1] 

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

## [0.2.0] 

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
