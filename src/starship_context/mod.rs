//! Starship-based context collection for enhanced command generation
//!
//! This module uses the starship crate to collect rich contextual information
//! about the current environment, including:
//! - Git repository state (branch, status, remote)
//! - Directory contents for project type detection
//! - Enhanced shell detection
//! - Environment variable access
//!
//! This context is used to improve command generation by providing the LLM
//! with more detailed information about the user's environment.

use serde::{Deserialize, Serialize};
use starship::context::{Context, Properties, Shell as StarshipShell, Target};
use std::collections::HashMap;
use std::path::PathBuf;

/// Git repository information extracted from starship context
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GitContext {
    /// Whether we're inside a git repository
    pub is_git_repo: bool,

    /// Current branch name (e.g., "main", "feature/foo")
    pub branch: Option<String>,

    /// Root directory of the git repository
    pub repo_root: Option<PathBuf>,

    /// Path to the .git directory
    pub git_dir: Option<PathBuf>,

    /// Repository state (normal, rebasing, merging, etc.)
    pub state: Option<String>,

    /// Remote URL if available
    pub remote_url: Option<String>,

    /// Remote name (e.g., "origin")
    pub remote_name: Option<String>,
}

impl GitContext {
    /// Create a summary string for LLM prompts
    pub fn to_prompt_summary(&self) -> String {
        if !self.is_git_repo {
            return String::from("Not in a git repository");
        }

        let mut parts = Vec::new();

        if let Some(ref branch) = self.branch {
            parts.push(format!("branch: {}", branch));
        }

        if let Some(ref state) = self.state {
            if state != "Clean" {
                parts.push(format!("state: {}", state));
            }
        }

        if let Some(ref remote) = self.remote_name {
            parts.push(format!("remote: {}", remote));
        }

        if parts.is_empty() {
            String::from("Git repository (details unavailable)")
        } else {
            format!("Git: {}", parts.join(", "))
        }
    }
}

/// Project type detected from directory contents
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ProjectType {
    /// Rust project (Cargo.toml present)
    Rust,
    /// Node.js project (package.json present)
    Node,
    /// Python project (pyproject.toml, setup.py, or requirements.txt present)
    Python,
    /// Go project (go.mod present)
    Go,
    /// Java/Maven project (pom.xml present)
    Maven,
    /// Java/Gradle project (build.gradle present)
    Gradle,
    /// Ruby project (Gemfile present)
    Ruby,
    /// Docker project (Dockerfile present)
    Docker,
    /// Kubernetes project (*.yaml with kind: present)
    Kubernetes,
    /// Terraform project (.tf files present)
    Terraform,
    /// Generic project (no specific type detected)
    Generic,
}

impl ProjectType {
    /// Get command hints specific to this project type
    pub fn command_hints(&self) -> Vec<&'static str> {
        match self {
            ProjectType::Rust => vec![
                "cargo build - compile the project",
                "cargo run - run the project",
                "cargo test - run tests",
                "cargo clippy - run linter",
                "cargo fmt - format code",
            ],
            ProjectType::Node => vec![
                "npm install - install dependencies",
                "npm run build - build project",
                "npm test - run tests",
                "npm start - start the application",
            ],
            ProjectType::Python => vec![
                "pip install -r requirements.txt - install dependencies",
                "python -m pytest - run tests",
                "python main.py - run main script",
                "pip install -e . - install in editable mode",
            ],
            ProjectType::Go => vec![
                "go build - compile packages",
                "go run . - compile and run",
                "go test ./... - run tests",
                "go mod tidy - clean up dependencies",
            ],
            ProjectType::Maven => vec![
                "mvn clean install - build project",
                "mvn test - run tests",
                "mvn package - create JAR/WAR",
            ],
            ProjectType::Gradle => vec![
                "./gradlew build - build project",
                "./gradlew test - run tests",
                "./gradlew run - run application",
            ],
            ProjectType::Ruby => vec![
                "bundle install - install dependencies",
                "bundle exec rspec - run tests",
                "bundle exec rails s - start Rails server",
            ],
            ProjectType::Docker => vec![
                "docker build -t <name> . - build image",
                "docker run <name> - run container",
                "docker-compose up - start services",
            ],
            ProjectType::Kubernetes => vec![
                "kubectl apply -f <file> - apply configuration",
                "kubectl get pods - list pods",
                "kubectl logs <pod> - view logs",
            ],
            ProjectType::Terraform => vec![
                "terraform init - initialize",
                "terraform plan - preview changes",
                "terraform apply - apply changes",
            ],
            ProjectType::Generic => vec![],
        }
    }
}

