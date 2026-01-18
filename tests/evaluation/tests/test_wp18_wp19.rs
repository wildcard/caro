//! WP18-19: Advanced Analytics & Visualization Tests
//!
//! These tests validate time-series analysis and interactive dashboard
//! generation for stakeholder-friendly reporting and trend analysis.

/// WP18: Test evaluation result storage
#[tokio::test]
async fn test_store_evaluation_result() {
    use caro_evaluation::timeseries::{EvaluationRecord, TimeSeriesStore};

    let store = TimeSeriesStore::new_in_memory();

    let record = EvaluationRecord {
        timestamp: "2026-01-18T10:00:00Z".to_string(),
        git_commit: Some("abc123".to_string()),
        backend: "smollm".to_string(),
        category: "correctness".to_string(),
        total_tests: 55,
        passed_tests: 28,
        pass_rate: 0.51,
        metadata: None,
    };

    let result = store.store(&record);

    assert!(result.is_ok(), "Should store evaluation record");
}

/// WP18: Test querying historical data
#[tokio::test]
async fn test_query_historical_data() {
    use caro_evaluation::timeseries::{EvaluationRecord, TimeSeriesStore};

    let store = TimeSeriesStore::new_in_memory();

    // Store multiple records
    for i in 0..5 {
        let record = EvaluationRecord {
            timestamp: format!("2026-01-{:02}T10:00:00Z", 14 + i),
            git_commit: Some(format!("commit{}", i)),
            backend: "smollm".to_string(),
            category: "correctness".to_string(),
            total_tests: 55,
            passed_tests: 25 + i,
            pass_rate: (25.0 + i as f64) / 55.0,
            metadata: None,
        };
        store.store(&record).unwrap();
    }

    let history = store
        .query("smollm", Some("correctness"), 30)
        .expect("Should query history");

    assert_eq!(history.len(), 5, "Should retrieve 5 records");
    assert!(
        history[0].timestamp < history[4].timestamp,
        "Should be in chronological order"
    );
}

/// WP18: Test trend calculation
#[tokio::test]
async fn test_calculate_trend() {
    use caro_evaluation::timeseries::{EvaluationRecord, TrendAnalyzer};

    let analyzer = TrendAnalyzer::new();

    let history = vec![
        EvaluationRecord {
            timestamp: "2026-01-14T10:00:00Z".to_string(),
            git_commit: None,
            backend: "smollm".to_string(),
            category: "correctness".to_string(),
            total_tests: 55,
            passed_tests: 25,
            pass_rate: 0.45,
            metadata: None,
        },
        EvaluationRecord {
            timestamp: "2026-01-15T10:00:00Z".to_string(),
            git_commit: None,
            backend: "smollm".to_string(),
            category: "correctness".to_string(),
            total_tests: 55,
            passed_tests: 28,
            pass_rate: 0.51,
            metadata: None,
        },
        EvaluationRecord {
            timestamp: "2026-01-16T10:00:00Z".to_string(),
            git_commit: None,
            backend: "smollm".to_string(),
            category: "correctness".to_string(),
            total_tests: 55,
            passed_tests: 30,
            pass_rate: 0.55,
            metadata: None,
        },
    ];

    let trend = analyzer.calculate_trend(&history);

    assert!(trend > 0.0, "Should show positive trend (improving)");
}

/// WP18: Test anomaly detection
#[tokio::test]
async fn test_detect_anomaly() {
    use caro_evaluation::timeseries::{AnomalyDetector, EvaluationRecord};

    let detector = AnomalyDetector::new(2.0); // 2 standard deviations

    // Historical data with stable pass rate ~50%
    let history = vec![
        EvaluationRecord {
            timestamp: "2026-01-10T10:00:00Z".to_string(),
            git_commit: None,
            backend: "smollm".to_string(),
            category: "correctness".to_string(),
            total_tests: 55,
            passed_tests: 28,
            pass_rate: 0.51,
            metadata: None,
        },
        EvaluationRecord {
            timestamp: "2026-01-11T10:00:00Z".to_string(),
            git_commit: None,
            backend: "smollm".to_string(),
            category: "correctness".to_string(),
            total_tests: 55,
            passed_tests: 27,
            pass_rate: 0.49,
            metadata: None,
        },
        EvaluationRecord {
            timestamp: "2026-01-12T10:00:00Z".to_string(),
            git_commit: None,
            backend: "smollm".to_string(),
            category: "correctness".to_string(),
            total_tests: 55,
            passed_tests: 28,
            pass_rate: 0.51,
            metadata: None,
        },
    ];

    // Normal value
    assert!(
        !detector.detect(&history, 0.50),
        "Should not detect anomaly for normal value"
    );

    // Significant drop
    assert!(
        detector.detect(&history, 0.20),
        "Should detect anomaly for significant drop"
    );
}

/// WP18: Test regression alerting
#[tokio::test]
async fn test_regression_alert() {
    use caro_evaluation::timeseries::{AlertConfig, RegressionAlerter};

    let config = AlertConfig {
        threshold_pct: 10.0, // Alert on >10% drop
        lookback_days: 7,
    };

    let alerter = RegressionAlerter::new(config);

    let current_pass_rate = 0.40;
    let previous_pass_rate = 0.55;

    let should_alert = alerter.should_alert(previous_pass_rate, current_pass_rate);

    assert!(should_alert, "Should alert on >10% regression (55% -> 40%)");
}

