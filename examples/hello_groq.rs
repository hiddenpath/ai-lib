use ai_lib::{AiClient, Provider, ChatCompletionRequest, Message, Role};
use ai_lib::types::common::Content;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Check environment variables
    if std::env::var("GROQ_API_KEY").is_err() {
        println!("❌ Please set GROQ_API_KEY environment variable");
        println!("   Example: export GROQ_API_KEY=your_api_key_here");
        println!("   Or set it in .env file");
        return Ok(());
    }
    
    println!("🔧 Creating Groq client using new Provider classification system...");
    
    // Create Groq client - using new provider classification system
    let client = AiClient::new(Provider::Groq)?;
    
    // fetch available models before have a konwn model
    let models = client.list_models().await?;
    
    // print models list
    println!("📝 Available models: {:?}", models);

    // Create chat request to a model from the list
    let request = ChatCompletionRequest::new(
        "llama-3.1-8b-instant".to_string(), // Available Groq model
        vec![Message {
            role: Role::User,
            content: Content::Text("Hello! Please respond with a simple greeting.".to_string()),
            function_call: None,
        }],
    );
    
    println!("🚀 Sending request to Groq...");
    println!("📝 Request: Hello! Please respond with a simple greeting.");
    println!();
    
    // Send request and get response
    let response = client.chat_completion(request).await?;
    
    println!("✅ Groq Response:");
    match &response.choices[0].message.content {
        Content::Text(text) => println!("{}", text),
        Content::Json(json) => println!("JSON: {:?}", json),
        Content::Image { url, mime, name } => println!("Image: url={:?}, mime={:?}, name={:?}", url, mime, name),
        Content::Audio { url, mime } => println!("Audio: url={:?}, mime={:?}", url, mime),
    }
    
    Ok(())
}
