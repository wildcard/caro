//! Query analysis for determining context needs and ambiguity
//!
//! The analyzer examines user queries to determine:
//! - Whether the query is ambiguous
//! - What type of command is being requested
//! - What context would help improve generation
//! - Whether the query is relative to current directory

use crate::context::ExecutionContext;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Classification of the query type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum QueryClassification {
    /// File system operations (ls, find, mv, cp, rm, etc.)
    FileSystem,
    /// Process management (ps, kill, top, etc.)
    Process,
    /// Network operations (netstat, curl, ping, etc.)
    Network,
    /// Package/build operations (npm, cargo, pip, make, etc.)
    Package,
    /// Version control (git, svn, etc.)
    VersionControl,
    /// System information (uname, df, free, etc.)
    SystemInfo,
    /// Text processing (grep, sed, awk, etc.)
    TextProcessing,
    /// Container operations (docker, kubectl, etc.)
    Container,
    /// General/unknown
    General,
}

/// Types of context that might be needed
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ContextNeed {
    /// Need to see files in current directory
    DirectoryListing,
    /// Need to see file/folder structure
    FileTree,
    /// Need to know available commands/tools
    AvailableTools,
    /// Need to know project type (Node, Rust, Python, etc.)
    ProjectType,
    /// Need OS-specific information
    OsInfo,
    /// Need to know running processes
    ProcessList,
    /// Need to know network state
    NetworkState,
    /// Need git repository status
    GitStatus,
    /// Need environment variables
    EnvironmentVars,
    /// Need to know package manager context
    PackageManager,
}

/// Result of query analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryAnalysis {
    /// Classification of the query type
    pub classification: QueryClassification,

    /// Confidence in the analysis (0.0 to 1.0)
    pub confidence: f64,

    /// Is the query relative to current directory?
    pub directory_relative: bool,

    /// Is the query too ambiguous to proceed?
    pub ambiguity_score: f64,

    /// What context would help?
    pub context_needs: HashSet<ContextNeed>,

    /// Keywords extracted from the query
    pub keywords: Vec<String>,

    /// Detected intent (action + target)
    pub detected_intent: Option<DetectedIntent>,

    /// Flags for specific ambiguity types
    pub ambiguous_target: bool,
    pub ambiguous_action: bool,
    pub needs_tool_context: bool,
}

impl Default for QueryAnalysis {
    fn default() -> Self {
        Self {
            classification: QueryClassification::General,
            confidence: 0.5,
            directory_relative: false,
            ambiguity_score: 0.0,
            context_needs: HashSet::new(),
            keywords: Vec::new(),
            detected_intent: None,
            ambiguous_target: false,
            ambiguous_action: false,
            needs_tool_context: false,
        }
    }
}

impl QueryAnalysis {
    /// Check if the query is too ambiguous to proceed
    pub fn is_too_ambiguous(&self) -> bool {
        self.ambiguity_score > 0.7
    }

    /// Check if query is relative to current directory
    pub fn is_directory_relative(&self) -> bool {
        self.directory_relative
    }

    /// Check if target is ambiguous
    pub fn has_ambiguous_target(&self) -> bool {
        self.ambiguous_target
    }

    /// Check if action is ambiguous
    pub fn has_ambiguous_action(&self) -> bool {
        self.ambiguous_action
    }

    /// Check if tool context is needed
    pub fn needs_tool_context(&self) -> bool {
        self.needs_tool_context
    }
}

/// Detected intent from the query
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectedIntent {
    /// The action to perform (list, find, delete, build, etc.)
    pub action: String,
    /// The target of the action (files, processes, etc.)
    pub target: Option<String>,
    /// Any modifiers (recursive, force, verbose, etc.)
    pub modifiers: Vec<String>,
}

/// Analyzes user queries to determine context needs
pub struct QueryAnalyzer {
    /// Patterns that indicate file system operations
    fs_patterns: Vec<&'static str>,
    /// Patterns that indicate directory-relative operations
    relative_patterns: Vec<&'static str>,
    /// Patterns that indicate build/package operations
    build_patterns: Vec<&'static str>,
    /// Keywords that suggest ambiguity
    ambiguous_keywords: Vec<&'static str>,
}

