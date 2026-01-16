//! Directory context detection for improved command generation
//!
//! This module scans the current directory to detect project type,
//! available tools, and relevant files that can help generate
//! more contextually appropriate shell commands.

use std::collections::HashSet;
use std::fs;
use std::path::Path;

/// Project type indicators detected from directory contents
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ProjectType {
    /// Node.js project (package.json present)
    NodeJs,
    /// Rust project (Cargo.toml present)
    Rust,
    /// Python project (pyproject.toml, setup.py, or requirements.txt present)
    Python,
    /// Go project (go.mod present)
    Go,
    /// Java/Kotlin project (pom.xml or build.gradle present)
    Java,
    /// Ruby project (Gemfile present)
    Ruby,
    /// Generic/unknown project type
    Generic,
}

impl std::fmt::Display for ProjectType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProjectType::NodeJs => write!(f, "Node.js"),
            ProjectType::Rust => write!(f, "Rust"),
            ProjectType::Python => write!(f, "Python"),
            ProjectType::Go => write!(f, "Go"),
            ProjectType::Java => write!(f, "Java/Kotlin"),
            ProjectType::Ruby => write!(f, "Ruby"),
            ProjectType::Generic => write!(f, "Generic"),
        }
    }
}

/// Context information about the current directory
#[derive(Debug, Clone, Default)]
pub struct DirectoryContext {
    /// Detected project types (can be multiple)
    pub project_types: HashSet<ProjectType>,
    /// Whether this is a Git repository
    pub has_git: bool,
    /// Whether a Makefile is present
    pub has_makefile: bool,
    /// Whether Docker configuration is present
    pub has_docker: bool,
    /// Whether docker-compose configuration is present
    pub has_docker_compose: bool,
    /// NPM scripts available (from package.json)
    pub npm_scripts: Vec<String>,
    /// Make targets available (from Makefile)
    pub make_targets: Vec<String>,
    /// Cargo commands/aliases available
    pub cargo_commands: Vec<String>,
    /// Python package manager detected (pip, poetry, uv)
    pub python_package_manager: Option<String>,
}

impl DirectoryContext {
    /// Scan a directory to detect project context
    pub fn scan(path: &Path) -> Self {
        let mut ctx = DirectoryContext::default();

        if !path.is_dir() {
            return ctx;
        }

        // Check for Git repository
        ctx.has_git = path.join(".git").exists();

        // Check for project markers
        if path.join("package.json").exists() {
            ctx.project_types.insert(ProjectType::NodeJs);
            ctx.npm_scripts = Self::extract_npm_scripts(path);
        }

        if path.join("Cargo.toml").exists() {
            ctx.project_types.insert(ProjectType::Rust);
            ctx.cargo_commands = Self::get_cargo_commands();
        }

        if path.join("pyproject.toml").exists()
            || path.join("setup.py").exists()
            || path.join("requirements.txt").exists()
        {
            ctx.project_types.insert(ProjectType::Python);
            ctx.python_package_manager = Self::detect_python_package_manager(path);
        }

        if path.join("go.mod").exists() {
            ctx.project_types.insert(ProjectType::Go);
        }

        if path.join("pom.xml").exists() || path.join("build.gradle").exists() {
            ctx.project_types.insert(ProjectType::Java);
        }

        if path.join("Gemfile").exists() {
            ctx.project_types.insert(ProjectType::Ruby);
        }

        // Check for build tools
        if path.join("Makefile").exists() || path.join("makefile").exists() {
            ctx.has_makefile = true;
            ctx.make_targets = Self::extract_make_targets(path);
        }

        // Check for Docker
        if path.join("Dockerfile").exists() {
            ctx.has_docker = true;
        }

        if path.join("docker-compose.yml").exists() || path.join("docker-compose.yaml").exists() {
            ctx.has_docker_compose = true;
        }

        // If no specific project type detected, mark as generic
        if ctx.project_types.is_empty() {
            ctx.project_types.insert(ProjectType::Generic);
        }

        ctx
    }

