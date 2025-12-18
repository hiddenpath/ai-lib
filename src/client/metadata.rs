use crate::{client::Provider, provider::classification::ProviderClassification};

/// Lightweight runtime metadata describing the active chat provider.
///
/// This replaces the previous direct dependency on the `Provider` enum inside
/// `AiClient`, allowing the client to operate purely against trait objects
/// while still surfacing human-readable context (name, defaults, upload paths).
#[derive(Clone, Debug)]
pub(crate) struct ClientMetadata {
    provider: Provider,
    provider_name: String,
    default_chat_model: Option<String>,
    default_multimodal_model: Option<String>,
    base_url: Option<String>,
    upload_endpoint: Option<String>,
    models_endpoint: Option<String>,
}

impl ClientMetadata {
    pub(crate) fn new(provider: Provider, provider_name: impl Into<String>) -> Self {
        Self {
            provider,
            provider_name: provider_name.into(),
            default_chat_model: None,
            default_multimodal_model: None,
            base_url: None,
            upload_endpoint: None,
            models_endpoint: None,
        }
    }

    pub(crate) fn provider(&self) -> Provider {
        self.provider
    }

    pub(crate) fn provider_name(&self) -> &str {
        &self.provider_name
    }

    pub(crate) fn default_chat_model(&self) -> Option<&str> {
        self.default_chat_model.as_deref()
    }

    pub(crate) fn default_multimodal_model(&self) -> Option<&str> {
        self.default_multimodal_model.as_deref()
    }

    pub(crate) fn base_url(&self) -> Option<&str> {
        self.base_url.as_deref()
    }

    pub(crate) fn upload_endpoint(&self) -> Option<&str> {
        self.upload_endpoint.as_deref()
    }
}

/// Build metadata for a provider-driven client instance.
pub(crate) fn metadata_from_provider(
    provider: Provider,
    provider_name: impl Into<String>,
    base_url: Option<String>,
    default_chat_model: Option<String>,
    default_multimodal_model: Option<String>,
) -> ClientMetadata {
    let mut metadata = ClientMetadata::new(provider, provider_name);
    metadata.base_url = base_url;

    let chat_model = default_chat_model.or_else(|| Some(provider.default_chat_model().to_string()));
    metadata.default_chat_model = chat_model;

    let multimodal = default_multimodal_model
        .or_else(|| provider.default_multimodal_model().map(|s| s.to_string()));
    metadata.default_multimodal_model = multimodal;

    if provider.is_config_driven() {
        if let Ok(config) = provider.get_default_config() {
            metadata.upload_endpoint = config.upload_endpoint;
            metadata.models_endpoint = config.models_endpoint;
        }
    } else if provider == Provider::OpenAI {
        metadata.upload_endpoint = Some("/v1/files".to_string());
        metadata.models_endpoint = Some("/v1/models".to_string());
    }

    metadata
}
