//! Project context detection module
//!
//! This module detects project-specific information from the current directory,
//! including package managers, build tools, and programming languages.
//!
//! Detection is based on common lock files, configuration files, and project
//! structure patterns - similar to how Starship detects project context for
//! its status line.

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

    /// Project root path
    pub root_path: PathBuf,

    /// Lock files and config files found
    pub detected_files: Vec<String>,
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
        let files = Self::scan_project_files(cwd)?;
        context.detected_files = files.clone();

        // Detect package manager from lock files (priority order)
        context.package_manager = Self::detect_package_manager(&files);

        // Detect build tool
        context.build_tool = Self::detect_build_tool(&files);

        // Detect languages
        context.languages = Self::detect_languages(&files);

        debug!(
            "Detected: pm={:?}, build={:?}, langs={:?}",
            context.package_manager, context.build_tool, context.languages
        );

        Ok(context)
    }

    /// Scan directory for project-related files
    fn scan_project_files(cwd: &Path) -> Result<Vec<String>, std::io::Error> {
        let mut files = Vec::new();

        // Files to look for (order matters for priority)
        let target_files = [
            // Package managers
            "yarn.lock",
            "package-lock.json",
            "pnpm-lock.yaml",
            "bun.lockb",
            "package.json",
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
            // Ruby
            "Gemfile.lock",
            "Gemfile",
            // Build tools
            "Makefile",
            "CMakeLists.txt",
            "build.gradle",
            "build.gradle.kts",
            "pom.xml",
            "BUILD.bazel",
            "BUILD",
            // Docker
            "Dockerfile",
            "docker-compose.yml",
            "docker-compose.yaml",
            "compose.yml",
            "compose.yaml",
            // CI/CD
            ".github/workflows",
            ".gitlab-ci.yml",
            "Jenkinsfile",
            // TypeScript/JavaScript
            "tsconfig.json",
            "jsconfig.json",
            ".eslintrc.json",
            ".prettierrc",
            // Misc
            ".editorconfig",
            ".gitignore",
        ];

        for filename in target_files {
            let path = cwd.join(filename);
            if path.exists() {
                trace!("Found: {}", filename);
                files.push(filename.to_string());
            }
        }

        Ok(files)
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

        languages
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