impl std::fmt::Display for ProjectType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProjectType::Rust => write!(f, "Rust"),
            ProjectType::Node => write!(f, "Node.js"),
            ProjectType::Python => write!(f, "Python"),
            ProjectType::Go => write!(f, "Go"),
            ProjectType::Maven => write!(f, "Maven"),
            ProjectType::Gradle => write!(f, "Gradle"),
            ProjectType::Ruby => write!(f, "Ruby"),
            ProjectType::Docker => write!(f, "Docker"),
            ProjectType::Kubernetes => write!(f, "Kubernetes"),
            ProjectType::Terraform => write!(f, "Terraform"),
            ProjectType::Generic => write!(f, "Generic"),
        }
    }
}

/// Enhanced shell type from starship detection
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum EnhancedShellType {
    Bash,
    Zsh,
    Fish,
    PowerShell,
    Cmd,
    Elvish,
    Ion,
    Tcsh,
    Nu,
    Xonsh,
    Unknown,
}

impl EnhancedShellType {
    /// Convert from starship's Shell type
    pub fn from_starship(shell: StarshipShell) -> Self {
        match shell {
            StarshipShell::Bash => EnhancedShellType::Bash,
            StarshipShell::Zsh => EnhancedShellType::Zsh,
            StarshipShell::Fish => EnhancedShellType::Fish,
            StarshipShell::Pwsh | StarshipShell::PowerShell => EnhancedShellType::PowerShell,
            StarshipShell::Cmd => EnhancedShellType::Cmd,
            StarshipShell::Elvish => EnhancedShellType::Elvish,
            StarshipShell::Ion => EnhancedShellType::Ion,
            StarshipShell::Tcsh => EnhancedShellType::Tcsh,
            StarshipShell::Nu => EnhancedShellType::Nu,
            StarshipShell::Xonsh => EnhancedShellType::Xonsh,
            StarshipShell::Unknown => EnhancedShellType::Unknown,
        }
    }

    /// Convert to starship's Shell type
    pub fn to_starship(self) -> StarshipShell {
        match self {
            EnhancedShellType::Bash => StarshipShell::Bash,
            EnhancedShellType::Zsh => StarshipShell::Zsh,
            EnhancedShellType::Fish => StarshipShell::Fish,
            EnhancedShellType::PowerShell => StarshipShell::Pwsh,
            EnhancedShellType::Cmd => StarshipShell::Cmd,
            EnhancedShellType::Elvish => StarshipShell::Elvish,
            EnhancedShellType::Ion => StarshipShell::Ion,
            EnhancedShellType::Tcsh => StarshipShell::Tcsh,
            EnhancedShellType::Nu => StarshipShell::Nu,
            EnhancedShellType::Xonsh => StarshipShell::Xonsh,
            EnhancedShellType::Unknown => StarshipShell::Unknown,
        }
    }

    /// Detect shell from environment
    pub fn detect() -> Self {
        if let Ok(shell_path) = std::env::var("SHELL") {
            if shell_path.contains("bash") {
                return EnhancedShellType::Bash;
            } else if shell_path.contains("zsh") {
                return EnhancedShellType::Zsh;
            } else if shell_path.contains("fish") {
                return EnhancedShellType::Fish;
            } else if shell_path.contains("elvish") {
                return EnhancedShellType::Elvish;
            } else if shell_path.contains("ion") {
                return EnhancedShellType::Ion;
            } else if shell_path.contains("tcsh") {
                return EnhancedShellType::Tcsh;
            } else if shell_path.contains("nu") {
                return EnhancedShellType::Nu;
            } else if shell_path.contains("xonsh") {
                return EnhancedShellType::Xonsh;
            }
        }

        if std::env::var("PSModulePath").is_ok() {
            return EnhancedShellType::PowerShell;
        }

        #[cfg(target_os = "windows")]
        {
            return EnhancedShellType::Cmd;
        }

        #[cfg(not(target_os = "windows"))]
        EnhancedShellType::Unknown
    }

    /// Check if this is a POSIX-compatible shell
    pub fn is_posix(&self) -> bool {
        matches!(
            self,
            EnhancedShellType::Bash | EnhancedShellType::Zsh | EnhancedShellType::Fish
        )
    }

