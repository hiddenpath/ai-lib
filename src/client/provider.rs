#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Provider {
    // Config-driven providers
    Groq,
    XaiGrok,
    Ollama,
    DeepSeek,
    Anthropic,
    AzureOpenAI,
    HuggingFace,
    TogetherAI,
    OpenRouter,
    Replicate,
    // Chinese providers (OpenAI-compatible or config-driven)
    BaiduWenxin,
    TencentHunyuan,
    IflytekSpark,
    Moonshot,
    ZhipuAI,
    MiniMax,
    // Independent adapters
    OpenAI,
    Qwen,
    Gemini,
    Mistral,
    Cohere,
    Perplexity,
    AI21,
    // AWS Bedrock handled in ai-lib-pro
}

impl Provider {
    /// Get the provider's preferred default chat model.
    /// These should mirror the values used inside `ProviderConfigs`.
    pub fn default_chat_model(&self) -> &'static str {
        match self {
            Provider::Groq => "llama-3.1-8b-instant",
            Provider::XaiGrok => "grok-beta",
            Provider::Ollama => "llama3-8b",
            Provider::DeepSeek => "deepseek-chat",
            Provider::Anthropic => "claude-3-5-sonnet-20241022",
            Provider::AzureOpenAI => "gpt-35-turbo",
            Provider::HuggingFace => "microsoft/DialoGPT-medium",
            Provider::TogetherAI => "meta-llama/Llama-3-8b-chat-hf",
            Provider::OpenRouter => "openai/gpt-3.5-turbo",
            Provider::Replicate => "meta/llama-2-7b-chat",
            Provider::BaiduWenxin => "ernie-3.5",
            Provider::TencentHunyuan => "hunyuan-standard",
            Provider::IflytekSpark => "spark-v3.0",
            Provider::Moonshot => "moonshot-v1-8k",
            Provider::ZhipuAI => "glm-4",
            Provider::MiniMax => "abab6.5-chat",
            Provider::OpenAI => "gpt-3.5-turbo",
            Provider::Qwen => "qwen-turbo",
            Provider::Gemini => "gemini-1.5-flash",
            Provider::Mistral => "mistral-small",
            Provider::Cohere => "command-r",
            Provider::Perplexity => "llama-3.1-sonar-small-128k-online",
            Provider::AI21 => "j2-ultra",
        }
    }

    /// Get the provider's preferred multimodal model (if any).
    pub fn default_multimodal_model(&self) -> Option<&'static str> {
        match self {
            Provider::OpenAI => Some("gpt-4o"),
            Provider::AzureOpenAI => Some("gpt-4o"),
            Provider::Anthropic => Some("claude-3-5-sonnet-20241022"),
            Provider::Groq => None,
            Provider::Gemini => Some("gemini-1.5-flash"),
            Provider::Cohere => Some("command-r-plus"),
            Provider::OpenRouter => Some("openai/gpt-4o"),
            Provider::Replicate => Some("meta/llama-2-7b-chat"),
            Provider::ZhipuAI => Some("glm-4v"),
            Provider::MiniMax => Some("abab6.5-chat"),
            Provider::Perplexity => Some("llama-3.1-sonar-small-128k-online"),
            Provider::AI21 => Some("j2-ultra"),
            _ => None,
        }
    }

    /// Get the environment variable prefix for this provider.
    ///
    /// Used by `ConnectionOptions::hydrate_with_env` to look up provider-specific
    /// API keys (e.g., `OPENAI_API_KEY`, `GROQ_API_KEY`).
    pub fn env_prefix(&self) -> &'static str {
        match self {
            Provider::Groq => "GROQ",
            Provider::XaiGrok => "GROK",
            Provider::Ollama => "OLLAMA",
            Provider::DeepSeek => "DEEPSEEK",
            Provider::Anthropic => "ANTHROPIC",
            Provider::AzureOpenAI => "AZURE_OPENAI",
            Provider::HuggingFace => "HUGGINGFACE",
            Provider::TogetherAI => "TOGETHER",
            Provider::OpenRouter => "OPENROUTER",
            Provider::Replicate => "REPLICATE",
            Provider::BaiduWenxin => "BAIDU_WENXIN",
            Provider::TencentHunyuan => "TENCENT_HUNYUAN",
            Provider::IflytekSpark => "IFLYTEK",
            Provider::Moonshot => "MOONSHOT",
            Provider::ZhipuAI => "ZHIPU",
            Provider::MiniMax => "MINIMAX",
            Provider::OpenAI => "OPENAI",
            Provider::Qwen => "DASHSCOPE",
            Provider::Gemini => "GEMINI",
            Provider::Mistral => "MISTRAL",
            Provider::Cohere => "COHERE",
            Provider::Perplexity => "PERPLEXITY",
            Provider::AI21 => "AI21",
        }
    }
}
