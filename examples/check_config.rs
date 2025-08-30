use ai_lib::{AiClient, Provider};

fn main() {
    println!("🔧 AI-lib 配置检查工具");
    println!("======================");

    // 检查API密钥
    let api_keys = vec![
        ("GROQ_API_KEY", "Groq"),
        ("OPENAI_API_KEY", "OpenAI"),
        ("DEEPSEEK_API_KEY", "DeepSeek"),
    ];

    println!("\n🔑 API密钥检查:");
    let mut configured_count = 0;

    for (env_var, provider) in &api_keys {
        match std::env::var(env_var) {
            Ok(key) => {
                let masked = if key.len() > 8 {
                    format!("{}...{}", &key[..4], &key[key.len() - 4..])
                } else {
                    "*".repeat(key.len())
                };
                println!("   ✅ {}: {} ({})", provider, env_var, masked);
                configured_count += 1;
            }
            Err(_) => {
                println!("   ❌ {}: {} 未设置", provider, env_var);
            }
        }
    }

    // 检查代理配置
    println!("\n🌐 代理配置检查:");
    match std::env::var("AI_PROXY_URL") {
        Ok(proxy_url) => {
            println!("   ✅ AI_PROXY_URL: {}", proxy_url);
        }
        Err(_) => {
            println!("   ℹ️  AI_PROXY_URL: 未设置 (可选)");
        }
    }

    // 测试客户端创建
    println!("\n🚀 客户端创建测试:");
    let providers = vec![
        (Provider::Groq, "Groq"),
        (Provider::OpenAI, "OpenAI"),
        (Provider::DeepSeek, "DeepSeek"),
    ];

    for (provider, name) in providers {
        match AiClient::new(provider) {
            Ok(_) => println!("   ✅ {}: 客户端创建成功", name),
            Err(e) => println!("   ❌ {}: {}", name, e),
        }
    }

    // 配置建议
    println!("\n💡 配置建议:");
    if configured_count == 0 {
        println!("   • 至少设置一个API密钥来开始使用");
        println!("   • 推荐先设置 GROQ_API_KEY (免费且快速)");
    } else if configured_count < api_keys.len() {
        println!(
            "   • 已配置 {}/{} 个提供商",
            configured_count,
            api_keys.len()
        );
        println!("   • 可以设置更多API密钥来测试不同提供商");
    } else {
        println!("   • 🎉 所有提供商都已配置!");
        println!("   • 可以运行: cargo run --example test_all_providers");
    }

    println!("\n📝 设置示例 (Windows):");
    println!("   set GROQ_API_KEY=your_groq_key");
    println!("   set OPENAI_API_KEY=your_openai_key");
    println!("   set DEEPSEEK_API_KEY=your_deepseek_key");
    println!("   set AI_PROXY_URL=http://proxy.example.com:8080");

    println!("\n📝 设置示例 (Linux/Mac):");
    println!("   export GROQ_API_KEY=your_groq_key");
    println!("   export OPENAI_API_KEY=your_openai_key");
    println!("   export DEEPSEEK_API_KEY=your_deepseek_key");
    println!("   export AI_PROXY_URL=http://proxy.example.com:8080");
}