/// WP19: Test dashboard HTML generation
#[tokio::test]
async fn test_generate_dashboard_html() {
    use caro_evaluation::dashboard::DashboardGenerator;
    use caro_evaluation::timeseries::EvaluationRecord;

    let generator = DashboardGenerator::new();

    let data = vec![
        EvaluationRecord {
            timestamp: "2026-01-18T10:00:00Z".to_string(),
            git_commit: Some("abc123".to_string()),
            backend: "smollm".to_string(),
            category: "correctness".to_string(),
            total_tests: 55,
            passed_tests: 28,
            pass_rate: 0.51,
            metadata: None,
        },
        EvaluationRecord {
            timestamp: "2026-01-18T11:00:00Z".to_string(),
            git_commit: Some("def456".to_string()),
            backend: "qwen".to_string(),
            category: "correctness".to_string(),
            total_tests: 55,
            passed_tests: 35,
            pass_rate: 0.64,
            metadata: None,
        },
    ];

    let html = generator
        .generate(&data)
        .expect("Should generate dashboard");

    assert!(html.contains("<!DOCTYPE html>"), "Should be valid HTML");
    assert!(
        html.contains("Chart.js") || html.contains("chart"),
        "Should reference charts"
    );
    assert!(html.contains("smollm"), "Should include backend names");
    assert!(
        html.contains("51") || html.contains("0.51"),
        "Should include pass rates"
    );
}

/// WP19: Test chart data embedding
#[tokio::test]
async fn test_embed_chart_data() {
    use caro_evaluation::dashboard::ChartDataBuilder;
    use caro_evaluation::timeseries::EvaluationRecord;

    let builder = ChartDataBuilder::new();

    let data = vec![
        EvaluationRecord {
            timestamp: "2026-01-18T10:00:00Z".to_string(),
            git_commit: None,
            backend: "smollm".to_string(),
            category: "correctness".to_string(),
            total_tests: 55,
            passed_tests: 28,
            pass_rate: 0.51,
            metadata: None,
        },
        EvaluationRecord {
            timestamp: "2026-01-18T11:00:00Z".to_string(),
            git_commit: None,
            backend: "smollm".to_string(),
            category: "correctness".to_string(),
            total_tests: 55,
            passed_tests: 30,
            pass_rate: 0.55,
            metadata: None,
        },
    ];

    let chart_json = builder
        .build_trend_chart(&data)
        .expect("Should build chart data");

    // Should be valid JSON
    let parsed: serde_json::Value =
        serde_json::from_str(&chart_json).expect("Should be valid JSON");

    assert!(parsed.get("labels").is_some(), "Should have labels");
    assert!(parsed.get("datasets").is_some(), "Should have datasets");
}

/// WP19: Test model comparison heatmap
#[tokio::test]
async fn test_generate_heatmap() {
    use caro_evaluation::dashboard::HeatmapGenerator;
    use caro_evaluation::timeseries::EvaluationRecord;

    let generator = HeatmapGenerator::new();

    let data = vec![
        EvaluationRecord {
            timestamp: "2026-01-18T10:00:00Z".to_string(),
            git_commit: None,
            backend: "smollm".to_string(),
            category: "correctness".to_string(),
            total_tests: 55,
            passed_tests: 28,
            pass_rate: 0.51,
            metadata: None,
        },
        EvaluationRecord {
            timestamp: "2026-01-18T10:00:00Z".to_string(),
            git_commit: None,
            backend: "qwen".to_string(),
            category: "correctness".to_string(),
            total_tests: 55,
            passed_tests: 35,
            pass_rate: 0.64,
            metadata: None,
        },
        EvaluationRecord {
            timestamp: "2026-01-18T10:00:00Z".to_string(),
            git_commit: None,
            backend: "smollm".to_string(),
            category: "safety".to_string(),
            total_tests: 20,
            passed_tests: 17,
            pass_rate: 0.85,
            metadata: None,
        },
    ];

    let heatmap_html = generator.generate(&data).expect("Should generate heatmap");

    assert!(
        heatmap_html.contains("smollm"),
        "Should include model names"
    );
    assert!(
        heatmap_html.contains("correctness"),
        "Should include categories"
    );
    assert!(
        heatmap_html.contains("51") || heatmap_html.contains("0.51"),
        "Should include pass rates"
    );
}

/// WP19: Test summary statistics generation
#[tokio::test]
async fn test_generate_summary_stats() {
    use caro_evaluation::dashboard::SummaryStats;
    use caro_evaluation::timeseries::EvaluationRecord;

    let data = vec![
        EvaluationRecord {
            timestamp: "2026-01-18T10:00:00Z".to_string(),
            git_commit: None,
            backend: "smollm".to_string(),
            category: "correctness".to_string(),
            total_tests: 55,
            passed_tests: 28,
            pass_rate: 0.51,
            metadata: None,
        },
        EvaluationRecord {
            timestamp: "2026-01-18T11:00:00Z".to_string(),
            git_commit: None,
            backend: "smollm".to_string(),
            category: "correctness".to_string(),
            total_tests: 55,
            passed_tests: 30,
            pass_rate: 0.55,
            metadata: None,
        },
    ];

    let stats = SummaryStats::from_records(&data);

    assert_eq!(stats.total_evaluations, 2);
    assert_eq!(stats.latest_pass_rate, 0.55);
    assert!(stats.average_pass_rate > 0.0);
    assert!(stats.trend_direction == "improving" || stats.trend_direction == "stable");
}
