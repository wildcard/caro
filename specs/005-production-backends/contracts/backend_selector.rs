// Contract Test: Backend Selection System
// Tests intelligent backend routing with performance monitoring

use cmdai::backends::{BackendSelector, BackendType, SelectionCriteria, SelectionResult};
use anyhow::Result;
use std::time::Duration;

#[tokio::test]
async fn test_basic_backend_selection() -> Result<()> {
    let selector = BackendSelector::new().await?;
    
    // Register available backends
    selector.register_backend(BackendType::MLX).await?;
    selector.register_backend(BackendType::Ollama).await?;
    selector.register_backend(BackendType::VLLM).await?;
    
    let criteria = SelectionCriteria {
        preferred_backend: Some(BackendType::MLX),
        max_response_time: Some(Duration::from_secs(5)),
        require_local: true,
        model_requirements: None,
    };
    
    let result = selector.select_backend(&criteria).await?;
    
    assert_eq!(result.selected_backend, BackendType::MLX);
    assert!(!result.fallback_chain.is_empty());
    assert!(result.estimated_response_time > Duration::ZERO);
    
    Ok(())
}

#[tokio::test]
async fn test_backend_availability_checking() -> Result<()> {
    let selector = BackendSelector::new().await?;
    
    // Check availability of different backends
    let mlx_available = selector.is_backend_available(BackendType::MLX).await?;
    let ollama_available = selector.is_backend_available(BackendType::Ollama).await?;
    let vllm_available = selector.is_backend_available(BackendType::VLLM).await?;
    
    // At least one backend should be available in test environment
    assert!(mlx_available || ollama_available || vllm_available);
    
    Ok(())
}

#[tokio::test]
async fn test_performance_based_selection() -> Result<()> {
    let selector = BackendSelector::new().await?;
    
    // Simulate performance data
    selector.record_performance_data(BackendType::MLX, Duration::from_millis(1200), true).await?;
    selector.record_performance_data(BackendType::Ollama, Duration::from_millis(3000), true).await?;
    selector.record_performance_data(BackendType::VLLM, Duration::from_millis(800), true).await?;
    
    let criteria = SelectionCriteria {
        preferred_backend: None, // Let performance decide
        max_response_time: Some(Duration::from_secs(2)),
        require_local: false,
        model_requirements: None,
    };
    
    let result = selector.select_backend(&criteria).await?;
    
    // Should select fastest backend (VLLM in this case)
    assert_eq!(result.selected_backend, BackendType::VLLM);
    assert!(result.selection_reason.contains("performance"));
    
    Ok(())
}

#[tokio::test]
async fn test_fallback_chain_execution() -> Result<()> {
    let selector = BackendSelector::new().await?;
    
    // Simulate primary backend failure
    selector.mark_backend_unavailable(BackendType::MLX).await?;
    
    let criteria = SelectionCriteria {
        preferred_backend: Some(BackendType::MLX),
        max_response_time: Some(Duration::from_secs(5)),
        require_local: true,
        model_requirements: None,
    };
    
    let result = selector.select_backend(&criteria).await?;
    
    // Should fallback to another available backend
    assert_ne!(result.selected_backend, BackendType::MLX);
    assert!(result.selection_reason.contains("fallback"));
    assert!(result.fallback_chain.contains(&BackendType::MLX));
    
    Ok(())
}

#[tokio::test]
async fn test_model_requirement_matching() -> Result<()> {
    let selector = BackendSelector::new().await?;
    
    use cmdai::backends::ModelRequirements;
    
    let criteria = SelectionCriteria {
        preferred_backend: None,
        max_response_time: Some(Duration::from_secs(10)),
        require_local: true,
        model_requirements: Some(ModelRequirements {
            min_context_length: Some(4096),
            required_capabilities: vec!["code_generation".to_string()],
            preferred_model_family: Some("llama".to_string()),
        }),
    };
    
    let result = selector.select_backend(&criteria).await?;
    
    // Should select backend that meets model requirements
    assert!(result.selection_reason.contains("model_requirements"));
    
    Ok(())
}