impl QueryAnalyzer {
    /// Create a new query analyzer
    pub fn new() -> Self {
        Self {
            fs_patterns: vec![
                "file", "files", "folder", "directory", "directories",
                "list", "find", "search", "show", "display", "view",
                "move", "copy", "delete", "remove", "rename",
                "create", "make", "touch", "mkdir",
                "size", "count", "modified", "recent",
            ],
            relative_patterns: vec![
                "here", "current", "this folder", "this directory",
                "in this", "from here", "local",
                ".", "./", "./*",
            ],
            build_patterns: vec![
                "build", "run", "test", "install", "compile",
                "start", "serve", "dev", "development",
                "lint", "format", "check",
                "npm", "cargo", "pip", "make", "yarn", "pnpm",
                "go", "gradle", "maven",
            ],
            ambiguous_keywords: vec![
                "something", "stuff", "things", "it", "them",
                "some", "any", "whatever", "somehow",
            ],
        }
    }

    /// Analyze a query and return analysis results
    pub fn analyze(&self, query: &str, context: &ExecutionContext) -> QueryAnalysis {
        let query_lower = query.to_lowercase();
        let words: Vec<&str> = query_lower.split_whitespace().collect();

        let mut analysis = QueryAnalysis::default();

        // Extract keywords
        analysis.keywords = words.iter().map(|s| s.to_string()).collect();

        // Classify the query
        analysis.classification = self.classify_query(&query_lower);

        // Check if directory-relative
        analysis.directory_relative = self.is_directory_relative(&query_lower, &words);

        // Calculate ambiguity score
        analysis.ambiguity_score = self.calculate_ambiguity(&query_lower, &words);

        // Determine context needs based on classification
        analysis.context_needs = self.determine_context_needs(&analysis, context);

        // Detect specific ambiguities
        analysis.ambiguous_target = self.has_ambiguous_target(&query_lower, &words);
        analysis.ambiguous_action = self.has_ambiguous_action(&words);
        analysis.needs_tool_context = self.needs_tool_context(&query_lower, &analysis.classification);

        // Detect intent
        analysis.detected_intent = self.detect_intent(&query_lower, &words);

        // Calculate confidence
        analysis.confidence = self.calculate_confidence(&analysis);

        analysis
    }

    /// Classify the type of query
    fn classify_query(&self, query: &str) -> QueryClassification {
        // Check for container operations
        if query.contains("docker") || query.contains("container") || query.contains("kubectl") || query.contains("pod") {
            return QueryClassification::Container;
        }

        // Check for version control
        if query.contains("git") || query.contains("commit") || query.contains("branch") || query.contains("merge") {
            return QueryClassification::VersionControl;
        }

        // Check for network operations
        if query.contains("network") || query.contains("port") || query.contains("connection")
            || query.contains("curl") || query.contains("wget") || query.contains("ping")
        {
            return QueryClassification::Network;
        }

        // Check for process operations
        if query.contains("process") || query.contains("kill") || query.contains("running")
            || query.contains("pid") || query.contains("top") || query.contains("memory usage")
        {
            return QueryClassification::Process;
        }

        // Check for package/build operations
        for pattern in &self.build_patterns {
            if query.contains(pattern) {
                return QueryClassification::Package;
            }
        }

        // Check for text processing
        if query.contains("grep") || query.contains("search in") || query.contains("find text")
            || query.contains("replace") || query.contains("sed") || query.contains("awk")
        {
            return QueryClassification::TextProcessing;
        }

        // Check for system info
        if query.contains("disk") || query.contains("memory") || query.contains("cpu")
            || query.contains("system") || query.contains("uptime") || query.contains("os")
        {
            return QueryClassification::SystemInfo;
        }

        // Check for file system operations (most common)
        for pattern in &self.fs_patterns {
            if query.contains(pattern) {
                return QueryClassification::FileSystem;
            }
        }

        QueryClassification::General
    }

    /// Check if query is relative to current directory
    fn is_directory_relative(&self, query: &str, _words: &[&str]) -> bool {
        // Explicit relative patterns
        for pattern in &self.relative_patterns {
            if query.contains(pattern) {
                return true;
            }
        }

        // File system operations without explicit path are typically relative
        let classification = self.classify_query(query);
        if classification == QueryClassification::FileSystem {
            // Check if there's no absolute path mentioned
            let has_absolute_path = query.contains('/') && !query.contains("./");
            let has_home_path = query.contains('~');

            if !has_absolute_path && !has_home_path {
                return true;
            }
        }

        // Build commands are typically relative to project root
        if classification == QueryClassification::Package {
            return true;
        }

        false
    }

