//! Backend Performance Monitoring Implementation
//!
//! Provides real-time metrics collection, historical performance analysis,
//! and intelligent backend selection based on performance characteristics.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};
use tracing::{debug, info, warn};


/// Central performance monitoring system for backend metrics and selection
#[derive(Debug)]
pub struct PerformanceMonitor {
    /// Per-backend performance metrics
    pub metrics: Arc<RwLock<HashMap<String, BackendMetrics>>>,
    /// Historical performance snapshots for trend analysis
    pub historical_data: Arc<RwLock<RingBuffer<PerformanceSnapshot>>>,
    /// Real-time statistics aggregator
    pub real_time_stats: Arc<RwLock<RealTimeStats>>,
    /// Maximum historical data points to retain
    max_history_size: usize,
}

/// Performance metrics for a specific backend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackendMetrics {
    pub avg_response_time: Duration,
    pub success_rate: f64,
    pub availability: f64,
    pub queue_length: usize,
    pub last_health_check: DateTime<Utc>,
    pub total_requests: u64,
    pub failed_requests: u64,
    pub min_response_time: Duration,
    pub max_response_time: Duration,
    pub p95_response_time: Duration,
    pub last_error: Option<String>,
    pub consecutive_failures: u32,
}

/// Point-in-time performance snapshot for historical analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceSnapshot {
    pub timestamp: DateTime<Utc>,
    pub backend_name: String,
    pub response_time: Duration,
    pub success: bool,
    pub queue_depth: usize,
    pub memory_usage_mb: Option<u64>,
    pub cpu_usage_percent: Option<f32>,
    pub error_message: Option<String>,
}

/// Real-time performance statistics across all backends
#[derive(Debug, Clone, Default)]
pub struct RealTimeStats {
    pub total_requests_per_minute: f64,
    pub average_response_time: Duration,
    pub global_success_rate: f64,
    pub active_backends: usize,
    pub fastest_backend: Option<String>,
    pub most_reliable_backend: Option<String>,
    pub last_updated: Option<DateTime<Utc>>,
}

/// Circular buffer for efficient historical data storage
#[derive(Debug)]
pub struct RingBuffer<T> {
    data: VecDeque<T>,
    capacity: usize,
}

/// Backend selection strategy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SelectionStrategy {
    /// Prioritize lowest latency
    LowestLatency,
    /// Prioritize highest reliability  
    HighestReliability,
    /// Balance latency and reliability
    Balanced { latency_weight: f64, reliability_weight: f64 },
    /// Round-robin among healthy backends
    RoundRobin,
    /// Prefer specific backend if available
    PreferredBackend { preferred: String, fallback: Box<SelectionStrategy> },
}

/// Health status for backend monitoring
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
    Unknown,
}

/// Metrics collection utilities
pub struct MetricsCollector {
    start_time: Instant,
    backend_name: String,
}

impl PerformanceMonitor {
    /// Create a new performance monitor with default settings
    pub fn new() -> Self {
        Self::with_capacity(1000)
    }

    /// Create a performance monitor with specified historical data capacity
    pub fn with_capacity(max_history_size: usize) -> Self {
        Self {
            metrics: Arc::new(RwLock::new(HashMap::new())),
            historical_data: Arc::new(RwLock::new(RingBuffer::new(max_history_size))),
            real_time_stats: Arc::new(RwLock::new(RealTimeStats::default())),
            max_history_size,
        }
    }

