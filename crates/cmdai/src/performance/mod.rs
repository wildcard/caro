//! Performance Monitoring System
//!
//! This module provides comprehensive performance monitoring for backend selection,
//! real-time metrics collection, and historical data analysis to optimize
//! command generation performance across multiple backends.

pub mod monitor;

pub use monitor::{
    BackendMetrics, HealthStatus, MetricsCollector, PerformanceMonitor, PerformanceSnapshot,
    RealTimeStats, RingBuffer, SelectionStrategy,
};