    /// Calculate ambiguity score (0.0 to 1.0)
    fn calculate_ambiguity(&self, query: &str, words: &[&str]) -> f64 {
        let mut score: f64 = 0.0;

        // Very short queries are ambiguous
        if words.len() <= 2 {
            score += 0.3;
        }

        // Check for ambiguous keywords
        for keyword in &self.ambiguous_keywords {
            if query.contains(keyword) {
                score += 0.2;
            }
        }

        // Missing clear action verb
        let action_verbs = ["list", "show", "find", "delete", "create", "move", "copy",
                           "run", "build", "install", "search", "get", "set"];
        let has_action = action_verbs.iter().any(|v| query.contains(v));
        if !has_action {
            score += 0.2;
        }

        // Queries with "build" or "run" without context are ambiguous
        if (query.contains("build") || query.contains("run"))
            && !query.contains("npm")
            && !query.contains("cargo")
            && !query.contains("make")
            && !query.contains("go")
        {
            score += 0.3;
        }

        score.min(1.0)
    }

    /// Determine what context would help
    fn determine_context_needs(
        &self,
        analysis: &QueryAnalysis,
        _context: &ExecutionContext,
    ) -> HashSet<ContextNeed> {
        let mut needs = HashSet::new();

        match analysis.classification {
            QueryClassification::FileSystem => {
                if analysis.directory_relative {
                    needs.insert(ContextNeed::DirectoryListing);
                    needs.insert(ContextNeed::FileTree);
                }
            }
            QueryClassification::Package => {
                needs.insert(ContextNeed::ProjectType);
                needs.insert(ContextNeed::PackageManager);
                needs.insert(ContextNeed::DirectoryListing);
            }
            QueryClassification::Process => {
                needs.insert(ContextNeed::ProcessList);
            }
            QueryClassification::Network => {
                needs.insert(ContextNeed::NetworkState);
            }
            QueryClassification::VersionControl => {
                needs.insert(ContextNeed::GitStatus);
            }
            QueryClassification::Container => {
                needs.insert(ContextNeed::AvailableTools);
            }
            _ => {}
        }

        // Always useful to know OS info for platform-specific commands
        if analysis.confidence < 0.7 {
            needs.insert(ContextNeed::OsInfo);
        }

        needs
    }

    /// Check if target is ambiguous
    fn has_ambiguous_target(&self, query: &str, words: &[&str]) -> bool {
        // Queries like "delete files" without specifying which files
        let fs_actions = ["delete", "remove", "move", "copy", "rename"];
        let has_fs_action = fs_actions.iter().any(|a| query.contains(a));

        if has_fs_action {
            // Check if there's a clear target specified
            let has_pattern = query.contains('*') || query.contains("all") || query.contains("matching");
            let has_extension = query.contains(".rs") || query.contains(".js") || query.contains(".py")
                || query.contains(".txt") || query.contains(".log");
            let has_name = query.contains("named") || query.contains("called");

            if !has_pattern && !has_extension && !has_name && words.len() <= 3 {
                return true;
            }
        }

        false
    }

    /// Check if action is ambiguous
    fn has_ambiguous_action(&self, words: &[&str]) -> bool {
        // Single word queries are often ambiguous
        if words.len() == 1 {
            let word = words[0];
            // Some single words are clear commands
            let clear_commands = ["ls", "pwd", "whoami", "date", "uptime", "top", "ps"];
            if !clear_commands.contains(&word) {
                return true;
            }
        }

        false
    }

    /// Check if tool context is needed
    fn needs_tool_context(&self, query: &str, classification: &QueryClassification) -> bool {
        if *classification != QueryClassification::Package {
            return false;
        }

        // Generic build/run commands without explicit tool
        let generic_actions = ["build", "run", "test", "start", "serve", "install", "dev"];
        let explicit_tools = ["npm", "cargo", "pip", "make", "yarn", "pnpm", "go", "gradle", "maven"];

        let has_generic = generic_actions.iter().any(|a| query.contains(a));
        let has_explicit = explicit_tools.iter().any(|t| query.contains(t));

        has_generic && !has_explicit
    }

