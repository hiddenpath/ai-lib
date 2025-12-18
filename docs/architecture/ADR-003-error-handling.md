# ADR-003: Error Handling Mechanism

## Status
**Accepted** - 2024-12

## Context
ai-lib interacts with 20+ external APIs, each with different error formats. We need:
- Unified error representation
- Retryable vs permanent error classification
- Rich error context for debugging
- Type-safe error handling

## Decision
Implement **structured error handling** with classification:

### Error Type Hierarchy
```rust
pub enum AiLibError {
    // Transient errors (retryable)
    RateLimitExceeded(String),
    Timeout(String),
    NetworkError(String),
    
    // Permanent errors (not retryable)
    AuthenticationError(String),
    InvalidRequest(String),
    ModelNotFound(String),
    UnsupportedFeature(String),
    
    // Provider-specific
    ProviderError(String),
    
    // Configuration
    ConfigurationError(String),
}
```

### Error Classification
```rust
impl AiLibError {
    pub fn is_retryable(&self) -> bool {
        matches!(self,
            AiLibError::RateLimitExceeded(_)
            | AiLibError::Timeout(_)
            | AiLibError::NetworkError(_)
        )
    }
}
```

### Error Context
Using `thiserror` for automatic trait implementations:
```rust
#[derive(Error, Debug)]
pub enum AiLibError {
    #[error("Rate limit exceeded: {0}")]
    RateLimitExceeded(String),
    
    #[error("Authentication failed: {0}")]
    AuthenticationError(String),
}
```

## Consequences

### Positive
- **Clear Classification**: Easy to determine retry logic
- **Type Safety**: Compiler enforces error handling
- **Rich Context**: Detailed error messages for debugging
- **Consistent API**: Same error types across all providers

### Negative
- **Abstraction Loss**: Provider-specific details may be lost
- **Mapping Overhead**: Must translate provider errors

### Error Handling Patterns
```rust
// Automatic retry for transient errors
match client.chat_completion(req).await {
    Ok(resp) => Ok(resp),
    Err(e) if e.is_retryable() => retry_with_backoff(),
    Err(e) => Err(e),
}

// Pattern matching for specific handling
match client.chat_completion(req).await {
    Err(AiLibError::RateLimitExceeded(_)) => wait_and_retry(),
    Err(AiLibError::AuthenticationError(_)) => refresh_token(),
    Err(e) => Err(e),
}
```

## Implementation Details

### HTTP Status Code Mapping
- 429 → `RateLimitExceeded`
- 401, 403 → `AuthenticationError`
- 400 → `InvalidRequest`
- 404 → `ModelNotFound`
- 500-599 → `ProviderError` (retryable)
- Timeout → `Timeout`
- Connection errors → `NetworkError`

### Provider Error Translation
Each adapter translates provider-specific errors:
```rust
// OpenAI error response
{
  "error": {
    "message": "Rate limit exceeded",
    "type": "rate_limit_error",
    "code": "rate_limit_exceeded"
  }
}

// Translated to
AiLibError::RateLimitExceeded("Rate limit exceeded".to_string())
```

## Future Enhancements

### Error Source Chain (Planned)
```rust
pub enum AiLibError {
    ProviderError {
        message: String,
        source: Option<Box<dyn Error + Send + Sync>>,
    },
}
```

### Structured Error Details (Planned)
```rust
pub struct ErrorContext {
    pub provider: String,
    pub model: String,
    pub request_id: Option<String>,
    pub retry_after: Option<Duration>,
}
```

## Alternatives Considered
1. **String Errors**: Lost type information
2. **anyhow**: Too generic, no classification
3. **Multiple Error Types**: Fragmented error handling

## Best Practices

### For Library Users
```rust
// Handle specific errors
match result {
    Err(AiLibError::RateLimitExceeded(_)) => {
        // Wait and retry
    }
    Err(e) if e.is_retryable() => {
        // Generic retry logic
    }
    Err(e) => {
        // Log and fail
    }
    Ok(resp) => // Process response
}
```

### For Contributors
- Always classify errors correctly
- Include actionable error messages
- Preserve provider error codes when possible
- Test error paths thoroughly

## References
- `src/types/error.rs` - Error type definitions
- `src/error_handling/` - Error recovery strategies
- `src/interceptors/retry.rs` - Retry logic implementation