    /// Record a performance measurement for a backend
    pub async fn record_measurement(
        &self,
        backend_name: &str,
        response_time: Duration,
        success: bool,
        error_message: Option<String>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let snapshot = PerformanceSnapshot {
            timestamp: Utc::now(),
            backend_name: backend_name.to_string(),
            response_time,
            success,
            queue_depth: 0, // Will be updated by queue monitoring
            memory_usage_mb: None, // Optional system metrics
            cpu_usage_percent: None,
            error_message,
        };

        // Update historical data
        {
            let mut history = self.historical_data.write().map_err(|e| format!("Lock error: {}", e))?;
            history.push(snapshot.clone());
        }

        // Update backend metrics
        self.update_backend_metrics(backend_name, &snapshot).await?;

        // Update real-time statistics
        self.update_real_time_stats().await?;

        debug!(
            backend = backend_name,
            response_time_ms = response_time.as_millis(),
            success = success,
            "Recorded performance measurement"
        );

        Ok(())
    }

    /// Update metrics for a specific backend based on new measurement
    async fn update_backend_metrics(
        &self,
        backend_name: &str,
        snapshot: &PerformanceSnapshot,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut metrics = self.metrics.write().map_err(|e| format!("Lock error: {}", e))?;
        
        let backend_metrics = metrics.entry(backend_name.to_string()).or_insert_with(|| {
            BackendMetrics {
                avg_response_time: Duration::from_millis(0),
                success_rate: 1.0,
                availability: 1.0,
                queue_length: 0,
                last_health_check: Utc::now(),
                total_requests: 0,
                failed_requests: 0,
                min_response_time: Duration::from_secs(u64::MAX),
                max_response_time: Duration::from_millis(0),
                p95_response_time: Duration::from_millis(0),
                last_error: None,
                consecutive_failures: 0,
            }
        });

        // Update request counters
        backend_metrics.total_requests += 1;
        if !snapshot.success {
            backend_metrics.failed_requests += 1;
            backend_metrics.consecutive_failures += 1;
            backend_metrics.last_error = snapshot.error_message.clone();
        } else {
            backend_metrics.consecutive_failures = 0;
            backend_metrics.last_error = None;
        }

        // Update timing metrics
        backend_metrics.min_response_time = backend_metrics.min_response_time.min(snapshot.response_time);
        backend_metrics.max_response_time = backend_metrics.max_response_time.max(snapshot.response_time);
        
        // Calculate rolling average (exponential moving average)
        let alpha = 0.1; // Smoothing factor
        let current_avg_ms = backend_metrics.avg_response_time.as_millis() as f64;
        let new_time_ms = snapshot.response_time.as_millis() as f64;
        let new_avg_ms = (alpha * new_time_ms) + ((1.0 - alpha) * current_avg_ms);
        backend_metrics.avg_response_time = Duration::from_millis(new_avg_ms as u64);

        // Update success rate
        backend_metrics.success_rate = 
            1.0 - (backend_metrics.failed_requests as f64 / backend_metrics.total_requests as f64);

        // Update availability based on consecutive failures
        backend_metrics.availability = if backend_metrics.consecutive_failures > 5 {
            0.0
        } else if backend_metrics.consecutive_failures > 2 {
            0.5
        } else {
            1.0
        };

        backend_metrics.last_health_check = Utc::now();

        Ok(())
    }

    /// Update global real-time statistics
    async fn update_real_time_stats(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let metrics = self.metrics.read().map_err(|e| format!("Lock error: {}", e))?;
        let mut stats = self.real_time_stats.write().map_err(|e| format!("Lock error: {}", e))?;

        if metrics.is_empty() {
            return Ok(());
        }

        // Calculate global averages
        let total_requests: u64 = metrics.values().map(|m| m.total_requests).sum();
        let total_failed: u64 = metrics.values().map(|m| m.failed_requests).sum();
        let avg_response_times: Vec<Duration> = metrics.values().map(|m| m.avg_response_time).collect();

        stats.global_success_rate = if total_requests > 0 {
            1.0 - (total_failed as f64 / total_requests as f64)
        } else {
            1.0
        };

        stats.average_response_time = Duration::from_millis(
            avg_response_times.iter().map(|d| d.as_millis() as u64).sum::<u64>() / avg_response_times.len() as u64
        );

        stats.active_backends = metrics.values().filter(|m| m.availability > 0.0).count();

        // Find fastest and most reliable backends
        stats.fastest_backend = metrics
            .iter()
            .filter(|(_, m)| m.availability > 0.0)
            .min_by_key(|(_, m)| m.avg_response_time)
            .map(|(name, _)| name.clone());

        stats.most_reliable_backend = metrics
            .iter()
            .filter(|(_, m)| m.availability > 0.0)
            .max_by(|(_, a), (_, b)| a.success_rate.partial_cmp(&b.success_rate).unwrap())
            .map(|(name, _)| name.clone());

        stats.last_updated = Some(Utc::now());

        Ok(())
    }

