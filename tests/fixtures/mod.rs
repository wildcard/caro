//! Test Fixtures and Utilities
//!
//! Reusable test helpers for V2 testing:
//! - Project creation (Rust, Node.js, Python, etc.)
//! - Dangerous command dataset
//! - Assertion helpers
//! - Mock data generators

use cmdai::safety::RiskLevel;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

// ============================================================================
// Test Project Builders
// ============================================================================

/// Test project types
pub enum ProjectFixture {
    Rust,
    NodeJs,
    NextJs,
    Python,
    Go,
    Docker,
    Git,
}

/// Builder for creating test projects
pub struct TestProject {
    pub path: PathBuf,
    pub project_type: ProjectFixture,
}

impl TestProject {
    /// Create a new Rust project
    pub async fn rust_project(dir: &Path) -> std::io::Result<Self> {
        tokio::fs::write(
            dir.join("Cargo.toml"),
            r#"[package]
name = "test-rust-project"
version = "0.1.0"
edition = "2021"

[dependencies]
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
anyhow = "1"
clap = { version = "4", features = ["derive"] }
"#,
        )
        .await?;

        tokio::fs::create_dir_all(dir.join("src")).await?;
        tokio::fs::write(
            dir.join("src/main.rs"),
            r#"fn main() {
    println!("Hello from test Rust project!");
}
"#,
        )
        .await?;

