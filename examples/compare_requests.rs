/// 对比请求格式示例 - Compare request formats example
use ai_lib::types::common::Content;
use ai_lib::{AiClient, ChatCompletionRequest, Message, Provider, Role};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 Compare Request Formats");
    println!("==========================");

    let request = ChatCompletionRequest::new(
        "test-model".to_string(),
        vec![Message {
            role: Role::User,
            content: Content::Text("Hello!".to_string()),
            function_call: None,
        }],
    )
    .with_max_tokens(10);

    println!("📤 Test Request:");
    println!("   Model: {}", request.model);
    println!("   Message: {:?}", request.messages[0]);
    println!("   max_tokens: {:?}", request.max_tokens);

    // Test Groq (working normally)
    println!("\n🟢 Groq (working normally):");
    if let Ok(_groq_client) = AiClient::new(Provider::Groq) {
        // Groq uses independent adapter, we know it works normally
        println!("   ✅ Uses independent adapter (GroqAdapter)");
        println!("   ✅ Request format correct");
    }

    // Test OpenAI (has issues)
    println!("\n🔴 OpenAI (has issues):");
    if let Ok(_openai_client) = AiClient::new(Provider::OpenAI) {
        println!("   ❌ Uses config-driven adapter (GenericAdapter)");
        println!("   ❌ Request format error: 'you must provide a model parameter'");
        println!("   🔍 Possible issues:");
        println!("      - JSON serialization problem");
        println!("      - Field mapping error");
        println!("      - Request body construction error");
    }

    println!("\n💡 Solutions:");
    println!("   1. Check GenericAdapter's convert_request method");
    println!("   2. Ensure JSON field names are correct");
    println!("   3. Verify request body structure");
    println!("   4. Consider creating independent adapter for OpenAI");

    // Suggested fixes
    println!("\n🔧 Suggested Fixes:");
    println!("   Option 1: Fix GenericAdapter's request conversion logic");
    println!("   Option 2: Create independent adapter for OpenAI (like Groq)");
    println!("   Option 3: Add more debug information to locate the issue");

    Ok(())
}
