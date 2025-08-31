use ai_lib::{
    CustomModelManager, LoadBalancingStrategy, ModelArray, ModelCapabilities, ModelEndpoint,
    ModelInfo, ModelSelectionStrategy, PerformanceMetrics, PricingInfo, QualityTier, SpeedTier,
};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 AI-lib Model Management Tools Example");
    println!("======================================");

    // Example 1: Create a custom model manager for Groq
    println!("\n📋 Example 1: Custom Model Manager for Groq");
    println!("    Building a model manager with multiple models and selection strategies");

    let mut groq_manager =
        CustomModelManager::new("groq").with_strategy(ModelSelectionStrategy::PerformanceBased);

    // Add different Groq models with their capabilities
    let llama3_8b = ModelInfo {
        name: "llama3-8b-8192".to_string(),
        display_name: "Llama 3 8B".to_string(),
        description: "Fast and cost-effective model for general tasks".to_string(),
        capabilities: ModelCapabilities::new()
            .with_chat()
            .with_code_generation()
            .with_context_window(8192),
        pricing: PricingInfo::new(0.05, 0.10), // $0.05/1K input, $0.10/1K output
        performance: PerformanceMetrics::new()
            .with_speed(SpeedTier::Fast)
            .with_quality(QualityTier::Good)
            .with_avg_response_time(Duration::from_millis(500)),
        metadata: std::collections::HashMap::new(),
    };

    let llama3_70b = ModelInfo {
        name: "llama3-70b-8192".to_string(),
        display_name: "Llama 3 70B".to_string(),
        description: "High-performance model for complex tasks".to_string(),
        capabilities: ModelCapabilities::new()
            .with_chat()
            .with_code_generation()
            .with_function_calling()
            .with_context_window(8192),
        pricing: PricingInfo::new(0.59, 1.99), // $0.59/1K input, $1.99/1K output
        performance: PerformanceMetrics::new()
            .with_speed(SpeedTier::Slow)
            .with_quality(QualityTier::Excellent)
            .with_avg_response_time(Duration::from_secs(3)),
        metadata: std::collections::HashMap::new(),
    };

    let mixtral = ModelInfo {
        name: "mixtral-8x7b-32768".to_string(),
        display_name: "Mixtral 8x7B".to_string(),
        description: "Balanced performance and cost model".to_string(),
        capabilities: ModelCapabilities::new()
            .with_chat()
            .with_code_generation()
            .with_context_window(32768),
        pricing: PricingInfo::new(0.14, 0.42), // $0.14/1K input, $0.42/1K output
        performance: PerformanceMetrics::new()
            .with_speed(SpeedTier::Balanced)
            .with_quality(QualityTier::Good)
            .with_avg_response_time(Duration::from_secs(1)),
        metadata: std::collections::HashMap::new(),
    };

    // Add models to the manager
    groq_manager.add_model(llama3_8b);
    groq_manager.add_model(llama3_70b);
    groq_manager.add_model(mixtral);

    println!(
        "✅ Added {} models to Groq manager",
        groq_manager.models.len()
    );

    // Demonstrate model selection
    if let Some(selected_model) = groq_manager.select_model() {
        println!(
            "🎯 Selected model: {} ({})",
            selected_model.display_name, selected_model.name
        );
        println!(
            "   Cost: ${:.3}/1K input, ${:.3}/1K output",
            selected_model.pricing.input_cost_per_1k, selected_model.pricing.output_cost_per_1k
        );
    }

    // Example 2: Model recommendation for specific use cases
    println!("\n📋 Example 2: Model Recommendation for Use Cases");

    if let Some(recommended_model) = groq_manager.recommend_for("chat") {
        println!(
            "💬 Chat recommendation: {} ({})",
            recommended_model.display_name, recommended_model.name
        );
    }

    if let Some(recommended_model) = groq_manager.recommend_for("code_generation") {
        println!(
            "💻 Code generation recommendation: {} ({})",
            recommended_model.display_name, recommended_model.name
        );
    }

    // Example 3: Create a model array for load balancing
    println!("\n📋 Example 3: Model Array for Load Balancing");
    println!("    Building a load-balanced array of model endpoints");

    let mut groq_array =
        ModelArray::new("groq-production").with_strategy(LoadBalancingStrategy::RoundRobin);

    // Add multiple endpoints for the same model
    let endpoint1 = ModelEndpoint {
        name: "groq-us-east-1".to_string(),
        model_name: "llama3-8b-8192".to_string(),
        url: "https://api.groq.com/openai/v1".to_string(),
        weight: 1.0,
        healthy: true,
        connection_count: 0,
    };

    let endpoint2 = ModelEndpoint {
        name: "groq-us-west-1".to_string(),
        model_name: "llama3-8b-8192".to_string(),
        url: "https://api-west.groq.com/openai/v1".to_string(),
        weight: 1.0,
        healthy: true,
        connection_count: 0,
    };

    let endpoint3 = ModelEndpoint {
        name: "groq-eu-west-1".to_string(),
        model_name: "llama3-8b-8192".to_string(),
        url: "https://api-eu.groq.com/openai/v1".to_string(),
        weight: 0.8, // Slightly lower weight for EU region
        healthy: true,
        connection_count: 0,
    };

    groq_array.add_endpoint(endpoint1);
    groq_array.add_endpoint(endpoint2);
    groq_array.add_endpoint(endpoint3);

    println!(
        "✅ Added {} endpoints to Groq array",
        groq_array.endpoints.len()
    );

    // Demonstrate load balancing
    for i in 0..5 {
        if let Some(endpoint) = groq_array.select_endpoint() {
            println!(
                "🔄 Request {} routed to: {} ({})",
                i + 1,
                endpoint.name,
                endpoint.url
            );

            // Simulate connection tracking
            endpoint.connection_count += 1;
        }
    }

    // Example 4: Cost analysis and comparison
    println!("\n📋 Example 4: Cost Analysis and Comparison");

    let test_input_tokens = 1000;
    let test_output_tokens = 500;

    println!(
        "💰 Cost comparison for {} input + {} output tokens:",
        test_input_tokens, test_output_tokens
    );

    for model in groq_manager.list_models() {
        let cost = model
            .pricing
            .calculate_cost(test_input_tokens, test_output_tokens);
        println!("   {}: ${:.4}", model.display_name, cost);
    }

    // Example 5: Performance-based model selection
    println!("\n📋 Example 5: Performance-Based Model Selection");

    let mut performance_manager = groq_manager.clone();
    performance_manager =
        performance_manager.with_strategy(ModelSelectionStrategy::PerformanceBased);

    if let Some(best_model) = performance_manager.select_model() {
        println!(
            "🏆 Best performance model: {} ({})",
            best_model.display_name, best_model.name
        );
        println!(
            "   Speed: {:?}, Quality: {:?}",
            best_model.performance.speed, best_model.performance.quality
        );
    }

    // Example 6: Cost-based model selection
    println!("\n📋 Example 6: Cost-Based Model Selection");

    let mut cost_manager = groq_manager.clone();
    cost_manager = cost_manager.with_strategy(ModelSelectionStrategy::CostBased);

    if let Some(cheapest_model) = cost_manager.select_model() {
        println!(
            "💸 Most cost-effective model: {} ({})",
            cheapest_model.display_name, cheapest_model.name
        );
        println!(
            "   Cost per 1K tokens: ${:.3}",
            cheapest_model.pricing.input_cost_per_1k + cheapest_model.pricing.output_cost_per_1k
        );
    }

    println!("\n🎉 Model management examples completed successfully!");
    println!("\n💡 Key benefits of these tools:");
    println!("   • Build custom model managers for any provider");
    println!("   • Implement sophisticated model selection strategies");
    println!("   • Create load-balanced model arrays");
    println!("   • Analyze costs and performance metrics");
    println!("   • Recommend models for specific use cases");

    Ok(())
}
