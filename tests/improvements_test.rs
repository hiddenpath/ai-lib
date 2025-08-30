#[cfg(test)]
mod tests {
    use ai_lib::provider::config::{ProviderConfig, FieldMapping};
    use ai_lib::types::AiLibError;
    use ai_lib::metrics::{Metrics, NoopMetrics, MetricsExt};
    use ai_lib::utils::file::{is_image_file, is_audio_file, is_video_file, is_text_file, get_file_extension};
    use std::collections::HashMap;
    use std::path::Path;

    #[test]
    fn test_provider_config_validation() {
        // 测试有效配置
        let valid_config = ProviderConfig::openai_compatible("https://api.openai.com", "OPENAI_API_KEY");
        assert!(valid_config.validate().is_ok());

        // 测试无效base_url
        let mut invalid_config = ProviderConfig::openai_compatible("", "OPENAI_API_KEY");
        assert!(invalid_config.validate().is_err());

        let mut invalid_config = ProviderConfig::openai_compatible("ftp://invalid.com", "OPENAI_API_KEY");
        assert!(invalid_config.validate().is_err());

        // 测试无效api_key_env
        let mut invalid_config = ProviderConfig::openai_compatible("https://api.openai.com", "");
        assert!(invalid_config.validate().is_err());

        // 测试无效chat_endpoint
        let mut invalid_config = ProviderConfig::openai_compatible("https://api.openai.com", "OPENAI_API_KEY");
        invalid_config.chat_endpoint = "".to_string();
        assert!(invalid_config.validate().is_err());
    }

    #[test]
    fn test_field_mapping_validation() {
        // 测试有效字段映射
        let mut role_mapping = HashMap::new();
        role_mapping.insert("System".to_string(), "system".to_string());
        role_mapping.insert("User".to_string(), "user".to_string());
        role_mapping.insert("Assistant".to_string(), "assistant".to_string());

        let valid_field_mapping = FieldMapping {
            messages_field: "messages".to_string(),
            model_field: "model".to_string(),
            role_mapping: role_mapping.clone(),
            response_content_path: "choices.0.message.content".to_string(),
        };
        assert!(valid_field_mapping.validate().is_ok());

        // 测试无效字段映射
        let invalid_field_mapping = FieldMapping {
            messages_field: "".to_string(),
            model_field: "model".to_string(),
            role_mapping: role_mapping.clone(),
            response_content_path: "choices.0.message.content".to_string(),
        };
        assert!(invalid_field_mapping.validate().is_err());

        // 测试缺少必需角色
        let mut incomplete_role_mapping = HashMap::new();
        incomplete_role_mapping.insert("System".to_string(), "system".to_string());
        incomplete_role_mapping.insert("User".to_string(), "user".to_string());
        // 缺少 Assistant 角色

        let invalid_field_mapping = FieldMapping {
            messages_field: "messages".to_string(),
            model_field: "model".to_string(),
            role_mapping: incomplete_role_mapping,
            response_content_path: "choices.0.message.content".to_string(),
        };
        assert!(invalid_field_mapping.validate().is_err());
    }

    #[test]
    fn test_config_url_methods() {
        let config = ProviderConfig::openai_compatible("https://api.openai.com", "OPENAI_API_KEY");
        
        assert_eq!(config.chat_url(), "https://api.openai.com/chat/completions");
        assert_eq!(config.models_url(), Some("https://api.openai.com/models".to_string()));
        assert_eq!(config.upload_url(), Some("https://api.openai.com/v1/files".to_string()));
    }

    #[test]
    fn test_enhanced_metrics() {
        let metrics = NoopMetrics::new();
        
        // 测试基础指标方法
        let _ = tokio::runtime::Runtime::new().unwrap().block_on(async {
            metrics.incr_counter("test.counter", 1).await;
            metrics.record_gauge("test.gauge", 42.0).await;
            metrics.record_histogram("test.histogram", 10.5).await;
            metrics.record_error("test.error", "validation_error").await;
            metrics.record_success("test.success", true).await;
            
            // 测试带标签的指标
            metrics.incr_counter_with_tags("test.counter", 1, &[("tag1", "value1")]).await;
            metrics.record_gauge_with_tags("test.gauge", 42.0, &[("tag1", "value1")]).await;
            metrics.record_histogram_with_tags("test.histogram", 10.5, &[("tag1", "value1")]).await;
        });
    }

    #[test]
    fn test_metrics_ext() {
        let metrics = NoopMetrics::new();
        
        let _ = tokio::runtime::Runtime::new().unwrap().block_on(async {
            // 测试便捷方法
            metrics.record_request("test.request", None, true).await;
            metrics.record_request_with_tags("test.request", None, true, &[("status", "success")]).await;
            metrics.record_error_with_context("test.error", "network_error", "connection_failed").await;
        });
    }

    #[test]
    fn test_file_utilities() {
        // 测试文件类型检测
        assert!(is_image_file(Path::new("image.png")));
        assert!(is_image_file(Path::new("photo.JPG")));
        assert!(!is_image_file(Path::new("document.txt")));

        assert!(is_audio_file(Path::new("music.mp3")));
        assert!(is_audio_file(Path::new("sound.WAV")));
        assert!(!is_audio_file(Path::new("image.png")));

        assert!(is_video_file(Path::new("movie.mp4")));
        assert!(is_video_file(Path::new("clip.AVI")));
        assert!(!is_video_file(Path::new("music.mp3")));

        assert!(is_text_file(Path::new("readme.md")));
        assert!(is_text_file(Path::new("config.json")));
        assert!(is_text_file(Path::new("script.py")));
        assert!(!is_text_file(Path::new("image.png")));

        // 测试文件扩展名获取
        assert_eq!(get_file_extension(Path::new("file.txt")), Some("txt".to_string()));
        assert_eq!(get_file_extension(Path::new("IMAGE.PNG")), Some("png".to_string()));
        assert_eq!(get_file_extension(Path::new("noextension")), None);
    }

    #[test]
    fn test_error_handling() {
        // 测试错误类型判断
        let config_error = AiLibError::ConfigurationError("Invalid config".to_string());
        assert!(config_error.is_config_error());
        assert!(!config_error.is_auth_error());
        assert!(!config_error.is_request_error());

        let auth_error = AiLibError::AuthenticationError("Invalid token".to_string());
        assert!(auth_error.is_auth_error());
        assert!(!auth_error.is_config_error());

        let request_error = AiLibError::InvalidRequest("Bad request".to_string());
        assert!(request_error.is_request_error());
        assert!(!request_error.is_config_error());

        // 测试错误上下文
        assert_eq!(config_error.context(), "Configuration validation failed");
        assert_eq!(auth_error.context(), "Authentication failed");
        assert_eq!(request_error.context(), "Invalid request parameters");

        // 测试重试判断
        assert!(!config_error.is_retryable());
        assert!(!auth_error.is_retryable());
        assert!(!request_error.is_retryable());

        let network_error = AiLibError::NetworkError("Connection failed".to_string());
        assert!(network_error.is_retryable());
        assert_eq!(network_error.retry_delay_ms(), 1000);
    }
}