    /// Extract NPM scripts from package.json
    fn extract_npm_scripts(path: &Path) -> Vec<String> {
        let package_json = path.join("package.json");
        if let Ok(content) = fs::read_to_string(package_json) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(scripts) = json.get("scripts").and_then(|s| s.as_object()) {
                    return scripts.keys().cloned().collect();
                }
            }
        }
        Vec::new()
    }

    /// Extract make targets from Makefile
    fn extract_make_targets(path: &Path) -> Vec<String> {
        let makefile = if path.join("Makefile").exists() {
            path.join("Makefile")
        } else {
            path.join("makefile")
        };

        let mut targets = Vec::new();
        if let Ok(content) = fs::read_to_string(makefile) {
            for line in content.lines() {
                // Match lines like "target:" or "target: dependency"
                // Skip lines starting with dot (hidden targets) or tab (recipes)
                if !line.starts_with('\t')
                    && !line.starts_with('.')
                    && !line.starts_with('#')
                    && !line.is_empty()
                {
                    if let Some(idx) = line.find(':') {
                        let target = line[..idx].trim();
                        // Skip variable assignments and pattern rules
                        if !target.contains('=')
                            && !target.contains('%')
                            && !target.contains('$')
                            && !target.is_empty()
                        {
                            targets.push(target.to_string());
                        }
                    }
                }
            }
        }
        targets
    }

    /// Get common Cargo commands
    fn get_cargo_commands() -> Vec<String> {
        vec![
            "build".to_string(),
            "run".to_string(),
            "test".to_string(),
            "clippy".to_string(),
            "fmt".to_string(),
            "check".to_string(),
            "doc".to_string(),
        ]
    }

    /// Detect which Python package manager is being used
    fn detect_python_package_manager(path: &Path) -> Option<String> {
        if path.join("uv.lock").exists() {
            return Some("uv".to_string());
        }
        if path.join("poetry.lock").exists() {
            return Some("poetry".to_string());
        }
        if path.join("Pipfile.lock").exists() {
            return Some("pipenv".to_string());
        }
        if path.join("requirements.txt").exists() {
            return Some("pip".to_string());
        }
        None
    }

    /// Convert directory context to a string for LLM prompts
    pub fn to_context_string(&self) -> String {
        let mut parts = Vec::new();

        // Project types
        let types: Vec<String> = self.project_types.iter().map(|t| t.to_string()).collect();
        if !types.is_empty() && !self.project_types.contains(&ProjectType::Generic) {
            parts.push(format!("Project type: {}", types.join(", ")));
        }

        // Git
        if self.has_git {
            parts.push("Git repository: yes".to_string());
        }

        // Build tools
        if self.has_makefile {
            if !self.make_targets.is_empty() {
                parts.push(format!(
                    "Make targets: {}",
                    self.make_targets
                        .iter()
                        .take(10)
                        .cloned()
                        .collect::<Vec<_>>()
                        .join(", ")
                ));
            } else {
                parts.push("Makefile: present".to_string());
            }
        }

        // Docker
        if self.has_docker || self.has_docker_compose {
            let docker_info = match (self.has_docker, self.has_docker_compose) {
                (true, true) => "Docker: Dockerfile + docker-compose",
                (true, false) => "Docker: Dockerfile",
                (false, true) => "Docker: docker-compose",
                _ => "",
            };
            if !docker_info.is_empty() {
                parts.push(docker_info.to_string());
            }
        }

        // NPM scripts
        if !self.npm_scripts.is_empty() {
            parts.push(format!(
                "NPM scripts: {}",
                self.npm_scripts
                    .iter()
                    .take(10)
                    .cloned()
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }

        // Python package manager
        if let Some(ref pm) = self.python_package_manager {
            parts.push(format!("Python package manager: {}", pm));
        }

        // Cargo commands (only if Rust project)
        if self.project_types.contains(&ProjectType::Rust) && !self.cargo_commands.is_empty() {
            parts.push(format!(
                "Cargo commands: {}",
                self.cargo_commands.join(", ")
            ));
        }

        if parts.is_empty() {
            String::new()
        } else {
            format!("Directory context:\n{}", parts.join("\n"))
        }
    }

    /// Check if the directory has any meaningful context
    pub fn has_context(&self) -> bool {
        self.has_git
            || self.has_makefile
            || self.has_docker
            || self.has_docker_compose
            || !self.npm_scripts.is_empty()
            || !self.make_targets.is_empty()
            || self.python_package_manager.is_some()
            || self
                .project_types
                .iter()
                .any(|t| *t != ProjectType::Generic)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::TempDir;

    #[test]
    fn test_detect_nodejs_project() {
        let temp_dir = TempDir::new().unwrap();
        let package_json = temp_dir.path().join("package.json");
        let mut file = File::create(&package_json).unwrap();
        writeln!(
            file,
            r#"{{
            "name": "test",
            "scripts": {{
                "build": "tsc",
                "test": "jest",
                "start": "node index.js"
            }}
        }}"#
        )
        .unwrap();

        let ctx = DirectoryContext::scan(temp_dir.path());

        assert!(ctx.project_types.contains(&ProjectType::NodeJs));
        assert!(ctx.npm_scripts.contains(&"build".to_string()));
        assert!(ctx.npm_scripts.contains(&"test".to_string()));
        assert!(ctx.npm_scripts.contains(&"start".to_string()));
    }

    #[test]
    fn test_detect_rust_project() {
        let temp_dir = TempDir::new().unwrap();
        File::create(temp_dir.path().join("Cargo.toml")).unwrap();

        let ctx = DirectoryContext::scan(temp_dir.path());

        assert!(ctx.project_types.contains(&ProjectType::Rust));
        assert!(ctx.cargo_commands.contains(&"build".to_string()));
        assert!(ctx.cargo_commands.contains(&"test".to_string()));
    }

    #[test]
    fn test_detect_python_project() {
        let temp_dir = TempDir::new().unwrap();
        File::create(temp_dir.path().join("pyproject.toml")).unwrap();
        File::create(temp_dir.path().join("poetry.lock")).unwrap();

        let ctx = DirectoryContext::scan(temp_dir.path());

        assert!(ctx.project_types.contains(&ProjectType::Python));
        assert_eq!(ctx.python_package_manager, Some("poetry".to_string()));
    }

    #[test]
    fn test_detect_git_repository() {
        let temp_dir = TempDir::new().unwrap();
        fs::create_dir(temp_dir.path().join(".git")).unwrap();

        let ctx = DirectoryContext::scan(temp_dir.path());

        assert!(ctx.has_git);
    }

    #[test]
    fn test_detect_makefile() {
        let temp_dir = TempDir::new().unwrap();
        let makefile = temp_dir.path().join("Makefile");
        let mut file = File::create(&makefile).unwrap();
        writeln!(
            file,
            r#"build:
	cargo build

test:
	cargo test

clean:
	rm -rf target"#
        )
        .unwrap();

        let ctx = DirectoryContext::scan(temp_dir.path());

        assert!(ctx.has_makefile);
        assert!(ctx.make_targets.contains(&"build".to_string()));
        assert!(ctx.make_targets.contains(&"test".to_string()));
        assert!(ctx.make_targets.contains(&"clean".to_string()));
    }

    #[test]
    fn test_detect_docker() {
        let temp_dir = TempDir::new().unwrap();
        File::create(temp_dir.path().join("Dockerfile")).unwrap();
        File::create(temp_dir.path().join("docker-compose.yml")).unwrap();

        let ctx = DirectoryContext::scan(temp_dir.path());

        assert!(ctx.has_docker);
        assert!(ctx.has_docker_compose);
    }

    #[test]
    fn test_multiple_project_types() {
        let temp_dir = TempDir::new().unwrap();
        File::create(temp_dir.path().join("package.json")).unwrap();
        File::create(temp_dir.path().join("Cargo.toml")).unwrap();
        fs::create_dir(temp_dir.path().join(".git")).unwrap();

        let ctx = DirectoryContext::scan(temp_dir.path());

        assert!(ctx.project_types.contains(&ProjectType::NodeJs));
        assert!(ctx.project_types.contains(&ProjectType::Rust));
        assert!(ctx.has_git);
    }

    #[test]
    fn test_to_context_string() {
        let temp_dir = TempDir::new().unwrap();
        File::create(temp_dir.path().join("Cargo.toml")).unwrap();
        fs::create_dir(temp_dir.path().join(".git")).unwrap();

        let ctx = DirectoryContext::scan(temp_dir.path());
        let context_str = ctx.to_context_string();

        assert!(context_str.contains("Rust"));
        assert!(context_str.contains("Git repository: yes"));
    }

    #[test]
    fn test_empty_directory() {
        let temp_dir = TempDir::new().unwrap();

        let ctx = DirectoryContext::scan(temp_dir.path());

        assert!(ctx.project_types.contains(&ProjectType::Generic));
        assert!(!ctx.has_git);
        assert!(!ctx.has_makefile);
    }

    #[test]
    fn test_has_context() {
        let temp_dir = TempDir::new().unwrap();

        // Empty directory has no meaningful context
        let ctx = DirectoryContext::scan(temp_dir.path());
        assert!(!ctx.has_context());

        // Directory with Git has context
        fs::create_dir(temp_dir.path().join(".git")).unwrap();
        let ctx = DirectoryContext::scan(temp_dir.path());
        assert!(ctx.has_context());
    }
}
