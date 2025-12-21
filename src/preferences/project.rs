//! Project context detection module
//!
//! This module detects project-specific information from the current directory,
//! including package managers, build tools, programming languages, and infrastructure
//! tools. Detection is based on common lock files, configuration files, and project
//! structure patterns - similar to how Starship detects project context.
//!
//! # Supported Ecosystems
//!
//! - **Web/App**: Node.js, Bun, Deno, Python, Ruby, Go, Rust, Java, Scala, Kotlin
//! - **DevOps/SRE**: Kubernetes, Helm, Terraform, Ansible, Docker
//! - **Cloud**: AWS, GCP, Azure detection
//! - **CI/CD**: GitHub Actions, GitLab CI, Jenkins, CircleCI

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tracing::{debug, trace};

/// Project context detected from current directory
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProjectContext {
    /// Detected package manager
    pub package_manager: Option<PackageManager>,

    /// Detected build tool
    pub build_tool: Option<BuildTool>,

    /// Detected programming languages
    pub languages: Vec<Language>,

    /// Detected infrastructure/DevOps tools
    pub infra_tools: Vec<InfraTool>,

    /// Detected cloud provider context
    pub cloud_context: Option<CloudContext>,

    /// Project root path
    pub root_path: PathBuf,

    /// Lock files and config files found
    pub detected_files: Vec<String>,

    /// Raw signals for model fallback (files that might be relevant but aren't parsed)
    pub raw_signals: Vec<String>,
}

impl ProjectContext {
    /// Detect project context from the given directory
    ///
    /// Scans the directory for recognizable files like lock files,
    /// configuration files, and project markers.
    ///
    /// # Arguments
    ///
    /// * `cwd` - Directory to scan
    ///
    /// # Returns
    ///
    /// A ProjectContext with detected information
    pub fn detect(cwd: &Path) -> Result<Self, std::io::Error> {
        debug!("Scanning directory for project context: {:?}", cwd);

        let mut context = Self {
            root_path: cwd.to_path_buf(),
            ..Default::default()
        };

        // Scan for all detectable files
        let (files, raw_signals) = Self::scan_project_files(cwd)?;
        context.detected_files = files.clone();
        context.raw_signals = raw_signals;

        // Detect package manager from lock files (priority order)
        context.package_manager = Self::detect_package_manager(&files);

        // Detect build tool
        context.build_tool = Self::detect_build_tool(&files);

        // Detect languages
        context.languages = Self::detect_languages(&files);

        // Detect infrastructure tools (DevOps/SRE)
        context.infra_tools = Self::detect_infra_tools(&files);

        // Detect cloud context
        context.cloud_context = Self::detect_cloud_context();

        debug!(
            "Detected: pm={:?}, build={:?}, langs={:?}, infra={:?}, cloud={:?}",
            context.package_manager, context.build_tool, context.languages,
            context.infra_tools, context.cloud_context
        );

        Ok(context)
    }

