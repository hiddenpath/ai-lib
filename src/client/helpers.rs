//! Client helper methods.
//!
//! This module provides convenience methods for common operations:
//! - Quick one-shot requests
//! - Request building shortcuts
//! - Provider switching
//! - File uploads

use super::metadata::metadata_from_provider;
use super::{AiClient, ModelOptions, Provider, ProviderFactory};
use crate::types::{AiLibError, ChatCompletionRequest};

pub async fn list_models(client: &AiClient) -> Result<Vec<String>, AiLibError> {
    client.chat_provider.list_models().await
}

/// Switch the provider while preserving client configuration state.
///
/// This function rebuilds the chat provider adapter for the new provider
/// while retaining the existing client's:
/// - metrics implementation
/// - connection options (base_url, proxy, timeout, api_key)
/// - backpressure controller
/// - interceptor pipeline (if feature enabled)
/// - custom default models
///
/// Note: The connection options (proxy, timeout, etc.) from the original client
/// will be applied to the new provider. If the new provider requires different
/// configuration, consider creating a new client instead.
pub fn switch_provider(client: &mut AiClient, provider: Provider) -> Result<(), AiLibError> {
    // Determine base_url and transport based on existing connection options
    let (base_url, transport) = if let Some(opts) = &client.connection_options {
        let resolved_base_url = super::builder::resolve_base_url(provider, opts.base_url.clone())?;
        let transport = if opts.proxy.is_some() || opts.timeout.is_some() {
            let transport_config = crate::transport::HttpTransportConfig {
                timeout: opts.timeout.unwrap_or(std::time::Duration::from_secs(30)),
                proxy: opts.proxy.clone(),
                pool_max_idle_per_host: None,
                pool_idle_timeout: None,
            };
            Some(crate::transport::HttpTransport::new_with_config(transport_config)?.boxed())
        } else {
            None
        };
        (Some(resolved_base_url), transport)
    } else {
        let resolved_base_url = super::builder::resolve_base_url(provider, None)?;
        (Some(resolved_base_url), None)
    };

    // Get API key from connection options if available
    let api_key = client
        .connection_options
        .as_ref()
        .and_then(|opts| opts.api_key.clone());

    // Create new provider adapter
    let chat_provider =
        ProviderFactory::create_adapter(provider, api_key, base_url.clone(), transport)?;

    // Update metadata for new provider
    let metadata = metadata_from_provider(
        provider,
        chat_provider.name().to_string(),
        base_url,
        client.custom_default_chat_model.clone(),
        client.custom_default_multimodal_model.clone(),
    );

    // Update only the provider and metadata, preserving all other state
    client.chat_provider = chat_provider;
    client.metadata = metadata;

    Ok(())
}

pub fn build_simple_request<S: Into<String>>(
    client: &AiClient,
    prompt: S,
) -> ChatCompletionRequest {
    let model = client
        .custom_default_chat_model
        .clone()
        .or_else(|| client.metadata.default_chat_model().map(|s| s.to_string()))
        .expect("AiClient metadata missing default chat model");

    ChatCompletionRequest::new(
        model,
        vec![crate::types::Message {
            role: crate::types::Role::User,
            content: crate::types::common::Content::Text(prompt.into()),
            function_call: None,
        }],
    )
}

pub fn build_simple_request_with_model<S: Into<String>>(
    _client: &AiClient,
    prompt: S,
    model: S,
) -> ChatCompletionRequest {
    ChatCompletionRequest::new(
        model.into(),
        vec![crate::types::Message {
            role: crate::types::Role::User,
            content: crate::types::common::Content::Text(prompt.into()),
            function_call: None,
        }],
    )
}

pub fn build_multimodal_request<S: Into<String>>(
    client: &AiClient,
    prompt: S,
) -> Result<ChatCompletionRequest, AiLibError> {
    let model = client
        .custom_default_multimodal_model
        .clone()
        .or_else(|| {
            client
                .metadata
                .default_multimodal_model()
                .map(|s| s.to_string())
        })
        .ok_or_else(|| {
            AiLibError::ConfigurationError(format!(
                "No multimodal model available for provider {}",
                client.provider_name()
            ))
        })?;

    Ok(ChatCompletionRequest::new(
        model,
        vec![crate::types::Message {
            role: crate::types::Role::User,
            content: crate::types::common::Content::Text(prompt.into()),
            function_call: None,
        }],
    ))
}

