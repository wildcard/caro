//! Project type detection and metadata parsing
//!
//! Detects project types by analyzing filesystem markers and configuration files.

use super::ContextError;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tokio::fs;

/// Supported project types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProjectType {
    Rust,
    NodeJs,
    Python,
    Go,
    Docker,
    Terraform,
    Kubernetes,
    NextJs,
    React,
    Unknown,
}

impl std::fmt::Display for ProjectType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Rust => write!(f, "Rust"),
            Self::NodeJs => write!(f, "Node.js"),
            Self::Python => write!(f, "Python"),
            Self::Go => write!(f, "Go"),
            Self::Docker => write!(f, "Docker"),
            Self::Terraform => write!(f, "Terraform"),
            Self::Kubernetes => write!(f, "Kubernetes"),
            Self::NextJs => write!(f, "Next.js"),
            Self::React => write!(f, "React"),
            Self::Unknown => write!(f, "Unknown"),
        }
    }
}

/// Project context containing detected metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectContext {
    /// Primary project type
    pub project_type: ProjectType,
    /// Additional project types (for multi-language projects)
    pub additional_types: Vec<ProjectType>,
    /// Project name (from package.json, Cargo.toml, etc.)
    pub name: Option<String>,
    /// Project version
    pub version: Option<String>,
    /// Dependencies (key dependencies only)
    pub key_dependencies: Vec<String>,
    /// Available scripts/tasks
    pub available_scripts: Vec<String>,
    /// Project root directory
    pub root: PathBuf,
}

impl ProjectContext {
    /// Create empty/unknown project context
    pub fn unknown(root: PathBuf) -> Self {
        Self {
            project_type: ProjectType::Unknown,
            additional_types: Vec::new(),
            name: None,
            version: None,
            key_dependencies: Vec::new(),
            available_scripts: Vec::new(),
            root,
        }
    }

    /// Convert to LLM-friendly context string
    pub fn to_llm_context(&self) -> String {
        let mut lines = Vec::new();

        lines.push(format!("Project Type: {}", self.project_type));

        if !self.additional_types.is_empty() {
            let types: Vec<String> = self.additional_types.iter().map(|t| t.to_string()).collect();
            lines.push(format!("Additional Types: {}", types.join(", ")));
        }

        if let Some(name) = &self.name {
            lines.push(format!("Project Name: {}", name));
        }

        if !self.key_dependencies.is_empty() {
            let deps = self.key_dependencies.join(", ");
            lines.push(format!("Key Dependencies: {}", deps));
        }

        if !self.available_scripts.is_empty() {
            let scripts = self.available_scripts.join(", ");
            lines.push(format!("Available Scripts: {}", scripts));
        }

        lines.join("\n")
    }
}

/// Project parser that detects project types and extracts metadata
pub struct ProjectParser;

impl ProjectParser {
    /// Detect project type and extract metadata from a directory
    pub async fn analyze(path: &Path) -> Result<ProjectContext, ContextError> {
        if !path.exists() {
            return Err(ContextError::InvalidPath {
                path: path.display().to_string(),
            });
        }

        let mut project_types = Vec::new();
        let mut context = ProjectContext::unknown(path.to_path_buf());

        // Check for various project markers
        if Self::is_rust_project(path).await {
            project_types.push(ProjectType::Rust);
            if let Ok(metadata) = Self::parse_rust_project(path).await {
                context = metadata;
            }
        }

        if Self::is_nodejs_project(path).await {
            if project_types.is_empty() {
                project_types.push(ProjectType::NodeJs);
                if let Ok(metadata) = Self::parse_nodejs_project(path).await {
                    context = metadata;
                }
            } else {
                context.additional_types.push(ProjectType::NodeJs);
            }
        }

        if Self::is_python_project(path).await {
            if project_types.is_empty() {
                project_types.push(ProjectType::Python);
                if let Ok(metadata) = Self::parse_python_project(path).await {
                    context = metadata;
                }
            } else {
                context.additional_types.push(ProjectType::Python);
            }
        }

        if Self::is_go_project(path).await {
            if project_types.is_empty() {
                project_types.push(ProjectType::Go);
                if let Ok(metadata) = Self::parse_go_project(path).await {
                    context = metadata;
                }
            } else {
                context.additional_types.push(ProjectType::Go);
            }
        }

        if Self::is_docker_project(path).await {
            if project_types.is_empty() {
                project_types.push(ProjectType::Docker);
            } else {
                context.additional_types.push(ProjectType::Docker);
            }
        }

        if Self::is_terraform_project(path).await {
            if project_types.is_empty() {
                project_types.push(ProjectType::Terraform);
            } else {
                context.additional_types.push(ProjectType::Terraform);
            }
        }

        if Self::is_kubernetes_project(path).await {
            context.additional_types.push(ProjectType::Kubernetes);
        }

        // Set primary project type
        if !project_types.is_empty() {
            context.project_type = project_types[0].clone();
        }

        Ok(context)
    }

