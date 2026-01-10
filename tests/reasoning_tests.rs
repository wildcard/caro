//! Integration tests for the reasoning mode
//!
//! Tests the query analysis, project detection, and context enrichment features.

use caro::context::ExecutionContext;
use caro::reasoning::{
    ContextFetchPolicy, ProjectDetector, ProjectType, QueryAnalyzer, QueryClassification,
    ReasoningConfig, ReasoningEngine, ReasoningMode,
};
use std::path::PathBuf;
use tempfile::TempDir;

/// Create a test execution context
fn test_context() -> ExecutionContext {
    ExecutionContext {
        os: "linux".to_string(),
        arch: "x86_64".to_string(),
        os_version: "5.15.0".to_string(),
        distribution: Some("Ubuntu 22.04".to_string()),
        cwd: PathBuf::from("/home/user/project"),
        shell: "bash".to_string(),
        user: "testuser".to_string(),
        available_commands: vec![
            "ls".to_string(),
            "find".to_string(),
            "grep".to_string(),
            "git".to_string(),
        ],
    }
}

// ============================================================================
// Query Analyzer Tests
// ============================================================================

mod query_analyzer_tests {
    use super::*;

    #[test]
    fn test_classify_file_system_operations() {
        let analyzer = QueryAnalyzer::new();
        let context = test_context();

        let queries = [
            "list all files",
            "find rust files",
            "show directory contents",
            "delete old logs",
            "copy config files",
        ];

        for query in queries {
            let analysis = analyzer.analyze(query, &context);
            assert_eq!(
                analysis.classification,
                QueryClassification::FileSystem,
                "Query '{}' should be classified as FileSystem",
                query
            );
        }
    }

    #[test]
    fn test_classify_package_operations() {
        let analyzer = QueryAnalyzer::new();
        let context = test_context();

        let queries = [
            "run npm install",
            "cargo build",
            "pip install requirements",
            "make clean",
        ];

        for query in queries {
            let analysis = analyzer.analyze(query, &context);
            assert_eq!(
                analysis.classification,
                QueryClassification::Package,
                "Query '{}' should be classified as Package",
                query
            );
        }
    }

    #[test]
    fn test_classify_network_operations() {
        let analyzer = QueryAnalyzer::new();
        let context = test_context();

        let queries = [
            "show network ports",
            "curl the api endpoint",
            "ping google.com",
        ];

        for query in queries {
            let analysis = analyzer.analyze(query, &context);
            assert_eq!(
                analysis.classification,
                QueryClassification::Network,
                "Query '{}' should be classified as Network",
                query
            );
        }
    }

    #[test]
    fn test_directory_relative_detection() {
        let analyzer = QueryAnalyzer::new();
        let context = test_context();

        // Should be directory-relative
        let relative_queries = [
            "find files here",
            "list current directory",
            "files in this folder",
            "npm run build", // Package commands are relative to project
        ];

        for query in relative_queries {
            let analysis = analyzer.analyze(query, &context);
            assert!(
                analysis.is_directory_relative(),
                "Query '{}' should be directory-relative",
                query
            );
        }

        // Should NOT be directory-relative
        let general_queries = ["what's my IP", "system uptime"];

        for query in general_queries {
            let analysis = analyzer.analyze(query, &context);
            assert!(
                !analysis.is_directory_relative()
                    || analysis.classification == QueryClassification::General,
                "Query '{}' should not be directory-relative",
                query
            );
        }
    }

    #[test]
    fn test_ambiguity_detection() {
        let analyzer = QueryAnalyzer::new();
        let context = test_context();

        // Should be ambiguous
        let ambiguous_queries = [
            "build",        // No tool specified
            "run something", // Vague target
        ];

        for query in ambiguous_queries {
            let analysis = analyzer.analyze(query, &context);
            assert!(
                analysis.ambiguity_score > 0.3,
                "Query '{}' should have high ambiguity score, got {}",
                query,
                analysis.ambiguity_score
            );
        }

        // Should be clear (longer, more specific queries)
        let clear_queries = [
            "list all .rs files recursively",
            "npm run build in this project",
            "show git status for current repo",
        ];

        for query in clear_queries {
            let analysis = analyzer.analyze(query, &context);
            assert!(
                analysis.ambiguity_score < 0.5,
                "Query '{}' should have low ambiguity score, got {}",
                query,
                analysis.ambiguity_score
            );
        }
    }

