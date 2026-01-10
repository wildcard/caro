//! Project type detection (Starship-like functionality)
//!
//! Detects the type of project in a directory by looking for characteristic
//! files like package.json, Cargo.toml, Makefile, etc.
//!
//! This helps the reasoning engine understand what tools and commands
//! are available in the current context.

use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::{Path, PathBuf};

/// Types of projects that can be detected
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ProjectType {
    /// Rust project (Cargo.toml)
    Rust,
    /// Node.js project (package.json)
    Node,
    /// Python project (pyproject.toml, setup.py, requirements.txt)
    Python,
    /// Go project (go.mod)
    Go,
    /// Java/Maven project (pom.xml)
    Maven,
    /// Java/Gradle project (build.gradle)
    Gradle,
    /// Ruby project (Gemfile)
    Ruby,
    /// C/C++ with Makefile
    Make,
    /// CMake project (CMakeLists.txt)
    CMake,
    /// Docker project (Dockerfile)
    Docker,
    /// Kubernetes project (k8s manifests)
    Kubernetes,
    /// Terraform project
    Terraform,
    /// Git repository
    Git,
    /// Unknown/generic
    Unknown,
}

impl ProjectType {
    /// Get the primary build/run commands for this project type
    pub fn primary_commands(&self) -> Vec<&'static str> {
        match self {
            Self::Rust => vec!["cargo build", "cargo run", "cargo test", "cargo check"],
            Self::Node => vec!["npm run", "npm install", "npm test", "yarn", "pnpm"],
            Self::Python => vec!["python", "pip install", "pytest", "poetry run"],
            Self::Go => vec!["go build", "go run", "go test", "go mod"],
            Self::Maven => vec!["mvn compile", "mvn test", "mvn package"],
            Self::Gradle => vec!["gradle build", "gradle test", "./gradlew"],
            Self::Ruby => vec!["bundle install", "rake", "ruby"],
            Self::Make => vec!["make", "make all", "make clean", "make install"],
            Self::CMake => vec!["cmake", "cmake --build", "ctest"],
            Self::Docker => vec!["docker build", "docker run", "docker-compose"],
            Self::Kubernetes => vec!["kubectl apply", "kubectl get", "helm"],
            Self::Terraform => vec!["terraform plan", "terraform apply", "terraform init"],
            Self::Git => vec!["git status", "git commit", "git push", "git pull"],
            Self::Unknown => vec![],
        }
    }
}

/// Information about a detected toolchain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolchainInfo {
    /// The detected project type
    pub project_type: ProjectType,
    /// Version of the tool (if detectable)
    pub version: Option<String>,
    /// Configuration file found
    pub config_file: PathBuf,
    /// Available scripts/targets
    pub available_scripts: Vec<String>,
}

/// Complete project context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectContext {
    /// Root directory of the project
    pub root: PathBuf,
    /// Primary project type
    pub project_type: ProjectType,
    /// All detected toolchains
    pub toolchains: Vec<ToolchainInfo>,
    /// Available tools/commands in this project
    pub available_tools: Vec<String>,
    /// Is this a git repository?
    pub is_git_repo: bool,
    /// Package manager in use (npm, yarn, pnpm, cargo, pip, etc.)
    pub package_manager: Option<String>,
}

impl Default for ProjectContext {
    fn default() -> Self {
        Self {
            root: PathBuf::new(),
            project_type: ProjectType::Unknown,
            toolchains: Vec::new(),
            available_tools: Vec::new(),
            is_git_repo: false,
            package_manager: None,
        }
    }
}

/// Detects project type and available tools in a directory
pub struct ProjectDetector {
    /// File patterns to look for
    detection_rules: Vec<DetectionRule>,
}

/// Rule for detecting a project type
struct DetectionRule {
    project_type: ProjectType,
    indicator_files: Vec<&'static str>,
    priority: u8,
    script_extractor: Option<fn(&Path) -> Vec<String>>,
}

