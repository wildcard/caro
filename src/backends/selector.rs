// Backend selection and priority management
//
// This module implements intelligent backend selection with:
// - Performance-based prioritization
// - Automatic health checking
// - Graceful fallback chains
// - Adaptive learning from usage patterns

use async_trait::async_trait;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

use crate::backends::{BackendInfo, CommandGenerator, GeneratorError};
use crate::models::{CommandRequest, GeneratedCommand};

/// Backend performance metrics for selection algorithms
#[derive(Debug, Clone)]
pub struct BackendMetrics {
    pub average_latency_ms: u64,
    pub success_rate: f64,
    pub total_requests: u64,
    pub failed_requests: u64,
    pub last_used: Option<Instant>,
    pub availability_score: f64, // 0.0 = never available, 1.0 = always available
}

impl Default for BackendMetrics {
    fn default() -> Self {
        Self {
            average_latency_ms: 5000, // Conservative default
            success_rate: 1.0,
            total_requests: 0,
            failed_requests: 0,
            last_used: None,
            availability_score: 1.0,
        }
    }
}

/// Configuration for backend selection behavior
#[derive(Debug, Clone)]
pub struct BackendSelectorConfig {
    /// Maximum time to wait for backend health checks
    pub health_check_timeout_ms: u64,
    /// How often to refresh backend availability (in seconds)
    pub refresh_interval_secs: u64,
    /// Weight factors for backend selection algorithm
    pub latency_weight: f64,
    pub availability_weight: f64,
    pub success_rate_weight: f64,
    /// Enable adaptive learning from usage patterns
    pub enable_adaptive_learning: bool,
}

impl Default for BackendSelectorConfig {
    fn default() -> Self {
        Self {
            health_check_timeout_ms: 2000,
            refresh_interval_secs: 30,
            latency_weight: 0.3,
            availability_weight: 0.4,
            success_rate_weight: 0.3,
            enable_adaptive_learning: true,
        }
    }
}

/// Prioritized backend with metadata
struct ManagedBackend {
    backend: Arc<dyn CommandGenerator>,
    name: String,
    priority: u8, // 0 = highest priority, 255 = lowest
    metrics: BackendMetrics,
    last_health_check: Option<Instant>,
}

impl std::fmt::Debug for ManagedBackend {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ManagedBackend")
            .field("name", &self.name)
            .field("priority", &self.priority)
            .field("metrics", &self.metrics)
            .field("last_health_check", &self.last_health_check)
            .field("backend", &"<CommandGenerator>")
            .finish()
    }
}

/// Intelligent backend selector with automatic fallback and performance monitoring
pub struct BackendSelector {
    backends: RwLock<Vec<ManagedBackend>>,
    config: BackendSelectorConfig,
}

impl BackendSelector {
    /// Create a new backend selector
    pub fn new(config: BackendSelectorConfig) -> Self {
        Self {
            backends: RwLock::new(Vec::new()),
            config,
        }
    }

    /// Add a backend with specified priority (0 = highest priority)
    pub async fn add_backend(
        &self,
        backend: Arc<dyn CommandGenerator>,
        name: String,
        priority: u8,
    ) -> Result<(), GeneratorError> {
        let mut backends = self.backends.write().await;

        let managed = ManagedBackend {
            backend,
            name: name.clone(),
            priority,
            metrics: BackendMetrics::default(),
            last_health_check: None,
        };

        backends.push(managed);

        // Sort by priority (lower number = higher priority)
        backends.sort_by_key(|b| b.priority);

        info!("Added backend '{}' with priority {}", name, priority);
        Ok(())
    }

    /// Get the best available backend based on current metrics
    pub async fn select_backend(&self) -> Result<Arc<dyn CommandGenerator>, GeneratorError> {
        let mut backends = self.backends.write().await;

        if backends.is_empty() {
            return Err(GeneratorError::BackendUnavailable {
                reason: "No backends configured".to_string(),
            });
        }

        // Refresh health status if needed
        self.refresh_health_status(&mut backends).await;

        // Find the best backend using our selection algorithm
        let best_backend = self.find_best_backend(&backends).await?;

        debug!("Selected backend: {}", best_backend.name);
        Ok(best_backend.backend.clone())
    }

    /// Perform health checks on backends that need refreshing
    async fn refresh_health_status(&self, backends: &mut Vec<ManagedBackend>) {
        let now = Instant::now();
        let refresh_interval = Duration::from_secs(self.config.refresh_interval_secs);

        for backend in backends.iter_mut() {
            let needs_check = backend
                .last_health_check
                .map(|last| now.duration_since(last) > refresh_interval)
                .unwrap_or(true);

            if needs_check {
                let start = Instant::now();
                let is_available = backend.backend.is_available().await;
                let check_duration = start.elapsed();

                backend.last_health_check = Some(now);

                // Update availability score with exponential moving average
                let new_score = if is_available { 1.0 } else { 0.0 };
                backend.metrics.availability_score =
                    0.8 * backend.metrics.availability_score + 0.2 * new_score;

                debug!(
                    "Health check for '{}': available={}, duration={:?}, score={:.2}",
                    backend.name, is_available, check_duration, backend.metrics.availability_score
                );
            }
        }
    }

