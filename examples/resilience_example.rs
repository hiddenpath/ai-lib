//! Example demonstrating resilience features in ai-lib
//!
//! This example shows how to use the new resilience features including:
//! - Circuit breaker for fault tolerance
//! - Rate limiting with adaptive capabilities
//! - Enhanced error handling with recovery suggestions
//! - Comprehensive metrics collection

use ai_lib::{
    AiClientBuilder, Provider, 
    circuit_breaker::{CircuitBreaker, CircuitBreakerConfig},
    circuit_breaker::breaker::CircuitBreakerError,
    rate_limiter::{TokenBucket, RateLimiterConfig},
    error_handling::{ErrorContext, ErrorRecoveryManager},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 AI-lib Resilience Features Demo");
    println!("=====================================\n");

    // 1. Demonstrate Circuit Breaker
    println!("1. Circuit Breaker Demo");
    println!("-----------------------");
    demonstrate_circuit_breaker().await?;
    println!();

    // 2. Demonstrate Rate Limiting
    println!("2. Rate Limiting Demo");
    println!("--------------------");
    demonstrate_rate_limiting().await?;
    println!();

    // 3. Demonstrate Error Handling
    println!("3. Error Handling Demo");
    println!("----------------------");
    demonstrate_error_handling().await?;
    println!();

    // 4. Demonstrate Integrated Resilience
    println!("4. Integrated Resilience Demo");
    println!("-----------------------------");
    demonstrate_integrated_resilience().await?;
    println!();

    // 5. Demonstrate Smart Configuration
    println!("5. Smart Configuration Demo");
    println!("---------------------------");
    demonstrate_smart_configuration().await?;

    println!("\n✅ All resilience features demonstrated successfully!");
    Ok(())
}

async fn demonstrate_circuit_breaker() -> Result<(), Box<dyn std::error::Error>> {
    // Create a circuit breaker with development configuration
    let config = CircuitBreakerConfig::development();
    let breaker = CircuitBreaker::new(config);
    
    println!("Circuit Breaker State: {:?}", breaker.state());
    println!("Failure Count: {}", breaker.failure_count());
    println!("Success Rate: {:.2}%", breaker.success_rate());
    
    // Simulate some operations
    for i in 1..=5 {
        println!("  Operation {}: Circuit state = {:?}", i, breaker.state());
        
        // Simulate a failing operation
        let result = breaker.call(async {
            if i % 3 == 0 {
                Err(ai_lib::types::AiLibError::NetworkError("Simulated network error".to_string()))
            } else {
                Ok(format!("Success {}", i))
            }
        }).await;
        
        match result {
            Ok(response) => println!("    ✅ Success: {}", response),
            Err(e) => println!("    ❌ Failed: {}", e),
        }
        
        println!("    State: {:?}, Failures: {}", breaker.state(), breaker.failure_count());
    }
    
    // Show final metrics
    let metrics = breaker.get_metrics();
    println!("Final Metrics:");
    println!("  Total Requests: {}", metrics.total_requests);
    println!("  Successful: {}", metrics.successful_requests);
    println!("  Failed: {}", metrics.failed_requests);
    println!("  Circuit Opens: {}", metrics.circuit_open_count);
    
    Ok(())
}

async fn demonstrate_rate_limiting() -> Result<(), Box<dyn std::error::Error>> {
    // Create a rate limiter with conservative configuration
    let config = RateLimiterConfig::conservative();
    let rate_limiter = TokenBucket::new(config);
    
    println!("Rate Limiter Configuration:");
    let metrics = rate_limiter.get_metrics();
    println!("  Capacity: {}", metrics.capacity);
    println!("  Refill Rate: {} req/s", metrics.refill_rate);
    println!("  Adaptive: {}", metrics.is_adaptive);
    println!("  Current Tokens: {}", metrics.current_tokens);
    
    // Simulate rapid requests
    println!("\nSimulating rapid requests:");
    for i in 1..=10 {
        let start = std::time::Instant::now();
        let result = rate_limiter.acquire(1).await;
        let duration = start.elapsed();
        
        match result {
            Ok(_) => println!("  Request {}: ✅ Acquired in {:?}", i, duration),
            Err(e) => println!("  Request {}: ❌ Failed - {}", i, e),
        }
        
        // Show current state
        let metrics = rate_limiter.get_metrics();
        println!("    Tokens: {}, Success Rate: {:.1}%", 
                metrics.current_tokens, rate_limiter.success_rate());
    }
    
    // Test adaptive rate adjustment
    if metrics.is_adaptive {
        println!("\nTesting adaptive rate adjustment:");
        rate_limiter.adjust_rate(true);  // Success
        rate_limiter.adjust_rate(false); // Failure
        rate_limiter.adjust_rate(true);  // Success
        
        let metrics = rate_limiter.get_metrics();
        println!("  Adaptive Rate: {:?}", metrics.adaptive_rate);
    }
    
    Ok(())
}

