//! Manifest CLI工具
//!
//! 提供命令行工具来验证、预览和调试manifest文件。

use crate::manifest::{ManifestLoader, ManifestResult, ManifestValidator};
use clap::{Parser, Subcommand};
use std::path::PathBuf;

/// Manifest CLI工具
#[derive(Parser)]
#[command(name = "ai-lib-manifest")]
#[command(about = "AI-Lib Manifest CLI工具 - 验证、预览和调试manifest文件")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// 验证manifest文件
    Validate {
        /// Manifest文件路径
        #[arg(short, long)]
        file: PathBuf,

        /// 详细输出
        #[arg(short, long)]
        verbose: bool,
    },

    /// 预览payload构建
    Preview {
        /// Manifest文件路径
        #[arg(short, long)]
        file: PathBuf,

        /// Provider ID
        #[arg(short, long)]
        provider: String,

        /// Model ID
        #[arg(short, long)]
        model: String,

        /// 输出格式 (json/yaml)
        #[arg(short, long, default_value = "json")]
        format: String,
    },

    /// 显示manifest信息
    Info {
        /// Manifest文件路径
        #[arg(short, long)]
        file: PathBuf,
    },

    /// 导出JSON Schema用于编辑器支持
    /// 这实现了"Code-First"验证方式，Rust struct是唯一的真理来源
    ExportSchema {
        /// 输出文件路径（可选，默认输出到stdout）
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
}

/// CLI执行器
pub struct CliRunner;

impl CliRunner {
    /// 执行CLI命令
    pub fn run(cli: Cli) -> ManifestResult<()> {
        match cli.command {
            Commands::Validate { file, verbose } => Self::validate_manifest(file, verbose),
            Commands::Preview {
                file,
                provider,
                model,
                format,
            } => Self::preview_payload(file, provider, model, format),
            Commands::Info { file } => Self::show_manifest_info(file),
            Commands::ExportSchema { output } => Self::export_schema(output),
        }
    }

    /// 验证manifest文件
    fn validate_manifest(file: PathBuf, verbose: bool) -> ManifestResult<()> {
        println!("🔍 验证manifest文件: {}", file.display());

        // 加载manifest
        let manifest = ManifestLoader::load_from_file(file)?;

        // 验证manifest
        ManifestValidator::validate_manifest(&manifest)?;

        println!("✅ Manifest验证成功！");
        println!("📊 版本: {}", manifest.version);
        println!("🏢 提供商数量: {}", manifest.providers.len());
        println!("🤖 模型数量: {}", manifest.models.len());

        if verbose {
            println!("\n📋 提供商列表:");
            for (id, provider) in &manifest.providers {
                println!("  • {} (v{})", id, provider.version);
            }

            println!("\n🤖 模型列表:");
            for (id, model) in &manifest.models {
                println!(
                    "  • {} ({}) - {}",
                    id,
                    model.provider,
                    model.display_name.as_deref().unwrap_or("未命名")
                );
            }
        }

        Ok(())
    }

    /// 预览payload构建
    fn preview_payload(
        file: PathBuf,
        provider: String,
        model: String,
        format: String,
    ) -> ManifestResult<()> {
        println!("🔍 预览payload构建");
        println!("📁 Manifest: {}", file.display());
        println!("🏢 Provider: {}", provider);
        println!("🤖 Model: {}", model);
        println!("📄 Format: {}", format);

        // 加载manifest
        let manifest = ManifestLoader::load_from_file(file)?;

        // 验证provider和model存在
        if !manifest.providers.contains_key(&provider) {
            eprintln!("❌ Provider '{}' 未在manifest中定义", provider);
            std::process::exit(1);
        }

        if !manifest.models.contains_key(&model) {
            eprintln!("❌ Model '{}' 未在manifest中定义", model);
            std::process::exit(1);
        }

        let model_def = &manifest.models[&model];
        if model_def.provider != provider {
            eprintln!("❌ Model '{}' 不属于provider '{}'", model, provider);
            std::process::exit(1);
        }

        // 创建示例请求
        let example_request = create_example_request(&model);

        // 这里应该调用PayloadBuilder来生成payload
        // 暂时输出示例结构
        println!("\n📤 示例请求结构:");
        match format.as_str() {
            "json" => {
                let json = serde_json::to_string_pretty(&example_request)
                    .map_err(|e| crate::manifest::ManifestError::ValidationError(e.to_string()))?;
                println!("{}", json);
            }
            "yaml" => {
                let yaml = serde_yaml::to_string(&example_request)
                    .map_err(|e| crate::manifest::ManifestError::ValidationError(e.to_string()))?;
                println!("{}", yaml);
            }
            _ => {
                eprintln!("❌ 不支持的格式: {}", format);
                std::process::exit(1);
            }
        }

        println!("\n💡 注意: PayloadBuilder实现将在Phase 1完成");

        Ok(())
    }

