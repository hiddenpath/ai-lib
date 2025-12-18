use std::borrow::Cow;

use crate::{client::Provider, transport::TransportError, types::AiLibError};

use super::catalog;

#[derive(Debug, Clone)]
pub struct ModelResolver;

#[derive(Debug, Clone)]
pub struct ModelResolution {
    pub model: String,
    pub source: ModelResolutionSource,
    pub doc_url: &'static str,
}

impl ModelResolution {
    pub fn new(
        model: impl Into<String>,
        source: ModelResolutionSource,
        doc_url: &'static str,
    ) -> Self {
        Self {
            model: model.into(),
            source,
            doc_url,
        }
    }
}

#[derive(Debug, Clone)]
pub enum ModelResolutionSource {
    Explicit,
    CustomDefault,
    EnvOverride,
    ProviderDefault,
    ProfileFallback,
}

impl ModelResolver {
    pub fn new() -> Self {
        Self
    }

    pub fn resolve_chat_model(
        &self,
        provider: Provider,
        requested: Option<&str>,
    ) -> ModelResolution {
        let profile = catalog::profile(provider);
        let doc_url = profile.doc_url;
        if let Some(model) = requested {
            return ModelResolution::new(
                model.to_string(),
                ModelResolutionSource::Explicit,
                doc_url,
            );
        }

        if let Some(env_model) = self.env_override(provider) {
            return ModelResolution::new(env_model, ModelResolutionSource::EnvOverride, doc_url);
        }

        ModelResolution::new(
            profile.default_chat_model(),
            ModelResolutionSource::ProviderDefault,
            doc_url,
        )
    }

    pub fn fallback_after_invalid(
        &self,
        provider: Provider,
        failed_model: &str,
    ) -> Option<ModelResolution> {
        let profile = catalog::profile(provider);
        let doc_url = profile.doc_url;

        if let Some(env_model) = self.env_override(provider) {
            if !equals_ignore_case(&env_model, failed_model) {
                return Some(ModelResolution::new(
                    env_model,
                    ModelResolutionSource::EnvOverride,
                    doc_url,
                ));
            }
        }

        for candidate in profile.fallback_models {
            if !equals_ignore_case(candidate, failed_model) {
                return Some(ModelResolution::new(
                    *candidate,
                    ModelResolutionSource::ProfileFallback,
                    doc_url,
                ));
            }
        }

        if !equals_ignore_case(profile.default_chat_model(), failed_model) {
            return Some(ModelResolution::new(
                profile.default_chat_model(),
                ModelResolutionSource::ProviderDefault,
                doc_url,
            ));
        }

        None
    }

    pub fn doc_url(&self, provider: Provider) -> &'static str {
        catalog::profile(provider).doc_url
    }

    pub fn suggestions(&self, provider: Provider) -> Vec<String> {
        let mut list = Vec::new();

        if let Some(env_override) = self.env_override(provider) {
            push_unique(&mut list, env_override);
        }

        let profile = catalog::profile(provider);
        push_unique(&mut list, profile.default_chat_model().to_string());
        for candidate in profile.fallback_models {
            push_unique(&mut list, (*candidate).to_string());
        }

        list
    }

    pub fn looks_like_invalid_model(&self, err: &AiLibError) -> bool {
        match err {
            AiLibError::ModelNotFound(_) => true,
            AiLibError::InvalidRequest(msg)
            | AiLibError::ProviderError(msg)
            | AiLibError::InvalidModelResponse(msg) => contains_invalid_keyword(msg),
            AiLibError::TransportError(TransportError::ClientError { status, message })
            | AiLibError::TransportError(TransportError::ServerError { status, message }) => {
                (*status == 400 || *status == 404) && contains_invalid_keyword(message)
            }
            _ => false,
        }
    }

    pub fn decorate_invalid_model_error(
        &self,
        provider: Provider,
        requested_model: &str,
        err: AiLibError,
    ) -> AiLibError {
        let doc_url = self.doc_url(provider);
        let suggestions = self.suggestions(provider);
        let provider_name = format!("{provider:?}");
        let suggestion_text = if suggestions.is_empty() {
            Cow::Borrowed("no known fallback models configured")
        } else {
            Cow::Owned(suggestions.join(", "))
        };

        AiLibError::ModelNotFound(format!(
            "Model `{}` is not available for provider {}. Try: {}. Docs: {}. Original error: {}",
            requested_model, provider_name, suggestion_text, doc_url, err
        ))
    }

    fn env_override(&self, provider: Provider) -> Option<String> {
        let var = format!("{}_MODEL", provider.env_prefix());
        std::env::var(var)
            .ok()
            .filter(|value| !value.trim().is_empty())
    }
}

fn push_unique(list: &mut Vec<String>, value: String) {
    if !list
        .iter()
        .any(|existing| equals_ignore_case(existing, &value))
    {
        list.push(value);
    }
}

fn equals_ignore_case(a: &str, b: &str) -> bool {
    a.eq_ignore_ascii_case(b)
}

fn contains_invalid_keyword(message: &str) -> bool {
    let lower = message.to_ascii_lowercase();
    lower.contains("invalid model")
        || lower.contains("model_not_found")
        || lower.contains("model not found")
        || lower.contains("unknown model")
        || lower.contains("unsupported model")
        || lower.contains("\"code\":\"1500\"")
}
