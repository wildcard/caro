//! Contract Test: Backend Selector Intelligent Routing
//! 
//! This test validates the intelligent backend selection system with
//! performance monitoring, health checking, and fallback mechanisms.
//! 
//! MUST FAIL: These tests expect implementation of production backend system
//! that provides intelligent routing and health monitoring.

use cmdai::backends::{BackendSelector, PerformanceMonitor, HealthChecker, SelectionStrategy, SelectionResult, BackendMetrics};
use cmdai::backends::{MockBackend, MLXBackend};
use cmdai::config::BackendConfig;
use anyhow::Result;
use std::time::Duration;
use std::collections::HashMap;

#[tokio::test]
async fn test_backend_selector_initialization() -> Result<()> {
    let mut selector = BackendSelector::new().await?;
    
    // Register test backends
    selector.register_backend("mock", Box::new(MockBackend::new())).await?;
    selector.register_backend("mlx", Box::new(MLXBackend::new().await?)).await?;
    
    // Verify initialization
    assert_eq!(selector.get_available_backends().len(), 2);
    assert!(selector.has_performance_monitor());
    assert!(selector.has_health_checker());
    
    // Verify default selection strategy
    let strategy = selector.get_selection_strategy();
    assert!(matches!(strategy, SelectionStrategy::Performance));
    
    Ok(())
}

#[tokio::test]
async fn test_intelligent_backend_selection() -> Result<()> {
    let mut selector = BackendSelector::new().await?;
    
    // Register backends with different characteristics
    let mut fast_backend = MockBackend::new();
    fast_backend.set_latency(Duration::from_millis(100));
    fast_backend.set_reliability(0.99);
    
    let mut slow_backend = MockBackend::new();
    slow_backend.set_latency(Duration::from_millis(2000));
    slow_backend.set_reliability(0.95);
    
    selector.register_backend("fast", Box::new(fast_backend)).await?;
    selector.register_backend("slow", Box::new(slow_backend)).await?;
    
    // Test selection with performance strategy
    selector.set_selection_strategy(SelectionStrategy::Performance).await?;
    
    let start = std::time::Instant::now();
    let selection = selector.select_backend("test input", "/home/user").await?;
    let selection_time = start.elapsed();
    
    // Verify constitutional performance requirement (<50ms)
    assert!(selection_time.as_millis() < 50, 
        "Backend selection {} ms exceeds constitutional requirement of 50ms", 
        selection_time.as_millis());
    
    // Should prefer fast backend
    assert_eq!(selection.selected_backend, "fast");
    assert!(!selection.selection_reason.is_empty());
    assert!(!selection.fallback_chain.is_empty());
    assert!(selection.estimated_response_time < Duration::from_millis(500));
    
    Ok(())
}

#[tokio::test]
async fn test_health_monitoring_and_fallback() -> Result<()> {
    let mut selector = BackendSelector::new().await?;
    
    // Register healthy and unhealthy backends
    let healthy_backend = MockBackend::new();
    
    let mut unhealthy_backend = MockBackend::new();
    unhealthy_backend.set_availability(false);
    
    selector.register_backend("healthy", Box::new(healthy_backend)).await?;
    selector.register_backend("unhealthy", Box::new(unhealthy_backend)).await?;
    
    // Set preference for unhealthy backend
    selector.set_preferred_backend("unhealthy").await?;
    
    // Selection should fall back to healthy backend
    let selection = selector.select_backend("test input", "/home/user").await?;
    
    assert_eq!(selection.selected_backend, "healthy");
    assert!(selection.selection_reason.contains("fallback") || 
            selection.selection_reason.contains("unavailable"));
    assert_eq!(selection.fallback_chain[0], "unhealthy");
    
    Ok(())
}

#[tokio::test]
async fn test_performance_monitoring() -> Result<()> {
    let mut selector = BackendSelector::new().await?;
    let backend = MockBackend::new();
    
    selector.register_backend("test", Box::new(backend)).await?;
    
    // Perform several operations to build performance history
    for i in 0..10 {
        let input = format!("test input {}", i);
        let _selection = selector.select_backend(&input, "/home/user").await?;
        
        // Simulate backend usage and record metrics
        let start = std::time::Instant::now();
        tokio::time::sleep(Duration::from_millis(50 + i * 10)).await; // Varying response times
        let duration = start.elapsed();
        
        selector.record_backend_performance("test", duration, true).await?;
    }
    
    // Verify performance metrics are tracked
    let metrics = selector.get_backend_metrics("test").await?;
    assert!(metrics.is_some());
    
    let metrics = metrics.unwrap();
    assert!(metrics.avg_response_time > Duration::from_millis(0));
    assert!(metrics.success_rate > 0.0);
    assert!(metrics.availability > 0.0);
    assert!(metrics.last_health_check <= chrono::Utc::now());
    
    // Verify metrics influence selection
    let selection = selector.select_backend("performance test", "/home/user").await?;
    assert!(selection.estimated_response_time.as_millis() > 50);
    
    Ok(())
}

