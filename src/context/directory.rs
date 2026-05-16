//! Directory context detection for improved command generation
//!
//! This module scans the current directory to detect project type,
//! available tools, and relevant files that can help generate
//! more contextually appropriate shell commands.

use std::collections::HashSet;
use std::fs;
use std::path::Path;

/// Aggression level for context gathering and prompt inclusion.
///
/// `Normal` is the default and preserves the existing prompt shape. The other
/// levels let callers tune how much information goes into the LLM prompt:
///
/// - `Minimal` — only the most essential signals (git presence, primary
///   project type). Useful when caro is invoked under an outer agent that
///   already supplies project context, and the goal is to cut prompt tokens.
/// - `Normal` — current behavior: project types, npm scripts, make targets,
///   docker, python package manager, cargo commands.
/// - `Aggressive` — `Normal` plus a small bounded set of top-level code
///   signatures gathered cheaply (e.g. Rust `pub fn` names from `src/`).
///   Caps are deliberately tight (file + signature counts) to avoid prompt
///   bloat — "deeper scan, still compressed."
///
/// Idea-borrowed from rtk-ai/rtk's `rtk read -l <aggressive>` aggression
/// levels; reimplemented in caro's idiom.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ContextLevel {
    /// Smallest possible context payload.
    Minimal,
    /// Current/default behavior — full directory inventory.
    #[default]
    Normal,
    /// `Normal` plus bounded code-signature scan.
    Aggressive,
}

impl ContextLevel {
    /// Parse a level string ("minimal" | "normal" | "aggressive"), case-insensitive.
    pub fn parse(s: &str) -> Option<Self> {
        match s.trim().to_ascii_lowercase().as_str() {
            "minimal" | "min" => Some(Self::Minimal),
            "normal" | "default" => Some(Self::Normal),
            "aggressive" | "agg" | "deep" => Some(Self::Aggressive),
            _ => None,
        }
    }
}

impl std::fmt::Display for ContextLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Minimal => write!(f, "minimal"),
            Self::Normal => write!(f, "normal"),
            Self::Aggressive => write!(f, "aggressive"),
        }
    }
}

impl std::str::FromStr for ContextLevel {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s).ok_or_else(|| format!("Unknown context level: {}", s))
    }
}

/// Caps used by `ContextLevel::Aggressive` to keep prompt growth bounded.
const AGGRESSIVE_MAX_FILES: usize = 8;
const AGGRESSIVE_MAX_SIGNATURES: usize = 24;
const AGGRESSIVE_MAX_FILE_BYTES: u64 = 64 * 1024;

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
    /// Bounded list of code signatures gathered when scanned with
    /// `ContextLevel::Aggressive`. Always empty otherwise.
    pub code_signatures: Vec<String>,
}

impl DirectoryContext {
    /// Scan a directory to detect project context at the default
    /// (`ContextLevel::Normal`) level. Preserves prior behavior.
    pub fn scan(path: &Path) -> Self {
        Self::scan_with_level(path, ContextLevel::Normal)
    }

