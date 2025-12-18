use crate::client::Provider;

/// Static metadata describing recommended models per provider.
///
/// This data is intentionally lightweight so we can provide reasonable fallbacks
/// without hard-coding every possible model permutation. Enterprise/pro releases
/// can override/extend this registry via a custom `ModelResolver`.
#[derive(Debug, Clone)]
pub struct ProviderModelProfile {
    pub provider: Provider,
    pub doc_url: &'static str,
    pub fallback_models: &'static [&'static str],
}

impl ProviderModelProfile {
    pub fn default_chat_model(&self) -> &'static str {
        self.provider.default_chat_model()
    }
}

pub fn profile(provider: Provider) -> ProviderModelProfile {
    match provider {
        Provider::Groq => ProviderModelProfile {
            provider,
            doc_url: "https://console.groq.com/docs/models",
            fallback_models: &["llama-3.1-70b-versatile", "llama-3.2-90b-vision-preview"],
        },
        Provider::Mistral => ProviderModelProfile {
            provider,
            doc_url: "https://docs.mistral.ai/platform/models/",
            fallback_models: &["mistral-small-latest", "mistral-medium", "mistral-large"],
        },
        Provider::DeepSeek => ProviderModelProfile {
            provider,
            doc_url: "https://api-docs.deepseek.com/quick_start/model_list",
            fallback_models: &["deepseek-reasoner"],
        },
        Provider::OpenAI | Provider::AzureOpenAI => ProviderModelProfile {
            provider,
            doc_url: "https://platform.openai.com/docs/models",
            fallback_models: &["gpt-4o-mini", "gpt-4o"],
        },
        Provider::Anthropic => ProviderModelProfile {
            provider,
            doc_url: "https://docs.anthropic.com/claude/reference/selecting-a-model",
            fallback_models: &["claude-3-5-haiku-20241022", "claude-3-opus-20240229"],
        },
        Provider::Gemini => ProviderModelProfile {
            provider,
            doc_url: "https://ai.google.dev/gemini-api/docs/models",
            fallback_models: &["gemini-1.5-pro", "gemini-1.0-pro"],
        },
        Provider::Cohere => ProviderModelProfile {
            provider,
            doc_url: "https://docs.cohere.com/docs/models",
            fallback_models: &["command-r-plus", "command"],
        },
        Provider::Perplexity => ProviderModelProfile {
            provider,
            doc_url: "https://docs.perplexity.ai/docs/model-cards",
            fallback_models: &["llama-3.1-sonar-large-128k-online"],
        },
        Provider::AI21 => ProviderModelProfile {
            provider,
            doc_url: "https://docs.ai21.com/reference/jurassic-models",
            fallback_models: &["j2-grande", "jamba-instruct"],
        },
        Provider::OpenRouter => ProviderModelProfile {
            provider,
            doc_url: "https://openrouter.ai/docs/models",
            fallback_models: &["openai/gpt-4o-mini", "meta-llama/llama-3.1-70b-instruct"],
        },
        Provider::TogetherAI => ProviderModelProfile {
            provider,
            doc_url: "https://docs.together.ai/docs/inference-models",
            fallback_models: &["meta-llama/Llama-3.1-8B-Instruct-Turbo"],
        },
        Provider::Replicate => ProviderModelProfile {
            provider,
            doc_url: "https://replicate.com/explore",
            fallback_models: &["meta/llama-2-13b-chat"],
        },
        Provider::BaiduWenxin => ProviderModelProfile {
            provider,
            doc_url: "https://cloud.baidu.com/doc/WENXINWORKSHOP/s/hlrk4akp8",
            fallback_models: &["ernie-4.0"],
        },
        Provider::TencentHunyuan => ProviderModelProfile {
            provider,
            doc_url: "https://cloud.tencent.com/document/product/1729",
            fallback_models: &["hunyuan-lite"],
        },
        Provider::IflytekSpark => ProviderModelProfile {
            provider,
            doc_url: "https://www.xfyun.cn/doc/spark/Web.html",
            fallback_models: &["spark-v4.0"],
        },
        Provider::Moonshot => ProviderModelProfile {
            provider,
            doc_url: "https://platform.moonshot.cn/docs",
            fallback_models: &["moonshot-v1-32k"],
        },
        Provider::ZhipuAI => ProviderModelProfile {
            provider,
            doc_url: "https://open.bigmodel.cn/doc/overview",
            fallback_models: &["glm-4-air"],
        },
        Provider::MiniMax => ProviderModelProfile {
            provider,
            doc_url: "https://www.minimaxi.com/document/guides/chat-models",
            fallback_models: &["abab5.5-chat"],
        },
        Provider::Qwen => ProviderModelProfile {
            provider,
            doc_url: "https://dashscope.aliyun.com/api",
            fallback_models: &["qwen-plus", "qwen2.5-72b-instruct"],
        },
        Provider::HuggingFace => ProviderModelProfile {
            provider,
            doc_url: "https://huggingface.co/docs/api-inference/models",
            fallback_models: &["google/gemma-2b-it", "mistralai/Mistral-7B-Instruct-v0.2"],
        },
        Provider::XaiGrok => ProviderModelProfile {
            provider,
            doc_url: "https://console.x.ai/docs/api-reference",
            fallback_models: &["grok-2"],
        },
        Provider::Ollama => ProviderModelProfile {
            provider,
            doc_url: "https://github.com/ollama/ollama/blob/main/docs/modelfile.md",
            fallback_models: &["llama3.1:8b", "phi3:3.8b"],
        },
    }
}
