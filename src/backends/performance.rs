//! Performance monitoring for backend metrics and analytics
//!
//! Provides comprehensive performance tracking for backend selection,
//! load balancing, and health monitoring according to the production specification.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

/// Performance monitor for backend metrics collection and analysis
#[derive(Debug)]
pub struct PerformanceMonitor {
    /// Current metrics for each backend
    pub metrics: Arc<RwLock<HashMap<String, BackendMetrics>>>,
    /// Historical performance data for trend analysis
    pub historical_data: Arc<RwLock<RingBuffer<PerformanceSnapshot>>>,
    /// Real-time statistics aggregator
    pub real_time_stats: Arc<RwLock<RealTimeStats>>,
    /// Configuration for monitoring behavior
    config: MonitorConfig,
}

/// Metrics for a specific backend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackendMetrics {
    /// Average response time over recent requests
    pub avg_response_time: Duration,
    /// Success rate (0.0 to 1.0)
    pub success_rate: f64,
    /// Availability percentage (0.0 to 1.0)
    pub availability: f64,
    /// Current queue length for pending requests
    pub queue_length: usize,
    /// Timestamp of last health check
    pub last_health_check: DateTime<Utc>,
    /// Total number of requests processed
    pub total_requests: u64,
    /// Total number of successful requests
    pub successful_requests: u64,
    /// Total number of failed requests
    pub failed_requests: u64,
    /// Current error rate (errors per minute)
    pub error_rate: f64,
    /// Average tokens per second throughput
    pub tokens_per_second: f64,
    /// Memory usage in MB (if available)
    pub memory_usage_mb: Option<u64>,
    /// CPU usage percentage (if available)
    pub cpu_usage_percent: Option<f64>,
}

/// Historical performance snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceSnapshot {
    /// Timestamp of this snapshot
    pub timestamp: DateTime<Utc>,
    /// Backend performance metrics at this time
    pub backend_metrics: HashMap<String, BackendMetrics>,
    /// System-wide metrics
    pub system_metrics: SystemMetrics,
}

/// System-wide performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMetrics {
    /// Total requests across all backends
    pub total_requests: u64,
    /// Overall system success rate
    pub overall_success_rate: f64,
    /// Average response time across all backends
    pub avg_response_time: Duration,
    /// Number of active backends
    pub active_backends: usize,
    /// Memory usage across all backends
    pub total_memory_mb: u64,
}

/// Real-time statistics aggregator
#[derive(Debug, Clone)]
pub struct RealTimeStats {
    /// Sliding window of recent response times
    response_times: VecDeque<(Instant, Duration)>,
    /// Sliding window of recent requests
    recent_requests: VecDeque<(Instant, RequestResult)>,
    /// Current requests per second
    pub requests_per_second: f64,
    /// Current average response time
    pub current_avg_response_time: Duration,
    /// Current error rate
    pub current_error_rate: f64,
    /// Last update time
    last_update: Instant,
}

/// Result of a request for metrics tracking
#[derive(Debug, Clone)]
pub struct RequestResult {
    pub backend: String,
    pub success: bool,
    pub response_time: Duration,
    pub tokens_generated: usize,
    pub error_type: Option<String>,
}

/// Ring buffer for efficient historical data storage
#[derive(Debug)]
pub struct RingBuffer<T> {
    data: VecDeque<T>,
    capacity: usize,
}

/// Configuration for performance monitoring
#[derive(Debug, Clone)]
pub struct MonitorConfig {
    /// Maximum number of historical snapshots to keep
    pub max_history_size: usize,
    /// Interval for taking performance snapshots
    pub snapshot_interval: Duration,
    /// Window size for real-time statistics (number of requests)
    pub realtime_window_size: usize,
    /// Maximum age for real-time statistics
    pub realtime_window_duration: Duration,
    /// Health check interval for backends
    pub health_check_interval: Duration,
}

impl Default for MonitorConfig {
    fn default() -> Self {
        Self {
            max_history_size: 1000,
            snapshot_interval: Duration::from_secs(60),
            realtime_window_size: 100,
            realtime_window_duration: Duration::from_secs(300), // 5 minutes
            health_check_interval: Duration::from_secs(30),
        }
    }
}