impl ProjectDetector {
    /// Create a new project detector
    pub fn new() -> Self {
        Self {
            detection_rules: vec![
                // Rust
                DetectionRule {
                    project_type: ProjectType::Rust,
                    indicator_files: vec!["Cargo.toml"],
                    priority: 10,
                    script_extractor: Some(extract_cargo_scripts),
                },
                // Node.js
                DetectionRule {
                    project_type: ProjectType::Node,
                    indicator_files: vec!["package.json"],
                    priority: 10,
                    script_extractor: Some(extract_npm_scripts),
                },
                // Python
                DetectionRule {
                    project_type: ProjectType::Python,
                    indicator_files: vec!["pyproject.toml", "setup.py", "requirements.txt", "Pipfile"],
                    priority: 9,
                    script_extractor: None,
                },
                // Go
                DetectionRule {
                    project_type: ProjectType::Go,
                    indicator_files: vec!["go.mod"],
                    priority: 10,
                    script_extractor: None,
                },
                // Maven
                DetectionRule {
                    project_type: ProjectType::Maven,
                    indicator_files: vec!["pom.xml"],
                    priority: 10,
                    script_extractor: None,
                },
                // Gradle
                DetectionRule {
                    project_type: ProjectType::Gradle,
                    indicator_files: vec!["build.gradle", "build.gradle.kts"],
                    priority: 10,
                    script_extractor: None,
                },
                // Ruby
                DetectionRule {
                    project_type: ProjectType::Ruby,
                    indicator_files: vec!["Gemfile"],
                    priority: 9,
                    script_extractor: None,
                },
                // Make
                DetectionRule {
                    project_type: ProjectType::Make,
                    indicator_files: vec!["Makefile", "makefile", "GNUmakefile"],
                    priority: 8,
                    script_extractor: Some(extract_make_targets),
                },
                // CMake
                DetectionRule {
                    project_type: ProjectType::CMake,
                    indicator_files: vec!["CMakeLists.txt"],
                    priority: 9,
                    script_extractor: None,
                },
                // Docker
                DetectionRule {
                    project_type: ProjectType::Docker,
                    indicator_files: vec!["Dockerfile", "docker-compose.yml", "docker-compose.yaml", "compose.yaml"],
                    priority: 7,
                    script_extractor: None,
                },
                // Kubernetes
                DetectionRule {
                    project_type: ProjectType::Kubernetes,
                    indicator_files: vec!["k8s/", "kubernetes/", "Chart.yaml"],
                    priority: 6,
                    script_extractor: None,
                },
                // Terraform
                DetectionRule {
                    project_type: ProjectType::Terraform,
                    indicator_files: vec!["main.tf", "terraform.tf"],
                    priority: 8,
                    script_extractor: None,
                },
                // Git (lowest priority, just marks as git repo)
                DetectionRule {
                    project_type: ProjectType::Git,
                    indicator_files: vec![".git/"],
                    priority: 1,
                    script_extractor: None,
                },
            ],
        }
    }

    /// Detect project context in the given directory
    pub fn detect(&self, path: &Path) -> ProjectContext {
        let mut context = ProjectContext {
            root: path.to_path_buf(),
            ..Default::default()
        };

        let mut detected_toolchains: Vec<(ToolchainInfo, u8)> = Vec::new();

        // Check each detection rule
        for rule in &self.detection_rules {
            for indicator in &rule.indicator_files {
                let indicator_path = if indicator.ends_with('/') {
                    // It's a directory
                    path.join(indicator.trim_end_matches('/'))
                } else {
                    path.join(indicator)
                };

                if indicator_path.exists() {
                    let scripts = if let Some(extractor) = rule.script_extractor {
                        extractor(&indicator_path)
                    } else {
                        Vec::new()
                    };

                    let toolchain = ToolchainInfo {
                        project_type: rule.project_type,
                        version: None,
                        config_file: indicator_path,
                        available_scripts: scripts,
                    };

                    detected_toolchains.push((toolchain, rule.priority));

                    // Handle special cases
                    if rule.project_type == ProjectType::Git {
                        context.is_git_repo = true;
                    }

                    break; // Found for this rule, move to next
                }
            }
        }

        // Sort by priority and select primary project type
        detected_toolchains.sort_by(|a, b| b.1.cmp(&a.1));

        if !detected_toolchains.is_empty() {
            context.project_type = detected_toolchains[0].0.project_type;
            context.toolchains = detected_toolchains.into_iter().map(|(t, _)| t).collect();
        }

        // Detect package manager
        context.package_manager = self.detect_package_manager(path);

        // Build available tools list
        context.available_tools = self.build_available_tools(&context);

        context
    }

