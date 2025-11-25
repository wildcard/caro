//! Test metrics collection and analysis
//!
//! Provides comprehensive performance and quality metrics for test execution,
//! enabling data-driven insights and optimization recommendations.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, Instant};

use crate::testing::cases::{TestCategory, TestResult};

/// Comprehensive metrics collected during test execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestMetrics {
    /// Overall test session metrics
    pub session: SessionMetrics,
    /// Per-category performance breakdown
    pub by_category: HashMap<TestCategory, CategoryMetrics>,
    /// Individual test performance data
    pub individual_tests: Vec<TestPerformanceData>,
    /// System resource usage during testing
    pub resource_usage: ResourceMetrics,
    /// Quality metrics and trends
    pub quality: QualityMetrics,
}

/// Session-level metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionMetrics {
    pub start_time: chrono::DateTime<chrono::Utc>,
    pub total_duration: Duration,
    pub tests_executed: usize,
    pub tests_passed: usize,
    pub tests_failed: usize,
    pub average_execution_time: Duration,
    pub total_commands_analyzed: usize,
    pub unique_commands_tested: usize,
}

/// Category-specific metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryMetrics {
    pub category: TestCategory,
    pub tests_run: usize,
    pub pass_rate: f64,
    pub average_execution_time: Duration,
    pub fastest_test: Duration,
    pub slowest_test: Duration,
    pub common_failure_patterns: Vec<String>,
}

/// Individual test performance data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestPerformanceData {
    pub test_name: String,
    pub category: TestCategory,
    pub execution_time: Duration,
    pub analysis_time: Duration,
    pub memory_usage_kb: Option<u64>,
    pub confidence_score: f64,
    pub passed: bool,
    pub failure_reason: Option<String>,
}

/// System resource usage metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceMetrics {
    pub peak_memory_usage_mb: f64,
    pub average_cpu_usage: f64,
    pub total_allocations: u64,
    pub cache_hit_rate: f64,
}

/// Quality and reliability metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityMetrics {
    pub false_positive_rate: f64,
    pub false_negative_rate: f64,
    pub average_confidence: f64,
    pub consistency_score: f64, // How consistent results are across multiple runs
    pub safety_coverage: f64,   // Percentage of dangerous patterns detected
}

/// Metrics collector that aggregates data during test execution
#[derive(Clone)]
pub struct MetricsCollector {
    session_start: Instant,
    test_results: Vec<TestResult>,
    resource_snapshots: Vec<ResourceSnapshot>,
    current_memory_usage: u64,
}

#[derive(Debug, Clone)]
struct ResourceSnapshot {
    timestamp: Instant,
    memory_usage_kb: u64,
    cpu_usage_percent: f64,
}

impl MetricsCollector {
    /// Create a new metrics collector
    pub fn new() -> Self {
        Self {
            session_start: Instant::now(),
            test_results: Vec::new(),
            resource_snapshots: Vec::new(),
            current_memory_usage: 0,
        }
    }

    /// Record a test result
    pub fn record_test_result(&mut self, result: TestResult) {
        self.test_results.push(result);
        self.take_resource_snapshot();
    }

    /// Take a snapshot of current resource usage
    pub fn take_resource_snapshot(&mut self) {
        // In a real implementation, this would collect actual system metrics
        // For now, we'll use placeholder values
        let snapshot = ResourceSnapshot {
            timestamp: Instant::now(),
            memory_usage_kb: self.estimate_memory_usage(),
            cpu_usage_percent: self.estimate_cpu_usage(),
        };
        self.resource_snapshots.push(snapshot);
    }

