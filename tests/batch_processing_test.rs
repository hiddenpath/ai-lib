#[cfg(test)]
mod tests {
    use ai_lib::metrics::NoopMetrics;
    use ai_lib::provider::config::ProviderConfig;
    use ai_lib::provider::generic::GenericAdapter;
    use ai_lib::types::common::Content;
    use ai_lib::types::{ChatCompletionRequest, Message, Role};
    use std::sync::Arc;

    // Include test utility mock transport
    mod utils {
        include!("utils/mock_transport.rs");
    }

    #[tokio::test]
    async fn test_batch_processing_concurrent() {
        use ai_lib::batch_utils::process_batch_concurrent;
        use serde_json::json;

        // Mock response
        let post_resp = json!({
            "id": "test-batch",
            "object": "chat.completion",
            "created": 0,
            "model": "test-model",
            "choices": [{ "message": { "role": "assistant", "content": "test response" }, "finish_reason": null }],
            "usage": { "prompt_tokens": 10, "completion_tokens": 5, "total_tokens": 15 }
        });

        let transport = Arc::new(utils::MockTransport::new(post_resp));
        let config =
            ProviderConfig::openai_compatible("http://example", "API_KEY", "gpt-3.5-turbo", None);
        let metrics = Arc::new(NoopMetrics::new());
        let adapter =
            GenericAdapter::with_transport_ref_and_metrics(config, transport, metrics).unwrap();

        // Create multiple requests
        let requests = vec![
            ChatCompletionRequest::new(
                "test-model".to_string(),
                vec![Message {
                    role: Role::User,
                    content: Content::Text("Request 1".to_string()),
                    function_call: None,
                }],
            ),
            ChatCompletionRequest::new(
                "test-model".to_string(),
                vec![Message {
                    role: Role::User,
                    content: Content::Text("Request 2".to_string()),
                    function_call: None,
                }],
            ),
            ChatCompletionRequest::new(
                "test-model".to_string(),
                vec![Message {
                    role: Role::User,
                    content: Content::Text("Request 3".to_string()),
                    function_call: None,
                }],
            ),
        ];

        // Test concurrent processing with limit
        let results = process_batch_concurrent(&adapter, requests, Some(2))
            .await
            .unwrap();

        assert_eq!(results.len(), 3);
        assert!(results.iter().all(|r| r.is_ok()));
    }

    #[tokio::test]
    async fn test_batch_processing_sequential() {
        use ai_lib::batch_utils::process_batch_sequential;
        use serde_json::json;

        // Mock response
        let post_resp = json!({
            "id": "test-batch-seq",
            "object": "chat.completion",
            "created": 0,
            "model": "test-model",
            "choices": [{ "message": { "role": "assistant", "content": "test response" }, "finish_reason": null }],
            "usage": { "prompt_tokens": 10, "completion_tokens": 5, "total_tokens": 15 }
        });

        let transport = Arc::new(utils::MockTransport::new(post_resp));
        let config =
            ProviderConfig::openai_compatible("http://example", "API_KEY", "gpt-3.5-turbo", None);
        let metrics = Arc::new(NoopMetrics::new());
        let adapter =
            GenericAdapter::with_transport_ref_and_metrics(config, transport, metrics).unwrap();

        // Create multiple requests
        let requests = vec![
            ChatCompletionRequest::new(
                "test-model".to_string(),
                vec![Message {
                    role: Role::User,
                    content: Content::Text("Request 1".to_string()),
                    function_call: None,
                }],
            ),
            ChatCompletionRequest::new(
                "test-model".to_string(),
                vec![Message {
                    role: Role::User,
                    content: Content::Text("Request 2".to_string()),
                    function_call: None,
                }],
            ),
        ];

        // Test sequential processing
        let results = process_batch_sequential(&adapter, requests).await.unwrap();

        assert_eq!(results.len(), 2);
        assert!(results.iter().all(|r| r.is_ok()));
    }