    /// Get current metrics for a specific backend
    pub async fn get_backend_metrics(&self, backend_name: &str) -> Option<BackendMetrics> {
        let metrics = self.metrics.read().ok()?;
        metrics.get(backend_name).cloned()
    }

    /// Get current real-time statistics
    pub async fn get_real_time_stats(&self) -> RealTimeStats {
        match self.real_time_stats.read() {
            Ok(stats) => stats.clone(),
            Err(_) => {
                warn!("Failed to acquire real-time stats lock, returning default");
                RealTimeStats::default()
            }
        }
    }

    /// Get health status for a backend
    pub async fn get_health_status(&self, backend_name: &str) -> HealthStatus {
        match self.get_backend_metrics(backend_name).await {
            Some(metrics) => {
                if metrics.availability >= 0.9 && metrics.success_rate >= 0.95 {
                    HealthStatus::Healthy
                } else if metrics.availability >= 0.5 && metrics.success_rate >= 0.8 {
                    HealthStatus::Degraded
                } else if metrics.availability > 0.0 {
                    HealthStatus::Unhealthy
                } else {
                    HealthStatus::Unknown
                }
            }
            None => HealthStatus::Unknown,
        }
    }

    /// Get historical performance data for analysis
    pub async fn get_historical_data(&self, backend_name: Option<&str>, limit: Option<usize>) -> Vec<PerformanceSnapshot> {
        let history = match self.historical_data.read() {
            Ok(h) => h,
            Err(_) => {
                warn!("Failed to acquire historical data lock");
                return Vec::new();
            }
        };

        let mut data: Vec<PerformanceSnapshot> = history.iter()
            .filter(|snapshot| {
                backend_name.map_or(true, |name| snapshot.backend_name == name)
            })
            .cloned()
            .collect();

        data.sort_by(|a, b| b.timestamp.cmp(&a.timestamp)); // Most recent first

        if let Some(limit) = limit {
            data.truncate(limit);
        }

        data
    }

    /// Select best backend based on strategy
    pub async fn select_backend(
        &self,
        available_backends: &[String],
        strategy: &SelectionStrategy,
    ) -> Option<String> {
        if available_backends.is_empty() {
            return None;
        }

        let metrics = self.metrics.read().ok()?;

        match strategy {
            SelectionStrategy::LowestLatency => {
                available_backends
                    .iter()
                    .filter_map(|name| metrics.get(name).map(|m| (name, m)))
                    .filter(|(_, m)| m.availability > 0.0)
                    .min_by_key(|(_, m)| m.avg_response_time)
                    .map(|(name, _)| name.clone())
            }
            SelectionStrategy::HighestReliability => {
                available_backends
                    .iter()
                    .filter_map(|name| metrics.get(name).map(|m| (name, m)))
                    .filter(|(_, m)| m.availability > 0.0)
                    .max_by(|(_, a), (_, b)| a.success_rate.partial_cmp(&b.success_rate).unwrap())
                    .map(|(name, _)| name.clone())
            }
            SelectionStrategy::Balanced { latency_weight, reliability_weight } => {
                available_backends
                    .iter()
                    .filter_map(|name| metrics.get(name).map(|m| (name, m)))
                    .filter(|(_, m)| m.availability > 0.0)
                    .max_by(|(_, a), (_, b)| {
                        let score_a = self.calculate_balanced_score(a, *latency_weight, *reliability_weight);
                        let score_b = self.calculate_balanced_score(b, *latency_weight, *reliability_weight);
                        score_a.partial_cmp(&score_b).unwrap()
                    })
                    .map(|(name, _)| name.clone())
            }
            SelectionStrategy::RoundRobin => {
                // Simple round-robin among healthy backends
                let healthy_backends: Vec<_> = available_backends
                    .iter()
                    .filter(|name| {
                        metrics.get(*name).map_or(false, |m| m.availability > 0.0)
                    })
                    .collect();
                
                if healthy_backends.is_empty() {
                    None
                } else {
                    let index = (Utc::now().timestamp() as usize) % healthy_backends.len();
                    Some(healthy_backends[index].clone())
                }
            }
            SelectionStrategy::PreferredBackend { preferred, fallback } => {
                if available_backends.contains(preferred) {
                    let preferred_health = self.get_health_status(preferred).await;
                    if preferred_health == HealthStatus::Healthy || preferred_health == HealthStatus::Degraded {
                        return Some(preferred.clone());
                    }
                }
                // Fall back to alternative strategy
                Box::pin(self.select_backend(available_backends, fallback)).await
            }
        }
    }