    /// Scan directory for project-related files
    /// Returns (known_files, raw_signals) where raw_signals are files we don't have rules for
    fn scan_project_files(cwd: &Path) -> Result<(Vec<String>, Vec<String>), std::io::Error> {
        let mut files = Vec::new();
        let mut raw_signals = Vec::new();

        // Files we have deterministic rules for (order matters for priority)
        let known_files = [
            // Node.js Package managers
            "yarn.lock",
            "package-lock.json",
            "pnpm-lock.yaml",
            "bun.lockb",
            "package.json",
            "deno.json",
            "deno.jsonc",
            // Rust
            "Cargo.toml",
            "Cargo.lock",
            // Go
            "go.mod",
            "go.sum",
            // Python
            "Pipfile.lock",
            "Pipfile",
            "poetry.lock",
            "pyproject.toml",
            "requirements.txt",
            "setup.py",
            "setup.cfg",
            "tox.ini",
            // Ruby
            "Gemfile.lock",
            "Gemfile",
            ".ruby-version",
            // PHP
            "composer.json",
            "composer.lock",
            // Java/JVM
            "pom.xml",
            "build.gradle",
            "build.gradle.kts",
            "settings.gradle",
            "settings.gradle.kts",
            "build.sbt",
            ".metals",
            // .NET
            "*.csproj",
            "*.fsproj",
            "*.sln",
            "nuget.config",
            // Elixir/Erlang
            "mix.exs",
            "rebar.config",
            // Build tools
            "Makefile",
            "CMakeLists.txt",
            "BUILD.bazel",
            "BUILD",
            "WORKSPACE",
            "meson.build",
            "ninja.build",
            // Docker
            "Dockerfile",
            "docker-compose.yml",
            "docker-compose.yaml",
            "compose.yml",
            "compose.yaml",
            ".dockerignore",
            // Kubernetes
            "kustomization.yaml",
            "kustomization.yml",
            "skaffold.yaml",
            "tilt.json",
            "Tiltfile",
            // Helm
            "Chart.yaml",
            "values.yaml",
            "helmfile.yaml",
            // Terraform/OpenTofu/Pulumi
            "main.tf",
            "terraform.tfvars",
            "terragrunt.hcl",
            "Pulumi.yaml",
            "pulumi.yaml",
            // Ansible
            "ansible.cfg",
            "playbook.yml",
            "playbook.yaml",
            "inventory.yml",
            "inventory.ini",
            // CI/CD
            ".github/workflows",
            ".gitlab-ci.yml",
            "Jenkinsfile",
            ".circleci/config.yml",
            ".travis.yml",
            "azure-pipelines.yml",
            "bitbucket-pipelines.yml",
            ".drone.yml",
            // TypeScript/JavaScript
            "tsconfig.json",
            "jsconfig.json",
            ".eslintrc.json",
            ".eslintrc.js",
            ".prettierrc",
            "vite.config.ts",
            "vite.config.js",
            "next.config.js",
            "next.config.mjs",
            "nuxt.config.ts",
            "svelte.config.js",
            "astro.config.mjs",
            // Testing
            "jest.config.js",
            "vitest.config.ts",
            "playwright.config.ts",
            "cypress.config.ts",
            ".rspec",
            "pytest.ini",
            "conftest.py",
            // Misc config
            ".editorconfig",
            ".gitignore",
            ".nvmrc",
            ".node-version",
            ".python-version",
            ".tool-versions",
        ];

        // Additional files to scan as raw signals (for model fallback)
        let raw_signal_patterns = [
            // Config files that might indicate preferences
            ".env",
            ".env.local",
            ".env.example",
            // AWS
            "samconfig.toml",
            "serverless.yml",
            "serverless.yaml",
            "cdk.json",
            "template.yaml",
            // Kubernetes manifests (any .yaml could be k8s)
            "deployment.yaml",
            "service.yaml",
            "ingress.yaml",
            "configmap.yaml",
            "secret.yaml",
            // Database
            "prisma/schema.prisma",
            "drizzle.config.ts",
            "knexfile.js",
            // Documentation
            "README.md",
            "CONTRIBUTING.md",
            // Monorepo
            "lerna.json",
            "nx.json",
            "turbo.json",
            "pnpm-workspace.yaml",
            "rush.json",
        ];

        for filename in known_files {
            // Handle glob patterns
            if filename.contains('*') {
                if let Ok(entries) = std::fs::read_dir(cwd) {
                    for entry in entries.flatten() {
                        let name = entry.file_name().to_string_lossy().to_string();
                        let pattern = filename.replace('*', "");
                        if name.ends_with(&pattern) {
                            trace!("Found (glob): {}", name);
                            files.push(name);
                        }
                    }
                }
            } else {
                let path = cwd.join(filename);
                if path.exists() {
                    trace!("Found: {}", filename);
                    files.push(filename.to_string());
                }
            }
        }

        // Collect raw signals
        for filename in raw_signal_patterns {
            let path = cwd.join(filename);
            if path.exists() {
                trace!("Raw signal: {}", filename);
                raw_signals.push(filename.to_string());
            }
        }

        Ok((files, raw_signals))
    }