    /// Detect the package manager in use
    fn detect_package_manager(&self, path: &Path) -> Option<String> {
        // Node.js package managers
        if path.join("pnpm-lock.yaml").exists() {
            return Some("pnpm".to_string());
        }
        if path.join("yarn.lock").exists() {
            return Some("yarn".to_string());
        }
        if path.join("package-lock.json").exists() {
            return Some("npm".to_string());
        }

        // Python package managers
        if path.join("poetry.lock").exists() {
            return Some("poetry".to_string());
        }
        if path.join("Pipfile.lock").exists() {
            return Some("pipenv".to_string());
        }
        if path.join("requirements.txt").exists() {
            return Some("pip".to_string());
        }

        // Rust
        if path.join("Cargo.lock").exists() {
            return Some("cargo".to_string());
        }

        // Go
        if path.join("go.sum").exists() {
            return Some("go mod".to_string());
        }

        None
    }

    /// Build list of available tools based on detected context
    fn build_available_tools(&self, context: &ProjectContext) -> Vec<String> {
        let mut tools = HashSet::new();

        // Add tools from detected toolchains
        for toolchain in &context.toolchains {
            for cmd in toolchain.project_type.primary_commands() {
                tools.insert(cmd.to_string());
            }

            // Add available scripts
            for script in &toolchain.available_scripts {
                tools.insert(script.clone());
            }
        }

        // Add package manager specific tools
        if let Some(ref pm) = context.package_manager {
            match pm.as_str() {
                "npm" => {
                    tools.insert("npm run".to_string());
                    tools.insert("npx".to_string());
                }
                "yarn" => {
                    tools.insert("yarn".to_string());
                    tools.insert("yarn run".to_string());
                }
                "pnpm" => {
                    tools.insert("pnpm run".to_string());
                    tools.insert("pnpm exec".to_string());
                }
                "cargo" => {
                    tools.insert("cargo".to_string());
                }
                "pip" | "poetry" | "pipenv" => {
                    tools.insert("python".to_string());
                    tools.insert("pytest".to_string());
                }
                _ => {}
            }
        }

        tools.into_iter().collect()
    }
}

impl Default for ProjectDetector {
    fn default() -> Self {
        Self::new()
    }
}

/// Extract available scripts from package.json
fn extract_npm_scripts(path: &Path) -> Vec<String> {
    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };

    // Simple JSON parsing for scripts
    let mut scripts = Vec::new();

    // Find "scripts" section
    if let Some(start) = content.find("\"scripts\"") {
        if let Some(brace_start) = content[start..].find('{') {
            let after_brace = &content[start + brace_start..];
            if let Some(brace_end) = find_matching_brace(after_brace) {
                let scripts_section = &after_brace[1..brace_end];

                // Extract script names
                for line in scripts_section.lines() {
                    let trimmed = line.trim();
                    if let Some(quote_start) = trimmed.find('"') {
                        if let Some(quote_end) = trimmed[quote_start + 1..].find('"') {
                            let script_name = &trimmed[quote_start + 1..quote_start + 1 + quote_end];
                            scripts.push(format!("npm run {}", script_name));
                        }
                    }
                }
            }
        }
    }

    scripts
}