        tokio::fs::write(
            dir.join("src/lib.rs"),
            r#"pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add() {
        assert_eq!(add(2, 2), 4);
    }
}
"#,
        )
        .await?;

        Ok(Self {
            path: dir.to_path_buf(),
            project_type: ProjectFixture::Rust,
        })
    }

    /// Create a Node.js project
    pub async fn node_project(dir: &Path) -> std::io::Result<Self> {
        tokio::fs::write(
            dir.join("package.json"),
            r#"{
  "name": "test-node-project",
  "version": "1.0.0",
  "description": "Test Node.js project",
  "main": "index.js",
  "scripts": {
    "start": "node index.js",
    "test": "jest",
    "build": "webpack"
  },
  "dependencies": {
    "express": "^4.18.0",
    "lodash": "^4.17.21"
  },
  "devDependencies": {
    "jest": "^29.0.0",
    "webpack": "^5.0.0"
  }
}
"#,
        )
        .await?;

        tokio::fs::write(
            dir.join("index.js"),
            r#"const express = require('express');
const app = express();

app.get('/', (req, res) => {
    res.send('Hello from test Node.js project!');
});

app.listen(3000, () => {
    console.log('Server running on port 3000');
});
"#,
        )
        .await?;

        Ok(Self {
            path: dir.to_path_buf(),
            project_type: ProjectFixture::NodeJs,
        })
    }

    /// Create a Next.js project
    pub async fn nextjs_project(dir: &Path) -> std::io::Result<Self> {
        tokio::fs::write(
            dir.join("package.json"),
            r#"{
  "name": "test-nextjs-project",
  "version": "0.1.0",
  "private": true,
  "scripts": {
    "dev": "next dev",
    "build": "next build",
    "start": "next start",
    "lint": "next lint"
  },
  "dependencies": {
    "next": "14.0.0",
    "react": "^18.2.0",
    "react-dom": "^18.2.0"
  }
}
"#,
        )
        .await?;

        tokio::fs::create_dir_all(dir.join("app")).await?;
        tokio::fs::write(
            dir.join("app/page.tsx"),
            r#"export default function Home() {
  return <h1>Hello from Next.js!</h1>
}
"#,
        )
        .await?;

        Ok(Self {
            path: dir.to_path_buf(),
            project_type: ProjectFixture::NextJs,
        })
    }

    /// Create a Python project
    pub async fn python_project(dir: &Path) -> std::io::Result<Self> {
        tokio::fs::write(
            dir.join("pyproject.toml"),
            r#"[project]
name = "test-python-project"
version = "0.1.0"
description = "Test Python project"
requires-python = ">=3.8"
dependencies = [
    "fastapi>=0.100.0",
    "uvicorn>=0.23.0",
    "pydantic>=2.0.0",
    "sqlalchemy>=2.0.0"
]

[build-system]
requires = ["setuptools>=61.0"]
build-backend = "setuptools.build_meta"
"#,
        )
        .await?;

        tokio::fs::write(
            dir.join("main.py"),
            r#"from fastapi import FastAPI

app = FastAPI()

@app.get("/")
async def root():
    return {"message": "Hello from test Python project!"}

if __name__ == "__main__":
    import uvicorn
    uvicorn.run(app, host="0.0.0.0", port=8000)
"#,
        )
        .await?;

        Ok(Self {
            path: dir.to_path_buf(),
            project_type: ProjectFixture::Python,
        })
    }

    /// Create a Go project
    pub async fn go_project(dir: &Path) -> std::io::Result<Self> {
        tokio::fs::write(
            dir.join("go.mod"),
            r#"module example.com/test-go-project

go 1.21

require (
    github.com/gin-gonic/gin v1.9.1
    github.com/stretchr/testify v1.8.4
)
"#,
        )
        .await?;

        tokio::fs::write(
            dir.join("main.go"),
            r#"package main

import (
    "fmt"
    "github.com/gin-gonic/gin"
)

func main() {
    r := gin.Default()
    r.GET("/", func(c *gin.Context) {
        c.JSON(200, gin.H{
            "message": "Hello from test Go project!",
        })
    })
    fmt.Println("Starting server on :8080")
    r.Run(":8080")
}
"#,
        )
        .await?;

        Ok(Self {
            path: dir.to_path_buf(),
            project_type: ProjectFixture::Go,
        })
    }

    /// Create a Docker project
    pub async fn docker_project(dir: &Path) -> std::io::Result<Self> {
        tokio::fs::write(
            dir.join("Dockerfile"),
            r#"FROM node:18-alpine

WORKDIR /app

COPY package*.json ./
RUN npm install

COPY . .

EXPOSE 3000

CMD ["npm", "start"]
"#,
        )
        .await?;

        tokio::fs::write(
            dir.join("docker-compose.yml"),
            r#"version: '3.8'

services:
  web:
    build: .
    ports:
      - "3000:3000"
    environment:
      - NODE_ENV=production
    volumes:
      - ./data:/app/data

  db:
    image: postgres:15
    environment:
      - POSTGRES_PASSWORD=secret
    volumes:
      - postgres_data:/var/lib/postgresql/data

volumes:
  postgres_data:
"#,
        )
        .await?;

        Ok(Self {
            path: dir.to_path_buf(),
            project_type: ProjectFixture::Docker,
        })
    }

    /// Initialize as Git repository
    pub async fn init_git(&self) -> std::io::Result<()> {
        use tokio::process::Command;

        Command::new("git")
            .arg("init")
            .current_dir(&self.path)
            .output()
            .await?;

        Command::new("git")
            .args(&["config", "user.name", "Test User"])
            .current_dir(&self.path)
            .output()
            .await?;

        Command::new("git")
            .args(&["config", "user.email", "test@example.com"])
            .current_dir(&self.path)
            .output()
            .await?;

        tokio::fs::write(self.path.join("README.md"), "# Test Project").await?;

        Command::new("git")
            .args(&["add", "."])
            .current_dir(&self.path)
            .output()
            .await?;

        Command::new("git")
            .args(&["commit", "-m", "Initial commit"])
            .current_dir(&self.path)
            .output()
            .await?;

        Ok(())
    }
}

// ============================================================================
// Dangerous Commands Dataset
// ============================================================================

/// A test command from the dangerous commands dataset
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DangerousCommand {
    pub command: String,
    pub expected_risk: f32,
    pub category: String,
    pub description: String,
}

