use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryConfig {
    #[serde(default)]
    pub protocols: HashMap<String, ProtocolDefinition>,
    #[serde(default)]
    pub providers: HashMap<String, ProviderConfig>,
    #[serde(default)]
    pub models: Vec<ModelInfo>,
}

impl RegistryConfig {
    /// Convert Manifest to RegistryConfig for backward compatibility
    pub fn from_manifest(manifest: &crate::manifest::schema::Manifest) -> Self {
        use crate::manifest::schema::PayloadFormat;
        use std::collections::HashMap;

        let mut providers = HashMap::new();
        for (id, provider_def) in &manifest.providers {
            let base_url = provider_def.base_url.clone();
            let api_env = match &provider_def.auth {
                crate::manifest::schema::AuthConfig::Bearer { token_env, .. } => {
                    Some(token_env.clone())
                }
                crate::manifest::schema::AuthConfig::ApiKey { key_env, .. } => {
                    Some(key_env.clone())
                }
                crate::manifest::schema::AuthConfig::QueryParam { token_env, .. } => {
                    Some(token_env.clone())
                }
                _ => None,
            };

            let protocol = match provider_def.payload_format {
                PayloadFormat::OpenaiStyle => "openai".to_string(),
                PayloadFormat::AnthropicStyle => "anthropic".to_string(),
                PayloadFormat::GeminiStyle => "gemini".to_string(),
                PayloadFormat::CohereNative => "cohere".to_string(),
                PayloadFormat::Custom(ref s) => s.clone(),
            };

            let mut extra = HashMap::new();
            if let Some(event_format) = &provider_def.streaming.event_format {
                extra.insert(
                    "streaming_event_format".to_string(),
                    serde_json::json!(event_format),
                );
            }
            if let Some(cp) = &provider_def.streaming.content_path {
                extra.insert("streaming_content_path".to_string(), serde_json::json!(cp));
            }
            if let Some(tp) = &provider_def.streaming.tool_call_path {
                extra.insert(
                    "streaming_tool_call_path".to_string(),
                    serde_json::json!(tp),
                );
            }

            providers.insert(
                id.clone(),
                ProviderConfig {
                    protocol,
                    base_url,
                    api_env,
                    api_key: None,
                    headers: HashMap::new(),
                    extra,
                },
            );
        }

        let models: Vec<ModelInfo> = manifest
            .models
            .iter()
            .map(|(id, model_def)| {
                let mut capabilities: HashMap<String, serde_json::Value> = model_def
                    .capabilities
                    .iter()
                    .map(|cap| {
                        (
                            format!("{:?}", cap).to_lowercase(),
                            serde_json::Value::Bool(true),
                        )
                    })
                    .collect();

                if let Some(agentic) = &model_def.agentic_capabilities {
                    capabilities.insert(
                        "reasoning_effort".to_string(),
                        serde_json::json!(format!("{:?}", agentic.reasoning_effort).to_lowercase()),
                    );
                    capabilities.insert(
                        "thinking_blocks".to_string(),
                        serde_json::json!(agentic.thinking_blocks),
                    );
                    capabilities.insert(
                        "parallel_tools".to_string(),
                        serde_json::json!(agentic.parallel_tools),
                    );
                    if let Some(max_p) = agentic.max_parallel_tools {
                        capabilities
                            .insert("max_parallel_tools".to_string(), serde_json::json!(max_p));
                    }
                    if !agentic.builtin_tools.is_empty() {
                        capabilities.insert(
                            "builtin_tools".to_string(),
                            serde_json::json!(agentic.builtin_tools.clone()),
                        );
                    }
                }

                ModelInfo {
                    id: id.clone(),
                    provider: model_def.provider.clone(),
                    display_name: model_def.display_name.clone(),
                    context: model_def.context_window,
                    pricing: model_def.pricing.as_ref().map(|p| ModelPricing {
                        input: p.input_per_token,
                        output: p.output_per_token,
                        currency: p.currency.clone(),
                    }),
                    capabilities,
                }
            })
            .collect();

        RegistryConfig {
            protocols: HashMap::new(),
            providers,
            models,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolDefinition {
    pub adapter: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    pub protocol: String,
    #[serde(rename = "baseUrl")]
    pub base_url: Option<String>,
    #[serde(rename = "apiEnv")]
    pub api_env: Option<String>,
    /// API key override (takes precedence over api_env)
    /// This field is not serialized/deserialized from JSON, set programmatically
    #[serde(skip)]
    pub api_key: Option<String>,
    #[serde(default)]
    pub headers: HashMap<String, String>, // Added based on Lobe Chat insights
    #[serde(default)]
    pub extra: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub id: String,
    pub provider: String,
    #[serde(default, rename = "displayName")]
    pub display_name: Option<String>, // UI-friendly name
    #[serde(default)]
    pub context: usize,
    #[serde(default)]
    pub pricing: Option<ModelPricing>, // Cost tracking
    #[serde(default)]
    pub capabilities: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelPricing {
    #[serde(default)]
    pub input: f64, // per 1M tokens usually? Or 1k? Let's assume per 1M for precision
    #[serde(default)]
    pub output: f64,
    #[serde(default)]
    pub currency: String,
}