    /// Calculate balanced score for backend selection
    fn calculate_balanced_score(&self, metrics: &BackendMetrics, latency_weight: f64, reliability_weight: f64) -> f64 {
        // Normalize latency score (lower is better, so invert)
        let latency_score = 1.0 / (1.0 + metrics.avg_response_time.as_millis() as f64 / 1000.0);
        let reliability_score = metrics.success_rate * metrics.availability;
        
        (latency_weight * latency_score) + (reliability_weight * reliability_score)
    }

    /// Start a metrics collection session for timing operations
    pub fn start_collection(&self, backend_name: &str) -> MetricsCollector {
        MetricsCollector {
            start_time: Instant::now(),
            backend_name: backend_name.to_string(),
        }
    }

    /// Reset all metrics (useful for testing)
    pub async fn reset_metrics(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        {
            let mut metrics = self.metrics.write().map_err(|e| format!("Lock error: {}", e))?;
            metrics.clear();
        }
        
        {
            let mut history = self.historical_data.write().map_err(|e| format!("Lock error: {}", e))?;
            history.clear();
        }

        {
            let mut stats = self.real_time_stats.write().map_err(|e| format!("Lock error: {}", e))?;
            *stats = RealTimeStats::default();
        }

        info!("Performance metrics reset");
        Ok(())
    }
}

impl<T> RingBuffer<T> {
    pub fn new(capacity: usize) -> Self {
        Self {
            data: VecDeque::with_capacity(capacity),
            capacity,
        }
    }

    pub fn push(&mut self, item: T) {
        if self.data.len() >= self.capacity {
            self.data.pop_front();
        }
        self.data.push_back(item);
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.data.iter()
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    pub fn clear(&mut self) {
        self.data.clear();
    }
}

impl MetricsCollector {
    /// Finish collection and record the measurement
    pub async fn finish(
        self,
        monitor: &PerformanceMonitor,
        success: bool,
        error_message: Option<String>,
    ) -> Result<Duration, Box<dyn std::error::Error + Send + Sync>> {
        let duration = self.start_time.elapsed();
        
        monitor.record_measurement(&self.backend_name, duration, success, error_message).await?;
        
        Ok(duration)
    }
}

impl Default for PerformanceMonitor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_performance_monitor_creation() {
        let monitor = PerformanceMonitor::new();
        let stats = monitor.get_real_time_stats().await;
        
        assert_eq!(stats.active_backends, 0);
        assert_eq!(stats.global_success_rate, 0.0);
    }

