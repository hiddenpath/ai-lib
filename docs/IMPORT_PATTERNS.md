# Application Import Patterns

This guide explains the recommended import patterns for ai-lib applications and libraries, helping you choose the most appropriate approach for your use case.

## Quick Reference

| Use Case | Recommended Import | Example |
|----------|-------------------|---------|
| **Application Development** | `use ai_lib::prelude::*;` | Get minimal common set |
| **Explicit Control** | `use ai_lib::{AiClient, Provider, ...};` | Top-level re-exports |
| **Library Development** | Domain-specific imports | `use ai_lib::types::response::Usage;` |
| **Multimodal Content** | `use ai_lib::{Content, Message, Role};` | Content creation methods |

## Import Strategies

### 1. Prelude (Recommended for Applications)

For most application code, the `prelude` offers the most convenient way to import commonly used types and traits:

```rust
use ai_lib::prelude::*;

#[tokio::main]
async fn main() -> Result<(), AiLibError> {
    let client = AiClient::new(Provider::OpenAI)?;
    
    let request = ChatCompletionRequest::new(
        "gpt-4".to_string(),
        vec![Message {
            role: Role::User,
            content: Content::from_text("Hello, world!"),
            function_call: None,
        }],
    );
    
    let response = client.chat_completion(request).await?;
    println!("Response: {}", response.choices[0].message.content.as_text());
    Ok(())
}
```

**Included in prelude:**
- `AiClient`, `AiClientBuilder`, `Provider`
- `ChatCompletionRequest`, `ChatCompletionResponse`, `Choice`
- `Content`, `Message`, `Role`
- `Usage`, `UsageStatus`
- `AiLibError`

### 2. Top-level Re-exports (Explicit Control)

When you want explicit control over imports but still avoid deep module paths:

```rust
use ai_lib::{AiClient, AiClientBuilder, Provider};
use ai_lib::{ChatCompletionRequest, ChatCompletionResponse};
use ai_lib::{Content, Message, Role};
use ai_lib::{Usage, UsageStatus, AiLibError};

#[tokio::main]
async fn main() -> Result<(), AiLibError> {
    let client = AiClient::new(Provider::Groq)?;
    // ... rest of your code
    Ok(())
}
```

### 3. Domain-specific Imports (Library Development)

For library authors or when you need fine-grained control:

```rust
use ai_lib::types::request::ChatCompletionRequest;
use ai_lib::types::response::{ChatCompletionResponse, Usage, UsageStatus};
use ai_lib::types::common::{Content, Message, Role};
use ai_lib::types::error::AiLibError;
use ai_lib::client::{AiClient, Provider};
```

## Multimodal Content Creation

### Image Content

```rust
use ai_lib::prelude::*;

// From file path (automatic processing)
let image_content = Content::from_image_file("path/to/image.png");

// From URL
let image_content = Content::new_image(
    Some("https://example.com/image.png".to_string()),
    Some("image/png".to_string()),
    Some("image.png".to_string()),
);

// From data URL
let image_content = Content::from_data_url(
    "data:image/png;base64,iVBORw0KGgo...".to_string(),
    Some("image/png".to_string()),
    Some("image.png".to_string()),
);
```

### Audio Content

```rust
use ai_lib::prelude::*;

// From file path (automatic processing)
let audio_content = Content::from_audio_file("path/to/audio.mp3");

// From URL
let audio_content = Content::new_audio(
    Some("https://example.com/audio.mp3".to_string()),
    Some("audio/mpeg".to_string()),
);
```

### Mixed Content Messages

```rust
use ai_lib::prelude::*;

let messages = vec![
    Message {
        role: Role::User,
        content: Content::from_text("Please analyze this image"),
        function_call: None,
    },
    Message {
        role: Role::User,
        content: Content::from_image_file("path/to/image.png"),
        function_call: None,
    },
];
```

## Provider Selection

### Basic Provider Selection

