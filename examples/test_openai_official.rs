use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 OpenAI官方API格式测试");
    println!("========================");

    // 检查API密钥
    let api_key = match std::env::var("OPENAI_API_KEY") {
        Ok(key) => key,
        Err(_) => {
            println!("❌ 未设置OPENAI_API_KEY");
            return Ok(());
        }
    };

    // 检查代理
    let proxy_url = std::env::var("AI_PROXY_URL").ok();
    if let Some(ref url) = proxy_url {
        println!("🌐 使用代理: {}", url);
    }

    // 创建HTTP客户端
    let mut client_builder = reqwest::Client::builder();
    if let Some(proxy_url) = proxy_url {
        if let Ok(proxy) = reqwest::Proxy::all(&proxy_url) {
            client_builder = client_builder.proxy(proxy);
        }
    }
    let client = client_builder.build()?;

    // 按照OpenAI官方文档构建请求
    let request_body = json!({
        "model": "gpt-3.5-turbo",
        "messages": [
            {
                "role": "user",
                "content": "Say 'Hello from OpenAI API!' exactly."
            }
        ],
        "max_tokens": 20,
        "temperature": 0.0
    });

    println!("\n📤 发送请求:");
    println!("URL: https://api.openai.com/v1/chat/completions");
    println!("Body: {}", serde_json::to_string_pretty(&request_body)?);

    // 发送请求
    let response = client
        .post("https://api.openai.com/v1/chat/completions")
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&request_body)
        .send()
        .await?;

    let status = response.status();
    println!("\n📥 响应状态: {}", status);

    let response_text = response.text().await?;

    if status.is_success() {
        println!("✅ 请求成功!");

        // 解析响应
        if let Ok(response_json) = serde_json::from_str::<serde_json::Value>(&response_text) {
            println!(
                "响应内容: {}",
                serde_json::to_string_pretty(&response_json)?
            );

            if let Some(choices) = response_json["choices"].as_array() {
                if let Some(first_choice) = choices.first() {
                    if let Some(message) = first_choice["message"]["content"].as_str() {
                        println!("\n🎯 AI回复: '{}'", message);
                    }
                }
            }
        }
    } else {
        println!("❌ 请求失败!");
        println!("错误响应: {}", response_text);

        // 分析错误
        if let Ok(error_json) = serde_json::from_str::<serde_json::Value>(&response_text) {
            if let Some(error) = error_json["error"]["message"].as_str() {
                println!("错误信息: {}", error);
            }
        }
    }

    println!("\n💡 这个测试直接使用reqwest调用OpenAI API");
    println!("   如果成功，说明API密钥和网络连接都正常");
    println!("   如果失败，说明问题在于我们的适配器实现");

    Ok(())
}
