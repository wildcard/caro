//! Infrastructure tool detection
//!
//! Detects available infrastructure tools (Docker, Kubernetes, Terraform, cloud CLIs, etc.)

use super::ContextError;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::process::Stdio;
use tokio::process::Command;

/// Detected infrastructure tool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tool {
    /// Tool name
    pub name: String,
    /// Tool version (if available)
    pub version: Option<String>,
    /// Whether tool is available in PATH
    pub available: bool,
    /// Tool category (docker, kubernetes, cloud, etc.)
    pub category: ToolCategory,
}

/// Tool category for organization
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ToolCategory {
    Container,
    Orchestration,
    Cloud,
    Infrastructure,
    Database,
    Build,
    Other,
}

impl std::fmt::Display for ToolCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Container => write!(f, "container"),
            Self::Orchestration => write!(f, "orchestration"),
            Self::Cloud => write!(f, "cloud"),
            Self::Infrastructure => write!(f, "infrastructure"),
            Self::Database => write!(f, "database"),
            Self::Build => write!(f, "build"),
            Self::Other => write!(f, "other"),
        }
    }
}

/// Infrastructure context containing detected tools
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InfrastructureContext {
    /// All detected tools
    pub tools: Vec<Tool>,
    /// Whether Docker is available
    pub has_docker: bool,
    /// Whether Kubernetes tools are available
    pub has_kubernetes: bool,
    /// Whether Terraform is available
    pub has_terraform: bool,
    /// Cloud providers detected
    pub cloud_providers: Vec<String>,
}

impl InfrastructureContext {
    /// Create empty infrastructure context
    pub fn empty() -> Self {
        Self {
            tools: Vec::new(),
            has_docker: false,
            has_kubernetes: false,
            has_terraform: false,
            cloud_providers: Vec::new(),
        }
    }

    /// Convert to LLM-friendly context string
    pub fn to_llm_context(&self) -> String {
        if self.tools.is_empty() {
            return "Infrastructure: No tools detected".to_string();
        }

        let mut lines = Vec::new();

        // Group tools by category
        let mut by_category: std::collections::HashMap<String, Vec<&Tool>> =
            std::collections::HashMap::new();

        for tool in &self.tools {
            by_category
                .entry(tool.category.to_string())
                .or_default()
                .push(tool);
        }

        for (category, tools) in by_category {
            let tool_names: Vec<String> = tools
                .iter()
                .map(|t| {
                    if let Some(ver) = &t.version {
                        format!("{} ({})", t.name, ver)
                    } else {
                        t.name.clone()
                    }
                })
                .collect();
            lines.push(format!("{}: {}", category, tool_names.join(", ")));
        }

        if !self.cloud_providers.is_empty() {
            lines.push(format!(
                "Cloud Providers: {}",
                self.cloud_providers.join(", ")
            ));
        }

        format!("Infrastructure Tools:\n{}", lines.join("\n"))
    }
}

/// Tool detector
pub struct ToolDetector;

impl ToolDetector {
    /// Detect all available infrastructure tools
    pub async fn analyze(_path: &Path) -> Result<InfrastructureContext, ContextError> {
        let mut tools = Vec::new();
        let mut cloud_providers = Vec::new();

        // Container tools
        if let Some(docker) = Self::detect_docker().await {
            tools.push(docker);
        }
        if let Some(podman) = Self::detect_podman().await {
            tools.push(podman);
        }
        if let Some(docker_compose) = Self::detect_docker_compose().await {
            tools.push(docker_compose);
        }

        // Kubernetes tools
        if let Some(kubectl) = Self::detect_kubectl().await {
            tools.push(kubectl);
        }
        if let Some(helm) = Self::detect_helm().await {
            tools.push(helm);
        }
        if let Some(minikube) = Self::detect_minikube().await {
            tools.push(minikube);
        }

        // Infrastructure as Code
        if let Some(terraform) = Self::detect_terraform().await {
            tools.push(terraform);
        }
        if let Some(pulumi) = Self::detect_pulumi().await {
            tools.push(pulumi);
        }

        // Cloud CLIs
        if let Some(aws) = Self::detect_aws_cli().await {
            tools.push(aws);
            cloud_providers.push("AWS".to_string());
        }
        if let Some(gcloud) = Self::detect_gcloud().await {
            tools.push(gcloud);
            cloud_providers.push("GCP".to_string());
        }
        if let Some(az) = Self::detect_azure_cli().await {
            tools.push(az);
            cloud_providers.push("Azure".to_string());
        }
        if let Some(railway) = Self::detect_railway().await {
            tools.push(railway);
            cloud_providers.push("Railway".to_string());
        }

        // Database tools
        if let Some(psql) = Self::detect_psql().await {
            tools.push(psql);
        }
        if let Some(mysql) = Self::detect_mysql().await {
            tools.push(mysql);
        }
        if let Some(redis) = Self::detect_redis_cli().await {
            tools.push(redis);
        }

        // Build tools
        if let Some(make) = Self::detect_make().await {
            tools.push(make);
        }

        // Compute derived flags
        let has_docker = tools.iter().any(|t| t.name == "docker");
        let has_kubernetes = tools
            .iter()
            .any(|t| t.name == "kubectl" || t.name == "helm");
        let has_terraform = tools.iter().any(|t| t.name == "terraform");

        Ok(InfrastructureContext {
            tools,
            has_docker,
            has_kubernetes,
            has_terraform,
            cloud_providers,
        })
    }