    /// Detect the intent from the query
    fn detect_intent(&self, query: &str, words: &[&str]) -> Option<DetectedIntent> {
        if words.is_empty() {
            return None;
        }

        // Common action verbs mapping
        let action_map: Vec<(&str, &str)> = vec![
            ("list", "list"),
            ("show", "show"),
            ("display", "show"),
            ("find", "find"),
            ("search", "find"),
            ("delete", "delete"),
            ("remove", "delete"),
            ("rm", "delete"),
            ("create", "create"),
            ("make", "create"),
            ("mkdir", "create"),
            ("touch", "create"),
            ("move", "move"),
            ("mv", "move"),
            ("copy", "copy"),
            ("cp", "copy"),
            ("run", "run"),
            ("execute", "run"),
            ("build", "build"),
            ("compile", "build"),
            ("install", "install"),
            ("test", "test"),
        ];

        let mut action = None;
        for (pattern, normalized) in &action_map {
            if query.contains(pattern) {
                action = Some(normalized.to_string());
                break;
            }
        }

        let action = action?;

        // Try to extract target
        let target = if words.len() > 1 {
            // Look for common target patterns
            let target_indicators = ["files", "directories", "folders", "processes", "ports"];
            for indicator in target_indicators {
                if query.contains(indicator) {
                    return Some(DetectedIntent {
                        action,
                        target: Some(indicator.to_string()),
                        modifiers: Vec::new(),
                    });
                }
            }
            None
        } else {
            None
        };

        // Extract modifiers
        let modifiers: Vec<String> = vec![];
        // TODO: Extract modifiers like "recursive", "force", "verbose"

        Some(DetectedIntent {
            action,
            target,
            modifiers,
        })
    }

    /// Calculate confidence in the analysis
    fn calculate_confidence(&self, analysis: &QueryAnalysis) -> f64 {
        let mut confidence = 0.5;

        // Higher confidence for clear classification
        if analysis.classification != QueryClassification::General {
            confidence += 0.2;
        }

        // Lower confidence for ambiguous queries
        confidence -= analysis.ambiguity_score * 0.3;

        // Higher confidence if intent is detected
        if analysis.detected_intent.is_some() {
            confidence += 0.2;
        }

        // Lower confidence if context is needed
        confidence -= (analysis.context_needs.len() as f64) * 0.05;

        confidence.clamp(0.0, 1.0)
    }
}

impl Default for QueryAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn test_context() -> ExecutionContext {
        ExecutionContext {
            os: "linux".to_string(),
            arch: "x86_64".to_string(),
            os_version: "5.15.0".to_string(),
            distribution: Some("Ubuntu 22.04".to_string()),
            cwd: PathBuf::from("/home/user/project"),
            shell: "bash".to_string(),
            user: "testuser".to_string(),
            available_commands: vec!["ls".to_string(), "find".to_string()],
        }
    }

    #[test]
    fn test_classify_file_system() {
        let analyzer = QueryAnalyzer::new();
        let context = test_context();

        let analysis = analyzer.analyze("list all files", &context);
        assert_eq!(analysis.classification, QueryClassification::FileSystem);
    }

    #[test]
    fn test_classify_package() {
        let analyzer = QueryAnalyzer::new();
        let context = test_context();

        let analysis = analyzer.analyze("run npm install", &context);
        assert_eq!(analysis.classification, QueryClassification::Package);
    }

    #[test]
    fn test_directory_relative() {
        let analyzer = QueryAnalyzer::new();
        let context = test_context();

        let analysis = analyzer.analyze("find files here", &context);
        assert!(analysis.directory_relative);

        let analysis = analyzer.analyze("list files in current directory", &context);
        assert!(analysis.directory_relative);
    }

    #[test]
    fn test_ambiguity_detection() {
        let analyzer = QueryAnalyzer::new();
        let context = test_context();

        // Ambiguous query
        let analysis = analyzer.analyze("build", &context);
        assert!(analysis.ambiguity_score > 0.5);
        assert!(analysis.needs_tool_context);

        // Clear query
        let analysis = analyzer.analyze("list all .rs files recursively", &context);
        assert!(analysis.ambiguity_score < 0.5);
    }

    #[test]
    fn test_context_needs() {
        let analyzer = QueryAnalyzer::new();
        let context = test_context();

        let analysis = analyzer.analyze("find all rust files", &context);
        assert!(analysis.context_needs.contains(&ContextNeed::DirectoryListing));
    }
}