/// Extract available targets from Makefile
fn extract_make_targets(path: &Path) -> Vec<String> {
    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };

    let mut targets = Vec::new();

    for line in content.lines() {
        // Look for target definitions (name: or name::)
        let trimmed = line.trim();

        // Skip comments and empty lines
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        // Skip variable assignments
        if trimmed.contains('=') && !trimmed.contains(':') {
            continue;
        }

        // Look for target pattern
        if let Some(colon_pos) = trimmed.find(':') {
            let target = trimmed[..colon_pos].trim();

            // Skip special targets and patterns
            if target.starts_with('.')
                || target.starts_with('$')
                || target.contains('%')
                || target.is_empty()
            {
                continue;
            }

            // Skip if it looks like a file rule (has . in the name)
            if target.contains('.') && !["all", "install", "clean", "test", "check"].contains(&target) {
                continue;
            }

            targets.push(format!("make {}", target));
        }
    }

    targets
}

/// Extract available cargo commands from Cargo.toml
fn extract_cargo_scripts(path: &Path) -> Vec<String> {
    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };

    let mut scripts = vec![
        "cargo build".to_string(),
        "cargo run".to_string(),
        "cargo test".to_string(),
        "cargo check".to_string(),
    ];

    // Check for binary targets
    if content.contains("[[bin]]") {
        scripts.push("cargo build --bin".to_string());
    }

    // Check for examples
    if content.contains("[[example]]") {
        scripts.push("cargo run --example".to_string());
    }

    // Check for benches
    if content.contains("[[bench]]") {
        scripts.push("cargo bench".to_string());
    }

    scripts
}

/// Find the position of the matching closing brace
fn find_matching_brace(s: &str) -> Option<usize> {
    let mut depth = 0;
    let mut in_string = false;
    let mut escape_next = false;

    for (i, c) in s.char_indices() {
        if escape_next {
            escape_next = false;
            continue;
        }

        if c == '\\' {
            escape_next = true;
            continue;
        }

        if c == '"' {
            in_string = !in_string;
            continue;
        }

        if in_string {
            continue;
        }

        if c == '{' {
            depth += 1;
        } else if c == '}' {
            depth -= 1;
            if depth == 0 {
                return Some(i);
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_detect_rust_project() {
        let temp_dir = TempDir::new().unwrap();
        let cargo_path = temp_dir.path().join("Cargo.toml");
        fs::write(&cargo_path, "[package]\nname = \"test\"").unwrap();

        let detector = ProjectDetector::new();
        let context = detector.detect(temp_dir.path());

        assert_eq!(context.project_type, ProjectType::Rust);
        assert!(context.available_tools.iter().any(|t| t.contains("cargo")));
    }

    #[test]
    fn test_detect_node_project() {
        let temp_dir = TempDir::new().unwrap();
        let package_path = temp_dir.path().join("package.json");
        fs::write(
            &package_path,
            r#"{"name": "test", "scripts": {"build": "tsc", "test": "jest"}}"#,
        ).unwrap();

        let detector = ProjectDetector::new();
        let context = detector.detect(temp_dir.path());

        assert_eq!(context.project_type, ProjectType::Node);
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
    fn test_detect_package_manager() {
        let temp_dir = TempDir::new().unwrap();

        // npm
        fs::write(temp_dir.path().join("package-lock.json"), "{}").unwrap();
        let detector = ProjectDetector::new();
        let context = detector.detect(temp_dir.path());
        assert_eq!(context.package_manager, Some("npm".to_string()));
    }

    #[test]
    fn test_make_target_extraction() {
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

clean:
	rm -rf build
"#,
        ).unwrap();

        let targets = extract_make_targets(&makefile_path);
        assert!(targets.contains(&"make all".to_string()));
        assert!(targets.contains(&"make build".to_string()));
        assert!(targets.contains(&"make test".to_string()));
        assert!(targets.contains(&"make clean".to_string()));
    }
}
