//! WP22-23: Performance & Cost Optimization Tests
//!
//! These tests validate token tracking, cost analysis, caching,
//! and parallel execution optimization.

/// WP22: Test token usage tracking
#[tokio::test]
async fn test_track_token_usage() {
    use caro_evaluation::token_tracker::{TokenTracker, TokenUsage};

    let tracker = TokenTracker::new();

    let usage = TokenUsage {
        backend: "smollm".to_string(),
        test_id: "test_001".to_string(),
        input_tokens: 100,
        output_tokens: 50,
    };

    tracker.track(&usage);

    let total = tracker.get_total_tokens("smollm");
    assert_eq!(total, 150, "Should track total tokens");
}

/// WP22: Test cost calculation for different backends
#[tokio::test]
async fn test_calculate_backend_cost() {
    use caro_evaluation::token_tracker::TokenUsage;

    // Local backend - no cost
    let local_usage = TokenUsage {
        backend: "smollm".to_string(),
        test_id: "test_001".to_string(),
        input_tokens: 1000,
        output_tokens: 500,
    };

    assert_eq!(local_usage.calculate_cost(), 0.0, "Local backends are free");

    // OpenAI backend - has cost
    let openai_usage = TokenUsage {
        backend: "openai".to_string(),
        test_id: "test_002".to_string(),
        input_tokens: 1000,
        output_tokens: 500,
    };

    let cost = openai_usage.calculate_cost();
    assert!(cost > 0.0, "OpenAI backend should have cost");
    assert!(cost < 1.0, "Cost should be reasonable for test volumes");
}

/// WP22: Test cost analysis report generation
#[tokio::test]
async fn test_generate_cost_analysis() {
    use caro_evaluation::token_tracker::{CostAnalyzer, TokenUsage};

    let analyzer = CostAnalyzer::new();

    let usages = vec![
        TokenUsage {
            backend: "smollm".to_string(),
            test_id: "test_001".to_string(),
            input_tokens: 100,
            output_tokens: 50,
        },
        TokenUsage {
            backend: "smollm".to_string(),
            test_id: "test_002".to_string(),
            input_tokens: 120,
            output_tokens: 60,
        },
        TokenUsage {
            backend: "qwen".to_string(),
            test_id: "test_001".to_string(),
            input_tokens: 110,
            output_tokens: 55,
        },
    ];

    let report = analyzer.generate_report(&usages);

    assert!(report.contains("smollm"), "Should include backend names");
    assert!(report.contains("Token Usage"), "Should have usage section");
    assert!(report.contains("Cost"), "Should have cost section");
}

/// WP22: Test identifying token-heavy tests
#[tokio::test]
async fn test_identify_token_heavy_tests() {
    use caro_evaluation::token_tracker::{TokenAnalyzer, TokenUsage};

    let analyzer = TokenAnalyzer::new();

    let usages = vec![
        TokenUsage {
            backend: "smollm".to_string(),
            test_id: "test_light".to_string(),
            input_tokens: 50,
            output_tokens: 25,
        },
        TokenUsage {
            backend: "smollm".to_string(),
            test_id: "test_heavy".to_string(),
            input_tokens: 1000,
            output_tokens: 500,
        },
        TokenUsage {
            backend: "smollm".to_string(),
            test_id: "test_medium".to_string(),
            input_tokens: 200,
            output_tokens: 100,
        },
    ];

    let heavy_tests = analyzer.find_heavy_tests(&usages, 1000);

    assert_eq!(heavy_tests.len(), 1, "Should find 1 heavy test");
    assert_eq!(heavy_tests[0].test_id, "test_heavy");
}

/// WP22: Test prompt optimization for token reduction
#[tokio::test]
async fn test_optimize_prompt_for_tokens() {
    use caro_evaluation::token_tracker::PromptOptimizer;

    let optimizer = PromptOptimizer::new();

    let verbose_prompt = "Please generate a shell command that will find all files in the current directory and all subdirectories that have the .txt extension and then display them.";

    let optimized = optimizer.optimize(verbose_prompt);

    assert!(
        optimized.len() < verbose_prompt.len(),
        "Optimized prompt should be shorter"
    );
    assert!(
        optimized.contains("find") || optimized.contains("txt"),
        "Should preserve key information"
    );
}