    /// Detect package manager from detected files
    fn detect_package_manager(files: &[String]) -> Option<PackageManager> {
        // Priority order: More specific lock files first
        if files.contains(&"yarn.lock".to_string()) {
            return Some(PackageManager::Yarn);
        }
        if files.contains(&"pnpm-lock.yaml".to_string()) {
            return Some(PackageManager::Pnpm);
        }
        if files.contains(&"bun.lockb".to_string()) {
            return Some(PackageManager::Bun);
        }
        if files.contains(&"package-lock.json".to_string()) {
            return Some(PackageManager::Npm);
        }
        if files.contains(&"Cargo.toml".to_string()) {
            return Some(PackageManager::Cargo);
        }
        if files.contains(&"go.mod".to_string()) {
            return Some(PackageManager::Go);
        }
        if files.contains(&"poetry.lock".to_string()) {
            return Some(PackageManager::Poetry);
        }
        if files.contains(&"Pipfile.lock".to_string()) {
            return Some(PackageManager::Pipenv);
        }
        if files.contains(&"Gemfile.lock".to_string()) {
            return Some(PackageManager::Bundler);
        }
        // package.json without lock file - default to npm
        if files.contains(&"package.json".to_string()) {
            return Some(PackageManager::Npm);
        }
        if files.contains(&"requirements.txt".to_string()) {
            return Some(PackageManager::Pip);
        }

        None
    }

    /// Detect build tool from detected files
    fn detect_build_tool(files: &[String]) -> Option<BuildTool> {
        if files.contains(&"Makefile".to_string()) {
            return Some(BuildTool::Make);
        }
        if files.contains(&"CMakeLists.txt".to_string()) {
            return Some(BuildTool::Cmake);
        }
        if files.contains(&"build.gradle".to_string())
            || files.contains(&"build.gradle.kts".to_string())
        {
            return Some(BuildTool::Gradle);
        }
        if files.contains(&"pom.xml".to_string()) {
            return Some(BuildTool::Maven);
        }
        if files.contains(&"BUILD.bazel".to_string()) || files.contains(&"BUILD".to_string()) {
            return Some(BuildTool::Bazel);
        }

        None
    }

    /// Detect programming languages from detected files
    fn detect_languages(files: &[String]) -> Vec<Language> {
        let mut languages = Vec::new();

        if files.contains(&"Cargo.toml".to_string()) {
            languages.push(Language::Rust);
        }
        if files.contains(&"tsconfig.json".to_string()) {
            languages.push(Language::TypeScript);
        } else if files.contains(&"package.json".to_string())
            || files.contains(&"jsconfig.json".to_string())
            || files.contains(&"deno.json".to_string())
        {
            languages.push(Language::JavaScript);
        }
        if files.contains(&"go.mod".to_string()) {
            languages.push(Language::Go);
        }
        if files.contains(&"pyproject.toml".to_string())
            || files.contains(&"setup.py".to_string())
            || files.contains(&"requirements.txt".to_string())
        {
            languages.push(Language::Python);
        }
        if files.contains(&"Gemfile".to_string()) {
            languages.push(Language::Ruby);
        }
        if files.contains(&"pom.xml".to_string()) || files.contains(&"build.gradle".to_string()) {
            languages.push(Language::Java);
        }
        if files.contains(&"build.sbt".to_string()) || files.contains(&".metals".to_string()) {
            languages.push(Language::Scala);
        }
        if files.contains(&"build.gradle.kts".to_string()) {
            languages.push(Language::Kotlin);
        }
        if files.contains(&"composer.json".to_string()) {
            languages.push(Language::Php);
        }
        if files.iter().any(|f| f.ends_with(".csproj") || f.ends_with(".sln")) {
            languages.push(Language::CSharp);
        }
        if files.iter().any(|f| f.ends_with(".fsproj")) {
            languages.push(Language::FSharp);
        }
        if files.contains(&"mix.exs".to_string()) {
            languages.push(Language::Elixir);
        }
        if files.contains(&"rebar.config".to_string()) {
            languages.push(Language::Erlang);
        }

        languages
    }