    // Tool detection helpers

    async fn detect_tool(
        name: &str,
        version_arg: &str,
        category: ToolCategory,
    ) -> Option<Tool> {
        let output = Command::new(name)
            .arg(version_arg)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await;

        match output {
            Ok(output) if output.status.success() => {
                let version_output = String::from_utf8_lossy(&output.stdout);
                let version = Self::extract_version(&version_output);

                Some(Tool {
                    name: name.to_string(),
                    version,
                    available: true,
                    category,
                })
            }
            _ => None,
        }
    }

    fn extract_version(output: &str) -> Option<String> {
        // Extract version from common formats
        let lines = output.lines().collect::<Vec<_>>();
        if lines.is_empty() {
            return None;
        }

        let first_line = lines[0];

        // Try to find version pattern (e.g., "1.2.3", "v1.2.3")
        if let Some(version) = first_line
            .split_whitespace()
            .find(|s| s.contains('.') && s.chars().any(|c| c.is_ascii_digit()))
        {
            return Some(version.trim_start_matches('v').to_string());
        }

        None
    }

    async fn detect_docker() -> Option<Tool> {
        Self::detect_tool("docker", "--version", ToolCategory::Container).await
    }

    async fn detect_podman() -> Option<Tool> {
        Self::detect_tool("podman", "--version", ToolCategory::Container).await
    }

    async fn detect_docker_compose() -> Option<Tool> {
        Self::detect_tool("docker-compose", "--version", ToolCategory::Container).await
    }

    async fn detect_kubectl() -> Option<Tool> {
        Self::detect_tool("kubectl", "version", ToolCategory::Orchestration).await
    }

    async fn detect_helm() -> Option<Tool> {
        Self::detect_tool("helm", "version", ToolCategory::Orchestration).await
    }

    async fn detect_minikube() -> Option<Tool> {
        Self::detect_tool("minikube", "version", ToolCategory::Orchestration).await
    }

    async fn detect_terraform() -> Option<Tool> {
        Self::detect_tool("terraform", "--version", ToolCategory::Infrastructure).await
    }

    async fn detect_pulumi() -> Option<Tool> {
        Self::detect_tool("pulumi", "version", ToolCategory::Infrastructure).await
    }

    async fn detect_aws_cli() -> Option<Tool> {
        Self::detect_tool("aws", "--version", ToolCategory::Cloud).await
    }

    async fn detect_gcloud() -> Option<Tool> {
        Self::detect_tool("gcloud", "--version", ToolCategory::Cloud).await
    }

    async fn detect_azure_cli() -> Option<Tool> {
        Self::detect_tool("az", "--version", ToolCategory::Cloud).await
    }

    async fn detect_railway() -> Option<Tool> {
        Self::detect_tool("railway", "--version", ToolCategory::Cloud).await
    }

    async fn detect_psql() -> Option<Tool> {
        Self::detect_tool("psql", "--version", ToolCategory::Database).await
    }

    async fn detect_mysql() -> Option<Tool> {
        Self::detect_tool("mysql", "--version", ToolCategory::Database).await
    }

    async fn detect_redis_cli() -> Option<Tool> {
        Self::detect_tool("redis-cli", "--version", ToolCategory::Database).await
    }

    async fn detect_make() -> Option<Tool> {
        Self::detect_tool("make", "--version", ToolCategory::Build).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_category_display() {
        assert_eq!(ToolCategory::Container.to_string(), "container");
        assert_eq!(ToolCategory::Cloud.to_string(), "cloud");
    }

    #[test]
    fn test_infrastructure_context_empty() {
        let ctx = InfrastructureContext::empty();
        assert!(ctx.tools.is_empty());
        assert!(!ctx.has_docker);
    }

    #[test]
    fn test_extract_version() {
        assert_eq!(
            ToolDetector::extract_version("Docker version 24.0.5, build ced0996"),
            Some("24.0.5".to_string())
        );
        assert_eq!(
            ToolDetector::extract_version("kubectl v1.28.0"),
            Some("1.28.0".to_string())
        );
    }

    #[tokio::test]
    async fn test_detect_tools() {
        let cwd = std::env::current_dir().unwrap();
        let result = ToolDetector::analyze(&cwd).await;
        assert!(result.is_ok());
        // Don't assert specific tools since it depends on environment
    }

    #[tokio::test]
    async fn test_detect_docker() {
        // May or may not be available
        let _result = ToolDetector::detect_docker().await;
        // Just ensure it doesn't panic
    }
}
