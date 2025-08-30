use super::config::ProviderConfig;

/// 预定义的提供商配置，支持多种AI服务
///
/// Predefined provider configurations for multiple AI services
pub struct ProviderConfigs;

impl ProviderConfigs {
    pub fn groq() -> ProviderConfig {
        ProviderConfig::openai_compatible("https://api.groq.com/openai/v1", "GROQ_API_KEY")
    }

    pub fn openai() -> ProviderConfig {
        ProviderConfig::openai_compatible("https://api.openai.com/v1", "OPENAI_API_KEY")
    }

    pub fn deepseek() -> ProviderConfig {
        ProviderConfig::openai_compatible("https://api.deepseek.com/v1", "DEEPSEEK_API_KEY")
    }

    pub fn ollama() -> ProviderConfig {
        // Ollama is commonly run locally and is OpenAI-compatible in many setups.
        // Allow developers to override the local base URL via OLLAMA_BASE_URL.
        // Default remains the common local address used by Ollama.
        let base_url = std::env::var("OLLAMA_BASE_URL")
            .unwrap_or_else(|_| "http://localhost:11434/api".to_string());

        // Ollama typically doesn't require an API key for local installs. We keep
        // the env variable name for parity, but it's optional for users.
        ProviderConfig::openai_compatible(&base_url, "OLLAMA_API_KEY")
    }

    /// xAI / Grok configuration - OpenAI-compatible hosted offering
    pub fn xai_grok() -> ProviderConfig {
        let base_url = std::env::var("GROK_BASE_URL")
            .unwrap_or_else(|_| "https://api.grok.com/openai/v1".to_string());
        ProviderConfig::openai_compatible(&base_url, "GROK_API_KEY")
    }

    /// Azure OpenAI configuration - highly compatible but often uses resource-specific base URL
    pub fn azure_openai() -> ProviderConfig {
        // Expect developers to set AZURE_OPENAI_BASE_URL to their resource endpoint
        let base_url = std::env::var("AZURE_OPENAI_BASE_URL")
            .unwrap_or_else(|_| "https://api.openai.azure.com/v1".to_string());
        ProviderConfig::openai_compatible(&base_url, "AZURE_OPENAI_API_KEY")
    }

    /// Hugging Face Inference API - configured to reuse generic adapter; may need adjustments per model
    pub fn huggingface() -> ProviderConfig {
        // Hugging Face inference API default (used for inference calls)
        let base_url = std::env::var("HUGGINGFACE_BASE_URL")
            .unwrap_or_else(|_| "https://api-inference.huggingface.co".to_string());

        use std::collections::HashMap;
        let mut headers = HashMap::new();
        headers.insert("Content-Type".to_string(), "application/json".to_string());

        // Hugging Face model listing is served from the Hub API under huggingface.co.
        // Use an absolute models endpoint so we can query the Hub independently of the
        // inference base URL (inference and hub are different services).
        ProviderConfig {
            base_url: base_url.clone(),
            api_key_env: "HUGGINGFACE_API_KEY".to_string(),
            chat_endpoint: "/models/{model}:predict".to_string(), // placeholder; per-model inference often requires model in path
            upload_endpoint: None,
            upload_size_limit: None,
            models_endpoint: Some("https://huggingface.co/api/models".to_string()),
            headers,
            field_mapping: crate::provider::config::FieldMapping {
                messages_field: "messages".to_string(),
                model_field: "model".to_string(),
                role_mapping: {
                    let mut role_mapping = std::collections::HashMap::new();
                    role_mapping.insert("System".to_string(), "system".to_string());
                    role_mapping.insert("User".to_string(), "user".to_string());
                    role_mapping.insert("Assistant".to_string(), "assistant".to_string());
                    role_mapping
                },
                response_content_path: "choices.0.message.content".to_string(),
            },
        }
    }

    /// Together AI - OpenAI-compatible chat API
    pub fn together_ai() -> ProviderConfig {
        let base_url = std::env::var("TOGETHER_BASE_URL")
            .unwrap_or_else(|_| "https://api.together.ai".to_string());
        ProviderConfig::openai_compatible(&base_url, "TOGETHER_API_KEY")
    }

    /// Groq configuration - proving OpenAI compatibility
    pub fn groq_as_generic() -> ProviderConfig {
        ProviderConfig::openai_compatible("https://api.groq.com/openai/v1", "GROQ_API_KEY")
    }

