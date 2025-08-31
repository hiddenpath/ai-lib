use ai_lib::{AiClient, Provider};

#[tokio::main]
async fn main() {
    let providers = vec![
        Provider::Groq,
        Provider::XaiGrok,
        Provider::Ollama,
        Provider::DeepSeek,
        Provider::Anthropic,
        Provider::AzureOpenAI,
        Provider::HuggingFace,
        Provider::TogetherAI,
        Provider::Qwen,
        Provider::OpenAI,
        Provider::Gemini,
        Provider::Mistral,
        Provider::Cohere,
        // Provider::Bedrock, // Removed: Bedrock deferred implementation/not in public API
    ];

    for p in providers {
        println!("--- Provider: {:?} ---", p);
        match AiClient::new(p) {
            Ok(client) => match client.list_models().await {
                Ok(models) => {
                    println!("Found {} models (showing up to 5):", models.len());
                    for m in models.into_iter().take(5) {
                        println!(" - {}", m);
                    }
                }
                Err(e) => println!("list_models error: {:?}", e),
            },
            Err(e) => println!("client init error: {:?}", e),
        }
    }
}