#[tokio::test]
async fn test_load_balancing_across_backends() -> Result<()> {
    let mut selector = BackendSelector::new().await?;
    
    // Register multiple equivalent backends
    for i in 0..3 {
        let backend = MockBackend::new();
        selector.register_backend(&format!("backend_{}", i), Box::new(backend)).await?;
    }
    
    selector.set_selection_strategy(SelectionStrategy::RoundRobin).await?;
    
    // Perform multiple selections and track distribution
    let mut selection_counts = HashMap::new();
    
    for i in 0..15 {
        let input = format!("load balance test {}", i);
        let selection = selector.select_backend(&input, "/home/user").await?;
        
        *selection_counts.entry(selection.selected_backend).or_insert(0) += 1;
    }
    
    // Verify load is distributed fairly
    assert_eq!(selection_counts.len(), 3, "Should use all backends");
    
    for count in selection_counts.values() {
        assert!(*count >= 4 && *count <= 6, "Load should be balanced: count = {}", count);
    }
    
    Ok(())
}

#[tokio::test]
async fn test_backend_specific_optimization() -> Result<()> {
    let mut selector = BackendSelector::new().await?;
    
    // Register backends with different characteristics
    let mut cpu_backend = MockBackend::new();
    cpu_backend.set_compute_type("cpu");
    cpu_backend.set_model_size("large");
    
    let mut gpu_backend = MockBackend::new();
    gpu_backend.set_compute_type("gpu");
    gpu_backend.set_model_size("small");
    
    selector.register_backend("cpu", Box::new(cpu_backend)).await?;
    selector.register_backend("gpu", Box::new(gpu_backend)).await?;
    
    selector.set_selection_strategy(SelectionStrategy::TaskOptimized).await?;
    
    // Test different task types
    let test_cases = vec![
        ("simple file listing", "cpu"), // Simple tasks to CPU
        ("complex code analysis requiring detailed understanding", "gpu"), // Complex tasks to GPU
        ("quick status check", "cpu"),
        ("analyze large codebase and generate comprehensive report", "gpu"),
    ];
    
    for (input, expected_backend_type) in test_cases {
        let selection = selector.select_backend(input, "/home/user").await?;
        
        // Verify selection reasoning
        assert!(
            selection.selected_backend.contains(expected_backend_type) ||
            selection.selection_reason.contains("optimized") ||
            selection.selection_reason.contains("task"),
            "Input '{}' should select {} backend, got {} with reason: {}", 
            input, expected_backend_type, selection.selected_backend, selection.selection_reason
        );
    }
    
    Ok(())
}

#[tokio::test]
async fn test_fallback_chain_execution() -> Result<()> {
    let mut selector = BackendSelector::new().await?;
    
    // Create backends with varying reliability
    let mut primary = MockBackend::new();
    primary.set_availability(false); // Primary is down
    
    let mut secondary = MockBackend::new();
    secondary.set_availability(false); // Secondary is also down
    
    let tertiary = MockBackend::new(); // Tertiary is working
    
    selector.register_backend("primary", Box::new(primary)).await?;
    selector.register_backend("secondary", Box::new(secondary)).await?;
    selector.register_backend("tertiary", Box::new(tertiary)).await?;
    
    // Set explicit fallback chain
    selector.set_fallback_chain(vec!["primary", "secondary", "tertiary"]).await?;
    
    // Selection should fall through to tertiary
    let selection = selector.select_backend("fallback test", "/home/user").await?;
    
    assert_eq!(selection.selected_backend, "tertiary");
    assert_eq!(selection.fallback_chain, vec!["primary", "secondary"]);
    assert!(selection.selection_reason.contains("fallback"));
    
    Ok(())
}

#[tokio::test]
async fn test_real_time_health_monitoring() -> Result<()> {
    let mut selector = BackendSelector::new().await?;
    
    let mut backend = MockBackend::new();
    backend.set_availability(true);
    
    selector.register_backend("monitored", Box::new(backend)).await?;
    
    // Start health monitoring
    selector.start_health_monitoring(Duration::from_millis(100)).await?;
    
    // Initially healthy
    let initial_health = selector.get_backend_health("monitored").await?;
    assert!(initial_health.is_available);
    
    // Simulate backend becoming unhealthy
    selector.simulate_backend_failure("monitored").await?;
    
    // Wait for health check to detect failure
    tokio::time::sleep(Duration::from_millis(200)).await;
    
    let updated_health = selector.get_backend_health("monitored").await?;
    assert!(!updated_health.is_available);
    
    // Selection should avoid unhealthy backend
    if selector.get_available_backends().len() > 1 {
        let selection = selector.select_backend("health test", "/home/user").await?;
        assert_ne!(selection.selected_backend, "monitored");
    }
    
    Ok(())
}