    /// Check if this is a Windows shell
    pub fn is_windows(&self) -> bool {
        matches!(self, EnhancedShellType::PowerShell | EnhancedShellType::Cmd)
    }

    /// Get shell-specific syntax hints
    pub fn syntax_hints(&self) -> Vec<&'static str> {
        match self {
            EnhancedShellType::Bash | EnhancedShellType::Zsh => vec![
                "Use $VAR or ${VAR} for variable expansion",
                "Use $(cmd) for command substitution",
                "Use && for command chaining",
            ],
            EnhancedShellType::Fish => vec![
                "Use $VAR for variable expansion (no braces)",
                "Use (cmd) for command substitution",
                "Use ; and for command chaining",
            ],
            EnhancedShellType::PowerShell => vec![
                "Use $VAR for variable expansion",
                "Use $(cmd) for command substitution",
                "Use ; or | for command chaining",
                "Cmdlets use Verb-Noun naming",
            ],
            EnhancedShellType::Nu => vec![
                "Use $VAR for variable expansion",
                "Pipelines pass structured data",
                "Use | for command chaining",
            ],
            _ => vec![],
        }
    }
}

impl std::fmt::Display for EnhancedShellType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EnhancedShellType::Bash => write!(f, "bash"),
            EnhancedShellType::Zsh => write!(f, "zsh"),
            EnhancedShellType::Fish => write!(f, "fish"),
            EnhancedShellType::PowerShell => write!(f, "powershell"),
            EnhancedShellType::Cmd => write!(f, "cmd"),
            EnhancedShellType::Elvish => write!(f, "elvish"),
            EnhancedShellType::Ion => write!(f, "ion"),
            EnhancedShellType::Tcsh => write!(f, "tcsh"),
            EnhancedShellType::Nu => write!(f, "nu"),
            EnhancedShellType::Xonsh => write!(f, "xonsh"),
            EnhancedShellType::Unknown => write!(f, "unknown"),
        }
    }
}

/// Comprehensive context collected from starship
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StarshipContext {
    /// Current working directory
    pub current_dir: PathBuf,

    /// Logical directory (may differ from current_dir in some shells)
    pub logical_dir: PathBuf,

    /// Detected shell type
    pub shell: EnhancedShellType,

    /// Terminal width in columns
    pub terminal_width: usize,

    /// Git repository context
    pub git: GitContext,

    /// Detected project type
    pub project_type: ProjectType,

    /// Notable files in current directory
    pub notable_files: Vec<String>,

    /// Relevant environment variables (non-sensitive)
    pub environment: HashMap<String, String>,
}

impl StarshipContext {
    /// Create a new StarshipContext by detecting the current environment
    pub fn detect() -> Self {
        Self::detect_with_shell(None)
    }

    /// Create a new StarshipContext with a specific shell override
    pub fn detect_with_shell(shell_override: Option<EnhancedShellType>) -> Self {
        let properties = Properties::default();
        let target = Target::Main;

        let shell = shell_override.unwrap_or_else(EnhancedShellType::detect);

        let current_dir = std::env::current_dir().unwrap_or_else(|_| {
            PathBuf::from(std::env::var("HOME").unwrap_or_else(|_| "/".to_string()))
        });
        let logical_dir = current_dir.clone();

        // Create starship context using the simpler constructor
        let ctx = Context::new(properties, target);

        // Extract git information
        let git = Self::extract_git_context(&ctx);

        // Detect project type from directory contents
        let (project_type, notable_files) = Self::detect_project_type(&ctx);

        // Get terminal width from starship context
        let terminal_width = ctx.width;

        // Get safe environment variables
        let environment = Self::filter_environment();

        StarshipContext {
            current_dir,
            logical_dir,
            shell,
            terminal_width,
            git,
            project_type,
            notable_files,
            environment,
        }
    }

    /// Extract git repository information from starship context
    fn extract_git_context(ctx: &Context) -> GitContext {
        let mut git_ctx = GitContext::default();

        if let Ok(repo) = ctx.get_repo() {
            git_ctx.is_git_repo = true;
            git_ctx.branch = repo.branch.clone();
            git_ctx.repo_root = repo.workdir.clone();
            git_ctx.git_dir = Some(repo.path.clone());

            // Extract repository state
            git_ctx.state = repo.state.as_ref().map(|state| format!("{:?}", state));

            // Extract remote information
            if let Some(ref remote) = repo.remote {
                git_ctx.remote_name = remote.name.clone();
                // Note: Remote struct only has name and branch fields, not url
            }
        }

        git_ctx
    }

