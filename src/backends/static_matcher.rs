//! Static Pattern Matcher Backend
//!
//! Provides deterministic command generation for known website-advertised examples.
//! This backend matches natural language patterns to exact shell commands,
//! ensuring consistent output for documented use cases.
//!
//! The static matcher runs BEFORE LLM backends, providing instant, predictable
//! results for common queries advertised on the website.

use async_trait::async_trait;
use regex::Regex;
use std::sync::Arc;

use crate::backends::{BackendInfo, CommandGenerator, GeneratorError};
use crate::models::{BackendType, CommandRequest, GeneratedCommand, RiskLevel, ShellType};
use crate::prompts::CapabilityProfile;
use crate::safety::{SafetyConfig, SafetyValidator};

/// Static pattern matcher for deterministic command generation
#[derive(Clone)]
pub struct StaticMatcher {
    patterns: Arc<Vec<PatternEntry>>,
    profile: CapabilityProfile,
    safety_validator: Arc<SafetyValidator>,
}

/// A single pattern entry mapping natural language to shell command
#[derive(Debug, Clone)]
struct PatternEntry {
    /// Keywords that must be present (all required)
    required_keywords: Vec<String>,
    /// Keywords that boost match confidence
    optional_keywords: Vec<String>,
    /// Regex pattern for more precise matching (optional)
    regex_pattern: Option<Regex>,
    /// Command for GNU/Linux systems
    gnu_command: String,
    /// Command for BSD systems (macOS)
    bsd_command: Option<String>,
    /// Description for debugging
    description: String,
}

impl StaticMatcher {
    /// Create a new static matcher with detected capabilities
    pub fn new(profile: CapabilityProfile) -> Self {
        // Initialize safety validator with moderate config
        // This will panic on invalid configuration, which is acceptable for initialization
        let safety_validator = Arc::new(
            SafetyValidator::new(SafetyConfig::moderate())
                .expect("Failed to initialize SafetyValidator with default config"),
        );

        Self {
            patterns: Arc::new(Self::build_patterns()),
            profile,
            safety_validator,
        }
    }