    /// 显示manifest信息
    fn show_manifest_info(file: PathBuf) -> ManifestResult<()> {
        println!("📋 Manifest信息: {}", file.display());

        let manifest = ManifestLoader::load_from_file(file)?;

        println!("📊 基本信息:");
        println!("  版本: {}", manifest.version);
        println!(
            "  描述: {}",
            manifest.metadata.description.as_deref().unwrap_or("无")
        );
        println!("  作者: {}", manifest.metadata.authors.join(", "));
        println!(
            "  更新时间: {}",
            manifest.metadata.last_updated.as_deref().unwrap_or("未知")
        );

        println!("\n🏢 提供商统计:");
        println!("  总数: {}", manifest.providers.len());

        let mut capabilities_count = std::collections::HashMap::new();
        for provider in manifest.providers.values() {
            for cap in &provider.capabilities {
                *capabilities_count.entry(cap.clone()).or_insert(0) += 1;
            }
        }

        println!("  能力分布:");
        for (cap, count) in capabilities_count {
            println!("    • {:?}: {}", cap, count);
        }

        println!("\n🤖 模型统计:");
        println!("  总数: {}", manifest.models.len());

        let mut provider_models = std::collections::HashMap::new();
        for model in manifest.models.values() {
            *provider_models.entry(model.provider.clone()).or_insert(0) += 1;
        }

        println!("  按提供商分布:");
        for (provider, count) in provider_models {
            println!("    • {}: {} 个模型", provider, count);
        }

        println!("\n🎯 2025年特性支持:");

        // 检查agentic loop支持
        let agentic_supported = manifest.standard_schema.agentic_loop.is_some();
        println!(
            "  • Agentic Loop: {}",
            if agentic_supported { "✅" } else { "❌" }
        );

        // 检查streaming events支持
        let streaming_supported = manifest.standard_schema.streaming_events.is_some();
        println!(
            "  • Streaming Events: {}",
            if streaming_supported { "✅" } else { "❌" }
        );

        // 检查工具映射支持
        let tools_mapping_count = manifest
            .providers
            .values()
            .filter(|p| p.tools_mapping.is_some())
            .count();
        println!("  • Tools Mapping: {} 个提供商", tools_mapping_count);

        // 检查prompt caching支持
        let prompt_caching_count = manifest
            .providers
            .values()
            .filter(|p| {
                p.prompt_caching
                    .as_ref()
                    .map(|c| c.enabled)
                    .unwrap_or(false)
            })
            .count();
        println!("  • Prompt Caching: {} 个提供商", prompt_caching_count);

        Ok(())
    }

    /// 导出JSON Schema
    /// 实现"Code-First"验证方式：Rust struct -> JSON Schema -> 编辑器支持
    fn export_schema(output: Option<PathBuf>) -> ManifestResult<()> {
        use crate::manifest::export_json_schema;

        println!("📋 导出Manifest JSON Schema");
        println!("🎯 验证方式: Code-First (Rust struct是唯一的真理来源)");

        let schema_json = export_json_schema();

        match output {
            Some(path) => {
                std::fs::write(&path, &schema_json)?;
                println!("✅ JSON Schema已导出到: {}", path.display());
                println!("💡 在YAML文件顶部添加: #$schema: {}", path.display());
            }
            None => {
                // 输出到stdout
                println!("📄 JSON Schema内容:");
                println!("{}", schema_json);
                println!("\n💡 复制以上内容保存为schema.json，然后在YAML文件顶部添加:");
                println!("   #$schema: ./schema.json");
            }
        }

        println!("\n🎉 这将为VS Code等编辑器提供完整的自动补全和验证支持！");

        Ok(())
    }
}

/// 创建示例请求（用于预览）
fn create_example_request(model: &str) -> serde_json::Value {
    serde_json::json!({
        "model": model,
        "messages": [
            {
                "role": "system",
                "content": "You are a helpful assistant."
            },
            {
                "role": "user",
                "content": "Hello! How can I help you today?"
            }
        ],
        "temperature": 0.7,
        "max_tokens": 1000,
        "stream": false,
        "tools": [
            {
                "id": "weather_tool",
                "name": "get_weather",
                "description": "Get current weather for a location",
                "input_schema": {
                    "type": "object",
                    "properties": {
                        "location": {
                            "type": "string",
                            "description": "The city and state, e.g. San Francisco, CA"
                        }
                    },
                    "required": ["location"]
                }
            }
        ]
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_validate_command() {
        // 创建临时manifest文件
        let mut temp_file = NamedTempFile::new().unwrap();
        let manifest_content = r#"
version: "1.1"
metadata:
  description: "Test Manifest"
  authors: ["Test"]
standard_schema:
  parameters:
    temperature:
      type: float
      range: [0.0, 2.0]
  tools:
    schema: "standard"
    choice_policy: ["auto"]
    strict_mode: false
    parallel_calls: false
  response_format:
    types: ["text"]
    schema_validation: false
providers:
  test_provider:
    version: "v1"
    base_url: "https://api.test.com"
    auth:
      type: bearer
      token_env: "TEST_KEY"
    payload_format: "openai_style"
    parameter_mappings:
      temperature: "temperature"
    response_format: "openai_style"
    response_paths:
      content: "choices[0].message.content"
models:
  test_model:
    provider: "test_provider"
    model_id: "test-model"
    context_window: 4096
    capabilities: ["chat"]
"#;
        temp_file.write_all(manifest_content.as_bytes()).unwrap();

        // 测试验证命令
        let cli = Cli {
            command: Commands::Validate {
                file: temp_file.path().to_path_buf(),
                verbose: false,
            },
        };

        let result = CliRunner::run(cli);
        assert!(result.is_ok());
    }
}
