/// Model configuration options for explicit model selection
#[derive(Debug, Clone)]
pub struct ModelOptions {
    pub chat_model: Option<String>,
    pub multimodal_model: Option<String>,
    pub fallback_models: Vec<String>,
    pub auto_discovery: bool,
}

impl Default for ModelOptions {
    fn default() -> Self {
        Self {
            chat_model: None,
            multimodal_model: None,
            fallback_models: Vec::new(),
            auto_discovery: true,
        }
    }
}

impl ModelOptions {
    /// Create default model options
    pub fn new() -> Self {
        Self::default()
    }

    /// Set chat model
    pub fn with_chat_model(mut self, model: &str) -> Self {
        self.chat_model = Some(model.to_string());
        self
    }

    /// Set multimodal model
    pub fn with_multimodal_model(mut self, model: &str) -> Self {
        self.multimodal_model = Some(model.to_string());
        self
    }

    /// Set fallback models
    pub fn with_fallback_models(mut self, models: Vec<&str>) -> Self {
        self.fallback_models = models.into_iter().map(|s| s.to_string()).collect();
        self
    }

    /// Enable or disable auto discovery
    pub fn with_auto_discovery(mut self, enabled: bool) -> Self {
        self.auto_discovery = enabled;
        self
    }
}
