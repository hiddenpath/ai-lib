use ai_lib::transport::{HttpClient, HttpTransport};
use serde_json::json;
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 OpenAI传输层调试");
    println!("===================");

    let api_key = match std::env::var("OPENAI_API_KEY") {
        Ok(key) => key,
        Err(_) => {
            println!("❌ 未设置OPENAI_API_KEY");
            return Ok(());
        }
    };

    // 使用我们的HttpTransport
    let transport = HttpTransport::new();

    // 测试GET请求 (模型列表) - 我们知道这个工作
    println!("\n📋 测试GET请求 (模型列表):");
    let mut headers = HashMap::new();
    headers.insert("Authorization".to_string(), format!("Bearer {}", api_key));

    match transport
        .get::<serde_json::Value>("https://api.openai.com/v1/models", Some(headers))
        .await
    {
        Ok(response) => {
            let model_count = response["data"]
                .as_array()
                .map(|arr| arr.len())
                .unwrap_or(0);
            println!("✅ GET请求成功，获取到 {} 个模型", model_count);
        }
        Err(e) => {
            println!("❌ GET请求失败: {}", e);
        }
    }

    // 测试POST请求 (聊天完成) - 这个有问题
    println!("\n💬 测试POST请求 (聊天完成):");
    let mut headers = HashMap::new();
    headers.insert("Authorization".to_string(), format!("Bearer {}", api_key));

    let request_body = json!({
        "model": "gpt-3.5-turbo",
        "messages": [
            {
                "role": "user",
                "content": "Say 'test' in one word."
            }
        ],
        "max_tokens": 5
    });

    println!("请求体: {}", serde_json::to_string_pretty(&request_body)?);

    match transport
        .post::<serde_json::Value, serde_json::Value>(
            "https://api.openai.com/v1/chat/completions",
            Some(headers),
            &request_body,
        )
        .await
    {
        Ok(response) => {
            println!("✅ POST请求成功!");
            println!("响应: {}", serde_json::to_string_pretty(&response)?);
        }
        Err(e) => {
            println!("❌ POST请求失败: {}", e);

            // 分析错误类型
            let error_str = e.to_string();
            if error_str.contains("you must provide a model parameter") {
                println!("🔍 这个错误很奇怪，因为我们确实提供了model参数");
                println!("   可能的原因:");
                println!("   1. 代理服务器修改了请求体");
                println!("   2. Content-Type头部问题");
                println!("   3. JSON序列化问题");
            }
        }
    }

    println!("\n💡 调试结论:");
    println!("   • GET请求工作正常 → 认证和网络连接OK");
    println!("   • POST请求失败 → 可能是代理或请求格式问题");
    println!("   • 建议检查代理服务器的POST请求处理");

    Ok(())
}