#[tokio::test]
async fn test_backend_configuration_updates() -> Result<()> {
    let mut selector = BackendSelector::new().await?;
    
    let backend = MockBackend::new();
    selector.register_backend("configurable", Box::new(backend)).await?;
    
    // Update backend configuration
    let new_config = BackendConfig {
        endpoint: "http://localhost:8080".to_string(),
        timeout_ms: 15000,
        max_retries: 5,
        headers: {
            let mut headers = HashMap::new();
            headers.insert("Authorization".to_string(), "Bearer token".to_string());
            headers
        },
    };
    
    selector.update_backend_config("configurable", new_config.clone()).await?;
    
    // Verify configuration was applied
    let stored_config = selector.get_backend_config("configurable").await?;
    assert!(stored_config.is_some());
    
    let stored = stored_config.unwrap();
    assert_eq!(stored.endpoint, new_config.endpoint);
    assert_eq!(stored.timeout_ms, new_config.timeout_ms);
    assert_eq!(stored.max_retries, new_config.max_retries);
    
    Ok(())
}

#[tokio::test]
async fn test_concurrent_backend_selection() -> Result<()> {
    let selector = std::sync::Arc::new({
        let mut s = BackendSelector::new().await?;
        s.register_backend("concurrent", Box::new(MockBackend::new())).await?;
        s
    });
    
    // Perform concurrent selections
    let mut handles = Vec::new();
    
    for i in 0..20 {
        let selector_clone = selector.clone();
        let handle = tokio::spawn(async move {
            let input = format!("concurrent test {}", i);
            let selection = selector_clone.select_backend(&input, "/home/user").await.unwrap();
            
            // Verify selection is valid
            assert!(!selection.selected_backend.is_empty());
            assert!(!selection.selection_reason.is_empty());
            selection.selected_backend
        });
        handles.push(handle);
    }
    
    // Wait for all selections to complete
    let mut results = Vec::new();
    for handle in handles {
        let result = handle.await?;
        results.push(result);
    }
    
    // All selections should succeed
    assert_eq!(results.len(), 20);
    for backend_name in results {
        assert_eq!(backend_name, "concurrent");
    }
    
    Ok(())
}

#[tokio::test]
async fn test_selection_strategy_switching() -> Result<()> {
    let mut selector = BackendSelector::new().await?;
    
    // Register multiple backends
    selector.register_backend("backend_a", Box::new(MockBackend::new())).await?;
    selector.register_backend("backend_b", Box::new(MockBackend::new())).await?;
    
    // Test different strategies produce different selections
    let input = "strategy test";
    
    // Performance strategy
    selector.set_selection_strategy(SelectionStrategy::Performance).await?;
    let perf_selection = selector.select_backend(input, "/home/user").await?;
    
    // Round robin strategy
    selector.set_selection_strategy(SelectionStrategy::RoundRobin).await?;
    let rr_selection1 = selector.select_backend(input, "/home/user").await?;
    let rr_selection2 = selector.select_backend(input, "/home/user").await?;
    
    // Random strategy
    selector.set_selection_strategy(SelectionStrategy::Random).await?;
    let random_selection = selector.select_backend(input, "/home/user").await?;
    
    // Verify strategies work differently
    assert!(!perf_selection.selection_reason.is_empty());
    
    // Round robin should alternate (if we have 2 backends)
    if selector.get_available_backends().len() == 2 {
        assert_ne!(rr_selection1.selected_backend, rr_selection2.selected_backend);
    }
    
    assert!(!random_selection.selected_backend.is_empty());
    
    Ok(())
}

#[tokio::test]
async fn test_performance_requirements_compliance() -> Result<()> {
    let mut selector = BackendSelector::new().await?;
    
    // Register multiple backends to make selection more complex
    for i in 0..5 {
        selector.register_backend(&format!("backend_{}", i), Box::new(MockBackend::new())).await?;
    }
    
    // Test selection performance under load
    let mut total_duration = Duration::new(0, 0);
    let iterations = 100;
    
    for i in 0..iterations {
        let input = format!("performance test {}", i);
        
        let start = std::time::Instant::now();
        let _selection = selector.select_backend(&input, "/home/user").await?;
        total_duration += start.elapsed();
    }
    
    let avg_duration = total_duration / iterations;
    
    // Verify constitutional performance requirement (<50ms average)
    assert!(avg_duration.as_millis() < 50, 
        "Average backend selection time {} ms exceeds constitutional requirement of 50ms", 
        avg_duration.as_millis());
    
    Ok(())
}