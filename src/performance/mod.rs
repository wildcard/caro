//! Performance Monitoring System
//!
//! This module provides comprehensive performance monitoring for backend selection,
//! real-time metrics collection, and historical data analysis to optimize
//! command generation performance across multiple backends.

pub mod monitor;

pub use monitor::{
    PerformanceMonitor, BackendMetrics, PerformanceSnapshot, RealTimeStats, RingBuffer,
    SelectionStrategy, HealthStatus, MetricsCollector,
};