pub fn build_multimodal_request_with_model<S: Into<String>>(
    _client: &AiClient,
    prompt: S,
    model: S,
) -> ChatCompletionRequest {
    ChatCompletionRequest::new(
        model.into(),
        vec![crate::types::Message {
            role: crate::types::Role::User,
            content: crate::types::common::Content::Text(prompt.into()),
            function_call: None,
        }],
    )
}

pub async fn quick_chat_text<P: Into<String>>(
    provider: Provider,
    prompt: P,
) -> Result<String, AiLibError> {
    let client = AiClient::new(provider)?;
    let req = client.build_simple_request(prompt.into());
    let resp = client.chat_completion(req).await?;
    resp.first_text().map(|s| s.to_string())
}

pub async fn quick_chat_text_with_model<P: Into<String>, M: Into<String>>(
    provider: Provider,
    prompt: P,
    model: M,
) -> Result<String, AiLibError> {
    let client = AiClient::new(provider)?;
    let req = client.build_simple_request_with_model(prompt.into(), model.into());
    let resp = client.chat_completion(req).await?;
    resp.first_text().map(|s| s.to_string())
}

pub async fn quick_multimodal_text<P: Into<String>>(
    provider: Provider,
    prompt: P,
) -> Result<String, AiLibError> {
    let client = AiClient::new(provider)?;
    let req = client.build_multimodal_request(prompt.into())?;
    let resp = client.chat_completion(req).await?;
    resp.first_text().map(|s| s.to_string())
}

pub async fn quick_multimodal_text_with_model<P: Into<String>, M: Into<String>>(
    provider: Provider,
    prompt: P,
    model: M,
) -> Result<String, AiLibError> {
    let client = AiClient::new(provider)?;
    let req = client.build_multimodal_request_with_model(prompt.into(), model.into());
    let resp = client.chat_completion(req).await?;
    resp.first_text().map(|s| s.to_string())
}

pub async fn quick_chat_text_with_options<P: Into<String>>(
    provider: Provider,
    prompt: P,
    options: ModelOptions,
) -> Result<String, AiLibError> {
    let client = AiClient::new(provider)?;

    // Determine which model to use based on options
    let model = if let Some(chat_model) = options.chat_model {
        chat_model
    } else {
        provider.default_chat_model().to_string()
    };

    let req = client.build_simple_request_with_model(prompt.into(), model);
    let resp = client.chat_completion(req).await?;
    resp.first_text().map(|s| s.to_string())
}

pub async fn upload_file(client: &AiClient, path: &str) -> Result<String, AiLibError> {
    // Determine base_url precedence: explicit connection_options > metadata default
    let base_url = if let Some(opts) = &client.connection_options {
        if let Some(b) = &opts.base_url {
            Some(b.clone())
        } else {
            client.metadata.base_url().map(|s| s.to_string())
        }
    } else {
        client.metadata.base_url().map(|s| s.to_string())
    }
    .ok_or_else(|| {
        AiLibError::ConfigurationError(format!(
            "No base URL available for provider {}",
            client.provider_name()
        ))
    })?;

    let endpoint = client
        .metadata
        .upload_endpoint()
        .map(|s| s.to_string())
        .ok_or_else(|| {
            AiLibError::UnsupportedFeature(format!(
                "Provider {} does not expose an upload endpoint in OSS",
                client.provider_name()
            ))
        })?;

    // Compose URL (avoid double slashes)
    let upload_url = if base_url.ends_with('/') {
        format!(
            "{}{}",
            base_url.trim_end_matches('/'),
            endpoint.trim_start_matches('/')
        )
    } else {
        format!("{}{}", base_url, endpoint)
    };

    // Perform upload using unified transport helper (uses injected transport when None)
    crate::provider::utils::upload_file_with_transport(None, &upload_url, path, "file")
        .await
        .map_err(|err| err.with_context("client helpers upload_file"))
}