/// Load dangerous commands from JSON file
pub fn load_dangerous_commands() -> Vec<DangerousCommand> {
    let dataset_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/dangerous_commands.json");

    let contents = std::fs::read_to_string(&dataset_path)
        .expect("Failed to read dangerous commands dataset");

    serde_json::from_str(&contents).expect("Failed to parse dangerous commands dataset")
}

/// Get commands by category
pub fn commands_by_category(category: &str) -> Vec<DangerousCommand> {
    load_dangerous_commands()
        .into_iter()
        .filter(|cmd| cmd.category == category)
        .collect()
}

/// Get safe commands only
pub fn safe_commands() -> Vec<DangerousCommand> {
    commands_by_category("safe")
}

/// Get critical commands only
pub fn critical_commands() -> Vec<DangerousCommand> {
    load_dangerous_commands()
        .into_iter()
        .filter(|cmd| cmd.expected_risk >= 8.0)
        .collect()
}

// ============================================================================
// Assertion Helpers
// ============================================================================

/// Assert that a command has the expected risk level
pub fn assert_risk_level(command: &str, expected: RiskLevel) {
    use cmdai::safety::{CommandFeatures, RuleBasedPredictor};

    let features = CommandFeatures::extract(command);
    let predictor = RuleBasedPredictor::new();
    let prediction = predictor
        .predict_risk(command, &features)
        .expect("Risk prediction should succeed");

    let actual_level = prediction.risk_level();

    assert_eq!(
        actual_level, expected,
        "Command '{}' should have risk level {:?}, got {:?} (score: {:.1})",
        command, expected, actual_level, prediction.risk_score
    );
}

/// Assert that a command is safe (risk < 2.0)
pub fn assert_safe(command: &str) {
    assert_risk_level(command, RiskLevel::Safe);
}

/// Assert that a command is dangerous (risk >= 5.0)
pub fn assert_dangerous(command: &str) {
    use cmdai::safety::{CommandFeatures, RuleBasedPredictor};

    let features = CommandFeatures::extract(command);
    let predictor = RuleBasedPredictor::new();
    let prediction = predictor
        .predict_risk(command, &features)
        .expect("Risk prediction should succeed");

    assert!(
        prediction.risk_score >= 5.0,
        "Command '{}' should be dangerous (>=5.0), got {}",
        command,
        prediction.risk_score
    );
}

/// Assert that a command is critical (risk >= 8.0)
pub fn assert_critical(command: &str) {
    use cmdai::safety::{CommandFeatures, RuleBasedPredictor};

    let features = CommandFeatures::extract(command);
    let predictor = RuleBasedPredictor::new();
    let prediction = predictor
        .predict_risk(command, &features)
        .expect("Risk prediction should succeed");

    assert!(
        prediction.risk_score >= 8.0,
        "Command '{}' should be critical (>=8.0), got {}",
        command,
        prediction.risk_score
    );
}

// ============================================================================
// Mock Data Generators
// ============================================================================

/// Generate random command pattern for testing
pub fn generate_random_pattern(index: usize) -> (String, String, String) {
    let prompts = vec![
        "list files",
        "find logs",
        "build project",
        "run tests",
        "deploy app",
        "show status",
        "check errors",
        "install dependencies",
    ];

    let commands = vec![
        "ls -la",
        "find . -name '*.log'",
        "cargo build --release",
        "npm test",
        "git push origin main",
        "git status",
        "grep -r ERROR .",
        "npm install",
    ];

    let contexts = vec![
        "Rust project, Git repo",
        "Node.js project",
        "Python project",
        "Docker project",
        "Generic project",
    ];

    let prompt = prompts[index % prompts.len()].to_string();
    let command = commands[index % commands.len()].to_string();
    let context = contexts[index % contexts.len()].to_string();

    (prompt, command, context)
}