    /// Detect project type from directory contents using starship's DirContents
    fn detect_project_type(ctx: &Context) -> (ProjectType, Vec<String>) {
        let mut notable_files = Vec::new();

        // Check for project files - dir_contents returns Result
        let dir_contents = match ctx.dir_contents() {
            Ok(contents) => contents,
            Err(_) => return (ProjectType::Generic, notable_files),
        };

        // Rust project
        if dir_contents.has_file_name("Cargo.toml") {
            notable_files.push("Cargo.toml".to_string());
            return (ProjectType::Rust, notable_files);
        }

        // Node.js project
        if dir_contents.has_file_name("package.json") {
            notable_files.push("package.json".to_string());
            if dir_contents.has_file_name("package-lock.json") {
                notable_files.push("package-lock.json".to_string());
            }
            if dir_contents.has_file_name("yarn.lock") {
                notable_files.push("yarn.lock".to_string());
            }
            return (ProjectType::Node, notable_files);
        }

        // Python project
        if dir_contents.has_file_name("pyproject.toml") {
            notable_files.push("pyproject.toml".to_string());
            return (ProjectType::Python, notable_files);
        }
        if dir_contents.has_file_name("setup.py") {
            notable_files.push("setup.py".to_string());
            return (ProjectType::Python, notable_files);
        }
        if dir_contents.has_file_name("requirements.txt") {
            notable_files.push("requirements.txt".to_string());
            return (ProjectType::Python, notable_files);
        }

        // Go project
        if dir_contents.has_file_name("go.mod") {
            notable_files.push("go.mod".to_string());
            return (ProjectType::Go, notable_files);
        }

        // Maven project
        if dir_contents.has_file_name("pom.xml") {
            notable_files.push("pom.xml".to_string());
            return (ProjectType::Maven, notable_files);
        }

        // Gradle project
        if dir_contents.has_file_name("build.gradle")
            || dir_contents.has_file_name("build.gradle.kts")
        {
            notable_files.push("build.gradle".to_string());
            return (ProjectType::Gradle, notable_files);
        }

        // Ruby project
        if dir_contents.has_file_name("Gemfile") {
            notable_files.push("Gemfile".to_string());
            return (ProjectType::Ruby, notable_files);
        }

        // Docker project
        if dir_contents.has_file_name("Dockerfile") {
            notable_files.push("Dockerfile".to_string());
            if dir_contents.has_file_name("docker-compose.yml")
                || dir_contents.has_file_name("docker-compose.yaml")
            {
                notable_files.push("docker-compose.yml".to_string());
            }
            return (ProjectType::Docker, notable_files);
        }

        // Terraform project
        if dir_contents.has_extension("tf") {
            notable_files.push("*.tf".to_string());
            return (ProjectType::Terraform, notable_files);
        }

        // Check for common files
        if dir_contents.has_file_name("Makefile") {
            notable_files.push("Makefile".to_string());
        }
        if dir_contents.has_file_name("README.md") {
            notable_files.push("README.md".to_string());
        }
        if dir_contents.has_file_name(".gitignore") {
            notable_files.push(".gitignore".to_string());
        }

        (ProjectType::Generic, notable_files)
    }

    /// Filter environment variables to exclude sensitive data
    fn filter_environment() -> HashMap<String, String> {
        let sensitive_patterns = [
            "API_KEY",
            "TOKEN",
            "SECRET",
            "PASSWORD",
            "PASSWD",
            "CREDENTIAL",
            "AUTH",
            "PRIVATE",
            "KEY",
            "AWS_",
            "GITHUB_",
            "GITLAB_",
            "ANTHROPIC_",
            "OPENAI_",
        ];

        // List of useful environment variables to include
        let useful_vars = [
            "HOME",
            "USER",
            "SHELL",
            "PATH",
            "PWD",
            "TERM",
            "LANG",
            "EDITOR",
            "VISUAL",
            "PAGER",
            "HOSTNAME",
            "DISPLAY",
            "XDG_CONFIG_HOME",
            "XDG_DATA_HOME",
            "XDG_CACHE_HOME",
            "CARGO_HOME",
            "RUSTUP_HOME",
            "GOPATH",
            "GOROOT",
            "PYENV_ROOT",
            "NVM_DIR",
            "VIRTUAL_ENV",
            "CONDA_PREFIX",
        ];

        std::env::vars()
            .filter(|(key, value)| {
                // Only include useful variables
                if !useful_vars.contains(&key.as_str()) {
                    return false;
                }
                // Filter out sensitive variables and empty values
                !key.is_empty()
                    && !value.is_empty()
                    && !sensitive_patterns
                        .iter()
                        .any(|pattern| key.to_uppercase().contains(pattern))
            })
            .collect()
    }