    /// Build the pattern library from website-advertised examples
    fn build_patterns() -> Vec<PatternEntry> {
        vec![
            // Pattern 1: "find all Python files modified today" (SPECIFIC - moved from Pattern 46)
            PatternEntry {
                required_keywords: vec!["python".to_string(), "modified".to_string(), "today".to_string()],
                optional_keywords: vec!["find".to_string(), "all".to_string(), "files".to_string()],
                regex_pattern: Some(Regex::new(r"(?i)(find|locate|search).*(all)?.*(python|\.py).*(files?).*(modified|changed).*(today)").unwrap()),
                gnu_command: r#"find . -name "*.py" -type f -mtime 0"#.to_string(),
                bsd_command: Some(r#"find . -name "*.py" -type f -mtime 0"#.to_string()),
                description: "Find Python files modified today".to_string(),
            },

            // Pattern 2: "list all files modified today" (GENERAL - was Pattern 1)
            PatternEntry {
                required_keywords: vec!["file".to_string(), "today".to_string()],
                optional_keywords: vec!["list".to_string(), "all".to_string(), "modified".to_string(), "changed".to_string()],
                regex_pattern: Some(Regex::new(r"(?i)(list|show|find|get|files?).*(modified|changed|updated).*(today|last 24 hours?)").unwrap()),
                gnu_command: "find . -type f -mtime 0".to_string(),
                bsd_command: Some("find . -type f -mtime 0".to_string()),
                description: "List files modified today".to_string(),
            },

            // Pattern 2a: "files modified yesterday" (Cycle 1 - Edge Case)
            PatternEntry {
                required_keywords: vec!["file".to_string(), "yesterday".to_string()],
                optional_keywords: vec!["list".to_string(), "all".to_string(), "find".to_string(), "modified".to_string(), "changed".to_string()],
                regex_pattern: Some(Regex::new(r"(?i)(list|show|find|get|files?).*(modified|changed|updated).*(yesterday)").unwrap()),
                gnu_command: "find . -type f -mtime 1".to_string(),
                bsd_command: Some("find . -type f -mtime 1".to_string()),
                description: "List files modified yesterday".to_string(),
            },

            // Pattern 2: "find large files over 100MB"
            PatternEntry {
                required_keywords: vec!["file".to_string(), "100".to_string()],
                optional_keywords: vec!["find".to_string(), "over".to_string(), "mb".to_string(), "large".to_string(), "big".to_string(), "bigger".to_string()],
                regex_pattern: Some(Regex::new(r"(?i)(find|locate|show|list).*(large|big|bigger).*(files?).*(over|above|bigger|greater|than).*(100|100mb|100m|megabyte)").unwrap()),
                gnu_command: "find . -type f -size +100M".to_string(),
                bsd_command: Some("find . -type f -size +100M".to_string()),
                description: "Find large files over 100MB".to_string(),
            },

            // Pattern 3: "show me disk usage/space by directory, sorted" (SPECIFIC - moved from Pattern 48)
            // Must come BEFORE Pattern 2a to match first when "sorted" is present
            // "usage" is optional to also match "disk space" phrasing
            PatternEntry {
                required_keywords: vec!["disk".to_string(), "directory".to_string(), "sorted".to_string()],
                optional_keywords: vec!["show".to_string(), "by".to_string(), "usage".to_string(), "space".to_string()],
                regex_pattern: Some(Regex::new(r"(?i)(show|display|list).*(me)?.*(disk|space).*(usage|use).*(by)?.*(directory|dir|folder).*(sorted|sort)").unwrap()),
                gnu_command: "du -h --max-depth=1 | sort -hr".to_string(),
                bsd_command: Some("du -h -d 1 | sort -hr".to_string()),
                description: "Show disk usage by directory, sorted".to_string(),
            },

            // Pattern 2a: "show disk space by directory" (SIMPLE - without "sorted" requirement)
            // Fixes Issue #406 - handles common query without requiring "sorted" keyword
            // Placed AFTER Pattern 3 so specific "sorted" pattern matches first
            // Regex only matches "directory|dir" (NOT "folder") to avoid conflicting with Pattern 4
            PatternEntry {
                required_keywords: vec!["disk".to_string(), "space".to_string(), "directory".to_string()],
                optional_keywords: vec!["show".to_string(), "by".to_string(), "usage".to_string()],
                regex_pattern: Some(Regex::new(r"(?i)(show|display|list|get).*(disk|storage).*(space|usage).*(by)?.*(directory|directories|dir)").unwrap()),
                gnu_command: "du -h --max-depth=1".to_string(),
                bsd_command: Some("du -h -d 1".to_string()),
                description: "Show disk space by directory".to_string(),
            },

            // Pattern 4: "show disk usage by folder" (GENERAL - was Pattern 3)
            PatternEntry {
                required_keywords: vec!["disk".to_string(), "folder".to_string()],
                optional_keywords: vec!["show".to_string(), "display".to_string(), "by".to_string(), "usage".to_string(), "space".to_string(), "used".to_string(), "each".to_string()],
                regex_pattern: Some(Regex::new(r"(?i)(show|display|list|get|disk).*(disk|space).*(usage|size|used).*(by |per |each )?(folder|director)").unwrap()),
                gnu_command: "du -sh */ | sort -rh | head -10".to_string(),
                bsd_command: Some("du -sh */ | sort -rh | head -10".to_string()),
                description: "Show disk usage by folder".to_string(),
            },

            // Pattern 4: "find python files modified/from last week"
            // Fixes Issue #406 - updated regex to handle "from" in addition to "modified"
            PatternEntry {
                required_keywords: vec!["python".to_string(), "file".to_string(), "week".to_string()],
                optional_keywords: vec!["find".to_string(), "last".to_string(), "modified".to_string(), "from".to_string()],
                regex_pattern: Some(Regex::new(r"(?i)(find|locate|list|show).*(python|\.py).*(files?).*(modified|changed|updated|from).*(last week|past week)").unwrap()),
                gnu_command: "find . -name \"*.py\" -type f -mtime -7".to_string(),
                bsd_command: Some("find . -name \"*.py\" -type f -mtime -7".to_string()),
                description: "Find Python files modified last week".to_string(),
            },

            // ===== FILE SIZE PATTERNS (Cycle 1 Priority 1) =====

            // Pattern 5: "find all PDF files larger than 10MB in Downloads" (SPECIFIC - moved from Pattern 41)
            PatternEntry {
                required_keywords: vec!["pdf".to_string(), "downloads".to_string()],
                optional_keywords: vec!["find".to_string(), "all".to_string(), "files".to_string(), "10".to_string(), "mb".to_string(), "larger".to_string()],
                regex_pattern: Some(Regex::new(r"(?i)(find|locate|search).*(all)?.*(pdf).*(files?).*(larger|bigger|over).*(10|10mb|10m).*(in|from)?.*(downloads|~/downloads)").unwrap()),
                gnu_command: r#"find ~/Downloads -name "*.pdf" -size +10M -ls"#.to_string(),
                bsd_command: Some(r#"find ~/Downloads -name "*.pdf" -size +10M -ls"#.to_string()),
                description: "Find PDF files larger than 10MB in Downloads".to_string(),
            },

            // Pattern 6: "find files larger than 10MB" (GENERAL - was Pattern 5)
            PatternEntry {
                required_keywords: vec!["file".to_string(), "10".to_string()],
                optional_keywords: vec!["find".to_string(), "larger".to_string(), "bigger".to_string(), "mb".to_string()],
                regex_pattern: Some(Regex::new(r"(?i)(find|locate|list|show).*(files?).*(larger|bigger|over|above|greater).*(10|10mb|10m)").unwrap()),
                gnu_command: "find . -type f -size +10M".to_string(),
                bsd_command: Some("find . -type f -size +10M".to_string()),
                description: "Find files larger than 10MB".to_string(),
            },

            // Pattern 6: "Find all files larger than 1GB" with exec (SPECIFIC - moved from Pattern 40)
            PatternEntry {
                required_keywords: vec!["find".to_string(), "all".to_string(), "file".to_string(), "larger".to_string(), "1gb".to_string()],
                optional_keywords: vec![],
                regex_pattern: Some(Regex::new(r"(?i)^find\s+all\s+files?\s+(larger|bigger|over|above|greater).*1\s*(gb?|g)").unwrap()),
                gnu_command: "find . -type f -size +1G -exec ls -lh {} \\;".to_string(),
                bsd_command: Some("find . -type f -size +1G -exec ls -lh {} \\;".to_string()),
                description: "Find files larger than 1GB with exec".to_string(),
            },

            // Pattern 7: "find files larger than 1GB" (GENERAL - was Pattern 6)
            PatternEntry {
                required_keywords: vec!["file".to_string(), "1".to_string()],
                optional_keywords: vec!["find".to_string(), "larger".to_string(), "gb".to_string()],
                regex_pattern: Some(Regex::new(r"(?i)(find|locate|list|show).*(files?).*(larger|bigger|over|above|greater).*(1|1gb|1g)").unwrap()),
                gnu_command: "find . -type f -size +1G".to_string(),
                bsd_command: Some("find . -type f -size +1G".to_string()),
                description: "Find files larger than 1GB".to_string(),
            },

            // Pattern 7a: "large javascript files over 50MB" (SPECIFIC - Cycle 1)
            PatternEntry {
                required_keywords: vec!["javascript".to_string(), "50".to_string()],
                optional_keywords: vec!["large".to_string(), "files".to_string(), "over".to_string(), "mb".to_string()],
                regex_pattern: Some(Regex::new(r"(?i)(find|locate|list|show).*(large|big)?.*(javascript|\.js|js).*(files?).*(over|above|bigger|greater).*(50|50mb|50m)").unwrap()),
                gnu_command: r#"find . -name "*.js" -type f -size +50M"#.to_string(),
                bsd_command: Some(r#"find . -name "*.js" -type f -size +50M"#.to_string()),
                description: "Find large JavaScript files over 50MB".to_string(),
            },

            // Pattern 8: "find files larger than 50MB" (GENERAL - was Pattern 7)
            PatternEntry {
                required_keywords: vec!["file".to_string(), "50".to_string()],
                optional_keywords: vec!["find".to_string(), "larger".to_string(), "mb".to_string()],
                regex_pattern: Some(Regex::new(r"(?i)(find|locate|list|show).*(files?).*(larger|bigger|over|above|greater).*(50|50mb|50m)").unwrap()),
                gnu_command: "find . -type f -size +50M".to_string(),
                bsd_command: Some("find . -type f -size +50M".to_string()),
                description: "Find files larger than 50MB".to_string(),
            },

            // Pattern 8: "find files larger than 500MB"
            PatternEntry {
                required_keywords: vec!["file".to_string(), "500".to_string()],
                optional_keywords: vec!["find".to_string(), "larger".to_string(), "mb".to_string()],
                regex_pattern: Some(Regex::new(r"(?i)(find|locate|list|show).*(files?).*(larger|bigger|over|above|greater).*(500|500mb|500m)").unwrap()),
                gnu_command: "find . -type f -size +500M".to_string(),
                bsd_command: Some("find . -type f -size +500M".to_string()),
                description: "Find files larger than 500MB".to_string(),
            },

            // ===== TIME FILTER PATTERNS - MINUTES (Cycle 1 Priority 3) =====

            // Pattern 9: "find files changed in last hour"
            PatternEntry {
                required_keywords: vec!["file".to_string(), "hour".to_string()],
                optional_keywords: vec!["find".to_string(), "changed".to_string(), "modified".to_string(), "last".to_string()],
                regex_pattern: Some(Regex::new(r"(?i)(find|locate|list|show).*(files?).*(changed|modified|updated).*(last|past).*(hour|60 min)").unwrap()),
                gnu_command: "find . -type f -mmin -60".to_string(),
                bsd_command: Some("find . -type f -mmin -60".to_string()),
                description: "Find files modified in the last hour".to_string(),
            },

            // Pattern 10: "find python files modified in the last 7 days" (SPECIFIC - moved from Pattern 42)
            PatternEntry {
                required_keywords: vec!["python".to_string(), "7".to_string()],
                optional_keywords: vec!["find".to_string(), "files".to_string(), "last".to_string(), "days".to_string(), "modified".to_string(), "from".to_string()],
                regex_pattern: Some(Regex::new(r"(?i)(find|locate|search|python).*(python|py|\.py).*(files?).*(modified|changed|from).*(in)?.*(the)?.*(last)?.*(7|seven).*(days?)").unwrap(),),
                gnu_command: r#"find . -name "*.py" -type f -mtime -7"#.to_string(),
                bsd_command: Some(r#"find . -name "*.py" -type f -mtime -7"#.to_string()),
                description: "Find Python files modified in last 7 days".to_string(),
            },

            // Pattern 11: "find files modified in last 7 days" (GENERAL - was Pattern 10)
            PatternEntry {
                required_keywords: vec!["file".to_string(), "7".to_string()],
                optional_keywords: vec!["find".to_string(), "modified".to_string(), "days".to_string()],
                regex_pattern: Some(Regex::new(r"(?i)(find|locate|list|show).*(files?).*(modified|changed|updated).*(last|past).*(7|seven).*days?").unwrap()),
                gnu_command: "find . -type f -mtime -7".to_string(),
                bsd_command: Some("find . -type f -mtime -7".to_string()),
                description: "Find files modified in the last 7 days".to_string(),
            },

            // ===== EXTENSION + TIME PATTERNS (Cycle 1 Priority 2) =====

            // Pattern 11: "find PNG images modified in last 7 days"
            PatternEntry {
                required_keywords: vec!["png".to_string(), "7".to_string()],
                optional_keywords: vec!["find".to_string(), "image".to_string(), "modified".to_string(), "days".to_string()],
                regex_pattern: Some(Regex::new(r"(?i)(find|locate|list|show).*(png|\.png).*(image|file)s?.*(modified|changed|updated).*(last|past).*(7|seven).*days?").unwrap()),
                gnu_command: "find . -name '*.png' -type f -mtime -7".to_string(),
                bsd_command: Some("find . -name '*.png' -type f -mtime -7".to_string()),
                description: "Find PNG images modified in the last 7 days".to_string(),
            },

            // ===== PROCESS MONITORING PATTERNS (Cycle 1 Priority 4) =====

            // Pattern 12: "show top 10 memory-consuming processes"
            PatternEntry {
                required_keywords: vec!["process".to_string(), "memory".to_string()],
                optional_keywords: vec!["top".to_string(), "10".to_string(), "consuming".to_string()],
                regex_pattern: Some(Regex::new(r"(?i)(show|display|list|find).*(top|most).*(memory|mem|ram).*(consuming|using|hogging).*process").unwrap()),
                gnu_command: "ps aux --sort=-%mem | head -n 11".to_string(),
                bsd_command: Some("ps aux -m | head -n 11".to_string()),
                description: "Show top memory-consuming processes".to_string(),
            },

            // Pattern 13: "check which process is using port 8080"
            PatternEntry {
                required_keywords: vec!["process".to_string(), "port".to_string()],
                optional_keywords: vec!["check".to_string(), "using".to_string(), "8080".to_string()],
                regex_pattern: Some(Regex::new(r"(?i)(check|find|show|which).*(process|program|service).*(using|listening|on).*(port|:)\s*\d+").unwrap()),
                gnu_command: "lsof -i :8080".to_string(),
                bsd_command: Some("lsof -i :8080".to_string()),
                description: "Check which process is using a port".to_string(),
            },

            // ===== TEXT PROCESSING PATTERNS (Cycle 2 Priority 1) =====

            // Pattern 14: "find all python files that import requests library"
            PatternEntry {
                required_keywords: vec!["python".to_string(), "import".to_string()],
                optional_keywords: vec!["find".to_string(), "files".to_string(), "requests".to_string()],
                regex_pattern: Some(Regex::new(r"(?i)(find|search|grep|locate).*(python|\.py).*(files?).*(import|importing).*requests").unwrap()),
                gnu_command: "grep -r 'import requests' --include='*.py'".to_string(),
                bsd_command: Some("grep -r 'import requests' --include='*.py'".to_string()),
                description: "Find Python files importing requests library".to_string(),
            },

            // Pattern 15: "Extract unique email addresses from a file"
            PatternEntry {
                required_keywords: vec!["email".to_string(), "extract".to_string()],
                optional_keywords: vec!["addresses".to_string(), "unique".to_string(), "file".to_string()],
                regex_pattern: Some(Regex::new(r"(?i)(extract|find|get|list).*(unique|all)?.*(email|e-mail).*(addresses?|addrs?)").unwrap()),
                gnu_command: r#"grep -Eo '\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Z|a-z]{2,}\b' file.txt | sort -u"#.to_string(),
                bsd_command: Some(r#"grep -Eo '\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Z|a-z]{2,}\b' file.txt | sort -u"#.to_string()),
                description: "Extract unique email addresses from a file".to_string(),
            },

            // Pattern 16: "Replace all occurrences in multiple files"
            PatternEntry {
                required_keywords: vec!["replace".to_string(), "files".to_string()],
                optional_keywords: vec!["all".to_string(), "occurrences".to_string(), "multiple".to_string()],
                regex_pattern: Some(Regex::new(r"(?i)(replace|substitute|change).*(all|every).*(occurrences?|instances?).*(multiple|many|several).*(files?)").unwrap()),
                gnu_command: "sed -i 's/old_text/new_text/g' *.txt".to_string(),
                bsd_command: Some("sed -i '' 's/old_text/new_text/g' *.txt".to_string()),
                description: "Replace text in multiple files".to_string(),
            },

            // Pattern 17: "compress this directory for transfer"
            PatternEntry {
                required_keywords: vec!["compress".to_string(), "directory".to_string()],
                optional_keywords: vec!["tar".to_string(), "transfer".to_string(), "archive".to_string()],
                regex_pattern: Some(Regex::new(r"(?i)(compress|archive|tar|zip).*(this|the)?.*(directory|folder|dir)").unwrap()),
                gnu_command: "tar -czf archive.tar.gz directory/".to_string(),
                bsd_command: Some("tar -czf archive.tar.gz directory/".to_string()),
                description: "Compress directory for transfer".to_string(),
            },

            // ===== GIT VERSION CONTROL PATTERNS (Cycle 2 Priority 2) =====

            // Pattern 18: "Show commits from the last week"
            PatternEntry {
                required_keywords: vec!["commits".to_string(), "week".to_string()],
                optional_keywords: vec!["show".to_string(), "last".to_string(), "git".to_string()],
                regex_pattern: Some(Regex::new(r"(?i)(show|list|display|get|find).*(commits?|changes?).*(from|in|during).*(last|past).*(week|7 days?)").unwrap()),
                gnu_command: "git log --since='1 week ago' --oneline".to_string(),
                bsd_command: Some("git log --since='1 week ago' --oneline".to_string()),
                description: "Show commits from the last week".to_string(),
            },

            // Pattern 19: "List all branches sorted by last commit date"
            PatternEntry {
                required_keywords: vec!["branches".to_string(), "sorted".to_string()],
                optional_keywords: vec!["list".to_string(), "all".to_string(), "commit".to_string(), "date".to_string()],
                regex_pattern: Some(Regex::new(r"(?i)(list|show|display).*(all|every)?.*(branches?).*(sorted|ordered|by).*(last|recent)?.*(commit|date)").unwrap()),
                gnu_command: "git for-each-ref --sort=-committerdate refs/heads/ --format='%(committerdate:short) %(refname:short)'".to_string(),
                bsd_command: Some("git for-each-ref --sort=-committerdate refs/heads/ --format='%(committerdate:short) %(refname:short)'".to_string()),
                description: "List branches sorted by commit date".to_string(),
            },

            // Pattern 20: "Find who changed a specific file"
            PatternEntry {
                required_keywords: vec!["who".to_string(), "changed".to_string(), "file".to_string()],
                optional_keywords: vec!["find".to_string(), "specific".to_string()],
                regex_pattern: Some(Regex::new(r"(?i)(find|show|who|which|see).*(who|author|user).*(changed|modified|edited).*(specific|this|a)?.*(file)").unwrap()),
                gnu_command: "git log --follow -p -- <filename>".to_string(),
                bsd_command: Some("git log --follow -p -- <filename>".to_string()),
                description: "Find who changed a specific file".to_string(),
            },

            // ===== NETWORK OPERATIONS PATTERNS (Cycle 2 Priority 3) =====

            // Pattern 21: "Test connection to a remote server"
            PatternEntry {
                required_keywords: vec!["test".to_string(), "connection".to_string()],
                optional_keywords: vec!["remote".to_string(), "server".to_string(), "ping".to_string()],
                regex_pattern: Some(Regex::new(r"(?i)(test|check|verify|ping).*(connection|connectivity|network).*(to|with)?.*(remote|external)?.*(server|host|machine)").unwrap()),
                gnu_command: "ping -c 4 example.com".to_string(),
                bsd_command: Some("ping -c 4 example.com".to_string()),
                description: "Test connection to a remote server".to_string(),
            },

            // Pattern 22: "show all established connections to port 443" (SPECIFIC - moved from Pattern 49)
            PatternEntry {
                required_keywords: vec!["established".to_string(), "connections".to_string(), "443".to_string()],
                optional_keywords: vec!["show".to_string(), "all".to_string(), "port".to_string()],
                regex_pattern: Some(Regex::new(r"(?i)(show|display|list).*(all)?.*(established|active).*(connections?|sockets?).*(to|on)?.*(port)?.*\b443\b").unwrap()),
                gnu_command: "ss -tn state established '( dport = :443 )'".to_string(),
                bsd_command: Some("netstat -an | grep ESTABLISHED | grep :443".to_string()),
                description: "Show established connections to port 443".to_string(),
            },

            // Pattern 23: "Show all listening TCP ports" (GENERAL - was Pattern 22)
            PatternEntry {
                required_keywords: vec!["listening".to_string(), "port".to_string()],
                optional_keywords: vec!["show".to_string(), "all".to_string(), "tcp".to_string()],
                regex_pattern: Some(Regex::new(r"(?i)(show|list|display|find).*(all|listening|open).*(tcp|network)?.*(ports?|sockets?)").unwrap()),
                gnu_command: "ss -tlnp".to_string(),
                bsd_command: Some("netstat -an | grep LISTEN".to_string()),
                description: "Show all listening TCP ports".to_string(),
            },

            // Pattern 23: "Download a file with resume support"
            PatternEntry {
                required_keywords: vec!["download".to_string(), "file".to_string()],
                optional_keywords: vec!["resume".to_string(), "support".to_string(), "wget".to_string()],
                regex_pattern: Some(Regex::new(r"(?i)(download|fetch|get).*(file|url).*(with|using)?.*(resume|continue|restart)").unwrap()),
                gnu_command: "wget -c https://example.com/file.tar.gz".to_string(),
                bsd_command: Some("curl -C - -O https://example.com/file.tar.gz".to_string()),
                description: "Download file with resume support".to_string(),
            },

            // Pattern 24: "show all network interfaces and their status"
            PatternEntry {
                required_keywords: vec!["network".to_string(), "interface".to_string()],
                optional_keywords: vec!["show".to_string(), "all".to_string(), "status".to_string()],
                regex_pattern: Some(Regex::new(r"(?i)(show|list|display|get).*(all|network)?.*(interfaces?|adapters?|nics?).*(and|with)?.*(status|info|information)").unwrap()),
                gnu_command: "ip addr show".to_string(),
                bsd_command: Some("ifconfig".to_string()),
                description: "Show network interfaces and status".to_string(),
            },

            // Pattern 25: "show all established connections to port 443"
            PatternEntry {
                required_keywords: vec!["established".to_string(), "connections".to_string(), "port".to_string()],
                optional_keywords: vec!["show".to_string(), "all".to_string(), "443".to_string()],
                regex_pattern: Some(Regex::new(r"(?i)(show|list|display|get).*(all|established).*(connections?|sockets?).*(to|on)?.*(port|:)\s*\d+").unwrap()),
                gnu_command: "ss -tn state established '( dport = :443 )'".to_string(),
                bsd_command: Some("netstat -an | grep ESTABLISHED | grep :443".to_string()),
                description: "Show established connections to a port".to_string(),
            },

            // ===== SYSTEM MONITORING PATTERNS (Cycle 3) =====

            // Pattern 26: "show me the top 5 processes by CPU usage" (SPECIFIC - moved from Pattern 47)
            PatternEntry {
                required_keywords: vec!["top".to_string(), "5".to_string(), "cpu".to_string()],
                optional_keywords: vec!["show".to_string(), "processes".to_string()],
                regex_pattern: Some(Regex::new(r"(?i)(show|display|list).*(me)?.*(top).*(5|\bfive\b).*(processes?).*(by)?.*(cpu|processor)").unwrap()),
                gnu_command: "ps aux --sort=-%cpu | head -n 6".to_string(),
                bsd_command: Some("ps aux -r | head -n 6".to_string()),
                description: "Show top 5 processes by CPU usage".to_string(),
            },

            // Pattern 27: "Monitor CPU usage in real-time" (GENERAL - was Pattern 26)
            PatternEntry {
                required_keywords: vec!["cpu".to_string(), "usage".to_string()],
                optional_keywords: vec!["monitor".to_string(), "real-time".to_string(), "top".to_string()],
                regex_pattern: Some(Regex::new(r"(?i)(monitor|watch|show|display).*(cpu|processor).*(usage|utilization|consumption).*(real-time|realtime|live)?").unwrap()),
                gnu_command: "top -b -n 1 | head -n 20".to_string(),
                bsd_command: Some("top -l 1 | head -n 20".to_string()),
                description: "Monitor CPU usage in real-time".to_string(),
            },

            // Pattern 27: "what's using disk space"
            PatternEntry {
                required_keywords: vec!["using".to_string(), "disk".to_string(), "space".to_string()],
                optional_keywords: vec!["what".to_string(), "show".to_string()],
                regex_pattern: Some(Regex::new(r"(?i)(what|which|show).*(using|consuming|taking).*(disk|storage).*(space|usage)").unwrap()),
                gnu_command: "du -sh /* | sort -rh | head -10".to_string(),
                bsd_command: Some("du -sh /* | sort -rh | head -10".to_string()),
                description: "Show what's using disk space".to_string(),
            },

            // Pattern 28: "show me the top 5 processes by CPU usage"
            PatternEntry {
                required_keywords: vec!["top".to_string(), "processes".to_string(), "cpu".to_string()],
                optional_keywords: vec!["show".to_string(), "5".to_string(), "usage".to_string()],
                regex_pattern: Some(Regex::new(r"(?i)(show|list|display|find).*(top|most).*(processes?).*(by|using).*(cpu|processor)").unwrap()),
                gnu_command: "ps aux --sort=-%cpu | head -n 6".to_string(),
                bsd_command: Some("ps aux -r | head -n 6".to_string()),
                description: "Show top processes by CPU usage".to_string(),
            },

            // ===== DEVOPS/KUBERNETES PATTERNS (Cycle 3 Priority 2) =====

            // Pattern 29: "check if all kubernetes deployments are ready"
            PatternEntry {
                required_keywords: vec!["kubernetes".to_string(), "deployments".to_string(), "ready".to_string()],
                optional_keywords: vec!["check".to_string(), "all".to_string(), "production".to_string()],
                regex_pattern: Some(Regex::new(r"(?i)(check|verify|show).*(all|kubernetes|k8s)?.*(deployments?|deploy).*(ready|status|health)").unwrap()),
                gnu_command: r#"kubectl get deployments -n production -o jsonpath='{range .items[*]}{.metadata.name}{"\t"}{.status.readyReplicas}/{.status.replicas}{"\n"}{end}'"#.to_string(),
                bsd_command: Some(r#"kubectl get deployments -n production -o jsonpath='{range .items[*]}{.metadata.name}{"\t"}{.status.readyReplicas}/{.status.replicas}{"\n"}{end}'"#.to_string()),
                description: "Check Kubernetes deployment readiness".to_string(),
            },

            // Pattern 30: "clean up docker images and volumes"
            PatternEntry {
                required_keywords: vec!["clean".to_string(), "docker".to_string()],
                optional_keywords: vec!["up".to_string(), "images".to_string(), "volumes".to_string()],
                regex_pattern: Some(Regex::new(r"(?i)(clean|cleanup|remove|prune).*(up)?.*(docker|container).*(images?|volumes?|unused|not being used)").unwrap()),
                gnu_command: "docker system prune -f && docker volume prune -f".to_string(),
                bsd_command: Some("docker system prune -f && docker volume prune -f".to_string()),
                description: "Clean up unused Docker resources".to_string(),
            },

            // Pattern 31: "check if redis, postgres, and nginx are running"
            PatternEntry {
                required_keywords: vec!["check".to_string(), "running".to_string()],
                optional_keywords: vec!["redis".to_string(), "postgres".to_string(), "nginx".to_string()],
                regex_pattern: Some(Regex::new(r"(?i)(check|verify|see).*(if|whether).*(redis|postgres|postgresql|nginx|mysql|mongodb).*(running|active|up|status)").unwrap()),
                gnu_command: r#"for svc in redis postgres nginx; do systemctl is-active "$svc"; done"#.to_string(),
                bsd_command: Some(r#"for svc in redis postgres nginx; do service "$svc" status; done"#.to_string()),
                description: "Check if services are running".to_string(),
            },

            // Pattern 32: "show SSL certificates expiring in the next 30 days"
            PatternEntry {
                required_keywords: vec!["ssl".to_string(), "certificates".to_string(), "expiring".to_string()],
                optional_keywords: vec!["show".to_string(), "30".to_string(), "days".to_string()],
                regex_pattern: Some(Regex::new(r"(?i)(show|list|find|check).*(ssl|tls).*(certificates?|certs?).*(expiring|expire|expiration).*(next|in)?.*(30|days?)").unwrap()),
                gnu_command: r#"find /etc/ssl -name "*.pem" -exec sh -c 'openssl x509 -enddate -noout -in "{}" 2>/dev/null' \;"#.to_string(),
                bsd_command: Some(r#"find /etc/ssl -name "*.pem" -exec sh -c 'openssl x509 -enddate -noout -in "{}" 2>/dev/null' \;"#.to_string()),
                description: "Show SSL certificates expiring soon".to_string(),
            },

            // Pattern 33: "show all AWS EC2 instances in terraform state"
            PatternEntry {
                required_keywords: vec!["terraform".to_string(), "state".to_string()],
                optional_keywords: vec!["show".to_string(), "aws".to_string(), "ec2".to_string(), "instances".to_string()],
                regex_pattern: Some(Regex::new(r"(?i)(show|list|display).*(all|aws)?.*(ec2|instances?).*(terraform|tf).*(state)").unwrap()),
                gnu_command: "terraform state list | grep aws_instance".to_string(),
                bsd_command: Some("terraform state list | grep aws_instance".to_string()),
                description: "Show AWS EC2 instances in Terraform state".to_string(),
            },

            // ===== TEXT PROCESSING PATTERNS (Cycle 3 Remaining) =====

            // Pattern 34: "Count lines, words, and characters in all .txt files"
            PatternEntry {
                required_keywords: vec!["count".to_string(), "lines".to_string(), "txt".to_string()],
                optional_keywords: vec!["words".to_string(), "characters".to_string(), "files".to_string()],
                regex_pattern: Some(Regex::new(r"(?i)(count|calculate|show).*(lines?|words?|characters?).*(in|of).*(all|txt|\.txt).*(files?)").unwrap()),
                gnu_command: "wc $(find . -name '*.txt')".to_string(),
                bsd_command: Some("find . -name '*.txt' -exec wc {} +".to_string()),
                description: "Count lines, words, and characters in text files".to_string(),
            },

            // Pattern 35: "your first command" / "hello world"
            PatternEntry {
                required_keywords: vec!["first".to_string(), "command".to_string()],
                optional_keywords: vec!["your".to_string(), "hello".to_string(), "world".to_string()],
                regex_pattern: Some(Regex::new(r"(?i)(your|my|the)?.*(first|hello|hi).*(command|world)").unwrap()),
                gnu_command: r#"echo "Hello, World!""#.to_string(),
                bsd_command: Some(r#"echo "Hello, World!""#.to_string()),
                description: "First command - Hello World".to_string(),
            },

            // ===== LOG ANALYSIS PATTERNS (Cycle 4) =====

            // Pattern 36: "find all ERROR lines in logs from the last 24 hours" (SPECIFIC - moved from Pattern 39)
            PatternEntry {
                required_keywords: vec!["error".to_string(), "log".to_string(), "last".to_string()],
                optional_keywords: vec!["find".to_string(), "all".to_string(), "24".to_string(), "hours".to_string()],
                regex_pattern: Some(Regex::new(r"(?i)(find|search|grep).*(all)?.*(error|errors).*(lines?|entries?).*(in)?.*(logs?).*(from|in)?.*(last|past).*(24|1440)?.*(hours?|day)").unwrap()),
                gnu_command: r#"find /var/log -name "*.log" -mmin -1440 -exec grep -l "ERROR" {} \;"#.to_string(),
                bsd_command: Some(r#"find /var/log -name "*.log" -mmin -1440 -exec grep -l "ERROR" {} \;"#.to_string()),
                description: "Find ERROR lines in logs from last 24 hours".to_string(),
            },

            // Pattern 37: "Find all ERROR entries in application logs" (GENERAL - was Pattern 36)
            PatternEntry {
                required_keywords: vec!["error".to_string(), "log".to_string()],
                optional_keywords: vec!["find".to_string(), "all".to_string(), "entries".to_string(), "application".to_string()],
                regex_pattern: Some(Regex::new(r"(?i)(find|show|search|grep).*(all)?.*(error|errors).*(entries?|lines?|messages?).*(in)?.*(application|app)?.*logs?").unwrap()),
                gnu_command: "grep -i 'error' /var/log/app.log | tail -n 50".to_string(),
                bsd_command: Some("grep -i 'error' /var/log/app.log | tail -n 50".to_string()),
                description: "Find ERROR entries in application logs".to_string(),
            },

            // Pattern 37: "Count HTTP status codes in access log"
            PatternEntry {
                required_keywords: vec!["count".to_string(), "status".to_string(), "code".to_string()],
                optional_keywords: vec!["http".to_string(), "access".to_string(), "log".to_string()],
                regex_pattern: Some(Regex::new(r"(?i)(count|show|display|analyze).*(http)?.*(status|response)?.*(codes?|responses?).*(in)?.*(access|nginx)?.*logs?").unwrap()),
                gnu_command: "awk '{print $9}' /var/log/nginx/access.log | sort | uniq -c | sort -rn".to_string(),
                bsd_command: Some("awk '{print $9}' /var/log/nginx/access.log | sort | uniq -c | sort -rn".to_string()),
                description: "Count HTTP status codes in access log".to_string(),
            },

            // Pattern 38: "Search for TODO/FIXME comments in code" - Issue #10 fix
            PatternEntry {
                required_keywords: vec!["todo".to_string()],
                optional_keywords: vec!["search".to_string(), "find".to_string(), "code".to_string(), "for".to_string(), "fixme".to_string()],
                regex_pattern: Some(Regex::new(r"(?i)(search|find|look|grep).*(for)?.*(TODO|FIXME|HACK|XXX|NOTE).*(in)?.*(code|files?)").unwrap()),
                gnu_command: "grep -rn 'TODO' .".to_string(),
                bsd_command: Some("grep -rn 'TODO' .".to_string()),
                description: "Search for TODO/FIXME comments in code".to_string(),
            },

            // Pattern 39: "Show last 100 system errors"
            PatternEntry {
                required_keywords: vec!["last".to_string(), "system".to_string(), "error".to_string()],
                optional_keywords: vec!["show".to_string(), "100".to_string()],
                regex_pattern: Some(Regex::new(r"(?i)(show|display|list|get).*(last|recent).*(100|\d+)?.*(system|systemd)?.*errors?").unwrap()),
                gnu_command: "journalctl -p err -n 100".to_string(),
                bsd_command: Some("grep -i error /var/log/messages | tail -n 100".to_string()),
                description: "Show last N system errors".to_string(),
            },

            // ===== FILE MANAGEMENT REFINED PATTERNS (Cycle 4) =====

            // Pattern 41: "find python files" (simple variant - was Pattern 43)
            PatternEntry {
                required_keywords: vec!["find".to_string(), "python".to_string()],
                optional_keywords: vec!["files".to_string(), "all".to_string()],
                regex_pattern: Some(Regex::new(r"(?i)^(find|locate|search).*(python|\.py).*(files?)?\s*$").unwrap()),
                gnu_command: r#"find . -name "*.py" -type f"#.to_string(),
                bsd_command: Some(r#"find . -name "*.py" -type f"#.to_string()),
                description: "Find Python files (simple)".to_string(),
            },

            // Pattern 42: "list hidden files" - Issue #11 fix (MOVED BEFORE "list files" for priority)
            PatternEntry {
                required_keywords: vec!["hidden".to_string()],
                optional_keywords: vec!["list".to_string(), "show".to_string(), "files".to_string(), "dot".to_string()],
                regex_pattern: Some(Regex::new(r"(?i)(list|show|display|find).*(hidden|dot).*(files?)?").unwrap()),
                gnu_command: "ls -d .*".to_string(),
                bsd_command: Some("ls -d .*".to_string()),
                description: "List hidden files".to_string(),
            },

            // Pattern 43: "list files" (very simple variant - was Pattern 44)
            PatternEntry {
                required_keywords: vec!["list".to_string(), "files".to_string()],
                optional_keywords: vec!["all".to_string()],
                regex_pattern: Some(Regex::new(r"(?i)^(list|show).*(all)?.*(files?)\s*$").unwrap()),
                gnu_command: "ls -la".to_string(),
                bsd_command: Some("ls -la".to_string()),
                description: "List files (simple)".to_string(),
            },

            // Pattern 44: "find large files" (simple variant without size specified - was Pattern 45)
            PatternEntry {
                required_keywords: vec!["find".to_string(), "large".to_string()],
                optional_keywords: vec!["files".to_string(), "big".to_string()],
                regex_pattern: Some(Regex::new(r"(?i)^(find|locate|search).*(large|big).*(files?)?\s*$").unwrap()),
                gnu_command: "find . -type f -size +100M".to_string(),
                bsd_command: Some("find . -type f -size +100M".to_string()),
                description: "Find large files (default 100MB)".to_string(),
            },

            // ===== ADDITIONAL PATTERNS (Cycle 5) =====

            // Pattern 49: "check disk health on all drives"
            PatternEntry {
                required_keywords: vec!["check".to_string(), "disk".to_string(), "health".to_string()],
                optional_keywords: vec!["all".to_string(), "drives".to_string(), "smart".to_string()],
                regex_pattern: Some(Regex::new(r"(?i)(check|test|verify|show).*(disk|drive|hdd|ssd).*(health|status|smart)").unwrap()),
                gnu_command: "smartctl -a /dev/sda".to_string(),
                bsd_command: Some("smartctl -a /dev/sda".to_string()),
                description: "Check disk health with smartctl".to_string(),
            },

            // Pattern 50: "日本語のファイルを検索" (Find Japanese filename files)
            // Note: No keywords - matches ONLY via regex to avoid false positives
            PatternEntry {
                required_keywords: vec![],
                optional_keywords: vec![],  // Empty to force regex-only matching
                regex_pattern: Some(Regex::new(r"[ぁ-んァ-ヶー一-龯]").unwrap()),
                gnu_command: "find . -name '*日本語*' -type f".to_string(),
                bsd_command: Some("find . -name '*日本語*' -type f".to_string()),
                description: "Find files with Japanese characters in name".to_string(),
            },

        ]
    }

    /// Try to match the query against known patterns
    fn try_match(&self, query: &str) -> Option<&PatternEntry> {
        let query_lower = query.to_lowercase();

        for pattern in self.patterns.iter() {
            // Check regex pattern first (most precise)
            if let Some(ref regex) = pattern.regex_pattern {
                if regex.is_match(&query_lower) {
                    return Some(pattern);
                }
            }

            // Fallback to keyword matching
            let all_required = pattern
                .required_keywords
                .iter()
                .all(|kw| query_lower.contains(kw));

            if all_required {
                // Count optional keywords for confidence boost
                let optional_count = pattern
                    .optional_keywords
                    .iter()
                    .filter(|kw| query_lower.contains(*kw))
                    .count();

                // Require at least some optional keywords for keyword-only match
                if optional_count > 0 || pattern.regex_pattern.is_none() {
                    return Some(pattern);
                }
            }
        }

        None
    }

    /// Select the appropriate command based on platform
    fn select_command(&self, pattern: &PatternEntry) -> String {
        // For now, use GNU commands as default
        // In the future, we can use self.profile to detect platform
        pattern.gnu_command.clone()
    }
}

#[async_trait]
impl CommandGenerator for StaticMatcher {
    async fn generate_command(
        &self,
        request: &CommandRequest,
    ) -> Result<GeneratedCommand, GeneratorError> {
        // Try to match the query
        if let Some(pattern) = self.try_match(&request.input) {
            let command = self.select_command(pattern);

            // SAFETY VALIDATION: Validate the GENERATED command
            // This happens after pattern matching to check if the generated command is safe
            let safety_result = self
                .safety_validator
                .validate_command(&command, request.shell)
                .await
                .map_err(|e| GeneratorError::ValidationFailed {
                    reason: format!("Safety validation error: {}", e),
                })?;

            // If generated command is unsafe, return error with safety information
            if !safety_result.allowed {
                return Err(GeneratorError::Unsafe {
                    reason: safety_result.explanation.clone(),
                    risk_level: safety_result.risk_level,
                    warnings: safety_result.warnings.clone(),
                });
            }

            Ok(GeneratedCommand {
                command: command.clone(),
                explanation: format!("Matched pattern: {}", pattern.description),
                safety_level: safety_result.risk_level, // Use actual risk level from validation
                estimated_impact: if safety_result.warnings.is_empty() {
                    "Safe to execute".to_string()
                } else {
                    format!("Warnings: {}", safety_result.warnings.join(", "))
                },
                alternatives: vec![],
                backend_used: "static-matcher".to_string(),
                generation_time_ms: 0, // Instant - no LLM call
                confidence_score: 1.0, // Deterministic match
            })
        } else {
            // No match - return error so we can fall through to LLM
            Err(GeneratorError::BackendUnavailable {
                reason: "No static pattern match found".to_string(),
            })
        }
    }

    async fn is_available(&self) -> bool {
        // Static matcher is always available
        true
    }

    fn backend_info(&self) -> BackendInfo {
        BackendInfo {
            backend_type: BackendType::Embedded,
            model_name: "static-matcher".to_string(),
            supports_streaming: false,
            max_tokens: 0,
            typical_latency_ms: 0,
            memory_usage_mb: 1,
            version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }

    async fn shutdown(&self) -> Result<(), GeneratorError> {
        // Nothing to clean up
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_website_example_1() {
        let profile = CapabilityProfile::ubuntu();
        let matcher = StaticMatcher::new(profile);

        let request = CommandRequest::new("list all files modified today", ShellType::Bash);

        let result = matcher.generate_command(&request).await;
        assert!(result.is_ok());

        let cmd = result.unwrap();
        assert_eq!(cmd.command, "find . -type f -mtime 0");
        assert_eq!(cmd.safety_level, RiskLevel::Safe);
    }

    #[tokio::test]
    async fn test_website_example_2() {
        let profile = CapabilityProfile::ubuntu();
        let matcher = StaticMatcher::new(profile);

        let request = CommandRequest::new("find large files over 100MB", ShellType::Bash);

        let result = matcher.generate_command(&request).await;
        assert!(result.is_ok());

        let cmd = result.unwrap();
        assert_eq!(cmd.command, "find . -type f -size +100M");
    }

    #[tokio::test]
    async fn test_website_example_3() {
        let profile = CapabilityProfile::ubuntu();
        let matcher = StaticMatcher::new(profile);

        let request = CommandRequest::new("show disk usage by folder", ShellType::Bash);

        let result = matcher.generate_command(&request).await;
        assert!(result.is_ok());

        let cmd = result.unwrap();
        assert_eq!(cmd.command, "du -sh */ | sort -rh | head -10");
    }

    #[tokio::test]
    async fn test_website_example_4() {
        let profile = CapabilityProfile::ubuntu();
        let matcher = StaticMatcher::new(profile);

        let request = CommandRequest::new("find python files modified last week", ShellType::Bash);

        let result = matcher.generate_command(&request).await;
        assert!(result.is_ok());

        let cmd = result.unwrap();
        assert_eq!(cmd.command, "find . -name \"*.py\" -type f -mtime -7");
    }

    #[tokio::test]
    async fn test_variant_phrasing() {
        let profile = CapabilityProfile::ubuntu();
        let matcher = StaticMatcher::new(profile);

        // Should still match with different phrasing
        let request = CommandRequest::new(
            "show me all files that were modified today",
            ShellType::Bash,
        );

        let result = matcher.generate_command(&request).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_no_match() {
        let profile = CapabilityProfile::ubuntu();
        let matcher = StaticMatcher::new(profile);

        let request = CommandRequest::new("compile my rust project", ShellType::Bash);

        let result = matcher.generate_command(&request).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_search_todo_in_code() {
        let profile = CapabilityProfile::ubuntu();
        let matcher = StaticMatcher::new(profile);

        let request = CommandRequest::new("search for TODO in code", ShellType::Bash);

        let result = matcher.generate_command(&request).await;
        assert!(result.is_ok());
        let cmd = result.unwrap();
        assert!(
            cmd.command.contains("-rn"),
            "Command should include -rn flag"
        );
        assert!(
            cmd.command.contains("TODO"),
            "Command should search for TODO"
        );
    }

    #[tokio::test]
    async fn test_list_hidden_files() {
        let profile = CapabilityProfile::ubuntu();
        let matcher = StaticMatcher::new(profile);

        let request = CommandRequest::new("list hidden files", ShellType::Bash);

        let result = matcher.generate_command(&request).await;
        assert!(result.is_ok());
        let cmd = result.unwrap();
        assert_eq!(cmd.command, "ls -d .*", "Command should be 'ls -d .*'");
    }
}