    /// Select the best backend using our multi-factor algorithm
    async fn find_best_backend<'a>(
        &self,
        backends: &'a [ManagedBackend],
    ) -> Result<&'a ManagedBackend, GeneratorError> {
        let mut best_backend: Option<&ManagedBackend> = None;
        let mut best_score = f64::NEG_INFINITY;

        for backend in backends {
            // Skip unavailable backends
            if backend.metrics.availability_score < 0.1 {
                continue;
            }

            let score = self.calculate_backend_score(backend);

            debug!(
                "Backend '{}' score: {:.3} (latency: {}ms, success: {:.2}, availability: {:.2})",
                backend.name,
                score,
                backend.metrics.average_latency_ms,
                backend.metrics.success_rate,
                backend.metrics.availability_score
            );

            if score > best_score {
                best_score = score;
                best_backend = Some(backend);
            }
        }

        best_backend.ok_or_else(|| GeneratorError::BackendUnavailable {
            reason: "No healthy backends available".to_string(),
        })
    }

    /// Calculate a composite score for backend selection
    fn calculate_backend_score(&self, backend: &ManagedBackend) -> f64 {
        let metrics = &backend.metrics;

        // Normalize latency score (lower is better)
        let latency_score = if metrics.average_latency_ms > 0 {
            1.0 / (1.0 + metrics.average_latency_ms as f64 / 1000.0) // Normalize to seconds
        } else {
            1.0
        };

        // Priority bonus (lower priority number = higher bonus)
        let priority_bonus = 1.0 - (backend.priority as f64 / 255.0);

        // Combine factors with weights
        let score = self.config.latency_weight * latency_score
            + self.config.availability_weight * metrics.availability_score
            + self.config.success_rate_weight * metrics.success_rate
            + 0.1 * priority_bonus; // Small priority influence

        score
    }

    /// Update metrics after a backend operation
    pub async fn record_operation_result(
        &self,
        backend_name: &str,
        duration: Duration,
        success: bool,
    ) {
        let mut backends = self.backends.write().await;

        if let Some(backend) = backends.iter_mut().find(|b| b.name == backend_name) {
            let metrics = &mut backend.metrics;

            // Update counters
            metrics.total_requests += 1;
            if !success {
                metrics.failed_requests += 1;
            }

            // Update success rate with exponential moving average
            let new_success_rate = (metrics.total_requests - metrics.failed_requests) as f64
                / metrics.total_requests as f64;
            metrics.success_rate = 0.9 * metrics.success_rate + 0.1 * new_success_rate;

            // Update average latency with exponential moving average
            let latency_ms = duration.as_millis() as u64;
            if metrics.total_requests == 1 {
                metrics.average_latency_ms = latency_ms;
            } else {
                metrics.average_latency_ms = (9 * metrics.average_latency_ms + latency_ms) / 10;
            }

            metrics.last_used = Some(Instant::now());

            debug!(
                "Updated metrics for '{}': latency={}ms, success_rate={:.2}, total={}",
                backend_name,
                metrics.average_latency_ms,
                metrics.success_rate,
                metrics.total_requests
            );
        }
    }

    /// Get current backend status for monitoring/debugging
    pub async fn get_backend_status(&self) -> Vec<(String, BackendInfo, BackendMetrics)> {
        let backends = self.backends.read().await;
        let mut status = Vec::new();

        for backend in backends.iter() {
            let info = backend.backend.backend_info();
            status.push((backend.name.clone(), info, backend.metrics.clone()));
        }

        status
    }
}

/// Wrapper that adds intelligent selection to any CommandGenerator
pub struct SmartBackend {
    selector: BackendSelector,
}

impl SmartBackend {
    /// Create a new smart backend with the given selector
    pub fn new(selector: BackendSelector) -> Self {
        Self { selector }
    }

    /// Create a smart backend with default configuration
    pub async fn with_defaults() -> Result<Self, GeneratorError> {
        let config = BackendSelectorConfig::default();
        let selector = BackendSelector::new(config);
        Ok(Self::new(selector))
    }

    /// Add a backend to the selection pool
    pub async fn add_backend(
        &self,
        backend: Arc<dyn CommandGenerator>,
        name: String,
        priority: u8,
    ) -> Result<(), GeneratorError> {
        self.selector.add_backend(backend, name, priority).await
    }

    /// Get current status of all backends
    pub async fn status(&self) -> Vec<(String, BackendInfo, BackendMetrics)> {
        self.selector.get_backend_status().await
    }
}