    /// Detect infrastructure/DevOps tools from detected files
    fn detect_infra_tools(files: &[String]) -> Vec<InfraTool> {
        let mut tools = Vec::new();

        // Docker
        if files.contains(&"Dockerfile".to_string())
            || files.contains(&"docker-compose.yml".to_string())
            || files.contains(&"docker-compose.yaml".to_string())
            || files.contains(&"compose.yml".to_string())
        {
            tools.push(InfraTool::Docker);
        }

        // Kubernetes
        if files.contains(&"kustomization.yaml".to_string())
            || files.contains(&"kustomization.yml".to_string())
            || files.contains(&"skaffold.yaml".to_string())
        {
            tools.push(InfraTool::Kubernetes);
        }

        // Helm
        if files.contains(&"Chart.yaml".to_string())
            || files.contains(&"helmfile.yaml".to_string())
        {
            tools.push(InfraTool::Helm);
        }

        // Terraform
        if files.contains(&"main.tf".to_string())
            || files.contains(&"terraform.tfvars".to_string())
            || files.contains(&"terragrunt.hcl".to_string())
        {
            tools.push(InfraTool::Terraform);
        }

        // Pulumi
        if files.contains(&"Pulumi.yaml".to_string())
            || files.contains(&"pulumi.yaml".to_string())
        {
            tools.push(InfraTool::Pulumi);
        }

        // Ansible
        if files.contains(&"ansible.cfg".to_string())
            || files.contains(&"playbook.yml".to_string())
            || files.contains(&"playbook.yaml".to_string())
        {
            tools.push(InfraTool::Ansible);
        }

        tools
    }

    /// Detect cloud provider context from environment and config files
    fn detect_cloud_context() -> Option<CloudContext> {
        let mut context = CloudContext::default();
        let home = dirs::home_dir()?;

        // AWS detection
        let aws_dir = home.join(".aws");
        if aws_dir.exists() {
            context.aws_configured = true;
            // Try to get current profile
            context.aws_profile = std::env::var("AWS_PROFILE").ok();
            if context.aws_profile.is_none() {
                context.aws_profile = std::env::var("AWS_DEFAULT_PROFILE").ok();
            }
        }

        // GCP detection
        let gcloud_dir = home.join(".config/gcloud");
        if gcloud_dir.exists() {
            context.gcp_configured = true;
            // Try to get current project
            context.gcp_project = std::env::var("GCLOUD_PROJECT").ok()
                .or_else(|| std::env::var("GOOGLE_CLOUD_PROJECT").ok());
        }

        // Azure detection
        let azure_dir = home.join(".azure");
        if azure_dir.exists() {
            context.azure_configured = true;
            context.azure_subscription = std::env::var("AZURE_SUBSCRIPTION_ID").ok();
        }

        // kubectl context detection
        if let Ok(output) = std::process::Command::new("kubectl")
            .args(["config", "current-context"])
            .output()
        {
            if output.status.success() {
                context.kubectl_context = String::from_utf8(output.stdout)
                    .ok()
                    .map(|s| s.trim().to_string());
            }
        }

        // Only return if at least one cloud is configured
        if context.aws_configured || context.gcp_configured || context.azure_configured {
            Some(context)
        } else {
            None
        }
    }

    /// Get the package manager command, if detected
    pub fn package_command(&self) -> Option<&str> {
        self.package_manager.as_ref().map(|pm| pm.command())
    }

    /// Check if a command uses a different package manager than detected
    pub fn is_wrong_package_manager(&self, command: &str) -> Option<PackageManager> {
        let expected = self.package_manager.as_ref()?;
        let alternatives = expected.alternatives();

        for alt in alternatives {
            if command.starts_with(&format!("{} ", alt))
                || command.contains(&format!(" {} ", alt))
            {
                return PackageManager::from_command(alt);
            }
        }

        None
    }
}

