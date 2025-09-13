# Using ai-lib with OpenAI-compatible Gateways

> INTERNAL DRAFT — NOT FOR PUBLICATION

This guide shows how to route ai-lib traffic through an OpenAI-compatible gateway (e.g., Helicone AI Gateway) while keeping your Rust code unchanged at the application layer.

> Scope: client-side configuration only. No changes to ai-lib modules are required.

## When to use a gateway

- Centralized API key management and rotation
- Organization-wide rate limits, budgets, and usage analytics
- Policy-driven routing (latency/cost/health/region) across providers
- Team/multi-tenant boundaries and access controls

Gateways add an extra network hop. For minimum latency and air-gapped use cases, prefer direct provider access.

## Quick start

1) Configure ai-lib to point to your gateway base URL

```rust
use ai_lib::{AiClient, Provider, ConnectionOptions};
use std::time::Duration;

let client = AiClient::with_options(
    Provider::OpenAI,
    ConnectionOptions {
        base_url: Some("http://localhost:8080/ai".into()), // or router-specific path
        proxy: None,
        api_key: Some("placeholder-api-key".into()), // gateway typically verifies/attaches real keys
        timeout: Some(Duration::from_secs(30)),
        disable_proxy: false,
    },
)?;
```

2) Use provider-prefixed model identifiers when required by the gateway

```rust
use ai_lib::{ChatCompletionRequest, Message, Role, Content};

let req = ChatCompletionRequest::new(
    // examples: "openai/gpt-4o-mini", "anthropic/claude-3-5-sonnet"
    "openai/gpt-4o-mini".to_string(),
    vec![Message {
        role: Role::User,
        content: Content::new_text("Hello from ai-lib via gateway!"),
        function_call: None,
    }],
);
```

3) Optional tenant/team hints

If your gateway supports tenant scoping, attach identifiers as headers (or query parameters) via a custom transport or interceptor. Suggested headers:

- `X-AI-Tenant`: logical tenant or organization id
- `X-AI-Team`: team or project id
- `X-AI-Request-Id`: end-to-end correlation id

These can be injected using ai-lib's transport abstraction or interceptor pipeline.

## Best practices

- Keep a single source of truth for the gateway base URL (env var) and inject through `ConnectionOptions`.
- Use correlation IDs end-to-end for troubleshooting.
- Be explicit about timeouts and implement retries with backoff.
- Treat gateway-side budgets/quotas as authoritative; surface errors clearly to callers.

## Security notes

- Gateways often remove the need to ship provider secrets to clients. Ensure that only a placeholder key is used in application configuration if the gateway enforces auth.
- Avoid logging request bodies and secrets. Prefer structured logs with correlation IDs.

## Interoperability

ai-lib works with both direct provider endpoints and OpenAI-compatible gateways. The same code can switch between the two by changing `ConnectionOptions.base_url` and, if required, model identifiers.

References:
- Helicone AI Gateway (OpenAI-compatible) `https://github.com/Helicone/ai-gateway`