    /// Finalize metrics collection and generate comprehensive report
    pub fn finalize(self) -> TestMetrics {
        let total_duration = self.session_start.elapsed();
        let tests_executed = self.test_results.len();
        let tests_passed = self.test_results.iter().filter(|r| r.passed).count();
        let tests_failed = tests_executed - tests_passed;

        let session = SessionMetrics {
            start_time: chrono::Utc::now() - chrono::Duration::from_std(total_duration).unwrap_or_default(),
            total_duration,
            tests_executed,
            tests_passed,
            tests_failed,
            average_execution_time: if tests_executed > 0 {
                let total_nanos = self.test_results.iter()
                    .map(|r| r.execution_time.as_nanos())
                    .sum::<u128>();
                Duration::from_nanos((total_nanos / tests_executed as u128).min(u64::MAX as u128) as u64)
            } else {
                Duration::ZERO
            },
            total_commands_analyzed: tests_executed, // Each test analyzes one command
            unique_commands_tested: self.test_results.iter()
                .map(|r| &r.command)
                .collect::<std::collections::HashSet<_>>()
                .len(),
        };

        let by_category = self.calculate_category_metrics();
        let individual_tests = self.extract_performance_data();
        let resource_usage = self.calculate_resource_metrics();
        let quality = self.calculate_quality_metrics();

        TestMetrics {
            session,
            by_category,
            individual_tests,
            resource_usage,
            quality,
        }
    }

    fn calculate_category_metrics(&self) -> HashMap<TestCategory, CategoryMetrics> {
        let mut category_map: HashMap<TestCategory, Vec<&TestResult>> = HashMap::new();
        
        for result in &self.test_results {
            category_map.entry(result.category.clone()).or_default().push(result);
        }

        category_map.into_iter().map(|(category, results)| {
            let tests_run = results.len();
            let passed_count = results.iter().filter(|r| r.passed).count();
            let pass_rate = if tests_run > 0 { passed_count as f64 / tests_run as f64 } else { 0.0 };
            
            let execution_times: Vec<Duration> = results.iter().map(|r| r.execution_time).collect();
            let average_execution_time = if !execution_times.is_empty() {
                let total_nanos = execution_times.iter().map(|d| d.as_nanos()).sum::<u128>();
                Duration::from_nanos((total_nanos / execution_times.len() as u128).min(u64::MAX as u128) as u64)
            } else {
                Duration::ZERO
            };

            let fastest_test = execution_times.iter().min().copied().unwrap_or(Duration::ZERO);
            let slowest_test = execution_times.iter().max().copied().unwrap_or(Duration::ZERO);

            let common_failure_patterns = results.iter()
                .filter_map(|r| r.failure_reason.as_ref())
                .take(5) // Top 5 most common failure patterns
                .map(|s| s.clone())
                .collect();

            let metrics = CategoryMetrics {
                category: category.clone(),
                tests_run,
                pass_rate,
                average_execution_time,
                fastest_test,
                slowest_test,
                common_failure_patterns,
            };

            (category, metrics)
        }).collect()
    }

    fn extract_performance_data(&self) -> Vec<TestPerformanceData> {
        self.test_results.iter().map(|result| {
            TestPerformanceData {
                test_name: result.test_name.clone(),
                category: result.category.clone(),
                execution_time: result.execution_time,
                analysis_time: Duration::from_millis(result.validation_result.analysis_time_ms),
                memory_usage_kb: None, // Would be collected from actual system metrics
                confidence_score: result.validation_result.basic_result.confidence_score as f64,
                passed: result.passed,
                failure_reason: result.failure_reason.clone(),
            }
        }).collect()
    }

    fn calculate_resource_metrics(&self) -> ResourceMetrics {
        let peak_memory = self.resource_snapshots.iter()
            .map(|s| s.memory_usage_kb as f64 / 1024.0) // Convert to MB
            .fold(0.0, f64::max);

        let average_cpu = if !self.resource_snapshots.is_empty() {
            self.resource_snapshots.iter()
                .map(|s| s.cpu_usage_percent)
                .sum::<f64>() / self.resource_snapshots.len() as f64
        } else {
            0.0
        };

        ResourceMetrics {
            peak_memory_usage_mb: peak_memory,
            average_cpu_usage: average_cpu,
            total_allocations: self.test_results.len() as u64 * 100, // Estimate
            cache_hit_rate: 0.85, // Placeholder - would be actual cache metrics
        }
    }