/// Supported package managers
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PackageManager {
    Yarn,
    Npm,
    Pnpm,
    Bun,
    Cargo,
    Go,
    Pip,
    Pipenv,
    Poetry,
    Bundler,
}

impl PackageManager {
    /// Get the command name for this package manager
    pub fn command(&self) -> &'static str {
        match self {
            Self::Yarn => "yarn",
            Self::Npm => "npm",
            Self::Pnpm => "pnpm",
            Self::Bun => "bun",
            Self::Cargo => "cargo",
            Self::Go => "go",
            Self::Pip => "pip",
            Self::Pipenv => "pipenv",
            Self::Poetry => "poetry",
            Self::Bundler => "bundle",
        }
    }

    /// Get the display name
    pub fn name(&self) -> &'static str {
        match self {
            Self::Yarn => "Yarn",
            Self::Npm => "npm",
            Self::Pnpm => "pnpm",
            Self::Bun => "Bun",
            Self::Cargo => "Cargo",
            Self::Go => "Go Modules",
            Self::Pip => "pip",
            Self::Pipenv => "Pipenv",
            Self::Poetry => "Poetry",
            Self::Bundler => "Bundler",
        }
    }

    /// Get alternative package managers in the same ecosystem
    pub fn alternatives(&self) -> Vec<&'static str> {
        match self {
            Self::Yarn => vec!["npm", "pnpm", "bun", "npx"],
            Self::Npm => vec!["yarn", "pnpm", "bun"],
            Self::Pnpm => vec!["npm", "yarn", "bun", "npx"],
            Self::Bun => vec!["npm", "yarn", "pnpm", "npx"],
            Self::Pip => vec!["pipenv", "poetry"],
            Self::Pipenv => vec!["pip", "poetry"],
            Self::Poetry => vec!["pip", "pipenv"],
            // No alternatives for language-specific ones
            Self::Cargo | Self::Go | Self::Bundler => vec![],
        }
    }

    /// Create from command name
    pub fn from_command(cmd: &str) -> Option<Self> {
        match cmd {
            "yarn" => Some(Self::Yarn),
            "npm" | "npx" => Some(Self::Npm),
            "pnpm" => Some(Self::Pnpm),
            "bun" | "bunx" => Some(Self::Bun),
            "cargo" => Some(Self::Cargo),
            "go" => Some(Self::Go),
            "pip" | "pip3" => Some(Self::Pip),
            "pipenv" => Some(Self::Pipenv),
            "poetry" => Some(Self::Poetry),
            "bundle" | "bundler" => Some(Self::Bundler),
            _ => None,
        }
    }

    /// Get equivalent command for translation
    pub fn translate_from(&self, other: &PackageManager, args: &str) -> Option<String> {
        // Handle npm <-> yarn <-> pnpm translations
        match (other, self) {
            // npm -> yarn
            (PackageManager::Npm, PackageManager::Yarn) => {
                let args = args.replace("install", "").trim().to_string();
                if args.is_empty() || args == "--save" || args == "-S" {
                    Some("yarn".to_string())
                } else if args.starts_with("--save-dev") || args.starts_with("-D") {
                    Some(format!("yarn add --dev {}", args.trim_start_matches("--save-dev").trim_start_matches("-D").trim()))
                } else {
                    Some(format!("yarn add {}", args))
                }
            }
            // yarn -> npm
            (PackageManager::Yarn, PackageManager::Npm) => {
                if args.is_empty() {
                    Some("npm install".to_string())
                } else if args.starts_with("add ") {
                    let pkg = args.strip_prefix("add ").unwrap();
                    Some(format!("npm install {}", pkg))
                } else {
                    Some(format!("npm {}", args))
                }
            }
            // Same ecosystem but different tool
            (from, to) if from.alternatives().contains(&to.command()) => {
                // Generic translation - just swap the command
                Some(format!("{} {}", to.command(), args))
            }
            _ => None,
        }
    }
}