    /// Qwen / 通义千问 (Alibaba Cloud) - OpenAI-compatible mode
    /// Uses DASHSCOPE_API_KEY and optional DASHSCOPE_BASE_URL to override the base URL.
    pub fn qwen() -> ProviderConfig {
        // Default to Alibaba DashScope public base URL and use the generation endpoint.
        let base_url = std::env::var("DASHSCOPE_BASE_URL")
            .unwrap_or_else(|_| "https://dashscope.aliyuncs.com".to_string());

        // DashScope generation endpoint example: /api/v1/services/aigc/text-generation/generation
        let mut cfg = ProviderConfig::openai_compatible(&base_url, "DASHSCOPE_API_KEY");
        cfg.chat_endpoint = "/api/v1/services/aigc/text-generation/generation".to_string();
        cfg
    }

    /// 百度文心一言（Wenxin / ERNIE） - OpenAI 兼容模式（通过千帆/控制台配置）
    /// 使用环境变量: BAIDU_WENXIN_BASE_URL (可选), BAIDU_WENXIN_API_KEY, BAIDU_WENXIN_SECRET
    pub fn baidu_wenxin() -> ProviderConfig {
        let base_url = std::env::var("BAIDU_WENXIN_BASE_URL")
            .unwrap_or_else(|_| "https://aip.baidubce.com".to_string());

        // Baidu Wenxin example path (model-specific):
        // /rpc/2.0/ai_custom/v1/wenxinworkshop/chat/{model_name}
        let mut cfg = ProviderConfig::openai_compatible(&base_url, "BAIDU_WENXIN_API_KEY");
        cfg.chat_endpoint = "/rpc/2.0/ai_custom/v1/wenxinworkshop/chat/{model}".to_string();
        // Baidu may use different upload endpoints; keep defaults but don't assume model listing.
        cfg.models_endpoint = None;
        cfg
    }

    /// 腾讯混元（Hunyuan） - 腾讯云 OpenAI 兼容端点
    /// 使用环境变量: TENCENT_HUNYUAN_BASE_URL (可选), TENCENT_HUNYUAN_API_KEY
    pub fn tencent_hunyuan() -> ProviderConfig {
        let base_url = std::env::var("TENCENT_HUNYUAN_BASE_URL")
            .unwrap_or_else(|_| "https://hunyuan.tencentcloudapi.com".to_string());

        // Tencent Hunyuan example uses an Action query parameter: ?Action=ChatCompletions
        let mut cfg = ProviderConfig::openai_compatible(&base_url, "TENCENT_HUNYUAN_API_KEY");
        cfg.chat_endpoint = "/?Action=ChatCompletions".to_string();
        cfg.models_endpoint = None;
        cfg
    }

    /// 讯飞星火（iFlytek Spark） - OpenAI 兼容端点示例
    /// 使用环境变量: IFLYTEK_BASE_URL (可选), IFLYTEK_API_KEY
    pub fn iflytek_spark() -> ProviderConfig {
        let base_url = std::env::var("IFLYTEK_BASE_URL").unwrap_or_else(|_| "https://api.xf-yun.com".to_string());
        let mut cfg = ProviderConfig::openai_compatible(&base_url, "IFLYTEK_API_KEY");
        cfg.chat_endpoint = "/v1/chat".to_string();
        cfg
    }

    /// 月之暗面 / Moonshot (Kimi) - OpenAI 兼容端点
    /// 使用环境变量: MOONSHOT_BASE_URL (可选), MOONSHOT_API_KEY
    pub fn moonshot() -> ProviderConfig {
        let base_url = std::env::var("MOONSHOT_BASE_URL").unwrap_or_else(|_| "https://api.moonshot.cn".to_string());
        let mut cfg = ProviderConfig::openai_compatible(&base_url, "MOONSHOT_API_KEY");
        cfg.chat_endpoint = "/v1/chat/completions".to_string();
        cfg
    }

    /// Anthropic Claude configuration - requires special handling
    pub fn anthropic() -> ProviderConfig {
        use std::collections::HashMap;

        let mut headers = HashMap::new();
        headers.insert("Content-Type".to_string(), "application/json".to_string());
        headers.insert("anthropic-version".to_string(), "2023-06-01".to_string());
        // Note: Anthropic uses x-api-key instead of Authorization: Bearer

        let mut role_mapping = HashMap::new();
        role_mapping.insert("System".to_string(), "system".to_string());
        role_mapping.insert("User".to_string(), "user".to_string());
        role_mapping.insert("Assistant".to_string(), "assistant".to_string());

        ProviderConfig {
            base_url: "https://api.anthropic.com/v1".to_string(),
            api_key_env: "ANTHROPIC_API_KEY".to_string(),
            chat_endpoint: "/messages".to_string(),
            upload_endpoint: None,
            upload_size_limit: None,
            models_endpoint: None, // Claude doesn't have a public model list endpoint
            headers,
            field_mapping: crate::provider::config::FieldMapping {
                messages_field: "messages".to_string(),
                model_field: "model".to_string(),
                role_mapping,
                response_content_path: "content.0.text".to_string(), // Claude's response format is different
            },
        }
    }
}