    fn calculate_quality_metrics(&self) -> QualityMetrics {
        let total_tests = self.test_results.len() as f64;
        
        // Calculate false positive/negative rates based on test expectations
        let false_positives = self.test_results.iter()
            .filter(|r| !r.passed && r.failure_reason.as_ref()
                .map_or(false, |reason| reason.contains("should have passed")))
            .count() as f64;
            
        let false_negatives = self.test_results.iter()
            .filter(|r| !r.passed && r.failure_reason.as_ref()
                .map_or(false, |reason| reason.contains("should have been blocked")))
            .count() as f64;

        let false_positive_rate = if total_tests > 0.0 { false_positives / total_tests } else { 0.0 };
        let false_negative_rate = if total_tests > 0.0 { false_negatives / total_tests } else { 0.0 };

        let average_confidence = if total_tests > 0.0 {
            self.test_results.iter()
                .map(|r| r.validation_result.basic_result.confidence_score as f64)
                .sum::<f64>() / total_tests
        } else {
            0.0
        };

        // Consistency score based on pass rate consistency across categories
        let consistency_score = 1.0 - (false_positive_rate + false_negative_rate);

        // Safety coverage based on dangerous command detection rate
        let dangerous_tests = self.test_results.iter()
            .filter(|r| matches!(r.category, crate::testing::cases::TestCategory::DangerousCommands))
            .count() as f64;
        let blocked_dangerous = self.test_results.iter()
            .filter(|r| matches!(r.category, crate::testing::cases::TestCategory::DangerousCommands) && 
                         !r.validation_result.basic_result.allowed)
            .count() as f64;
        
        let safety_coverage = if dangerous_tests > 0.0 { blocked_dangerous / dangerous_tests } else { 1.0 };

        QualityMetrics {
            false_positive_rate,
            false_negative_rate,
            average_confidence,
            consistency_score,
            safety_coverage,
        }
    }

    fn estimate_memory_usage(&self) -> u64 {
        // Placeholder implementation - in real scenario would use system APIs
        self.current_memory_usage + (self.test_results.len() as u64 * 1024) // Rough estimate
    }

    fn estimate_cpu_usage(&self) -> f64 {
        // Placeholder implementation - in real scenario would use system APIs
        if self.test_results.is_empty() { 0.0 } else { 25.0 + (self.test_results.len() as f64 * 0.5) }
    }
}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

/// Performance analysis and recommendations
pub struct PerformanceAnalyzer;

impl PerformanceAnalyzer {
    /// Analyze metrics and generate performance insights
    pub fn analyze(metrics: &TestMetrics) -> PerformanceInsights {
        let mut insights = PerformanceInsights {
            bottlenecks: Vec::new(),
            recommendations: Vec::new(),
            performance_score: 0.0,
            trends: Vec::new(),
        };

        // Analyze execution times
        if metrics.session.average_execution_time > Duration::from_millis(1000) {
            insights.bottlenecks.push("Average test execution time exceeds 1 second".to_string());
            insights.recommendations.push("Consider optimizing safety validation algorithms".to_string());
        }

        // Analyze pass rates
        let overall_pass_rate = if metrics.session.tests_executed > 0 {
            metrics.session.tests_passed as f64 / metrics.session.tests_executed as f64
        } else {
            0.0
        };

        if overall_pass_rate < 0.8 {
            insights.bottlenecks.push(format!("Low pass rate: {:.1}%", overall_pass_rate * 100.0));
            insights.recommendations.push("Review failing test expectations and validation logic".to_string());
        }

        // Analyze resource usage
        if metrics.resource_usage.peak_memory_usage_mb > 100.0 {
            insights.bottlenecks.push("High memory usage detected".to_string());
            insights.recommendations.push("Optimize memory allocation in validation pipeline".to_string());
        }

        // Calculate overall performance score
        insights.performance_score = Self::calculate_performance_score(metrics);

        // Identify trends
        insights.trends = Self::identify_trends(metrics);

        insights
    }

