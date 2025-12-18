use ai_lib::provider::config::ProviderConfig;

#[test]
fn test_valid_config() {
    let config = ProviderConfig::openai_compatible(
        "https://api.example.com",
        "API_KEY",
        "gpt-3.5-turbo",
        None,
    );
    assert!(config.validate().is_ok());
}

#[test]
fn test_empty_base_url() {
    let mut config = ProviderConfig::openai_compatible(
        "https://api.example.com",
        "API_KEY",
        "gpt-3.5-turbo",
        None,
    );
    config.base_url = String::new();
    assert!(config.validate().is_err());
}

#[test]
fn test_invalid_url_scheme() {
    let mut config = ProviderConfig::openai_compatible(
        "https://api.example.com",
        "API_KEY",
        "gpt-3.5-turbo",
        None,
    );
    config.base_url = "ftp://api.example.com".to_string();
    let result = config.validate();
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("HTTP/HTTPS URL"));
}

#[test]
fn test_trailing_slash_in_base_url() {
    let mut config = ProviderConfig::openai_compatible(
        "https://api.example.com",
        "API_KEY",
        "gpt-3.5-turbo",
        None,
    );
    config.base_url = "https://api.example.com/".to_string();
    let result = config.validate();
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("trailing slash"));
}

#[test]
fn test_endpoint_without_leading_slash() {
    let mut config = ProviderConfig::openai_compatible(
        "https://api.example.com",
        "API_KEY",
        "gpt-3.5-turbo",
        None,
    );
    config.chat_endpoint = "chat/completions".to_string();
    let result = config.validate();
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("must start with /"));
}

#[test]
fn test_empty_chat_model() {
    let mut config = ProviderConfig::openai_compatible(
        "https://api.example.com",
        "API_KEY",
        "gpt-3.5-turbo",
        None,
    );
    config.chat_model = String::new();
    let result = config.validate();
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("chat_model cannot be empty"));
}

#[test]
fn test_empty_api_key_env() {
    let mut config = ProviderConfig::openai_compatible(
        "https://api.example.com",
        "API_KEY",
        "gpt-3.5-turbo",
        None,
    );
    config.api_key_env = String::new();
    let result = config.validate();
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("api_key_env cannot be empty"));
}

#[test]
fn test_upload_endpoint_validation() {
    let mut config = ProviderConfig::openai_compatible(
        "https://api.example.com",
        "API_KEY",
        "gpt-3.5-turbo",
        None,
    );
    config.upload_endpoint = Some("files".to_string());
    let result = config.validate();
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("upload_endpoint must start with /"));
}

#[test]
fn test_models_endpoint_validation() {
    let mut config = ProviderConfig::openai_compatible(
        "https://api.example.com",
        "API_KEY",
        "gpt-3.5-turbo",
        None,
    );
    config.models_endpoint = Some("models".to_string());
    let result = config.validate();
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("models_endpoint must start with /"));
}