#[tokio::test]
async fn test_health_monitoring() -> Result<()> {
    let selector = BackendSelector::new().await?;
    
    // Start health monitoring
    selector.start_health_monitoring().await?;
    
    // Wait for health checks to complete
    tokio::time::sleep(Duration::from_millis(200)).await;
    
    // Get health status
    let health_status = selector.get_health_status().await?;
    
    assert!(!health_status.is_empty());
    
    for (backend, status) in health_status {
        assert!(status.last_check.is_some());
        // Health check should be recent (within last minute)
        let elapsed = chrono::Utc::now() - status.last_check.unwrap();
        assert!(elapsed.num_seconds() < 60);
    }
    
    Ok(())
}

#[tokio::test]
async fn test_backend_load_balancing() -> Result<()> {
    let selector = BackendSelector::new().await?;
    
    // Register multiple instances of same backend type
    selector.register_backend_instance(BackendType::VLLM, "instance1", "http://localhost:8001").await?;
    selector.register_backend_instance(BackendType::VLLM, "instance2", "http://localhost:8002").await?;
    
    let criteria = SelectionCriteria {
        preferred_backend: Some(BackendType::VLLM),
        max_response_time: Some(Duration::from_secs(5)),
        require_local: false,
        model_requirements: None,
    };
    
    // Make multiple selections
    let mut selected_instances = Vec::new();
    for _ in 0..4 {
        let result = selector.select_backend(&criteria).await?;
        selected_instances.push(result.backend_instance);
    }
    
    // Should distribute load across instances
    let unique_instances: std::collections::HashSet<_> = selected_instances.into_iter().collect();
    assert!(unique_instances.len() > 1, "Load balancing should use multiple instances");
    
    Ok(())
}

#[tokio::test]
async fn test_performance_metrics_collection() -> Result<()> {
    let selector = BackendSelector::new().await?;
    
    // Record various performance metrics
    selector.record_performance_data(BackendType::MLX, Duration::from_millis(1500), true).await?;
    selector.record_performance_data(BackendType::MLX, Duration::from_millis(1200), true).await?;
    selector.record_performance_data(BackendType::MLX, Duration::from_millis(1800), false).await?;
    
    let metrics = selector.get_performance_metrics(BackendType::MLX).await?;
    
    assert_eq!(metrics.total_requests, 3);
    assert_eq!(metrics.successful_requests, 2);
    assert_eq!(metrics.failed_requests, 1);
    assert!((metrics.success_rate - 0.666).abs() < 0.01);
    assert!(metrics.avg_response_time > Duration::ZERO);
    assert!(metrics.p95_response_time >= metrics.avg_response_time);
    
    Ok(())
}

#[tokio::test]
async fn test_selection_timeout() -> Result<()> {
    let selector = BackendSelector::new().await?;
    
    // Set very short timeout
    let criteria = SelectionCriteria {
        preferred_backend: Some(BackendType::VLLM),
        max_response_time: Some(Duration::from_millis(1)), // Very short
        require_local: false,
        model_requirements: None,
    };
    
    let result = selector.select_backend(&criteria).await;
    
    // Should handle timeout gracefully
    if result.is_err() {
        let error = result.unwrap_err();
        assert!(error.to_string().contains("timeout") || error.to_string().contains("unavailable"));
    }
    
    Ok(())
}

#[tokio::test]
async fn test_selection_performance() -> Result<()> {
    let selector = BackendSelector::new().await?;
    
    let criteria = SelectionCriteria {
        preferred_backend: None,
        max_response_time: Some(Duration::from_secs(5)),
        require_local: true,
        model_requirements: None,
    };
    
    let start = std::time::Instant::now();
    let _result = selector.select_backend(&criteria).await?;
    let selection_time = start.elapsed();
    
    // Backend selection should complete within 50ms (constitutional requirement)
    assert!(selection_time.as_millis() < 50, 
        "Backend selection took {}ms, expected <50ms", selection_time.as_millis());
    
    Ok(())
}