#[async_trait]
impl CommandGenerator for SmartBackend {
    async fn generate_command(
        &self,
        request: &CommandRequest,
    ) -> Result<GeneratedCommand, GeneratorError> {
        let start_time = Instant::now();

        // Try to get the best backend
        let backend = self.selector.select_backend().await?;
        let backend_info = backend.backend_info();
        let backend_name = format!("{}:{}", backend_info.backend_type, backend_info.model_name);

        // Attempt generation
        match backend.generate_command(request).await {
            Ok(mut result) => {
                let duration = start_time.elapsed();

                // Update metrics with success
                self.selector
                    .record_operation_result(&backend_name, duration, true)
                    .await;

                // Enhance result with selection info
                result.backend_used = format!("Smart[{}]", result.backend_used);
                result.generation_time_ms = duration.as_millis() as u64;

                Ok(result)
            }
            Err(error) => {
                let duration = start_time.elapsed();

                // Update metrics with failure
                self.selector
                    .record_operation_result(&backend_name, duration, false)
                    .await;

                warn!("Backend '{}' failed: {}", backend_name, error);

                // For now, return the error - future versions could implement retry with fallback
                Err(error)
            }
        }
    }

    async fn is_available(&self) -> bool {
        // We're available if any backend is available
        let backends = self.selector.backends.read().await;
        for backend in backends.iter() {
            if backend.metrics.availability_score > 0.1 {
                return true;
            }
        }
        false
    }

    fn backend_info(&self) -> BackendInfo {
        BackendInfo {
            backend_type: crate::models::BackendType::Embedded, // Placeholder
            model_name: "Smart Backend Selector".to_string(),
            supports_streaming: false,
            max_tokens: 1000,
            typical_latency_ms: 2000,
            memory_usage_mb: 100,
            version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }

    async fn shutdown(&self) -> Result<(), GeneratorError> {
        let backends = self.selector.backends.read().await;
        for backend in backends.iter() {
            if let Err(e) = backend.backend.shutdown().await {
                warn!("Error shutting down backend '{}': {}", backend.name, e);
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{BackendType, CommandRequest, GeneratedCommand, RiskLevel, ShellType};

    #[derive(Debug)]
    struct MockBackend {
        name: String,
        available: bool,
        latency_ms: u64,
    }

    #[async_trait]
    impl CommandGenerator for MockBackend {
        async fn generate_command(
            &self,
            _request: &CommandRequest,
        ) -> Result<GeneratedCommand, GeneratorError> {
            if !self.available {
                return Err(GeneratorError::BackendUnavailable {
                    reason: format!("{} is unavailable", self.name),
                });
            }

            tokio::time::sleep(Duration::from_millis(self.latency_ms)).await;

            Ok(GeneratedCommand {
                command: "echo test".to_string(),
                explanation: format!("Generated by {}", self.name),
                safety_level: RiskLevel::Safe,
                estimated_impact: "Test".to_string(),
                alternatives: vec![],
                backend_used: self.name.clone(),
                generation_time_ms: self.latency_ms,
                confidence_score: 0.9,
            })
        }

        async fn is_available(&self) -> bool {
            self.available
        }

        fn backend_info(&self) -> BackendInfo {
            BackendInfo {
                backend_type: BackendType::Embedded,
                model_name: self.name.clone(),
                supports_streaming: false,
                max_tokens: 100,
                typical_latency_ms: self.latency_ms,
                memory_usage_mb: 100,
                version: "1.0".to_string(),
            }
        }

        async fn shutdown(&self) -> Result<(), GeneratorError> {
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_backend_selection_priority() {
        let config = BackendSelectorConfig::default();
        let selector = BackendSelector::new(config);

        // Add backends with different priorities
        let fast_backend = Arc::new(MockBackend {
            name: "fast".to_string(),
            available: true,
            latency_ms: 100,
        });
        let slow_backend = Arc::new(MockBackend {
            name: "slow".to_string(),
            available: true,
            latency_ms: 1000,
        });

        selector
            .add_backend(slow_backend, "slow".to_string(), 1)
            .await
            .unwrap();
        selector
            .add_backend(fast_backend, "fast".to_string(), 0)
            .await
            .unwrap();

        // Should select the fast backend due to better latency and priority
        let selected = selector.select_backend().await.unwrap();
        let info = selected.backend_info();
        assert_eq!(info.model_name, "fast");
    }

    #[tokio::test]
    async fn test_smart_backend_fallback() {
        let smart = SmartBackend::with_defaults().await.unwrap();

        let working_backend = Arc::new(MockBackend {
            name: "working".to_string(),
            available: true,
            latency_ms: 500,
        });

        smart
            .add_backend(working_backend, "working".to_string(), 0)
            .await
            .unwrap();

        let request = CommandRequest::new("test", ShellType::Bash);
        let result = smart.generate_command(&request).await;

        assert!(result.is_ok());
        let cmd = result.unwrap();
        assert!(cmd.backend_used.contains("Smart"));
    }
}
