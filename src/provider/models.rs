use crate::types::AiLibError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

/// Model information structure for custom model management
///
/// This struct provides detailed information about AI models,
/// allowing developers to build custom model managers and arrays.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    /// Model name/identifier
    pub name: String,
    /// Display name for user interface
    pub display_name: String,
    /// Model description
    pub description: String,
    /// Model capabilities
    pub capabilities: ModelCapabilities,
    /// Pricing information
    pub pricing: PricingInfo,
    /// Performance metrics
    pub performance: PerformanceMetrics,
    /// Provider-specific metadata
    pub metadata: HashMap<String, String>,
}

/// Model capabilities enumeration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelCapabilities {
    /// Chat capabilities
    pub chat: bool,
    /// Code generation capabilities
    pub code_generation: bool,
    /// Multimodal capabilities (text + image/audio)
    pub multimodal: bool,
    /// Function calling capabilities
    pub function_calling: bool,
    /// Tool use capabilities
    pub tool_use: bool,
    /// Multilingual support
    pub multilingual: bool,
    /// Context window size in tokens
    pub context_window: Option<u32>,
}

impl ModelCapabilities {
    /// Create new capabilities with default values
    pub fn new() -> Self {
        Self {
            chat: true,
            code_generation: false,
            multimodal: false,
            function_calling: false,
            tool_use: false,
            multilingual: false,
            context_window: None,
        }
    }

    /// Enable chat capabilities
    pub fn with_chat(mut self) -> Self {
        self.chat = true;
        self
    }

    /// Enable code generation capabilities
    pub fn with_code_generation(mut self) -> Self {
        self.code_generation = true;
        self
    }

    /// Enable multimodal capabilities
    pub fn with_multimodal(mut self) -> Self {
        self.multimodal = true;
        self
    }

    /// Enable function calling capabilities
    pub fn with_function_calling(mut self) -> Self {
        self.function_calling = true;
        self
    }

    /// Enable tool use capabilities
    pub fn with_tool_use(mut self) -> Self {
        self.tool_use = true;
        self
    }

    /// Enable multilingual support
    pub fn with_multilingual(mut self) -> Self {
        self.multilingual = true;
        self
    }

    /// Set context window size
    pub fn with_context_window(mut self, size: u32) -> Self {
        self.context_window = Some(size);
        self
    }

    /// Check if model supports a specific capability
    pub fn supports(&self, capability: &str) -> bool {
        match capability {
            "chat" => self.chat,
            "code_generation" => self.code_generation,
            "multimodal" => self.multimodal,
            "function_calling" => self.function_calling,
            "tool_use" => self.tool_use,
            "multilingual" => self.multilingual,
            _ => false,
        }
    }
}

/// Pricing information for models
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PricingInfo {
    /// Cost per input token (in USD)
    pub input_cost_per_1k: f64,
    /// Cost per output token (in USD)
    pub output_cost_per_1k: f64,
    /// Currency (default: USD)
    pub currency: String,
}

impl PricingInfo {
    /// Create new pricing information
    pub fn new(input_cost_per_1k: f64, output_cost_per_1k: f64) -> Self {
        Self {
            input_cost_per_1k,
            output_cost_per_1k,
            currency: "USD".to_string(),
        }
    }

    /// Set custom currency
    pub fn with_currency(mut self, currency: &str) -> Self {
        self.currency = currency.to_string();
        self
    }

    /// Calculate cost for a given number of tokens
    pub fn calculate_cost(&self, input_tokens: u32, output_tokens: u32) -> f64 {
        let input_cost = (input_tokens as f64 / 1000.0) * self.input_cost_per_1k;
        let output_cost = (output_tokens as f64 / 1000.0) * self.output_cost_per_1k;
        input_cost + output_cost
    }
}