    #[test]
    fn test_tool_context_detection() {
        let analyzer = QueryAnalyzer::new();
        let context = test_context();

        // Should need tool context
        let needs_tool = analyzer.analyze("build the project", &context);
        assert!(
            needs_tool.needs_tool_context(),
            "Generic 'build' should need tool context"
        );

        // Should NOT need tool context (explicit tool)
        let has_tool = analyzer.analyze("npm run build", &context);
        assert!(
            !has_tool.needs_tool_context(),
            "Explicit npm should not need tool context"
        );
    }
}

// ============================================================================
// Project Detection Tests
// ============================================================================

mod project_detection_tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_detect_rust_project() {
        let temp_dir = TempDir::new().unwrap();
        let cargo_path = temp_dir.path().join("Cargo.toml");
        fs::write(
            &cargo_path,
            r#"[package]
name = "test"
version = "0.1.0"
"#,
        )
        .unwrap();

        let detector = ProjectDetector::new();
        let context = detector.detect(temp_dir.path());

        assert_eq!(context.project_type, ProjectType::Rust);
        assert!(context
            .available_tools
            .iter()
            .any(|t| t.contains("cargo")));
    }

    #[test]
    fn test_detect_node_project() {
        let temp_dir = TempDir::new().unwrap();
        let package_path = temp_dir.path().join("package.json");
        fs::write(
            &package_path,
            r#"{
  "name": "test",
  "scripts": {
    "build": "tsc",
    "test": "jest"
  }
}"#,
        )
        .unwrap();

        let detector = ProjectDetector::new();
        let context = detector.detect(temp_dir.path());

        assert_eq!(context.project_type, ProjectType::Node);
    }

    #[test]
    fn test_detect_python_project() {
        let temp_dir = TempDir::new().unwrap();
        let pyproject_path = temp_dir.path().join("pyproject.toml");
        fs::write(&pyproject_path, "[project]\nname = \"test\"\n").unwrap();

        let detector = ProjectDetector::new();
        let context = detector.detect(temp_dir.path());

        assert_eq!(context.project_type, ProjectType::Python);
    }

    #[test]
    fn test_detect_make_project() {
        let temp_dir = TempDir::new().unwrap();
        let makefile_path = temp_dir.path().join("Makefile");
        fs::write(
            &makefile_path,
            r#"
.PHONY: all clean

all: build

build:
	echo "building"

test:
	echo "testing"
"#,
        )
        .unwrap();

        let detector = ProjectDetector::new();
        let context = detector.detect(temp_dir.path());

        assert_eq!(context.project_type, ProjectType::Make);
        assert!(context
            .available_tools
            .iter()
            .any(|t| t.contains("make")));
    }

    #[test]
    fn test_detect_git_repo() {
        let temp_dir = TempDir::new().unwrap();
        let git_dir = temp_dir.path().join(".git");
        fs::create_dir(&git_dir).unwrap();

        let detector = ProjectDetector::new();
        let context = detector.detect(temp_dir.path());

        assert!(context.is_git_repo);
    }

    #[test]
    fn test_detect_package_manager_npm() {
        let temp_dir = TempDir::new().unwrap();
        fs::write(temp_dir.path().join("package.json"), "{}").unwrap();
        fs::write(temp_dir.path().join("package-lock.json"), "{}").unwrap();

        let detector = ProjectDetector::new();
        let context = detector.detect(temp_dir.path());

        assert_eq!(context.package_manager, Some("npm".to_string()));
    }

    #[test]
    fn test_detect_package_manager_yarn() {
        let temp_dir = TempDir::new().unwrap();
        fs::write(temp_dir.path().join("package.json"), "{}").unwrap();
        fs::write(temp_dir.path().join("yarn.lock"), "").unwrap();

        let detector = ProjectDetector::new();
        let context = detector.detect(temp_dir.path());

        assert_eq!(context.package_manager, Some("yarn".to_string()));
    }

    #[test]
    fn test_multi_toolchain_project() {
        let temp_dir = TempDir::new().unwrap();

        // Create both Rust and Make indicators
        fs::write(temp_dir.path().join("Cargo.toml"), "[package]\nname = \"test\"").unwrap();
        fs::write(temp_dir.path().join("Makefile"), "all:\n\techo ok").unwrap();

        let detector = ProjectDetector::new();
        let context = detector.detect(temp_dir.path());

        // Should detect both, with Rust as primary (higher priority)
        assert_eq!(context.project_type, ProjectType::Rust);
        assert!(context.toolchains.len() >= 2);
    }
}

