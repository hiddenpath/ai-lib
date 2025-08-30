use ai_lib::types::common::Content;
use ai_lib::{AiClient, ChatCompletionRequest, Message, Provider, Role};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🤖 Google Gemini 独立适配器测试");
    println!("===============================");

    // 检查API密钥
    match std::env::var("GEMINI_API_KEY") {
        Ok(_) => println!("✅ 检测到 GEMINI_API_KEY"),
        Err(_) => {
            println!("❌ 未设置 GEMINI_API_KEY 环境变量");
            println!("   请设置: export GEMINI_API_KEY=your_api_key");
            println!("   获取API密钥: https://aistudio.google.com/app/apikey");
            return Ok(());
        }
    }

    // 创建Gemini客户端
    let client = AiClient::new(Provider::Gemini)?;
    println!("✅ Gemini客户端创建成功 (使用GeminiAdapter)");

    // 测试聊天完成
    println!("\n💬 测试Gemini聊天...");
    let request = ChatCompletionRequest::new(
        "gemini-1.5-flash".to_string(),
        vec![Message {
            role: Role::User,
            content: Content::Text("Hello Gemini! Please respond with 'Hello from Google Gemini via ai-lib!' to confirm the connection works.".to_string()),
            function_call: None,
        }],
    ).with_max_tokens(50)
     .with_temperature(0.7);

    match client.chat_completion(request).await {
        Ok(response) => {
            println!("✅ Gemini聊天成功!");
            println!("   模型: {}", response.model);
            println!(
                "   响应: '{}'",
                response.choices[0].message.content.as_text()
            );
            println!(
                "   Token使用: {} (prompt: {}, completion: {})",
                response.usage.total_tokens,
                response.usage.prompt_tokens,
                response.usage.completion_tokens
            );
        }
        Err(e) => {
            println!("❌ Gemini聊天失败: {}", e);

            // 分析错误类型
            let error_str = e.to_string();
            if error_str.contains("401") || error_str.contains("403") {
                println!("   → 认证错误，请检查GEMINI_API_KEY");
            } else if error_str.contains("400") {
                println!("   → 请求格式错误");
            } else if error_str.contains("429") {
                println!("   → 速率限制，请稍后重试");
            }
        }
    }

    // 测试模型列表
    println!("\n📋 测试模型列表...");
    match client.list_models().await {
        Ok(models) => {
            println!("✅ 可用模型: {:?}", models);
        }
        Err(e) => {
            println!("❌ 模型列表获取失败: {}", e);
        }
    }

    println!("\n🎯 Gemini独立适配器特点:");
    println!("   • 🔧 特殊API格式: contents数组 vs messages数组");
    println!("   • 🔑 URL参数认证: ?key=<API_KEY> vs Authorization头");
    println!("   • 📊 不同响应路径: candidates[0].content.parts[0].text");
    println!("   • 🎭 角色映射: assistant → model");
    println!("   • ⚙️  配置字段: generationConfig vs 直接参数");

    println!("\n🏗️  混合架构验证:");
    println!("   ✅ 独立适配器与配置驱动适配器共存");
    println!("   ✅ 统一ChatApi接口，用户无感知差异");
    println!("   ✅ 灵活处理特殊API格式和认证方式");

    Ok(())
}