/// Supported build tools
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BuildTool {
    Make,
    Cmake,
    Gradle,
    Maven,
    Bazel,
}

impl BuildTool {
    /// Get the command name
    pub fn command(&self) -> &'static str {
        match self {
            Self::Make => "make",
            Self::Cmake => "cmake",
            Self::Gradle => "./gradlew", // Prefer wrapper
            Self::Maven => "mvn",
            Self::Bazel => "bazel",
        }
    }

    /// Get the display name
    pub fn name(&self) -> &'static str {
        match self {
            Self::Make => "Make",
            Self::Cmake => "CMake",
            Self::Gradle => "Gradle",
            Self::Maven => "Maven",
            Self::Bazel => "Bazel",
        }
    }
}

/// Detected programming languages
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Language {
    Rust,
    JavaScript,
    TypeScript,
    Python,
    Go,
    Ruby,
    Java,
    Scala,
    Kotlin,
    Php,
    CSharp,
    FSharp,
    Elixir,
    Erlang,
}

impl Language {
    /// Get the display name
    pub fn name(&self) -> &'static str {
        match self {
            Self::Rust => "Rust",
            Self::JavaScript => "JavaScript",
            Self::TypeScript => "TypeScript",
            Self::Python => "Python",
            Self::Go => "Go",
            Self::Ruby => "Ruby",
            Self::Java => "Java",
            Self::Scala => "Scala",
            Self::Kotlin => "Kotlin",
            Self::Php => "PHP",
            Self::CSharp => "C#",
            Self::FSharp => "F#",
            Self::Elixir => "Elixir",
            Self::Erlang => "Erlang",
        }
    }
}

/// Infrastructure/DevOps tools detected in the project
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum InfraTool {
    Docker,
    Kubernetes,
    Helm,
    Terraform,
    Pulumi,
    Ansible,
}

impl InfraTool {
    /// Get the primary command for this tool
    pub fn command(&self) -> &'static str {
        match self {
            Self::Docker => "docker",
            Self::Kubernetes => "kubectl",
            Self::Helm => "helm",
            Self::Terraform => "terraform",
            Self::Pulumi => "pulumi",
            Self::Ansible => "ansible",
        }
    }

    /// Get the display name
    pub fn name(&self) -> &'static str {
        match self {
            Self::Docker => "Docker",
            Self::Kubernetes => "Kubernetes",
            Self::Helm => "Helm",
            Self::Terraform => "Terraform",
            Self::Pulumi => "Pulumi",
            Self::Ansible => "Ansible",
        }
    }
}

/// Cloud provider context detected from user environment
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CloudContext {
    /// AWS CLI is configured
    pub aws_configured: bool,
    /// Current AWS profile
    pub aws_profile: Option<String>,

    /// GCP CLI is configured
    pub gcp_configured: bool,
    /// Current GCP project
    pub gcp_project: Option<String>,

    /// Azure CLI is configured
    pub azure_configured: bool,
    /// Current Azure subscription
    pub azure_subscription: Option<String>,

    /// Current kubectl context
    pub kubectl_context: Option<String>,
}

impl CloudContext {
    /// Get the primary cloud provider (if only one is configured)
    pub fn primary_cloud(&self) -> Option<&'static str> {
        let configured = [
            (self.aws_configured, "AWS"),
            (self.gcp_configured, "GCP"),
            (self.azure_configured, "Azure"),
        ];