// ============================================================================
// Reasoning Configuration Tests
// ============================================================================

mod config_tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = ReasoningConfig::default();
        assert!(config.enabled);
        assert_eq!(config.mode, ReasoningMode::Auto);
        assert_eq!(config.context_fetch_policy, ContextFetchPolicy::SafeOnly);
    }

    #[test]
    fn test_fast_config() {
        let config = ReasoningConfig::fast();
        assert!(config.enabled);
        assert_eq!(config.mode, ReasoningMode::Fast);
        assert_eq!(config.max_tree_depth, 1);
    }

    #[test]
    fn test_thorough_config() {
        let config = ReasoningConfig::thorough();
        assert_eq!(config.mode, ReasoningMode::Thorough);
        assert_eq!(config.context_fetch_policy, ContextFetchPolicy::AutoFetch);
        assert_eq!(config.max_tree_depth, 5);
    }

    #[test]
    fn test_disabled_config() {
        let config = ReasoningConfig::disabled();
        assert!(!config.enabled);
    }

    #[test]
    fn test_mode_parsing() {
        assert_eq!("auto".parse::<ReasoningMode>().unwrap(), ReasoningMode::Auto);
        assert_eq!("fast".parse::<ReasoningMode>().unwrap(), ReasoningMode::Fast);
        assert_eq!("off".parse::<ReasoningMode>().unwrap(), ReasoningMode::Off);
        assert_eq!("thorough".parse::<ReasoningMode>().unwrap(), ReasoningMode::Thorough);
        assert!("invalid".parse::<ReasoningMode>().is_err());
    }

    #[test]
    fn test_mode_capabilities() {
        assert!(!ReasoningMode::Off.allows_clarification());
        assert!(!ReasoningMode::Fast.allows_clarification());
        assert!(ReasoningMode::Auto.allows_clarification());
        assert!(ReasoningMode::Thorough.allows_clarification());
        assert!(ReasoningMode::Interactive.is_interactive());
    }
}

// ============================================================================
// Reasoning Engine Tests
// ============================================================================

mod reasoning_engine_tests {
    use super::*;

    #[test]
    fn test_should_apply_reasoning() {
        let engine = ReasoningEngine::with_defaults();

        // Should apply to complex queries
        assert!(engine.should_apply_reasoning("find all rust files and count lines"));
        assert!(engine.should_apply_reasoning("build the project and run tests"));

        // Should skip very simple queries
        assert!(!engine.should_apply_reasoning("ls"));
        assert!(!engine.should_apply_reasoning("pwd"));
    }

    #[tokio::test]
    async fn test_analyze_simple_query() {
        let config = ReasoningConfig::default();
        let engine = ReasoningEngine::new(config);
        let context = test_context();

        let result = engine.analyze("list all files", &context).await;

        assert!(result.ready_to_generate);
        assert_eq!(
            result.analysis.classification,
            QueryClassification::FileSystem
        );
    }

    #[tokio::test]
    async fn test_analyze_package_query() {
        let config = ReasoningConfig::default();
        let engine = ReasoningEngine::new(config);
        let context = test_context();

        let result = engine.analyze("run npm install", &context).await;

        assert!(result.ready_to_generate);
        assert_eq!(result.analysis.classification, QueryClassification::Package);
    }

    #[tokio::test]
    async fn test_analyze_with_context_enrichment() {
        let temp_dir = TempDir::new().unwrap();
        std::fs::write(temp_dir.path().join("test.txt"), "content").unwrap();
        std::fs::create_dir(temp_dir.path().join("src")).unwrap();

        let context = ExecutionContext {
            os: "linux".to_string(),
            arch: "x86_64".to_string(),
            os_version: "5.15.0".to_string(),
            distribution: Some("Ubuntu 22.04".to_string()),
            cwd: temp_dir.path().to_path_buf(),
            shell: "bash".to_string(),
            user: "testuser".to_string(),
            available_commands: vec!["ls".to_string()],
        };

        let config = ReasoningConfig::thorough();
        let engine = ReasoningEngine::new(config);

        let result = engine.analyze("find files in this directory", &context).await;

        assert!(result.ready_to_generate);
        assert!(result.enriched_context.is_some());
    }
}