```rust
use ai_lib::prelude::*;

// Single provider
let client = AiClient::new(Provider::OpenAI)?;

// With failover
let client = AiClient::new(Provider::OpenAI)?
    .with_failover(vec![Provider::Anthropic, Provider::Groq]);
```

### Available Providers

```rust
// OpenAI-compatible providers
Provider::OpenAI
Provider::AzureOpenAI
Provider::OpenRouter
Provider::Replicate
Provider::ZhipuAI
Provider::MiniMax

// Independent providers
Provider::Anthropic
Provider::Groq
Provider::Gemini
Provider::Mistral
Provider::Cohere
Provider::Perplexity
Provider::AI21
Provider::DeepSeek
Provider::Qwen
Provider::Ollama
```

## Error Handling

```rust
use ai_lib::prelude::*;

#[tokio::main]
async fn main() -> Result<(), AiLibError> {
    let client = AiClient::new(Provider::OpenAI)?;
    
    match client.chat_completion(request).await {
        Ok(response) => {
            println!("Success: {}", response.choices[0].message.content.as_text());
        }
        Err(AiLibError::NetworkError(msg)) => {
            eprintln!("Network error: {}", msg);
        }
        Err(AiLibError::ProviderError(msg)) => {
            eprintln!("Provider error: {}", msg);
        }
        Err(e) => {
            eprintln!("Other error: {}", e);
        }
    }
    
    Ok(())
}
```

## Streaming

```rust
use ai_lib::prelude::*;
use futures::stream::StreamExt;

#[tokio::main]
async fn main() -> Result<(), AiLibError> {
    let client = AiClient::new(Provider::OpenAI)?;
    let request = ChatCompletionRequest::new(
        "gpt-4".to_string(),
        vec![Message {
            role: Role::User,
            content: Content::from_text("Tell me a story"),
            function_call: None,
        }],
    );
    
    let mut stream = client.chat_completion_stream(request).await?;
    
    while let Some(chunk) = stream.next().await {
        match chunk {
            Ok(chunk) => {
                if let Some(delta) = chunk.choices.get(0)
                    .and_then(|c| c.delta.as_ref())
                    .and_then(|d| d.content.as_ref()) {
                    print!("{}", delta);
                }
            }
            Err(e) => eprintln!("Stream error: {}", e),
        }
    }
    
    Ok(())
}
```

## Best Practices

### Do's
- ✅ Use `ai_lib::prelude::*` for application development
- ✅ Use explicit top-level imports when you need control
- ✅ Use `Content::from_image_file()` and `Content::from_audio_file()` for multimodal content
- ✅ Use `Provider` enum for provider selection
- ✅ Handle errors appropriately with `AiLibError`

### Don'ts
- ❌ Don't combine wildcard imports across domains
- ❌ Don't directly import from `ai_lib::provider::utils` (use `Content` methods instead)
- ❌ Don't import concrete adapter types directly (use `Provider` enum)
- ❌ Don't ignore error handling

## Migration from Previous Versions

### Usage and UsageStatus
```rust
// Old (deprecated)
use ai_lib::types::common::{Usage, UsageStatus};

// New (recommended)
use ai_lib::{Usage, UsageStatus};
// or
use ai_lib::types::response::{Usage, UsageStatus};
```

### Provider Utils
```rust
// Old (no longer available)
use ai_lib::provider::utils::upload_file_with_transport;

// New (use Content methods)
let content = Content::from_image_file("path/to/image.png");
```

## IDE Support

With the new import structure, your IDE's auto-completion will guide you to the correct types:
- Typing `ai_lib::` will show top-level exports
- Typing `ai_lib::prelude::` will show common items
- Search for types at the crate root first

## Further Reading

- [Module Tree and Import Patterns](MODULE_TREE_AND_IMPORTS.md) - Detailed module structure guide
- [API Reference](https://docs.rs/ai-lib/0.3.4) - Complete API documentation
- [Examples](../examples/) - Practical usage examples