        let active: Vec<_> = configured.iter().filter(|(c, _)| *c).collect();
        if active.len() == 1 {
            Some(active[0].1)
        } else {
            None
        }
    }

    /// Generate a prompt context string for the model
    pub fn to_prompt_context(&self) -> String {
        let mut parts = Vec::new();

        if self.aws_configured {
            if let Some(ref profile) = self.aws_profile {
                parts.push(format!("AWS (profile: {})", profile));
            } else {
                parts.push("AWS configured".to_string());
            }
        }

        if self.gcp_configured {
            if let Some(ref project) = self.gcp_project {
                parts.push(format!("GCP (project: {})", project));
            } else {
                parts.push("GCP configured".to_string());
            }
        }

        if self.azure_configured {
            if let Some(ref sub) = self.azure_subscription {
                parts.push(format!("Azure (subscription: {})", sub));
            } else {
                parts.push("Azure configured".to_string());
            }
        }

        if let Some(ref ctx) = self.kubectl_context {
            parts.push(format!("kubectl context: {}", ctx));
        }

        if parts.is_empty() {
            String::new()
        } else {
            format!("CLOUD: {}", parts.join(", "))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use tempfile::TempDir;

    #[test]
    fn test_detect_yarn_project() {
        let temp = TempDir::new().unwrap();
        File::create(temp.path().join("yarn.lock")).unwrap();
        File::create(temp.path().join("package.json")).unwrap();

        let context = ProjectContext::detect(temp.path()).unwrap();
        assert_eq!(context.package_manager, Some(PackageManager::Yarn));
        assert!(context.detected_files.contains(&"yarn.lock".to_string()));
    }

    #[test]
    fn test_detect_npm_project() {
        let temp = TempDir::new().unwrap();
        File::create(temp.path().join("package-lock.json")).unwrap();
        File::create(temp.path().join("package.json")).unwrap();

        let context = ProjectContext::detect(temp.path()).unwrap();
        assert_eq!(context.package_manager, Some(PackageManager::Npm));
    }

    #[test]
    fn test_detect_rust_project() {
        let temp = TempDir::new().unwrap();
        File::create(temp.path().join("Cargo.toml")).unwrap();

        let context = ProjectContext::detect(temp.path()).unwrap();
        assert_eq!(context.package_manager, Some(PackageManager::Cargo));
        assert!(context.languages.contains(&Language::Rust));
    }

    #[test]
    fn test_detect_typescript_project() {
        let temp = TempDir::new().unwrap();
        File::create(temp.path().join("tsconfig.json")).unwrap();
        File::create(temp.path().join("package.json")).unwrap();
        File::create(temp.path().join("yarn.lock")).unwrap();

        let context = ProjectContext::detect(temp.path()).unwrap();
        assert_eq!(context.package_manager, Some(PackageManager::Yarn));
        assert!(context.languages.contains(&Language::TypeScript));
    }

    #[test]
    fn test_detect_makefile() {
        let temp = TempDir::new().unwrap();
        File::create(temp.path().join("Makefile")).unwrap();

        let context = ProjectContext::detect(temp.path()).unwrap();
        assert_eq!(context.build_tool, Some(BuildTool::Make));
    }

    #[test]
    fn test_package_manager_alternatives() {
        let yarn = PackageManager::Yarn;
        assert!(yarn.alternatives().contains(&"npm"));
        assert!(yarn.alternatives().contains(&"pnpm"));
    }

    #[test]
    fn test_wrong_package_manager_detection() {
        let temp = TempDir::new().unwrap();
        File::create(temp.path().join("yarn.lock")).unwrap();

        let context = ProjectContext::detect(temp.path()).unwrap();

        // Should detect npm as wrong manager when yarn is expected
        let wrong = context.is_wrong_package_manager("npm install lodash");
        assert_eq!(wrong, Some(PackageManager::Npm));

        // yarn should be correct
        let correct = context.is_wrong_package_manager("yarn add lodash");
        assert!(correct.is_none());
    }

    #[test]
    fn test_npm_to_yarn_translation() {
        let yarn = PackageManager::Yarn;
        let npm = PackageManager::Npm;

        let translated = yarn.translate_from(&npm, "install lodash");
        assert_eq!(translated, Some("yarn add lodash".to_string()));
    }

    #[test]
    fn test_empty_project() {
        let temp = TempDir::new().unwrap();

        let context = ProjectContext::detect(temp.path()).unwrap();
        assert!(context.package_manager.is_none());
        assert!(context.build_tool.is_none());
        assert!(context.languages.is_empty());
    }
}