    /// Scan a directory at a specific [`ContextLevel`].
    ///
    /// `Minimal` returns only git + primary project type detection (cheapest).
    /// `Normal` matches the previous default scan.
    /// `Aggressive` additionally collects a small bounded set of code
    /// signatures from top-level source files, capped by
    /// `AGGRESSIVE_MAX_FILES` and `AGGRESSIVE_MAX_SIGNATURES`.
    pub fn scan_with_level(path: &Path, level: ContextLevel) -> Self {
        let mut ctx = DirectoryContext::default();

        if !path.is_dir() {
            return ctx;
        }

        // Always cheap: git presence.
        ctx.has_git = path.join(".git").exists();

        // Project marker detection (also cheap — just `path.join().exists()`).
        if path.join("package.json").exists() {
            ctx.project_types.insert(ProjectType::NodeJs);
        }
        if path.join("Cargo.toml").exists() {
            ctx.project_types.insert(ProjectType::Rust);
        }
        if path.join("pyproject.toml").exists()
            || path.join("setup.py").exists()
            || path.join("requirements.txt").exists()
        {
            ctx.project_types.insert(ProjectType::Python);
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

        // Normal and Aggressive levels do the existing deeper enrichment.
        if matches!(level, ContextLevel::Normal | ContextLevel::Aggressive) {
            if ctx.project_types.contains(&ProjectType::NodeJs) {
                ctx.npm_scripts = Self::extract_npm_scripts(path);
            }
            if ctx.project_types.contains(&ProjectType::Rust) {
                ctx.cargo_commands = Self::get_cargo_commands();
            }
            if ctx.project_types.contains(&ProjectType::Python) {
                ctx.python_package_manager = Self::detect_python_package_manager(path);
            }

            if path.join("Makefile").exists() || path.join("makefile").exists() {
                ctx.has_makefile = true;
                ctx.make_targets = Self::extract_make_targets(path);
            }
            if path.join("Dockerfile").exists() {
                ctx.has_docker = true;
            }
            if path.join("docker-compose.yml").exists()
                || path.join("docker-compose.yaml").exists()
            {
                ctx.has_docker_compose = true;
            }
        }

        // Aggressive: bounded signature scan on top of Normal data.
        if matches!(level, ContextLevel::Aggressive) {
            ctx.code_signatures = Self::collect_code_signatures(path);
        }

        // If no specific project type detected, mark as generic
        if ctx.project_types.is_empty() {
            ctx.project_types.insert(ProjectType::Generic);
        }

        ctx
    }

    /// Collect a bounded set of code signatures from top-level source files.
    ///
    /// Caps:
    /// - up to `AGGRESSIVE_MAX_FILES` files scanned
    /// - each file capped at `AGGRESSIVE_MAX_FILE_BYTES`
    /// - up to `AGGRESSIVE_MAX_SIGNATURES` signatures total returned
    ///
    /// Currently extracts Rust `pub fn`/`pub struct`/`pub enum` names and
    /// Python `def`/`class` names. Best-effort only — bad UTF-8 / IO errors
    /// are silently skipped so a noisy directory never blocks a query.
    fn collect_code_signatures(path: &Path) -> Vec<String> {
        let mut out = Vec::new();
        let mut files_scanned = 0usize;

        // Candidate top-level directories to look in.
        let candidate_dirs = [path.to_path_buf(), path.join("src")];

        for dir in candidate_dirs.iter() {
            if files_scanned >= AGGRESSIVE_MAX_FILES
                || out.len() >= AGGRESSIVE_MAX_SIGNATURES
            {
                break;
            }
            let Ok(entries) = fs::read_dir(dir) else {
                continue;
            };
            for entry in entries.flatten() {
                if files_scanned >= AGGRESSIVE_MAX_FILES
                    || out.len() >= AGGRESSIVE_MAX_SIGNATURES
                {
                    break;
                }
                let p = entry.path();
                if !p.is_file() {
                    continue;
                }
                let ext = match p.extension().and_then(|e| e.to_str()) {
                    Some(e) => e,
                    None => continue,
                };
                if !matches!(ext, "rs" | "py") {
                    continue;
                }
                let meta = match fs::metadata(&p) {
                    Ok(m) => m,
                    Err(_) => continue,
                };
                if meta.len() > AGGRESSIVE_MAX_FILE_BYTES {
                    continue;
                }
                let content = match fs::read_to_string(&p) {
                    Ok(c) => c,
                    Err(_) => continue,
                };
                files_scanned += 1;

                let fname = p
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("?")
                    .to_string();
                for line in content.lines() {
                    if out.len() >= AGGRESSIVE_MAX_SIGNATURES {
                        break;
                    }
                    let trimmed = line.trim_start();
                    let sig = match ext {
                        "rs" => Self::rust_signature(trimmed),
                        "py" => Self::python_signature(trimmed),
                        _ => None,
                    };
                    if let Some(s) = sig {
                        out.push(format!("{}: {}", fname, s));
                    }
                }
            }
        }
        out
    }

    fn rust_signature(line: &str) -> Option<String> {
        for prefix in [
            "pub fn ",
            "pub async fn ",
            "pub struct ",
            "pub enum ",
            "pub trait ",
        ] {
            if let Some(rest) = line.strip_prefix(prefix) {
                let name: String = rest
                    .chars()
                    .take_while(|c| c.is_alphanumeric() || *c == '_')
                    .collect();
                if !name.is_empty() {
                    return Some(format!("{}{}", prefix.trim_end(), {
                        let mut s = String::from(" ");
                        s.push_str(&name);
                        s
                    }));
                }
            }
        }
        None
    }

    fn python_signature(line: &str) -> Option<String> {
        for prefix in ["def ", "async def ", "class "] {
            if let Some(rest) = line.strip_prefix(prefix) {
                let name: String = rest
                    .chars()
                    .take_while(|c| c.is_alphanumeric() || *c == '_')
                    .collect();
                if !name.is_empty() {
                    return Some(format!("{}{}", prefix.trim_end(), {
                        let mut s = String::from(" ");
                        s.push_str(&name);
                        s
                    }));
                }
            }
        }
        None
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

    /// Convert directory context to a string for LLM prompts using the
    /// `Normal` level (preserves legacy output shape).
    pub fn to_context_string(&self) -> String {
        self.to_context_string_with_level(ContextLevel::Normal)
    }

    /// Convert directory context to a string for LLM prompts, tuned by
    /// [`ContextLevel`]:
    ///
    /// - `Minimal` — only project type and git presence (smallest payload)
    /// - `Normal` — current behavior
    /// - `Aggressive` — adds bounded code-signature listing
    pub fn to_context_string_with_level(&self, level: ContextLevel) -> String {
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

        // Minimal stops here. Other levels include build tools and beyond.
        if matches!(level, ContextLevel::Minimal) {
            return if parts.is_empty() {
                String::new()
            } else {
                format!("Directory context:\n{}", parts.join("\n"))
            };
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

        // Aggressive: bounded signature dump.
        if matches!(level, ContextLevel::Aggressive) && !self.code_signatures.is_empty() {
            parts.push(format!(
                "Top-level signatures ({}):\n  {}",
                self.code_signatures.len(),
                self.code_signatures.join("\n  ")
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
            || !self.code_signatures.is_empty()
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

    // ----- ContextLevel tests --------------------------------------------

    #[test]
    fn test_context_level_parse() {
        assert_eq!(ContextLevel::parse("minimal"), Some(ContextLevel::Minimal));
        assert_eq!(ContextLevel::parse("MIN"), Some(ContextLevel::Minimal));
        assert_eq!(ContextLevel::parse("normal"), Some(ContextLevel::Normal));
        assert_eq!(ContextLevel::parse("default"), Some(ContextLevel::Normal));
        assert_eq!(
            ContextLevel::parse("aggressive"),
            Some(ContextLevel::Aggressive)
        );
        assert_eq!(
            ContextLevel::parse(" Deep "),
            Some(ContextLevel::Aggressive)
        );
        assert_eq!(ContextLevel::parse("garbage"), None);
    }

    #[test]
    fn test_context_level_default_is_normal() {
        assert_eq!(ContextLevel::default(), ContextLevel::Normal);
    }

    #[test]
    fn test_minimal_skips_enrichment() {
        let temp_dir = TempDir::new().unwrap();
        // Rust + Makefile + package.json: Normal would enrich all three.
        File::create(temp_dir.path().join("Cargo.toml")).unwrap();
        File::create(temp_dir.path().join("package.json")).unwrap();
        let mut mk = File::create(temp_dir.path().join("Makefile")).unwrap();
        writeln!(mk, "build:\n\techo hi").unwrap();

        let minimal = DirectoryContext::scan_with_level(temp_dir.path(), ContextLevel::Minimal);
        // Project types still detected (cheap).
        assert!(minimal.project_types.contains(&ProjectType::Rust));
        assert!(minimal.project_types.contains(&ProjectType::NodeJs));
        // But enrichment is skipped.
        assert!(minimal.cargo_commands.is_empty());
        assert!(minimal.npm_scripts.is_empty());
        assert!(!minimal.has_makefile);
        assert!(minimal.make_targets.is_empty());
        // Output is correspondingly trimmed.
        let s = minimal.to_context_string_with_level(ContextLevel::Minimal);
        assert!(!s.contains("Cargo commands"));
        assert!(!s.contains("NPM scripts"));
        assert!(!s.contains("Make targets"));
    }

    #[test]
    fn test_normal_matches_legacy_scan() {
        let temp_dir = TempDir::new().unwrap();
        File::create(temp_dir.path().join("Cargo.toml")).unwrap();
        let legacy = DirectoryContext::scan(temp_dir.path());
        let normal = DirectoryContext::scan_with_level(temp_dir.path(), ContextLevel::Normal);
        assert_eq!(legacy.project_types, normal.project_types);
        assert_eq!(legacy.cargo_commands, normal.cargo_commands);
        assert_eq!(legacy.code_signatures, normal.code_signatures);
        assert!(legacy.code_signatures.is_empty());
    }

    #[test]
    fn test_aggressive_collects_rust_signatures() {
        let temp_dir = TempDir::new().unwrap();
        File::create(temp_dir.path().join("Cargo.toml")).unwrap();
        let mut f = File::create(temp_dir.path().join("lib.rs")).unwrap();
        writeln!(
            f,
            "pub fn alpha() {{}}\nfn private_skip() {{}}\npub struct Beta {{}}\npub async fn gamma() {{}}\npub trait Delta {{}}\n"
        )
        .unwrap();

        let agg = DirectoryContext::scan_with_level(temp_dir.path(), ContextLevel::Aggressive);
        let joined = agg.code_signatures.join("\n");
        assert!(joined.contains("alpha"));
        assert!(joined.contains("Beta"));
        assert!(joined.contains("gamma"));
        assert!(joined.contains("Delta"));
        // private items must NOT leak.
        assert!(!joined.contains("private_skip"));
    }

    #[test]
    fn test_aggressive_respects_signature_cap() {
        let temp_dir = TempDir::new().unwrap();
        File::create(temp_dir.path().join("Cargo.toml")).unwrap();
        let mut f = File::create(temp_dir.path().join("lib.rs")).unwrap();
        for i in 0..200 {
            writeln!(f, "pub fn item_{}() {{}}", i).unwrap();
        }

        let agg = DirectoryContext::scan_with_level(temp_dir.path(), ContextLevel::Aggressive);
        assert!(agg.code_signatures.len() <= AGGRESSIVE_MAX_SIGNATURES);
    }

    #[test]
    fn test_aggressive_to_context_string_includes_signatures() {
        let temp_dir = TempDir::new().unwrap();
        File::create(temp_dir.path().join("Cargo.toml")).unwrap();
        let mut f = File::create(temp_dir.path().join("lib.rs")).unwrap();
        writeln!(f, "pub fn observable() {{}}").unwrap();

        let agg = DirectoryContext::scan_with_level(temp_dir.path(), ContextLevel::Aggressive);
        let s = agg.to_context_string_with_level(ContextLevel::Aggressive);
        assert!(s.contains("observable"));
        assert!(s.contains("Top-level signatures"));
    }
}