impl PerformanceMonitor {
    /// Create a new performance monitor with default configuration
    pub fn new() -> Self {
        Self::with_config(MonitorConfig::default())
    }

    /// Create a new performance monitor with custom configuration
    pub fn with_config(config: MonitorConfig) -> Self {
        Self {
            metrics: Arc::new(RwLock::new(HashMap::new())),
            historical_data: Arc::new(RwLock::new(RingBuffer::new(config.max_history_size))),
            real_time_stats: Arc::new(RwLock::new(RealTimeStats::new(config.realtime_window_size))),
            config,
        }
    }

    /// Record a request result for performance tracking
    pub fn record_request(&self, result: RequestResult) {
        // Update backend-specific metrics
        {
            let mut metrics = self.metrics.write().unwrap();
            let backend_metrics = metrics
                .entry(result.backend.clone())
                .or_insert_with(|| BackendMetrics::new(&result.backend));

            backend_metrics.record_request(&result);
        }

        // Update real-time statistics
        {
            let mut stats = self.real_time_stats.write().unwrap();
            stats.record_request(result.clone());
        }
    }

    /// Get current metrics for a specific backend
    pub fn get_backend_metrics(&self, backend: &str) -> Option<BackendMetrics> {
        let metrics = self.metrics.read().unwrap();
        metrics.get(backend).cloned()
    }

    /// Get metrics for all backends
    pub fn get_all_metrics(&self) -> HashMap<String, BackendMetrics> {
        let metrics = self.metrics.read().unwrap();
        metrics.clone()
    }

    /// Get real-time statistics
    pub fn get_realtime_stats(&self) -> RealTimeStats {
        let stats = self.real_time_stats.read().unwrap();
        stats.clone()
    }

    /// Take a performance snapshot and add it to historical data
    pub fn take_snapshot(&self) -> PerformanceSnapshot {
        let metrics = self.get_all_metrics();
        let stats = self.get_realtime_stats();

        let system_metrics = SystemMetrics {
            total_requests: metrics.values().map(|m| m.total_requests).sum(),
            overall_success_rate: {
                let total_requests: u64 = metrics.values().map(|m| m.total_requests).sum();
                let successful_requests: u64 =
                    metrics.values().map(|m| m.successful_requests).sum();
                if total_requests > 0 {
                    successful_requests as f64 / total_requests as f64
                } else {
                    0.0
                }
            },
            avg_response_time: stats.current_avg_response_time,
            active_backends: metrics.len(),
            total_memory_mb: metrics.values().filter_map(|m| m.memory_usage_mb).sum(),
        };

        let snapshot = PerformanceSnapshot {
            timestamp: Utc::now(),
            backend_metrics: metrics,
            system_metrics,
        };

        // Add to historical data
        {
            let mut history = self.historical_data.write().unwrap();
            history.push(snapshot.clone());
        }

        snapshot
    }

    /// Get historical performance data
    pub fn get_history(&self) -> Vec<PerformanceSnapshot> {
        let history = self.historical_data.read().unwrap();
        history.data.iter().cloned().collect()
    }

    /// Get performance trends for a specific backend
    pub fn get_backend_trends(&self, backend: &str, duration: Duration) -> Vec<BackendMetrics> {
        let history = self.historical_data.read().unwrap();
        let cutoff = Utc::now() - chrono::Duration::from_std(duration).unwrap_or_default();

        history
            .data
            .iter()
            .filter(|snapshot| snapshot.timestamp >= cutoff)
            .filter_map(|snapshot| snapshot.backend_metrics.get(backend).cloned())
            .collect()
    }

    /// Update health status for a backend
    pub fn update_health_status(
        &self,
        backend: &str,
        is_healthy: bool,
        response_time: Option<Duration>,
    ) {
        let mut metrics = self.metrics.write().unwrap();
        let backend_metrics = metrics
            .entry(backend.to_string())
            .or_insert_with(|| BackendMetrics::new(backend));

        backend_metrics.last_health_check = Utc::now();

        // Update availability based on health check
        if is_healthy {
            backend_metrics.availability = (backend_metrics.availability * 0.9 + 0.1).min(1.0);
        } else {
            backend_metrics.availability = (backend_metrics.availability * 0.9).max(0.0);
        }

        // Update response time if provided
        if let Some(rt) = response_time {
            backend_metrics.update_response_time(rt);
        }
    }