    #[tokio::test]
    async fn test_record_measurement() {
        let monitor = PerformanceMonitor::new();
        
        let result = monitor.record_measurement(
            "test_backend",
            Duration::from_millis(150),
            true,
            None,
        ).await;
        
        assert!(result.is_ok());
        
        let metrics = monitor.get_backend_metrics("test_backend").await;
        assert!(metrics.is_some());
        let metrics = metrics.unwrap();
        assert_eq!(metrics.total_requests, 1);
        assert_eq!(metrics.failed_requests, 0);
        assert_eq!(metrics.success_rate, 1.0);
    }

    #[tokio::test]
    async fn test_backend_selection_lowest_latency() {
        let monitor = PerformanceMonitor::new();
        
        // Record measurements for different backends
        monitor.record_measurement("fast", Duration::from_millis(50), true, None).await.unwrap();
        monitor.record_measurement("slow", Duration::from_millis(200), true, None).await.unwrap();
        
        let backends = vec!["fast".to_string(), "slow".to_string()];
        let selected = monitor.select_backend(&backends, &SelectionStrategy::LowestLatency).await;
        
        assert_eq!(selected, Some("fast".to_string()));
    }

    #[tokio::test]
    async fn test_backend_selection_highest_reliability() {
        let monitor = PerformanceMonitor::new();
        
        // Record measurements with different success rates
        monitor.record_measurement("reliable", Duration::from_millis(100), true, None).await.unwrap();
        monitor.record_measurement("unreliable", Duration::from_millis(80), false, Some("Error".to_string())).await.unwrap();
        
        let backends = vec!["reliable".to_string(), "unreliable".to_string()];
        let selected = monitor.select_backend(&backends, &SelectionStrategy::HighestReliability).await;
        
        assert_eq!(selected, Some("reliable".to_string()));
    }

    #[tokio::test]
    async fn test_health_status_calculation() {
        let monitor = PerformanceMonitor::new();
        
        // Healthy backend
        monitor.record_measurement("healthy", Duration::from_millis(100), true, None).await.unwrap();
        let status = monitor.get_health_status("healthy").await;
        assert_eq!(status, HealthStatus::Healthy);
        
        // Unknown backend
        let status = monitor.get_health_status("unknown").await;
        assert_eq!(status, HealthStatus::Unknown);
    }

    #[tokio::test]
    async fn test_metrics_collector() {
        let monitor = PerformanceMonitor::new();
        let collector = monitor.start_collection("test_backend");
        
        // Simulate some work
        tokio::time::sleep(Duration::from_millis(10)).await;
        
        let duration = collector.finish(&monitor, true, None).await.unwrap();
        assert!(duration >= Duration::from_millis(10));
        
        let metrics = monitor.get_backend_metrics("test_backend").await.unwrap();
        assert_eq!(metrics.total_requests, 1);
    }

    #[tokio::test]
    async fn test_ring_buffer() {
        let mut buffer = RingBuffer::new(3);
        
        buffer.push(1);
        buffer.push(2);
        buffer.push(3);
        assert_eq!(buffer.len(), 3);
        
        buffer.push(4); // Should evict 1
        assert_eq!(buffer.len(), 3);
        assert_eq!(buffer.iter().copied().collect::<Vec<_>>(), vec![2, 3, 4]);
    }

    #[tokio::test]
    async fn test_balanced_selection_strategy() {
        let monitor = PerformanceMonitor::new();
        
        // Record different performance characteristics
        monitor.record_measurement("fast_unreliable", Duration::from_millis(50), false, None).await.unwrap();
        monitor.record_measurement("slow_reliable", Duration::from_millis(200), true, None).await.unwrap();
        
        let backends = vec!["fast_unreliable".to_string(), "slow_reliable".to_string()];
        
        // Favor reliability
        let strategy = SelectionStrategy::Balanced { 
            latency_weight: 0.2, 
            reliability_weight: 0.8 
        };
        let selected = monitor.select_backend(&backends, &strategy).await;
        assert_eq!(selected, Some("slow_reliable".to_string()));
    }
}