    #[tokio::test]
    async fn test_batch_processing_smart() {
        use ai_lib::batch_utils::process_batch_smart;
        use serde_json::json;

        // Mock response
        let post_resp = json!({
            "id": "test-batch-smart",
            "object": "chat.completion",
            "created": 0,
            "model": "test-model",
            "choices": [{ "message": { "role": "assistant", "content": "test response" }, "finish_reason": null }],
            "usage": { "prompt_tokens": 10, "completion_tokens": 5, "total_tokens": 15 }
        });

        let transport = Arc::new(utils::MockTransport::new(post_resp));
        let config =
            ProviderConfig::openai_compatible("http://example", "API_KEY", "gpt-3.5-turbo", None);
        let metrics = Arc::new(NoopMetrics::new());
        let adapter =
            GenericAdapter::with_transport_ref_and_metrics(config, transport, metrics).unwrap();

        // Test small batch (should use sequential)
        let small_requests = vec![ChatCompletionRequest::new(
            "test-model".to_string(),
            vec![Message {
                role: Role::User,
                content: Content::Text("Small request".to_string()),
                function_call: None,
            }],
        )];

        let results = process_batch_smart(&adapter, small_requests, Some(5))
            .await
            .unwrap();
        assert_eq!(results.len(), 1);
        assert!(results.iter().all(|r| r.is_ok()));

        // Test large batch (should use concurrent)
        let large_requests = vec![
            ChatCompletionRequest::new(
                "test-model".to_string(),
                vec![Message {
                    role: Role::User,
                    content: Content::Text("Large request 1".to_string()),
                    function_call: None,
                }],
            ),
            ChatCompletionRequest::new(
                "test-model".to_string(),
                vec![Message {
                    role: Role::User,
                    content: Content::Text("Large request 2".to_string()),
                    function_call: None,
                }],
            ),
            ChatCompletionRequest::new(
                "test-model".to_string(),
                vec![Message {
                    role: Role::User,
                    content: Content::Text("Large request 3".to_string()),
                    function_call: None,
                }],
            ),
            ChatCompletionRequest::new(
                "test-model".to_string(),
                vec![Message {
                    role: Role::User,
                    content: Content::Text("Large request 4".to_string()),
                    function_call: None,
                }],
            ),
        ];

        let results = process_batch_smart(&adapter, large_requests, Some(5))
            .await
            .unwrap();
        assert_eq!(results.len(), 4);
        assert!(results.iter().all(|r| r.is_ok()));
    }

    #[tokio::test]
    async fn test_batch_result_operations() {
        use ai_lib::types::ChatCompletionResponse;
        use ai_lib::BatchResult;

        let mut batch_result = BatchResult::new(5);

        // Test initial state
        assert_eq!(batch_result.total_requests, 5);
        assert_eq!(batch_result.total_successful, 0);
        assert_eq!(batch_result.total_failed, 0);
        assert_eq!(batch_result.success_rate(), 0.0);

        // Add successful responses
        let mock_response = ChatCompletionResponse {
            id: "test".to_string(),
            object: "chat.completion".to_string(),
            created: 0,
            model: "test-model".to_string(),
            choices: vec![],
            usage: ai_lib::types::Usage {
                prompt_tokens: 10,
                completion_tokens: 5,
                total_tokens: 15,
            },
            usage_status: ai_lib::types::UsageStatus::Finalized,
        };

        batch_result.add_success(mock_response.clone());
        batch_result.add_success(mock_response.clone());
        batch_result.add_success(mock_response);

        // Add failed responses
        batch_result.add_failure(
            3,
            ai_lib::types::AiLibError::NetworkError("test error".to_string()),
        );
        batch_result.add_failure(
            4,
            ai_lib::types::AiLibError::TimeoutError("timeout".to_string()),
        );

        // Test final state
        assert_eq!(batch_result.total_successful, 3);
        assert_eq!(batch_result.total_failed, 2);
        assert_eq!(batch_result.success_rate(), 60.0);
        assert!(!batch_result.all_successful());
    }

    #[tokio::test]
    async fn test_empty_batch() {
        use ai_lib::batch_utils::process_batch_concurrent;
        use ai_lib::batch_utils::process_batch_sequential;
        use ai_lib::batch_utils::process_batch_smart;
        use serde_json::json;

        // Mock response
        let post_resp = json!({
            "id": "test-empty",
            "object": "chat.completion",
            "created": 0,
            "model": "test-model",
            "choices": [{ "message": { "role": "assistant", "content": "test response" }, "finish_reason": null }],
            "usage": { "prompt_tokens": 10, "completion_tokens": 5, "total_tokens": 15 }
        });

        let transport = Arc::new(utils::MockTransport::new(post_resp));
        let config =
            ProviderConfig::openai_compatible("http://example", "API_KEY", "gpt-3.5-turbo", None);
        let metrics = Arc::new(NoopMetrics::new());
        let adapter =
            GenericAdapter::with_transport_ref_and_metrics(config, transport, metrics).unwrap();

        let empty_requests: Vec<ChatCompletionRequest> = vec![];

        // Test all methods with empty batch
        let concurrent_results =
            process_batch_concurrent(&adapter, empty_requests.clone(), Some(5))
                .await
                .unwrap();
        assert_eq!(concurrent_results.len(), 0);

        let sequential_results = process_batch_sequential(&adapter, empty_requests.clone())
            .await
            .unwrap();
        assert_eq!(sequential_results.len(), 0);

        let smart_results = process_batch_smart(&adapter, empty_requests, Some(5))
            .await
            .unwrap();
        assert_eq!(smart_results.len(), 0);
    }
}