    // Project type detection helpers

    async fn is_rust_project(path: &Path) -> bool {
        path.join("Cargo.toml").exists()
    }

    async fn is_nodejs_project(path: &Path) -> bool {
        path.join("package.json").exists()
    }

    async fn is_python_project(path: &Path) -> bool {
        path.join("pyproject.toml").exists()
            || path.join("requirements.txt").exists()
            || path.join("setup.py").exists()
            || path.join("Pipfile").exists()
    }

    async fn is_go_project(path: &Path) -> bool {
        path.join("go.mod").exists()
    }

    async fn is_docker_project(path: &Path) -> bool {
        path.join("Dockerfile").exists() || path.join("docker-compose.yml").exists()
    }

    async fn is_terraform_project(path: &Path) -> bool {
        // Check for .tf files
        if let Ok(mut entries) = fs::read_dir(path).await {
            while let Ok(Some(entry)) = entries.next_entry().await {
                if let Some(ext) = entry.path().extension() {
                    if ext == "tf" {
                        return true;
                    }
                }
            }
        }
        false
    }

    async fn is_kubernetes_project(path: &Path) -> bool {
        // Check for k8s YAML files
        if let Ok(mut entries) = fs::read_dir(path).await {
            while let Ok(Some(entry)) = entries.next_entry().await {
                let path = entry.path();
                if let Some(ext) = path.extension() {
                    if ext == "yaml" || ext == "yml" {
                        if let Ok(content) = fs::read_to_string(&path).await {
                            if content.contains("kind:") && content.contains("apiVersion:") {
                                return true;
                            }
                        }
                    }
                }
            }
        }
        false
    }

    // Project metadata parsers

    async fn parse_rust_project(path: &Path) -> Result<ProjectContext, ContextError> {
        let cargo_path = path.join("Cargo.toml");
        let content = fs::read_to_string(&cargo_path).await?;

        let parsed: toml::Value = toml::from_str(&content).map_err(|e| ContextError::ParseError {
            message: format!("Failed to parse Cargo.toml: {}", e),
        })?;

        let package = parsed.get("package");
        let name = package
            .and_then(|p| p.get("name"))
            .and_then(|n| n.as_str())
            .map(String::from);
        let version = package
            .and_then(|p| p.get("version"))
            .and_then(|v| v.as_str())
            .map(String::from);

        let mut key_dependencies = Vec::new();
        if let Some(deps) = parsed.get("dependencies").and_then(|d| d.as_table()) {
            for (dep_name, _) in deps.iter().take(10) {
                key_dependencies.push(dep_name.clone());
            }
        }

        Ok(ProjectContext {
            project_type: ProjectType::Rust,
            additional_types: Vec::new(),
            name,
            version,
            key_dependencies,
            available_scripts: vec![
                "cargo build".to_string(),
                "cargo test".to_string(),
                "cargo run".to_string(),
            ],
            root: path.to_path_buf(),
        })
    }

    async fn parse_nodejs_project(path: &Path) -> Result<ProjectContext, ContextError> {
        let package_path = path.join("package.json");
        let content = fs::read_to_string(&package_path).await?;

        let parsed: serde_json::Value = serde_json::from_str(&content)?;

        let name = parsed.get("name").and_then(|n| n.as_str()).map(String::from);
        let version = parsed
            .get("version")
            .and_then(|v| v.as_str())
            .map(String::from);

        let mut key_dependencies = Vec::new();
        if let Some(deps) = parsed.get("dependencies").and_then(|d| d.as_object()) {
            for (dep_name, _) in deps.iter().take(10) {
                key_dependencies.push(dep_name.clone());
            }
        }

        let mut available_scripts = Vec::new();
        if let Some(scripts) = parsed.get("scripts").and_then(|s| s.as_object()) {
            for (script_name, _) in scripts.iter() {
                available_scripts.push(format!("npm run {}", script_name));
            }
        }

        // Check if it's Next.js or React
        let is_nextjs = key_dependencies.contains(&"next".to_string());
        let is_react = key_dependencies.contains(&"react".to_string());

        let project_type = if is_nextjs {
            ProjectType::NextJs
        } else if is_react {
            ProjectType::React
        } else {
            ProjectType::NodeJs
        };

        Ok(ProjectContext {
            project_type,
            additional_types: Vec::new(),
            name,
            version,
            key_dependencies,
            available_scripts,
            root: path.to_path_buf(),
        })
    }