/// Generate bulk test patterns
pub fn generate_bulk_patterns(count: usize) -> Vec<(String, String, String)> {
    (0..count).map(generate_random_pattern).collect()
}

// ============================================================================
// Test Environment Helpers
// ============================================================================

/// Check if we're running in CI
pub fn is_ci() -> bool {
    std::env::var("CI").is_ok() || std::env::var("GITHUB_ACTIONS").is_ok()
}

/// Check if Git is available
pub fn has_git() -> bool {
    std::process::Command::new("git")
        .arg("--version")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

/// Check if Docker is available
pub fn has_docker() -> bool {
    std::process::Command::new("docker")
        .arg("--version")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

/// Skip test if not in CI (for long-running tests)
#[macro_export]
macro_rules! skip_if_not_ci {
    () => {
        if !$crate::fixtures::is_ci() {
            eprintln!("Skipping test (not in CI environment)");
            return;
        }
    };
}

// ============================================================================
// Performance Measurement Helpers
// ============================================================================

/// Simple performance timer
pub struct PerfTimer {
    start: std::time::Instant,
    label: String,
}

impl PerfTimer {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            start: std::time::Instant::now(),
            label: label.into(),
        }
    }

    pub fn elapsed_ms(&self) -> u128 {
        self.start.elapsed().as_millis()
    }

    pub fn assert_under(&self, max_ms: u128) {
        let elapsed = self.elapsed_ms();
        assert!(
            elapsed < max_ms,
            "{} took {}ms, expected <{}ms",
            self.label,
            elapsed,
            max_ms
        );
    }
}

impl Drop for PerfTimer {
    fn drop(&mut self) {
        println!("{}: {}ms", self.label, self.elapsed_ms());
    }
}

// ============================================================================
// Tests for Fixtures
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_rust_project_creation() {
        let temp_dir = TempDir::new().unwrap();
        let project = TestProject::rust_project(temp_dir.path()).await.unwrap();

        assert!(project.path.join("Cargo.toml").exists());
        assert!(project.path.join("src/main.rs").exists());
    }

    #[tokio::test]
    async fn test_node_project_creation() {
        let temp_dir = TempDir::new().unwrap();
        let project = TestProject::node_project(temp_dir.path()).await.unwrap();

        assert!(project.path.join("package.json").exists());
        assert!(project.path.join("index.js").exists());
    }

    #[test]
    fn test_dangerous_commands_loading() {
        let commands = load_dangerous_commands();
        assert!(commands.len() > 0, "Should load dangerous commands");
        assert!(commands.len() >= 40, "Should have at least 40 test commands");
    }

    #[test]
    fn test_safe_commands_filter() {
        let safe = safe_commands();
        assert!(safe.len() > 0, "Should have safe commands");

        for cmd in safe {
            assert_eq!(cmd.category, "safe");
            assert!(cmd.expected_risk < 3.0);
        }
    }

    #[test]
    fn test_critical_commands_filter() {
        let critical = critical_commands();
        assert!(critical.len() > 0, "Should have critical commands");

        for cmd in critical {
            assert!(cmd.expected_risk >= 8.0);
        }
    }

    #[test]
    fn test_assertion_helpers() {
        assert_safe("ls -la");
        assert_dangerous("rm -rf /tmp/*");
        assert_critical("rm -rf /");
    }

    #[test]
    fn test_bulk_pattern_generation() {
        let patterns = generate_bulk_patterns(100);
        assert_eq!(patterns.len(), 100);

        for (prompt, command, context) in patterns {
            assert!(!prompt.is_empty());
            assert!(!command.is_empty());
            assert!(!context.is_empty());
        }
    }

    #[test]
    fn test_perf_timer() {
        let timer = PerfTimer::new("test operation");
        std::thread::sleep(std::time::Duration::from_millis(10));
        assert!(timer.elapsed_ms() >= 10);
    }
}
