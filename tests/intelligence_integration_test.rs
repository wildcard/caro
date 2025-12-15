//! Integration tests for Context Intelligence Engine
//!
//! Tests the complete context building workflow across various project types.

use cmdai::intelligence::{ContextGraph, ContextOptions, ProjectType};
use std::path::PathBuf;
use std::time::Instant;

#[tokio::test]
async fn test_context_graph_build_performance() {
    let cwd = std::env::current_dir().unwrap();
    let start = Instant::now();

    let result = ContextGraph::build(&cwd).await;
    let elapsed = start.elapsed();

    assert!(result.is_ok(), "Context build should succeed");
    assert!(
        elapsed.as_millis() < 300,
        "Context build should complete in <300ms, took {}ms",
        elapsed.as_millis()
    );

    let context = result.unwrap();
    assert!(context.is_valid(), "Context should be valid");
    println!("Context build time: {}ms", context.build_time_ms);
    println!("Summary: {}", context.summary());
}

#[tokio::test]
async fn test_detect_rust_project() {
    let fixture_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/rust_project");

    let context = ContextGraph::build(&fixture_path).await.unwrap();

    assert_eq!(
        context.project.project_type,
        ProjectType::Rust,
        "Should detect Rust project"
    );
    assert_eq!(
        context.project.name,
        Some("test-rust-project".to_string()),
        "Should extract project name"
    );
    assert!(!context.project.key_dependencies.is_empty(), "Should extract dependencies");
}

#[tokio::test]
async fn test_detect_nodejs_nextjs_project() {
    let fixture_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/node_project");

    let context = ContextGraph::build(&fixture_path).await.unwrap();

    // Should detect as Next.js project
    assert_eq!(
        context.project.project_type,
        ProjectType::NextJs,
        "Should detect Next.js project from dependencies"
    );
    assert_eq!(
        context.project.name,
        Some("test-node-project".to_string())
    );
    assert!(context.project.key_dependencies.contains(&"next".to_string()));
    assert!(!context.project.available_scripts.is_empty(), "Should extract npm scripts");
}

#[tokio::test]
async fn test_detect_python_project() {
    let fixture_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/python_project");

    let context = ContextGraph::build(&fixture_path).await.unwrap();

    assert_eq!(context.project.project_type, ProjectType::Python);
    assert_eq!(
        context.project.name,
        Some("test-python-project".to_string())
    );
    assert!(context.project.key_dependencies.contains(&"fastapi>=0.100.0".to_string()) ||
            context.project.key_dependencies.iter().any(|d| d.starts_with("fastapi")));
}

#[tokio::test]
async fn test_detect_docker_project() {
    let fixture_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/docker_project");

    let context = ContextGraph::build(&fixture_path).await.unwrap();

    // Should detect Docker in additional types
    assert!(
        context.project.project_type == ProjectType::Docker
            || context.project.additional_types.contains(&ProjectType::Docker),
        "Should detect Docker project"
    );
}

#[tokio::test]
async fn test_git_analysis_on_cmdai_repo() {
    let cwd = std::env::current_dir().unwrap();
    let context = ContextGraph::build(&cwd).await.unwrap();

    // cmdai is a git repository
    assert!(context.git.is_repo, "Should detect cmdai as git repo");
    assert!(context.git.branch.is_some(), "Should detect current branch");
}

#[tokio::test]
async fn test_llm_context_generation() {
    let cwd = std::env::current_dir().unwrap();
    let context = ContextGraph::build(&cwd).await.unwrap();

    let llm_context = context.to_llm_context();
    assert!(!llm_context.is_empty(), "LLM context should not be empty");
    assert!(
        llm_context.contains("Shell:") || llm_context.contains("Platform:"),
        "Should contain environment info"
    );

    // Print for manual inspection
    println!("Generated LLM Context:\n{}", llm_context);
}

#[tokio::test]
async fn test_context_with_custom_options() {
    let cwd = std::env::current_dir().unwrap();

    // Test with history disabled (faster)
    let options = ContextOptions {
        enable_git: true,
        enable_tools: true,
        enable_history: false,
        timeout_ms: 200,
    };

    let result = ContextGraph::build_with_options(&cwd, options).await;
    assert!(result.is_ok());

    let context = result.unwrap();
    assert!(context.history.frequent_commands.is_empty(), "History should be disabled");
}

#[tokio::test]
async fn test_graceful_degradation_on_invalid_path() {
    let invalid_path = PathBuf::from("/nonexistent/path/12345");

    let result = ContextGraph::build(&invalid_path).await;

    // Should succeed with warnings, not fail
    if let Ok(context) = result {
        assert!(!context.warnings.is_empty(), "Should have warnings");
        assert!(context.environment.platform.len() > 0, "Should still have platform info");
    }
}

#[tokio::test]
async fn test_tool_detection() {
    let cwd = std::env::current_dir().unwrap();
    let context = ContextGraph::build(&cwd).await.unwrap();

    // Should detect at least some common tools (git should be available in CI)
    println!("Detected {} tools", context.infrastructure.tools.len());
    for tool in &context.infrastructure.tools {
        println!("  - {} {:?}", tool.name, tool.version);
    }
}

#[tokio::test]
async fn test_performance_metrics() {
    let cwd = std::env::current_dir().unwrap();
    let context = ContextGraph::build(&cwd).await.unwrap();

    let metrics = context.performance_metrics();
    assert!(metrics.total_time_ms < 300, "Should meet performance target");
    assert!(metrics.analyzers_run >= 1, "Should run at least environment analyzer");

    println!("Performance Metrics:");
    println!("  Total time: {}ms", metrics.total_time_ms);
    println!("  Analyzers run: {}", metrics.analyzers_run);
    println!("  Warnings: {}", metrics.warning_count);
}
