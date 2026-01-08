use ai_lib::manifest::schema::{
    AuthConfig, Capability, Manifest, ManifestMetadata, ManifestValidator, MappingRule,
    ModelDefinition, ModelStatus, MultimodalSchema, ParameterConstraints, ParameterDefinition,
    ParameterType, PayloadFormat, ProviderDefinition, ResponseFormat, ResponseFormatSchema,
    StandardSchema, StreamingConfig, ToolSchema,
};
use std::collections::HashMap;

fn base_manifest() -> Manifest {
    let mut params = HashMap::new();
    params.insert(
        "temperature".to_string(),
        ParameterDefinition {
            param_type: ParameterType::Float,
            constraints: ParameterConstraints {
                range: Some([0.0, 2.0]),
                min: None,
                max: None,
                values: vec![],
                pattern: None,
            },
            default: None,
            description: None,
        },
    );

    let standard_schema = StandardSchema {
        parameters: params,
        tools: ToolSchema {
            schema: "std".to_string(),
            choice_policy: vec!["auto".to_string()],
            strict_mode: false,
            parallel_calls: false,
        },
        response_format: ResponseFormatSchema {
            types: vec!["text".to_string()],
            schema_validation: false,
        },
        multimodal: MultimodalSchema::default(),
        agentic_loop: None,
        streaming_events: None,
    };

    let mut providers = HashMap::new();
    providers.insert(
        "openai".to_string(),
        ProviderDefinition {
            version: "v1".to_string(),
            base_url: Some("https://api.example.com/v1".to_string()),
            base_url_template: None,
            connection_vars: None,
            auth: AuthConfig::Bearer {
                token_env: "TOKEN".to_string(),
                extra_headers: vec![],
            },
            payload_format: PayloadFormat::OpenaiStyle,
            parameter_mappings: {
                let mut m = HashMap::new();
                m.insert(
                    "temperature".to_string(),
                    MappingRule::Direct("temperature".to_string()),
                );
                m
            },
            special_handling: HashMap::new(),
            response_format: ResponseFormat::OpenaiStyle,
            response_paths: {
                let mut rp = HashMap::new();
                rp.insert(
                    "content".to_string(),
                    ai_lib::manifest::schema::JsonPath("choices[0].message.content".to_string()),
                );
                rp
            },
            streaming: StreamingConfig::default(),
            experimental_features: vec![],
            capabilities: vec![Capability::Chat],
            response_strategy: None,
            tools_mapping: None,
            prompt_caching: None,
            service_tier: None,
            reasoning_tokens: None,
            features: None,
        },
    );

    let mut models = HashMap::new();
    models.insert(
        "gpt-4".to_string(),
        ModelDefinition {
            provider: "openai".to_string(),
            model_id: "gpt-4".to_string(),
            display_name: None,
            context_window: 8192,
            capabilities: vec![Capability::Chat],
            pricing: None,
            overrides: HashMap::new(),
            status: ModelStatus::Active,
            tags: vec![],
            agentic_capabilities: None,
        },
    );

    Manifest {
        version: "1.1".to_string(),
        metadata: ManifestMetadata::default(),
        standard_schema,
        providers,
        models,
    }
}

#[test]
fn validate_ok_manifest() {
    let manifest = base_manifest();
    assert!(ManifestValidator::validate_manifest(&manifest).is_ok());
}

#[test]
fn validate_missing_content_path_fails() {
    let mut manifest = base_manifest();
    if let Some(p) = manifest.providers.get_mut("openai") {
        p.response_paths.clear();
    }
    assert!(ManifestValidator::validate_manifest(&manifest).is_err());
}

#[test]
fn validate_template_requires_vars() {
    let mut manifest = base_manifest();
    if let Some(p) = manifest.providers.get_mut("openai") {
        p.base_url = None;
        p.base_url_template = Some("https://{resource}.example.com/{deployment}".to_string());
        p.connection_vars = None;
    }
    assert!(ManifestValidator::validate_manifest(&manifest).is_err());
}
