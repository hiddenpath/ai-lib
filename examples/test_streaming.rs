use ai_lib::types::common::Content;
use ai_lib::{AiClient, ChatCompletionRequest, Message, Provider, Role};
use futures::StreamExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🌊 Streaming Response Test");
    println!("================");

    // Check Groq API key
    if std::env::var("GROQ_API_KEY").is_err() {
        println!("❌ GROQ_API_KEY not set");
        return Ok(());
    }

    // Create Groq client
    let client = AiClient::new(Provider::Groq)?;
    println!("✅ Groq client created successfully");

    // Choose model from env or fallback to a sensible default
    let model = std::env::var("GROQ_MODEL").unwrap_or_else(|_| "llama-3.1-8b-instant".to_string());

    // Create streaming request
    let request = ChatCompletionRequest::new(
        model,
        vec![Message {
            role: Role::User,
            content: Content::Text(
                "Please write a short poem about AI in exactly 4 lines.".to_string(),
            ),
            function_call: None,
        }],
    )
    .with_max_tokens(100)
    .with_temperature(0.7);

    println!("\n📤 Sending streaming request...");
    println!("   Model: {}", request.model);
    println!("   Message: {}", request.messages[0].content.as_text());

    // Get streaming response
    match client.chat_completion_stream(request).await {
        Ok(mut stream) => {
            println!("\n🌊 Starting to receive streaming response:");
            println!("{}", "─".repeat(50));

            let mut full_content = String::new();
            let mut chunk_count = 0;

            while let Some(result) = stream.next().await {
                match result {
                    Ok(chunk) => {
                        chunk_count += 1;

                        if let Some(choice) = chunk.choices.first() {
                            if let Some(content) = &choice.delta.content {
                                print!("{}", content);
                                full_content.push_str(content);

                                // Flush output
                                use std::io::{self, Write};
                                io::stdout().flush().unwrap();
                            }

                            // Check if completed
                            if choice.finish_reason.is_some() {
                                println!("\n{}", "─".repeat(50));
                                println!("✅ Streaming response completed!");
                                println!("   Finish reason: {:?}", choice.finish_reason);
                                break;
                            }
                        }
                    }
                    Err(e) => {
                        println!("\n❌ Streaming response error: {}", e);
                        break;
                    }
                }
            }

            println!("\n📊 Streaming response statistics:");
            println!("   Chunk count: {}", chunk_count);
            println!("   Total content length: {} characters", full_content.len());
            println!("   Complete content: \"{}\"", full_content.trim());
        }
        Err(e) => {
            println!("❌ Streaming request failed: {}", e);
        }
    }

    println!("\n💡 Advantages of streaming responses:");
    println!("   • Real-time content generation display");
    println!("   • Better user experience");
    println!("   • Can stop generation early");
    println!("   • Suitable for long text generation");

    Ok(())
}