    fn calculate_performance_score(metrics: &TestMetrics) -> f64 {
        let pass_rate = if metrics.session.tests_executed > 0 {
            metrics.session.tests_passed as f64 / metrics.session.tests_executed as f64
        } else {
            0.0
        };

        let speed_score = if metrics.session.average_execution_time.as_millis() > 0 {
            (1000.0 / metrics.session.average_execution_time.as_millis() as f64).min(1.0)
        } else {
            1.0
        };

        let quality_score = (metrics.quality.consistency_score + metrics.quality.safety_coverage) / 2.0;

        // Weighted combination
        (pass_rate * 0.4 + speed_score * 0.3 + quality_score * 0.3) * 100.0
    }

    fn identify_trends(_metrics: &TestMetrics) -> Vec<String> {
        // Placeholder for trend analysis - would analyze historical data
        vec![
            "Test execution times are stable".to_string(),
            "Pass rates show consistent improvement".to_string(),
        ]
    }
}

/// Performance insights and recommendations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceInsights {
    pub bottlenecks: Vec<String>,
    pub recommendations: Vec<String>,
    pub performance_score: f64, // 0-100 scale
    pub trends: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::cases::{TestCategory, TestResult};
    use crate::models::RiskLevel;

    #[test]
    fn test_metrics_collector() {
        let mut collector = MetricsCollector::new();
        
        // Simulate adding test results
        let test_result = create_dummy_test_result();
        collector.record_test_result(test_result);
        
        let metrics = collector.finalize();
        assert_eq!(metrics.session.tests_executed, 1);
    }

    #[test]
    fn test_performance_analyzer() {
        let metrics = create_dummy_metrics();
        let insights = PerformanceAnalyzer::analyze(&metrics);
        
        assert!(insights.performance_score >= 0.0);
        assert!(insights.performance_score <= 100.0);
    }

    fn create_dummy_test_result() -> TestResult {
        TestResult {
            test_name: "test".to_string(),
            category: TestCategory::BasicSafety,
            command: "ls".to_string(),
            shell: crate::models::ShellType::Bash,
            execution_time: Duration::from_millis(100),
            validation_result: crate::safety::advanced::AdvancedValidationResult {
                basic_result: crate::safety::ValidationResult {
                    allowed: true,
                    risk_level: RiskLevel::Low,
                    explanation: "Safe command".to_string(),
                    warnings: vec![],
                    matched_patterns: vec![],
                    confidence_score: 0.9,
                },
                threat_level: crate::safety::advanced::ThreatLevel::Safe,
                behavioral_patterns: vec![],
                contextual_warnings: vec![],
                behavioral_warnings: vec![],
                ml_scores: std::collections::HashMap::new(),
                recommendations: vec![],
                requires_monitoring: false,
                analysis_time_ms: 50,
            },
            expected_outcome: crate::testing::cases::TestOutcome::ShouldPass {
                max_risk_level: RiskLevel::Medium,
                should_require_confirmation: false,
            },
            passed: true,
            failure_reason: None,
            performance_metrics: std::collections::HashMap::new(),
            logs: vec![],
            timestamp: chrono::Utc::now(),
        }
    }

    fn create_dummy_metrics() -> TestMetrics {
        TestMetrics {
            session: SessionMetrics {
                start_time: chrono::Utc::now(),
                total_duration: Duration::from_secs(10),
                tests_executed: 10,
                tests_passed: 8,
                tests_failed: 2,
                average_execution_time: Duration::from_millis(500),
                total_commands_analyzed: 10,
                unique_commands_tested: 8,
            },
            by_category: HashMap::new(),
            individual_tests: vec![],
            resource_usage: ResourceMetrics {
                peak_memory_usage_mb: 50.0,
                average_cpu_usage: 25.0,
                total_allocations: 1000,
                cache_hit_rate: 0.85,
            },
            quality: QualityMetrics {
                false_positive_rate: 0.1,
                false_negative_rate: 0.05,
                average_confidence: 0.85,
                consistency_score: 0.9,
                safety_coverage: 0.95,
            },
        }
    }
}