    /// Get the best performing backend based on current metrics
    pub fn get_best_backend(&self) -> Option<String> {
        let metrics = self.metrics.read().unwrap();

        metrics
            .iter()
            .filter(|(_, m)| m.availability > 0.8 && m.success_rate > 0.9)
            .min_by(|(_, a), (_, b)| {
                // Sort by response time, then by error rate
                a.avg_response_time
                    .partial_cmp(&b.avg_response_time)
                    .unwrap_or(std::cmp::Ordering::Equal)
                    .then_with(|| {
                        a.error_rate
                            .partial_cmp(&b.error_rate)
                            .unwrap_or(std::cmp::Ordering::Equal)
                    })
            })
            .map(|(name, _)| name.clone())
    }

    /// Check if a backend is performing well
    pub fn is_backend_healthy(&self, backend: &str) -> bool {
        if let Some(metrics) = self.get_backend_metrics(backend) {
            metrics.availability > 0.7
                && metrics.success_rate > 0.8
                && metrics.avg_response_time < Duration::from_secs(10)
                && metrics.error_rate < 0.1
        } else {
            false
        }
    }

    /// Reset metrics for a specific backend
    pub fn reset_backend_metrics(&self, backend: &str) {
        let mut metrics = self.metrics.write().unwrap();
        if let Some(backend_metrics) = metrics.get_mut(backend) {
            *backend_metrics = BackendMetrics::new(backend);
        }
    }

    /// Clean up old real-time statistics
    pub fn cleanup_old_stats(&self) {
        let mut stats = self.real_time_stats.write().unwrap();
        stats.cleanup_old_data(self.config.realtime_window_duration);
    }
}

impl Default for PerformanceMonitor {
    fn default() -> Self {
        Self::new()
    }
}

impl BackendMetrics {
    /// Create new metrics for a backend
    pub fn new(_backend_name: &str) -> Self {
        Self {
            avg_response_time: Duration::from_millis(1000), // Default 1s
            success_rate: 1.0,
            availability: 1.0,
            queue_length: 0,
            last_health_check: Utc::now(),
            total_requests: 0,
            successful_requests: 0,
            failed_requests: 0,
            error_rate: 0.0,
            tokens_per_second: 0.0,
            memory_usage_mb: None,
            cpu_usage_percent: None,
        }
    }

    /// Record a request result
    pub fn record_request(&mut self, result: &RequestResult) {
        self.total_requests += 1;

        if result.success {
            self.successful_requests += 1;
        } else {
            self.failed_requests += 1;
        }

        // Update success rate (exponential moving average)
        let current_success = if result.success { 1.0 } else { 0.0 };
        self.success_rate = self.success_rate * 0.9 + current_success * 0.1;

        // Update response time (exponential moving average)
        self.update_response_time(result.response_time);

        // Update tokens per second
        if result.response_time.as_secs_f64() > 0.0 {
            let tps = result.tokens_generated as f64 / result.response_time.as_secs_f64();
            self.tokens_per_second = self.tokens_per_second * 0.9 + tps * 0.1;
        }
    }

    /// Update average response time
    pub fn update_response_time(&mut self, response_time: Duration) {
        let current_ms = self.avg_response_time.as_millis() as f64;
        let new_ms = response_time.as_millis() as f64;
        let updated_ms = current_ms * 0.9 + new_ms * 0.1;
        self.avg_response_time = Duration::from_millis(updated_ms as u64);
    }

    /// Check if this backend is considered healthy
    pub fn is_healthy(&self) -> bool {
        self.availability > 0.7
            && self.success_rate > 0.8
            && self.avg_response_time < Duration::from_secs(10)
    }

    /// Get a performance score (0.0 to 1.0, higher is better)
    pub fn performance_score(&self) -> f64 {
        let response_score = 1.0 - (self.avg_response_time.as_secs_f64() / 10.0).min(1.0);
        let availability_weight = 0.4;
        let success_weight = 0.4;
        let response_weight = 0.2;

        self.availability * availability_weight
            + self.success_rate * success_weight
            + response_score * response_weight
    }
}