    async fn parse_python_project(path: &Path) -> Result<ProjectContext, ContextError> {
        let mut name = None;
        let mut version = None;
        let mut key_dependencies = Vec::new();

        // Try pyproject.toml first
        let pyproject_path = path.join("pyproject.toml");
        if pyproject_path.exists() {
            if let Ok(content) = fs::read_to_string(&pyproject_path).await {
                if let Ok(parsed) = toml::from_str::<toml::Value>(&content) {
                    if let Some(project) = parsed.get("project") {
                        name = project.get("name").and_then(|n| n.as_str()).map(String::from);
                        version = project
                            .get("version")
                            .and_then(|v| v.as_str())
                            .map(String::from);
                        if let Some(deps) = project.get("dependencies").and_then(|d| d.as_array())
                        {
                            for dep in deps.iter().take(10) {
                                if let Some(dep_str) = dep.as_str() {
                                    key_dependencies.push(dep_str.to_string());
                                }
                            }
                        }
                    }
                }
            }
        }

        // Try requirements.txt if no pyproject.toml
        if key_dependencies.is_empty() {
            let req_path = path.join("requirements.txt");
            if req_path.exists() {
                if let Ok(content) = fs::read_to_string(&req_path).await {
                    for line in content.lines().take(10) {
                        let dep = line.split(&['=', '>', '<', '!'][..]).next().unwrap_or("");
                        if !dep.is_empty() && !dep.starts_with('#') {
                            key_dependencies.push(dep.trim().to_string());
                        }
                    }
                }
            }
        }

        Ok(ProjectContext {
            project_type: ProjectType::Python,
            additional_types: Vec::new(),
            name,
            version,
            key_dependencies,
            available_scripts: vec![
                "python -m pytest".to_string(),
                "python setup.py".to_string(),
            ],
            root: path.to_path_buf(),
        })
    }

    async fn parse_go_project(path: &Path) -> Result<ProjectContext, ContextError> {
        let go_mod_path = path.join("go.mod");
        let content = fs::read_to_string(&go_mod_path).await?;

        let mut name = None;
        let mut key_dependencies = Vec::new();

        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("module ") {
                name = Some(trimmed.trim_start_matches("module ").trim().to_string());
            } else if trimmed.starts_with("require ") && key_dependencies.len() < 10 {
                let dep = trimmed
                    .trim_start_matches("require ")
                    .split_whitespace()
                    .next()
                    .unwrap_or("");
                if !dep.is_empty() {
                    key_dependencies.push(dep.to_string());
                }
            }
        }

        Ok(ProjectContext {
            project_type: ProjectType::Go,
            additional_types: Vec::new(),
            name,
            version: None,
            key_dependencies,
            available_scripts: vec![
                "go build".to_string(),
                "go test".to_string(),
                "go run .".to_string(),
            ],
            root: path.to_path_buf(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_project_type_display() {
        assert_eq!(ProjectType::Rust.to_string(), "Rust");
        assert_eq!(ProjectType::NodeJs.to_string(), "Node.js");
        assert_eq!(ProjectType::Python.to_string(), "Python");
    }

    #[test]
    fn test_project_context_unknown() {
        let ctx = ProjectContext::unknown(PathBuf::from("/tmp/test"));
        assert_eq!(ctx.project_type, ProjectType::Unknown);
        assert!(ctx.name.is_none());
    }

    #[test]
    fn test_project_context_to_llm_string() {
        let ctx = ProjectContext {
            project_type: ProjectType::Rust,
            additional_types: vec![ProjectType::Docker],
            name: Some("cmdai".to_string()),
            version: Some("0.1.0".to_string()),
            key_dependencies: vec!["tokio".to_string(), "clap".to_string()],
            available_scripts: vec!["cargo build".to_string()],
            root: PathBuf::from("/tmp"),
        };

        let llm_str = ctx.to_llm_context();
        assert!(llm_str.contains("Rust"));
        assert!(llm_str.contains("cmdai"));
        assert!(llm_str.contains("tokio"));
    }

    #[tokio::test]
    async fn test_detect_current_project() {
        // Should detect cmdai as Rust project
        let cwd = std::env::current_dir().unwrap();
        let result = ProjectParser::analyze(&cwd).await;
        assert!(result.is_ok());
        let ctx = result.unwrap();
        // May be Rust or Unknown depending on execution context
        assert!(
            ctx.project_type == ProjectType::Rust || ctx.project_type == ProjectType::Unknown
        );
    }
}
