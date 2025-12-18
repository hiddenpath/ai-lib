//! Example: Adding a custom provider to ai-lib
//!
//! This example demonstrates the minimal steps to add a new AI provider.

use ai_lib::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Note: In a real scenario, you would add 'YourProvider' to the Provider enum
    // and recompile. For this example, we'll use an existing provider to demonstrate
    // the client creation flow which would be identical.

    // Step 1: Use your new provider
    // let client = AiClient::new(Provider::YourProvider)?;
    let client = AiClient::new(Provider::OpenAI)?;

    // Step 2: Make a request
    let request = ChatCompletionRequest::new(
        "gpt-3.5-turbo".to_string(),
        vec![Message {
            role: Role::User,
            content: ai_lib::types::common::Content::Text("Hello!".to_string()),
            function_call: None,
        }],
    );

    // Note: This will fail without an API key, but demonstrates the API usage
    match client.chat_completion(request).await {
        Ok(response) => println!("Response: {:?}", response),
        Err(e) => println!("Error (expected if no key): {}", e),
    }

    Ok(())
}