impl RealTimeStats {
    /// Create new real-time statistics tracker
    pub fn new(window_size: usize) -> Self {
        Self {
            response_times: VecDeque::with_capacity(window_size),
            recent_requests: VecDeque::with_capacity(window_size),
            requests_per_second: 0.0,
            current_avg_response_time: Duration::from_millis(1000),
            current_error_rate: 0.0,
            last_update: Instant::now(),
        }
    }

    /// Record a request for real-time statistics
    pub fn record_request(&mut self, result: RequestResult) {
        let now = Instant::now();

        // Add to sliding windows
        self.response_times.push_back((now, result.response_time));
        self.recent_requests.push_back((now, result));

        // Update statistics
        self.update_statistics();
        self.last_update = now;
    }

    /// Update calculated statistics
    fn update_statistics(&mut self) {
        let now = Instant::now();
        let window_duration = Duration::from_secs(60); // 1 minute window

        // Calculate requests per second
        let recent_count = self
            .recent_requests
            .iter()
            .filter(|(timestamp, _)| now.duration_since(*timestamp) <= window_duration)
            .count();
        self.requests_per_second = recent_count as f64 / window_duration.as_secs_f64();

        // Calculate average response time
        let recent_response_times: Vec<Duration> = self
            .response_times
            .iter()
            .filter(|(timestamp, _)| now.duration_since(*timestamp) <= window_duration)
            .map(|(_, duration)| *duration)
            .collect();

        if !recent_response_times.is_empty() {
            let total_ms: u64 = recent_response_times
                .iter()
                .map(|d| d.as_millis() as u64)
                .sum();
            self.current_avg_response_time =
                Duration::from_millis(total_ms / recent_response_times.len() as u64);
        }

        // Calculate error rate
        let recent_errors = self
            .recent_requests
            .iter()
            .filter(|(timestamp, _)| now.duration_since(*timestamp) <= window_duration)
            .filter(|(_, result)| !result.success)
            .count();
        self.current_error_rate = if recent_count > 0 {
            recent_errors as f64 / recent_count as f64
        } else {
            0.0
        };
    }

    /// Clean up old data beyond the window duration
    pub fn cleanup_old_data(&mut self, max_age: Duration) {
        let now = Instant::now();

        // Clean up response times
        while let Some((timestamp, _)) = self.response_times.front() {
            if now.duration_since(*timestamp) > max_age {
                self.response_times.pop_front();
            } else {
                break;
            }
        }

        // Clean up requests
        while let Some((timestamp, _)) = self.recent_requests.front() {
            if now.duration_since(*timestamp) > max_age {
                self.recent_requests.pop_front();
            } else {
                break;
            }
        }
    }
}

impl<T> RingBuffer<T> {
    /// Create a new ring buffer with the specified capacity
    pub fn new(capacity: usize) -> Self {
        Self {
            data: VecDeque::with_capacity(capacity),
            capacity,
        }
    }

    /// Add an item to the buffer, removing the oldest if at capacity
    pub fn push(&mut self, item: T) {
        if self.data.len() >= self.capacity {
            self.data.pop_front();
        }
        self.data.push_back(item);
    }

    /// Get the current number of items in the buffer
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Check if the buffer is empty
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Get an iterator over the items in the buffer
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.data.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_performance_monitor_creation() {
        let monitor = PerformanceMonitor::new();
        assert!(monitor.get_all_metrics().is_empty());
    }

    #[test]
    fn test_request_recording() {
        let monitor = PerformanceMonitor::new();

        let result = RequestResult {
            backend: "test_backend".to_string(),
            success: true,
            response_time: Duration::from_millis(500),
            tokens_generated: 50,
            error_type: None,
        };

        monitor.record_request(result);

        let metrics = monitor.get_backend_metrics("test_backend").unwrap();
        assert_eq!(metrics.total_requests, 1);
        assert_eq!(metrics.successful_requests, 1);
        assert_eq!(metrics.failed_requests, 0);
        assert!(metrics.success_rate > 0.0);
    }

