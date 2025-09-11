//! Provider/model pricing table (indicative). Used for defaults and docs.
//! This scaffold provides a minimal lookup; for production, prefer env/remote config.
//!
//! Last updated: 2025-01-27
//! Sources: Official provider pricing pages (as of update date)
//! Note: Prices are USD per 1K tokens, subject to change by providers

use crate::client::Provider;
#[cfg(feature = "routing_mvp")]
use crate::provider::models::PricingInfo;

#[cfg(not(feature = "routing_mvp"))]
#[derive(Debug, Clone)]
pub struct PricingInfo {
    pub input_cost_per_1k: f64,
    pub output_cost_per_1k: f64,
}

#[cfg(not(feature = "routing_mvp"))]
impl PricingInfo {
    pub fn new(input_cost_per_1k: f64, output_cost_per_1k: f64) -> Self {
        Self { input_cost_per_1k, output_cost_per_1k }
    }
}

/// Return indicative pricing for a given provider/model, if known.
/// Values are USD per 1K input/output tokens.
pub fn get_pricing(provider: Provider, model: &str) -> Option<PricingInfo> {
    let m = model.to_ascii_lowercase();
    match provider {
        Provider::OpenAI => match m.as_str() {
            // Reference values are illustrative only
            "gpt-3.5-turbo" => Some(PricingInfo::new(0.50, 1.50)),
            "gpt-4o" => Some(PricingInfo::new(5.00, 15.00)),
            _ => None,
        },
        Provider::Groq => match m.as_str() {
            "llama-3.1-8b-instant" | "llama3-8b" => Some(PricingInfo::new(0.05, 0.08)),
            _ => None,
        },
        Provider::DeepSeek => match m.as_str() {
            "deepseek-chat" => Some(PricingInfo::new(0.27, 1.10)),
            "deepseek-reasoner" => Some(PricingInfo::new(0.55, 2.20)),
            _ => None,
        },
        Provider::Mistral => match m.as_str() {
            "mistral-small" => Some(PricingInfo::new(0.20, 0.60)),
            _ => None,
        },
        Provider::Cohere => match m.as_str() {
            "command-r" => Some(PricingInfo::new(0.50, 1.50)),
            _ => None,
        },
        Provider::Gemini => match m.as_str() {
            "gemini-pro" | "gemini-1.5-flash" => Some(PricingInfo::new(0.10, 0.30)),
            _ => None,
        },
        Provider::AzureOpenAI
        | Provider::Anthropic
        | Provider::HuggingFace
        | Provider::TogetherAI
        | Provider::Qwen
        | Provider::BaiduWenxin
        | Provider::TencentHunyuan
        | Provider::IflytekSpark
        | Provider::Moonshot
        | Provider::Ollama
        | Provider::XaiGrok => None,
    }
}
