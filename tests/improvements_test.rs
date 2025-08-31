#[cfg(test)]
mod tests {
    use ai_lib::metrics::{Metrics, MetricsExt, NoopMetrics};
    use ai_lib::provider::config::{FieldMapping, ProviderConfig};
    use ai_lib::types::AiLibError;
    use ai_lib::utils::file::{
        get_file_extension, is_audio_file, is_image_file, is_text_file, is_video_file,
    };
    use std::collections::HashMap;
    use std::path::Path;

    #[test]
    fn test_provider_config_validation() {
        // Test valid configuration
        let valid_config = ProviderConfig::openai_compatible(
            "https://api.openai.com",
            "OPENAI_API_KEY",
            "gpt-3.5-turbo",
            None,
        );
        assert!(valid_config.validate().is_ok());

        // Test invalid base_url
        let mut invalid_config =
            ProviderConfig::openai_compatible("", "OPENAI_API_KEY", "gpt-3.5-turbo", None);
        assert!(invalid_config.validate().is_err());

        let mut invalid_config = ProviderConfig::openai_compatible(
            "ftp://invalid.com",
            "OPENAI_API_KEY",
            "gpt-3.5-turbo",
            None,
        );
        assert!(invalid_config.validate().is_err());

        // Test invalid api_key_env
        let mut invalid_config =
            ProviderConfig::openai_compatible("https://api.openai.com", "", "gpt-3.5-turbo", None);
        assert!(invalid_config.validate().is_err());

        // Test invalid chat_endpoint
        let mut invalid_config = ProviderConfig::openai_compatible(
            "https://api.openai.com",
            "OPENAI_API_KEY",
            "gpt-3.5-turbo",
            None,
        );
        invalid_config.chat_endpoint = "".to_string();
        assert!(invalid_config.validate().is_err());
    }

    #[test]
    fn test_field_mapping_validation() {
        // Test valid field mapping
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

        // Test invalid field mapping
        let invalid_field_mapping = FieldMapping {
            messages_field: "".to_string(),
            model_field: "model".to_string(),
            role_mapping: role_mapping.clone(),
            response_content_path: "choices.0.message.content".to_string(),
        };
        assert!(invalid_field_mapping.validate().is_err());

        // Test missing required roles
        let mut incomplete_role_mapping = HashMap::new();
        incomplete_role_mapping.insert("System".to_string(), "system".to_string());
        incomplete_role_mapping.insert("User".to_string(), "user".to_string());
        // Missing Assistant role

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
        let config = ProviderConfig::openai_compatible(
            "https://api.openai.com",
            "OPENAI_API_KEY",
            "gpt-3.5-turbo",
            None,
        );

        assert_eq!(config.chat_url(), "https://api.openai.com/chat/completions");
        assert_eq!(
            config.models_url(),
            Some("https://api.openai.com/models".to_string())
        );
        assert_eq!(
            config.upload_url(),
            Some("https://api.openai.com/v1/files".to_string())
        );
    }

    #[test]
    fn test_enhanced_metrics() {
        let metrics = NoopMetrics::new();

        // Test basic metrics methods
        let _ = tokio::runtime::Runtime::new().unwrap().block_on(async {
            metrics.incr_counter("test.counter", 1).await;
            metrics.record_gauge("test.gauge", 42.0).await;
            metrics.record_histogram("test.histogram", 10.5).await;
            metrics.record_error("test.error", "validation_error").await;
            metrics.record_success("test.success", true).await;

            // Test metrics with tags
            metrics
                .incr_counter_with_tags("test.counter", 1, &[("tag1", "value1")])
                .await;
            metrics
                .record_gauge_with_tags("test.gauge", 42.0, &[("tag1", "value1")])
                .await;
            metrics
                .record_histogram_with_tags("test.histogram", 10.5, &[("tag1", "value1")])
                .await;
        });
    }

    #[test]
    fn test_metrics_ext() {
        let metrics = NoopMetrics::new();

        let _ = tokio::runtime::Runtime::new().unwrap().block_on(async {
            // Test convenience methods
            metrics.record_request("test.request", None, true).await;
            metrics
                .record_error_with_context("test.error", "network_error", "connection_failed")
                .await;

            // Test methods with tags
            metrics
                .record_request_with_tags("test.request", None, true, &[("status", "success")])
                .await;
            metrics
                .record_error_with_context("test.error", "network_error", "connection_failed")
                .await;
        });
    }

    #[test]
    fn test_file_utils() {
        // Test file type detection
        assert!(is_image_file(Path::new("image.png")));
        assert!(is_image_file(Path::new("IMAGE.JPG")));
        assert!(!is_image_file(Path::new("document.txt")));

        assert!(is_audio_file(Path::new("audio.mp3")));
        assert!(is_audio_file(Path::new("AUDIO.WAV")));
        assert!(!is_audio_file(Path::new("image.png")));

        assert!(is_video_file(Path::new("video.mp4")));
        assert!(is_video_file(Path::new("VIDEO.AVI")));
        assert!(!is_video_file(Path::new("audio.mp3")));

        assert!(is_text_file(Path::new("document.txt")));
        assert!(is_text_file(Path::new("code.rs")));
        assert!(!is_text_file(Path::new("image.png")));

        // Test file extension extraction
        assert_eq!(
            get_file_extension(Path::new("file.txt")),
            Some("txt".to_string())
        );
        assert_eq!(
            get_file_extension(Path::new("IMAGE.PNG")),
            Some("png".to_string())
        );
        assert_eq!(get_file_extension(Path::new("noextension")), None);
    }

    #[test]
    fn test_error_handling() {
        // Test error creation and conversion
        let network_error = AiLibError::NetworkError("connection failed".to_string());
        assert!(network_error.is_retryable());

        let auth_error = AiLibError::AuthenticationError("invalid key".to_string());
        assert!(!auth_error.is_retryable());

        let timeout_error = AiLibError::TimeoutError("request timeout".to_string());
        assert!(timeout_error.is_retryable());
    }
}