/// WP23: Test evaluation result caching
#[tokio::test]
async fn test_cache_evaluation_results() {
    use caro_evaluation::execution_cache::{CacheKey, EvalCache};

    let cache = EvalCache::new();

    let key = CacheKey {
        test_id: "test_001".to_string(),
        backend: "smollm".to_string(),
        prompt_version: "v1".to_string(),
    };

    let result = "find . -name '*.txt'".to_string();

    cache.put(key.clone(), result.clone());

    let cached = cache.get(&key);
    assert!(cached.is_some(), "Should retrieve cached result");
    assert_eq!(cached.unwrap(), result);
}

/// WP23: Test cache expiration
#[tokio::test]
async fn test_cache_expiration() {
    use caro_evaluation::execution_cache::{CacheKey, CachedResult, EvalCache};
    use chrono::{Duration, Utc};

    let cache = EvalCache::new();

    let key = CacheKey {
        test_id: "test_001".to_string(),
        backend: "smollm".to_string(),
        prompt_version: "v1".to_string(),
    };

    // Create an expired result (25 hours old)
    let expired_result = CachedResult {
        result: "old result".to_string(),
        timestamp: Utc::now() - Duration::hours(25),
    };

    cache.put_with_timestamp(key.clone(), expired_result);

    let cached = cache.get(&key);
    assert!(cached.is_none(), "Expired results should not be returned");
}

/// WP23: Test batch execution
#[tokio::test]
async fn test_batch_execution() {
    use caro_evaluation::batch_executor::BatchExecutor;
    use caro_evaluation::dataset::TestCase;

    let executor = BatchExecutor::new(3); // batch size 3

    let test_cases = vec![
        TestCase {
            id: "test_001".to_string(),
            prompt: "find files".to_string(),
            expected_command: "find .".to_string(),
            category: "correctness".to_string(),
            risk_level: "low".to_string(),
            posix_compliant: true,
            tags: vec![],
            metadata: None,
        },
        TestCase {
            id: "test_002".to_string(),
            prompt: "list files".to_string(),
            expected_command: "ls".to_string(),
            category: "correctness".to_string(),
            risk_level: "low".to_string(),
            posix_compliant: true,
            tags: vec![],
            metadata: None,
        },
    ];

    let batches = executor.create_batches(&test_cases);

    assert_eq!(
        batches.len(),
        1,
        "Should create 1 batch for 2 tests with batch size 3"
    );
    assert_eq!(batches[0].len(), 2);
}

/// WP23: Test parallel backend initialization
#[tokio::test]
async fn test_parallel_backend_initialization() {
    use caro_evaluation::batch_executor::BackendInitializer;

    let initializer = BackendInitializer::new();

    let backends = vec!["static_matcher".to_string(), "smollm".to_string()];

    let start = std::time::Instant::now();
    initializer.prewarm_parallel(&backends).await;
    let duration = start.elapsed();

    // Parallel should be faster than sequential
    // (This is a simplified test - real implementation would measure actual backends)
    assert!(
        duration.as_secs() < 10,
        "Parallel initialization should complete quickly"
    );
}

/// WP23: Test cache hit rate tracking
#[tokio::test]
async fn test_cache_hit_rate() {
    use caro_evaluation::execution_cache::{CacheKey, EvalCache};

    let cache = EvalCache::new();

    let key1 = CacheKey {
        test_id: "test_001".to_string(),
        backend: "smollm".to_string(),
        prompt_version: "v1".to_string(),
    };

    let key2 = CacheKey {
        test_id: "test_002".to_string(),
        backend: "smollm".to_string(),
        prompt_version: "v1".to_string(),
    };

    // Add one result
    cache.put(key1.clone(), "result1".to_string());

    // Hit
    let _ = cache.get(&key1);

    // Miss
    let _ = cache.get(&key2);

    let stats = cache.get_stats();
    assert_eq!(stats.hits, 1);
    assert_eq!(stats.misses, 1);
    assert_eq!(stats.hit_rate(), 0.5);
}
