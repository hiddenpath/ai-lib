use ai_lib::{AiClient, Provider, ChatCompletionRequest, Message, Role};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔄 重试机制和错误处理测试");
    println!("==========================");
    
    // 测试正常请求
    if std::env::var("GROQ_API_KEY").is_ok() {
        println!("\n✅ 测试正常请求 (Groq):");
        let client = AiClient::new(Provider::Groq)?;
        
        let request = ChatCompletionRequest::new(
            "llama3-8b-8192".to_string(),
            vec![Message {
                role: Role::User,
                content: "Say 'Retry test successful!' exactly.".to_string(),
            }],
        ).with_max_tokens(10);
        
        match client.chat_completion(request).await {
            Ok(response) => {
                println!("   ✅ 正常请求成功: '{}'", response.choices[0].message.content);
            }
            Err(e) => {
                println!("   ❌ 正常请求失败: {}", e);
                println!("   🔍 错误分析:");
                println!("      可重试: {}", e.is_retryable());
                println!("      建议延迟: {}ms", e.retry_delay_ms());
            }
        }
    }
    
    // 测试无效模型 (应该不重试)
    if std::env::var("GROQ_API_KEY").is_ok() {
        println!("\n❌ 测试无效模型 (不应重试):");
        let client = AiClient::new(Provider::Groq)?;
        
        let request = ChatCompletionRequest::new(
            "invalid-model-name".to_string(),
            vec![Message {
                role: Role::User,
                content: "Test invalid model".to_string(),
            }],
        ).with_max_tokens(10);
        
        match client.chat_completion(request).await {
            Ok(_) => {
                println!("   ⚠️  意外成功 (可能模型名称有效)");
            }
            Err(e) => {
                println!("   ✅ 预期失败: {}", e);
                println!("   🔍 错误分析:");
                println!("      可重试: {} (应该是false)", e.is_retryable());
                println!("      建议延迟: {}ms", e.retry_delay_ms());
            }
        }
    }
    
    // 测试网络错误模拟
    println!("\n🌐 测试网络错误处理:");
    println!("   (使用无效代理模拟网络错误)");
    
    // 临时设置无效代理
    let original_proxy = std::env::var("AI_PROXY_URL").ok();
    std::env::set_var("AI_PROXY_URL", "http://invalid-proxy:9999");
    
    if std::env::var("GROQ_API_KEY").is_ok() {
        let client = AiClient::new(Provider::Groq)?;
        
        let request = ChatCompletionRequest::new(
            "llama3-8b-8192".to_string(),
            vec![Message {
                role: Role::User,
                content: "Network error test".to_string(),
            }],
        ).with_max_tokens(10);
        
        match client.chat_completion(request).await {
            Ok(_) => {
                println!("   ⚠️  意外成功 (代理可能有效)");
            }
            Err(e) => {
                println!("   ✅ 预期网络错误: {}", e);
                println!("   🔍 错误分析:");
                println!("      可重试: {} (网络错误应该可重试)", e.is_retryable());
                println!("      建议延迟: {}ms", e.retry_delay_ms());
            }
        }
    }
    
    // 恢复原始代理设置
    match original_proxy {
        Some(proxy) => std::env::set_var("AI_PROXY_URL", proxy),
        None => std::env::remove_var("AI_PROXY_URL"),
    }
    
    println!("\n🎯 重试机制特性:");
    println!("   • 🔄 自动重试: 网络错误、超时、速率限制");
    println!("   • ⏱️  智能延迟: 根据错误类型调整重试间隔");
    println!("   • 🛑 永久错误: 认证、无效请求等不重试");
    println!("   • 📊 错误分类: 详细的错误类型和处理建议");
    
    println!("\n💡 错误处理最佳实践:");
    println!("   1. 检查 error.is_retryable() 决定是否重试");
    println!("   2. 使用 error.retry_delay_ms() 获取建议延迟");
    println!("   3. 实现指数退避避免过度重试");
    println!("   4. 记录错误日志便于调试和监控");
    
    Ok(())
}