    #[test]
    fn test_backend_metrics_updates() {
        let mut metrics = BackendMetrics::new("test");

        let result = RequestResult {
            backend: "test".to_string(),
            success: true,
            response_time: Duration::from_millis(200),
            tokens_generated: 25,
            error_type: None,
        };

        metrics.record_request(&result);

        assert_eq!(metrics.total_requests, 1);
        assert_eq!(metrics.successful_requests, 1);
        assert!(metrics.tokens_per_second > 0.0);
    }

    #[test]
    fn test_health_status_updates() {
        let monitor = PerformanceMonitor::new();

        monitor.update_health_status("backend1", true, Some(Duration::from_millis(100)));

        let metrics = monitor.get_backend_metrics("backend1").unwrap();
        assert!(metrics.availability > 0.9);
        assert!(metrics.last_health_check <= Utc::now());
    }

    #[test]
    fn test_performance_snapshot() {
        let monitor = PerformanceMonitor::new();

        // Record some requests
        for i in 0..5 {
            let result = RequestResult {
                backend: format!("backend_{}", i % 2),
                success: i % 4 != 3, // 3/4 success rate
                response_time: Duration::from_millis(100 + i * 50),
                tokens_generated: (20 + i * 5) as usize,
                error_type: if i % 4 == 3 {
                    Some("test_error".to_string())
                } else {
                    None
                },
            };
            monitor.record_request(result);
        }

        let snapshot = monitor.take_snapshot();
        assert_eq!(snapshot.backend_metrics.len(), 2);
        assert!(snapshot.system_metrics.total_requests > 0);
    }

    #[test]
    fn test_best_backend_selection() {
        let monitor = PerformanceMonitor::new();

        // Create metrics for two backends with different performance
        monitor.record_request(RequestResult {
            backend: "fast_backend".to_string(),
            success: true,
            response_time: Duration::from_millis(100),
            tokens_generated: 50,
            error_type: None,
        });

        monitor.record_request(RequestResult {
            backend: "slow_backend".to_string(),
            success: true,
            response_time: Duration::from_millis(1000),
            tokens_generated: 50,
            error_type: None,
        });

        let best = monitor.get_best_backend();
        assert_eq!(best, Some("fast_backend".to_string()));
    }

    #[test]
    fn test_ring_buffer() {
        let mut buffer = RingBuffer::new(3);

        assert!(buffer.is_empty());
        assert_eq!(buffer.len(), 0);

        buffer.push(1);
        buffer.push(2);
        buffer.push(3);
        assert_eq!(buffer.len(), 3);

        buffer.push(4); // Should remove 1
        assert_eq!(buffer.len(), 3);

        let items: Vec<&i32> = buffer.iter().collect();
        assert_eq!(items, vec![&2, &3, &4]);
    }

    #[test]
    fn test_realtime_stats() {
        let mut stats = RealTimeStats::new(10);

        let result = RequestResult {
            backend: "test".to_string(),
            success: true,
            response_time: Duration::from_millis(250),
            tokens_generated: 30,
            error_type: None,
        };

        stats.record_request(result);

        assert!(stats.requests_per_second >= 0.0);
        assert!(stats.current_avg_response_time > Duration::from_millis(0));
        assert_eq!(stats.current_error_rate, 0.0);
    }

    #[test]
    fn test_backend_health_check() {
        let monitor = PerformanceMonitor::new();

        // Record successful requests
        for _ in 0..10 {
            monitor.record_request(RequestResult {
                backend: "healthy_backend".to_string(),
                success: true,
                response_time: Duration::from_millis(100),
                tokens_generated: 25,
                error_type: None,
            });
        }

        assert!(monitor.is_backend_healthy("healthy_backend"));

        // Record failing requests
        for _ in 0..10 {
            monitor.record_request(RequestResult {
                backend: "unhealthy_backend".to_string(),
                success: false,
                response_time: Duration::from_millis(5000),
                tokens_generated: 0,
                error_type: Some("connection_failed".to_string()),
            });
        }

        assert!(!monitor.is_backend_healthy("unhealthy_backend"));
    }
}
