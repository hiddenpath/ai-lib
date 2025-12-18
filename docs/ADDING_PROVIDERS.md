# Adding a New Provider to ai-lib

## Quick Start

Adding a new provider requires changes to only 2-3 files:

1. **Define Provider** (`src/client/provider.rs`)
2. **Configure Provider** (`src/provider/configs.rs`)
3. **Register in Factory** (`src/client/provider_factory.rs`)

## Step-by-Step Guide

### Step 1: Add Provider Enum Variant

**File**: `src/client/provider.rs`

Add your provider to the `Provider` enum:

```rust
pub enum Provider {
    // ... existing providers
    YourProvider,  // Add here
}
```

Add default model:

```rust
impl Provider {
    pub fn default_chat_model(&self) -> &'static str {
        match self {
            // ... existing
            Provider::YourProvider => "your-default-model",
        }
    }
}
```

### Step 2: Create Provider Configuration

**File**: `src/provider/configs.rs`

```rust
impl ProviderConfigs {
    pub fn your_provider() -> ProviderConfig {
        ProviderConfig {
            api_base: "https://api.yourprovider.com/v1".to_string(),
            api_key_env: "YOUR_PROVIDER_API_KEY".to_string(),
            default_model: "your-default-model".to_string(),
            requires_auth: true,
            // ... other config options
        }
    }
}
```

### Step 3: Register in Provider Factory

**File**: `src/client/provider_factory.rs`

```rust
impl ProviderFactory {
    pub fn create_adapter(...) -> Result<Box<dyn ChatProvider>, AiLibError> {
        match provider {
            // ... existing
            Provider::YourProvider => create_generic(
                ProviderConfigs::your_provider(),
                api_key, base_url, transport
            ),
        }
    }
}
```

### Step 4: Test Your Provider

```rust
#[tokio::test]
async fn test_your_provider() {
    let client = AiClient::new(Provider::YourProvider).unwrap();
    // ... test code
}
```

That's it! 🎉

## Injecting Custom Providers (No Enum Changes)

For OpenAI-compatible backends (self-hosted gateways, vendor previews, etc.) you can now build a provider at runtime and inject it through `AiClientBuilder::with_strategy`:

```rust
use ai_lib::{
    client::{AiClientBuilder, Provider},
    provider::builders::CustomProviderBuilder,
    types::AiLibError,
};

fn connect_custom() -> Result<(), AiLibError> {
    let custom = CustomProviderBuilder::new("labs-staging")
        .with_base_url("https://labs.example.com/v1")
        .with_api_key_env("LABS_STAGING_TOKEN")
        .with_default_chat_model("labs-gpt-35")
        .build_provider()?;

    let client = AiClientBuilder::new(Provider::OpenAI)
        .with_strategy(custom)
        .build()?;

    // client.chat_completion(...).await?;
    Ok(())
}
```

### Rolling Your Own Adapter

If you already have a bespoke `ChatProvider` implementation (or the legacy `ChatApi` alias),
wrap it with `AdapterProvider` so it satisfies the unified trait:

```rust
use ai_lib::{
    client::{AiClientBuilder, Provider},
    provider::chat_provider::AdapterProvider,
    types::AiLibError,
};

fn connect_custom_adapter() -> Result<(), AiLibError> {
    let my_adapter = Box::new(MyInternalAdapter::new()?); // implements `ChatProvider`
    let provider = AdapterProvider::new("internal-sandbox".to_string(), my_adapter).boxed();

    let client = AiClientBuilder::new(Provider::OpenAI)
        .with_strategy(provider)
        .build()?;

    Ok(())
}
```

This keeps the `Provider` enum as a factory helper while allowing end users to plug in any strategy that implements the unified `ChatProvider` trait.

### Strategy Injection Checklist

1. Use `CustomProviderBuilder` for OpenAI-compatible gateways or wrap a bespoke adapter via `AdapterProvider::new`.
2. Pass the boxed `ChatProvider` to `AiClientBuilder::with_strategy`.
3. Reuse every `AiClient` API (chat, stream, batch, helpers) without modifying the `Provider` enum.

See `examples/custom_provider_injection.rs` for a ready-to-run sample that demonstrates the full workflow.

### Provider Builders & Request Extensions

- Prefer per-provider builders (`GroqBuilder`, `OpenAiBuilder`, etc.) when you want a typed façade around `AiClientBuilder`. They expose the same methods but pre-fill provider metadata:

```rust
use ai_lib::provider::GroqBuilder;

let groq = GroqBuilder::new()
    .with_base_url("https://api.groq.com")
    .build_provider()?; // -> Box<dyn ChatProvider>
```

- Use `ChatCompletionRequest::with_extension(key, value)` to pass vendor-specific parameters. This replaces the legacy `with_provider_specific()` helper and serializes custom JSON directly into the request body.