    /// Generate a comprehensive prompt context string for LLM
    pub fn to_prompt_context(&self) -> String {
        let mut sections = Vec::new();

        // Basic environment info
        sections.push(format!(
            "ENVIRONMENT:\n- Shell: {}\n- Directory: {}\n- Terminal Width: {} columns",
            self.shell,
            self.current_dir.display(),
            self.terminal_width
        ));

        // Project context
        if self.project_type != ProjectType::Generic {
            let mut project_section = format!("\nPROJECT TYPE: {}", self.project_type);
            let hints = self.project_type.command_hints();
            if !hints.is_empty() {
                project_section.push_str("\nCommon commands:");
                for hint in hints.iter().take(3) {
                    project_section.push_str(&format!("\n  - {}", hint));
                }
            }
            sections.push(project_section);
        }

        // Notable files
        if !self.notable_files.is_empty() {
            sections.push(format!(
                "\nNOTABLE FILES: {}",
                self.notable_files.join(", ")
            ));
        }

        // Git context
        if self.git.is_git_repo {
            sections.push(format!("\n{}", self.git.to_prompt_summary()));
        }

        // Shell-specific hints
        let shell_hints = self.shell.syntax_hints();
        if !shell_hints.is_empty() {
            let mut shell_section = format!("\nSHELL ({}) NOTES:", self.shell);
            for hint in shell_hints.iter().take(2) {
                shell_section.push_str(&format!("\n  - {}", hint));
            }
            sections.push(shell_section);
        }

        sections.join("\n")
    }

    /// Get just the git context summary (for brief context)
    pub fn git_summary(&self) -> String {
        self.git.to_prompt_summary()
    }

    /// Check if we're in a git repository
    pub fn is_git_repo(&self) -> bool {
        self.git.is_git_repo
    }

    /// Get the current git branch
    pub fn git_branch(&self) -> Option<&str> {
        self.git.branch.as_deref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_context() {
        let ctx = StarshipContext::detect();

        // Basic sanity checks
        assert!(!ctx.current_dir.as_os_str().is_empty());
        // Terminal width may be 0 in CI
    }

    #[test]
    fn test_git_context_prompt_summary() {
        let git = GitContext {
            is_git_repo: true,
            branch: Some("main".to_string()),
            repo_root: Some(PathBuf::from("/home/user/project")),
            git_dir: Some(PathBuf::from("/home/user/project/.git")),
            state: Some("Clean".to_string()),
            remote_url: Some("https://github.com/user/project.git".to_string()),
            remote_name: Some("origin".to_string()),
        };

        let summary = git.to_prompt_summary();
        assert!(summary.contains("branch: main"));
        assert!(summary.contains("remote: origin"));
    }

    #[test]
    fn test_project_type_hints() {
        let rust_hints = ProjectType::Rust.command_hints();
        assert!(!rust_hints.is_empty());
        assert!(rust_hints.iter().any(|h| h.contains("cargo")));

        let node_hints = ProjectType::Node.command_hints();
        assert!(!node_hints.is_empty());
        assert!(node_hints.iter().any(|h| h.contains("npm")));
    }

    #[test]
    fn test_shell_type_conversion() {
        let bash = EnhancedShellType::Bash;
        assert!(bash.is_posix());
        assert!(!bash.is_windows());

        let pwsh = EnhancedShellType::PowerShell;
        assert!(!pwsh.is_posix());
        assert!(pwsh.is_windows());
    }

    #[test]
    fn test_to_prompt_context() {
        let ctx = StarshipContext::detect();
        let prompt = ctx.to_prompt_context();

        // Should contain environment section
        assert!(prompt.contains("ENVIRONMENT:"));
        assert!(prompt.contains("Shell:"));
        assert!(prompt.contains("Directory:"));
    }

    #[test]
    fn test_shell_detection() {
        let shell = EnhancedShellType::detect();
        // Should detect something (might be Unknown in CI)
        let _ = shell.to_string();
    }
}
