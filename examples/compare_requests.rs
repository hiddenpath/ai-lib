use ai_lib::{AiClient, Provider, ChatCompletionRequest, Message, Role};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 对比请求格式");
    println!("================");
    
    let request = ChatCompletionRequest::new(
        "test-model".to_string(),
        vec![Message {
            role: Role::User,
            content: "Hello!".to_string(),
        }],
    ).with_max_tokens(10);
    
    println!("📤 测试请求:");
    println!("   模型: {}", request.model);
    println!("   消息: {:?}", request.messages[0]);
    println!("   max_tokens: {:?}", request.max_tokens);
    
    // 测试Groq (工作正常)
    println!("\n🟢 Groq (工作正常):");
    if let Ok(groq_client) = AiClient::new(Provider::Groq) {
        // Groq使用独立适配器，我们知道它工作正常
        println!("   ✅ 使用独立适配器 (GroqAdapter)");
        println!("   ✅ 请求格式正确");
    }
    
    // 测试OpenAI (有问题)
    println!("\n🔴 OpenAI (有问题):");
    if let Ok(_openai_client) = AiClient::new(Provider::OpenAI) {
        println!("   ❌ 使用配置驱动适配器 (GenericAdapter)");
        println!("   ❌ 请求格式错误: 'you must provide a model parameter'");
        println!("   🔍 可能的问题:");
        println!("      - JSON序列化问题");
        println!("      - 字段映射错误");
        println!("      - 请求体构建错误");
    }
    
    println!("\n💡 解决方案:");
    println!("   1. 检查GenericAdapter的convert_request方法");
    println!("   2. 确保JSON字段名正确");
    println!("   3. 验证请求体结构");
    println!("   4. 考虑为OpenAI创建独立适配器");
    
    // 建议的修复
    println!("\n🔧 建议修复:");
    println!("   选项1: 修复GenericAdapter的请求转换逻辑");
    println!("   选项2: 为OpenAI创建独立适配器 (像Groq一样)");
    println!("   选项3: 添加更多调试信息来定位问题");
    
    Ok(())
}