async fn demonstrate_error_handling() -> Result<(), Box<dyn std::error::Error>> {
    let error_manager = ErrorRecoveryManager::new();
    
    // Simulate different types of errors
    let errors = vec![
        ("Rate Limit", ai_lib::types::AiLibError::RateLimitExceeded("API rate limit exceeded".to_string())),
        ("Network", ai_lib::types::AiLibError::NetworkError("Connection timeout".to_string())),
        ("Authentication", ai_lib::types::AiLibError::AuthenticationError("Invalid API key".to_string())),
        ("Context Length", ai_lib::types::AiLibError::ContextLengthExceeded("Request too long".to_string())),
    ];
    
    println!("Error Handling and Pattern Analysis:");
    for (error_type, error) in errors {
        let context = ErrorContext::new("demo_provider".to_string(), "/demo/endpoint".to_string());
        let result = error_manager.handle_error(&error, &context).await;
        
        println!("  {} Error: {}", error_type, error);
        println!("    Suggested Action: {:?}", context.suggested_action);
        println!("    Recovery Result: {}", if result.is_ok() { "Success" } else { "Failed" });
    }
    
    // Show error statistics
    let stats = error_manager.get_error_statistics();
    println!("\nError Statistics:");
    println!("  Total Errors: {}", stats.total_errors);
    println!("  Unique Error Types: {}", stats.unique_error_types);
    println!("  Most Common: {:?}", stats.most_common_error);
    
    // Show error patterns
    let patterns = error_manager.get_error_patterns();
    for (error_type, pattern) in patterns {
        println!("  {:?}: {} occurrences, {:.2} errors/min", 
                error_type, pattern.count, pattern.frequency);
    }
    
    Ok(())
}

async fn demonstrate_integrated_resilience() -> Result<(), Box<dyn std::error::Error>> {
    // Create all resilience components
    let circuit_breaker = CircuitBreaker::new(CircuitBreakerConfig::development());
    let rate_limiter = TokenBucket::new(RateLimiterConfig::development());
    let error_manager = ErrorRecoveryManager::new();
    
    println!("Integrated Resilience Components:");
    println!("  Circuit Breaker: {:?}", circuit_breaker.state());
    println!("  Rate Limiter: {:.1}% success rate", rate_limiter.success_rate());
    println!("  Error Manager: {} total errors", error_manager.get_error_statistics().total_errors);
    
    // Simulate a resilient operation
    println!("\nSimulating resilient operation:");
    for i in 1..=5 {
        println!("  Operation {}:", i);
        
        // Step 1: Rate limiting
        match rate_limiter.acquire(1).await {
            Ok(_) => println!("    ✅ Rate limit passed"),
            Err(e) => {
                println!("    ❌ Rate limited: {}", e);
                continue;
            }
        }
        
        // Step 2: Circuit breaker protection
        let result = circuit_breaker.call(async {
            // Simulate API call with occasional failures
            if i % 4 == 0 {
                Err(ai_lib::types::AiLibError::ProviderError("Simulated provider error".to_string()))
            } else {
                Ok(format!("API response {}", i))
            }
        }).await;
        
        // Step 3: Error handling
        match result {
            Ok(response) => {
                println!("    ✅ Success: {}", response);
                rate_limiter.adjust_rate(true);
            }
            Err(e) => {
                println!("    ❌ Failed: {}", e);
                rate_limiter.adjust_rate(false);
                
                // Convert CircuitBreakerError to AiLibError for error manager
                let ai_error = match e {
                    CircuitBreakerError::Underlying(err) => err,
                    CircuitBreakerError::CircuitOpen(msg) => ai_lib::types::AiLibError::ProviderError(msg),
                    CircuitBreakerError::RequestTimeout(msg) => ai_lib::types::AiLibError::TimeoutError(msg),
                    CircuitBreakerError::Disabled => ai_lib::types::AiLibError::ConfigurationError("Circuit breaker disabled".to_string()),
                };
                
                let context = ErrorContext::new("demo_provider".to_string(), "/demo/endpoint".to_string());
                let _ = error_manager.handle_error(&ai_error, &context).await;
            }
        }
        
        // Show current state
        println!("    State: CB={:?}, RL={:.1}%", 
                circuit_breaker.state(), rate_limiter.success_rate());
    }
    
    Ok(())
}

async fn demonstrate_smart_configuration() -> Result<(), Box<dyn std::error::Error>> {
    println!("Smart Configuration Options:");
    
    // 1. Smart defaults
    println!("\n1. Smart Defaults:");
    let _smart_client = AiClientBuilder::new(Provider::Groq)
        .with_smart_defaults()
        .build()?;
    println!("  ✅ Client created with smart defaults");
    
    // 2. Production configuration
    println!("\n2. Production Configuration:");
    let _prod_client = AiClientBuilder::new(Provider::OpenAI)
        .for_production()
        .build()?;
    println!("  ✅ Production client created");
    
    // 3. Development configuration
    println!("\n3. Development Configuration:");
    let _dev_client = AiClientBuilder::new(Provider::Gemini)
        .for_development()
        .build()?;
    println!("  ✅ Development client created");
    
    // 4. Custom configuration
    println!("\n4. Custom Configuration:");
    let custom_config = ai_lib::config::ResilienceConfig {
        circuit_breaker: Some(CircuitBreakerConfig::conservative()),
        rate_limiter: Some(RateLimiterConfig::conservative()),
        backpressure: Some(ai_lib::config::BackpressureConfig {
            max_concurrent_requests: 10,
        }),
        error_handling: Some(ai_lib::config::ErrorHandlingConfig::default()),
    };
    
    let _custom_client = AiClientBuilder::new(Provider::Mistral)
        .with_resilience_config(custom_config)
        .build()?;
    println!("  ✅ Custom client created");
    
    println!("\nAll configuration options demonstrated successfully!");
    Ok(())
}