/// Performance metrics for models
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    /// Speed tier classification
    pub speed: SpeedTier,
    /// Quality tier classification
    pub quality: QualityTier,
    /// Average response time
    pub avg_response_time: Option<Duration>,
    /// Throughput (requests per second)
    pub throughput: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SpeedTier {
    Fast,
    Balanced,
    Slow,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QualityTier {
    Basic,
    Good,
    Excellent,
}

impl PerformanceMetrics {
    /// Create new performance metrics
    pub fn new() -> Self {
        Self {
            speed: SpeedTier::Balanced,
            quality: QualityTier::Good,
            avg_response_time: None,
            throughput: None,
        }
    }

    /// Set speed tier
    pub fn with_speed(mut self, speed: SpeedTier) -> Self {
        self.speed = speed;
        self
    }

    /// Set quality tier
    pub fn with_quality(mut self, quality: QualityTier) -> Self {
        self.quality = quality;
        self
    }

    /// Set average response time
    pub fn with_avg_response_time(mut self, time: Duration) -> Self {
        self.avg_response_time = Some(time);
        self
    }

    /// Set throughput
    pub fn with_throughput(mut self, tps: f64) -> Self {
        self.throughput = Some(tps);
        self
    }
}

/// Custom model manager for developers
///
/// This struct allows developers to build their own model management systems,
/// including model discovery, selection, and load balancing.
#[derive(Clone)]
pub struct CustomModelManager {
    /// Provider identifier
    pub provider: String,
    /// Available models
    pub models: HashMap<String, ModelInfo>,
    /// Model selection strategy
    pub selection_strategy: ModelSelectionStrategy,
}

#[derive(Debug, Clone)]
pub enum ModelSelectionStrategy {
    /// Round-robin selection
    RoundRobin,
    /// Weighted selection based on performance
    Weighted,
    /// Least connections selection
    LeastConnections,
    /// Performance-based selection
    PerformanceBased,
    /// Cost-based selection
    CostBased,
}

impl CustomModelManager {
    /// Create new model manager
    pub fn new(provider: &str) -> Self {
        Self {
            provider: provider.to_string(),
            models: HashMap::new(),
            selection_strategy: ModelSelectionStrategy::RoundRobin,
        }
    }

    /// Add a model to the manager
    pub fn add_model(&mut self, model: ModelInfo) {
        self.models.insert(model.name.clone(), model);
    }

    /// Remove a model from the manager
    pub fn remove_model(&mut self, model_name: &str) -> Option<ModelInfo> {
        self.models.remove(model_name)
    }

    /// Get model information
    pub fn get_model(&self, model_name: &str) -> Option<&ModelInfo> {
        self.models.get(model_name)
    }

    /// List all available models
    pub fn list_models(&self) -> Vec<&ModelInfo> {
        self.models.values().collect()
    }

    /// Set selection strategy
    pub fn with_strategy(mut self, strategy: ModelSelectionStrategy) -> Self {
        self.selection_strategy = strategy;
        self
    }

    /// Select model based on current strategy
    pub fn select_model(&self) -> Option<&ModelInfo> {
        if self.models.is_empty() {
            return None;
        }

        match self.selection_strategy {
            ModelSelectionStrategy::RoundRobin => {
                // Simple round-robin implementation
                let models: Vec<&ModelInfo> = self.models.values().collect();
                let index = (std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs() as usize)
                    % models.len();
                Some(models[index])
            }
            ModelSelectionStrategy::Weighted => {
                // Weighted selection based on performance
                self.models.values().max_by_key(|model| {
                    let speed_score = match model.performance.speed {
                        SpeedTier::Fast => 3,
                        SpeedTier::Balanced => 2,
                        SpeedTier::Slow => 1,
                    };
                    let quality_score = match model.performance.quality {
                        QualityTier::Excellent => 3,
                        QualityTier::Good => 2,
                        QualityTier::Basic => 1,
                    };
                    speed_score + quality_score
                })
            }
            ModelSelectionStrategy::LeastConnections => {
                // For now, return first available model
                // In a real implementation, this would track connection counts
                self.models.values().next()
            }
            ModelSelectionStrategy::PerformanceBased => {
                // Select model with best performance metrics
                self.models.values().max_by_key(|model| {
                    let speed_score = match model.performance.speed {
                        SpeedTier::Fast => 3,
                        SpeedTier::Balanced => 2,
                        SpeedTier::Slow => 1,
                    };
                    speed_score
                })
            }
            ModelSelectionStrategy::CostBased => {
                // Select model with lowest cost
                self.models.values().min_by(|a, b| {
                    let a_cost = a.pricing.input_cost_per_1k + a.pricing.output_cost_per_1k;
                    let b_cost = b.pricing.input_cost_per_1k + b.pricing.output_cost_per_1k;
                    a_cost
                        .partial_cmp(&b_cost)
                        .unwrap_or(std::cmp::Ordering::Equal)
                })
            }
        }
    }

    /// Recommend model for specific use case
    pub fn recommend_for(&self, use_case: &str) -> Option<&ModelInfo> {
        let supported_models: Vec<&ModelInfo> = self
            .models
            .values()
            .filter(|model| model.capabilities.supports(use_case))
            .collect();

        if supported_models.is_empty() {
            return None;
        }

        // For now, return the first supported model
        // In a real implementation, this could use more sophisticated logic
        supported_models.first().copied()
    }

    /// Load models from configuration file
    pub fn load_from_config(&mut self, config_path: &str) -> Result<(), AiLibError> {
        let config_content = std::fs::read_to_string(config_path)
            .map_err(|e| AiLibError::ConfigurationError(format!("Failed to read config: {}", e)))?;

        let models: Vec<ModelInfo> = serde_json::from_str(&config_content).map_err(|e| {
            AiLibError::ConfigurationError(format!("Failed to parse config: {}", e))
        })?;

        for model in models {
            self.add_model(model);
        }

        Ok(())
    }

    /// Save current model configuration to file
    pub fn save_to_config(&self, config_path: &str) -> Result<(), AiLibError> {
        let models: Vec<&ModelInfo> = self.models.values().collect();
        let config_content = serde_json::to_string_pretty(&models).map_err(|e| {
            AiLibError::ConfigurationError(format!("Failed to serialize config: {}", e))
        })?;

        std::fs::write(config_path, config_content).map_err(|e| {
            AiLibError::ConfigurationError(format!("Failed to write config: {}", e))
        })?;

        Ok(())
    }
}

/// Model array for load balancing and A/B testing
///
/// This struct allows developers to build model arrays with multiple endpoints,
/// supporting various load balancing strategies.
pub struct ModelArray {
    /// Array name/identifier
    pub name: String,
    /// Model endpoints in the array
    pub endpoints: Vec<ModelEndpoint>,
    /// Load balancing strategy
    pub strategy: LoadBalancingStrategy,
    /// Health check configuration
    pub health_check: HealthCheckConfig,
}

/// Model endpoint in an array
#[derive(Debug, Clone)]
pub struct ModelEndpoint {
    /// Endpoint name
    pub name: String,
    /// Model name
    pub model_name: String,
    /// Endpoint URL
    pub url: String,
    /// Weight for weighted load balancing
    pub weight: f32,
    /// Health status
    pub healthy: bool,
    /// Connection count
    pub connection_count: u32,
}

/// Load balancing strategies
#[derive(Debug, Clone)]
pub enum LoadBalancingStrategy {
    /// Round-robin load balancing
    RoundRobin,
    /// Weighted load balancing
    Weighted,
    /// Least connections load balancing
    LeastConnections,
    /// Health-based load balancing
    HealthBased,
}

/// Health check configuration
#[derive(Debug, Clone)]
pub struct HealthCheckConfig {
    /// Health check endpoint
    pub endpoint: String,
    /// Health check interval
    pub interval: Duration,
    /// Health check timeout
    pub timeout: Duration,
    /// Maximum consecutive failures
    pub max_failures: u32,
}

impl ModelArray {
    /// Create new model array
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            endpoints: Vec::new(),
            strategy: LoadBalancingStrategy::RoundRobin,
            health_check: HealthCheckConfig {
                endpoint: "/health".to_string(),
                interval: Duration::from_secs(30),
                timeout: Duration::from_secs(5),
                max_failures: 3,
            },
        }
    }

    /// Add endpoint to the array
    pub fn add_endpoint(&mut self, endpoint: ModelEndpoint) {
        self.endpoints.push(endpoint);
    }

    /// Set load balancing strategy
    pub fn with_strategy(mut self, strategy: LoadBalancingStrategy) -> Self {
        self.strategy = strategy;
        self
    }

    /// Configure health check
    pub fn with_health_check(mut self, config: HealthCheckConfig) -> Self {
        self.health_check = config;
        self
    }

    /// Select next endpoint based on strategy
    pub fn select_endpoint(&mut self) -> Option<&mut ModelEndpoint> {
        if self.endpoints.is_empty() {
            return None;
        }

        // Get indices of healthy endpoints
        let healthy_indices: Vec<usize> = self
            .endpoints
            .iter()
            .enumerate()
            .filter(|(_, endpoint)| endpoint.healthy)
            .map(|(index, _)| index)
            .collect();

        if healthy_indices.is_empty() {
            return None;
        }

        match self.strategy {
            LoadBalancingStrategy::RoundRobin => {
                // Simple round-robin implementation
                let index = (std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs() as usize)
                    % healthy_indices.len();
                let endpoint_index = healthy_indices[index];
                Some(&mut self.endpoints[endpoint_index])
            }
            LoadBalancingStrategy::Weighted => {
                // Weighted selection - simplified implementation
                let total_weight: f32 = healthy_indices
                    .iter()
                    .map(|&idx| self.endpoints[idx].weight)
                    .sum();
                let mut current_weight = 0.0;

                for &idx in &healthy_indices {
                    current_weight += self.endpoints[idx].weight;
                    if current_weight >= total_weight / 2.0 {
                        return Some(&mut self.endpoints[idx]);
                    }
                }

                // Fallback to first healthy endpoint
                let endpoint_index = healthy_indices[0];
                Some(&mut self.endpoints[endpoint_index])
            }
            LoadBalancingStrategy::LeastConnections => {
                // Select endpoint with least connections
                healthy_indices
                    .iter()
                    .min_by_key(|&&idx| self.endpoints[idx].connection_count)
                    .map(|&idx| &mut self.endpoints[idx])
            }
            LoadBalancingStrategy::HealthBased => {
                // Select first healthy endpoint
                let endpoint_index = healthy_indices[0];
                Some(&mut self.endpoints[endpoint_index])
            }
        }
    }

    /// Mark endpoint as unhealthy
    pub fn mark_unhealthy(&mut self, endpoint_name: &str) {
        if let Some(endpoint) = self.endpoints.iter_mut().find(|e| e.name == endpoint_name) {
            endpoint.healthy = false;
        }
    }

    /// Mark endpoint as healthy
    pub fn mark_healthy(&mut self, endpoint_name: &str) {
        if let Some(endpoint) = self.endpoints.iter_mut().find(|e| e.name == endpoint_name) {
            endpoint.healthy = true;
        }
    }

    /// Get array health status
    pub fn is_healthy(&self) -> bool {
        self.endpoints.iter().any(|endpoint| endpoint.healthy)
    }
}

impl Default for ModelCapabilities {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for PerformanceMetrics {
    fn default() -> Self {
        Self::new()
    }
}
