//! Natural language command suggestions from the pattern library
//!
//! This module provides command suggestions based on common shell patterns,
//! allowing users to discover relevant commands by typing partial descriptions.

/// A command suggestion with its description and generated command
#[derive(Debug, Clone)]
pub struct CommandSuggestion {
    /// Natural language description of what the command does
    pub description: String,
    /// The generated shell command
    pub command: String,
    /// Match score (0.0 to 1.0)
    pub score: f64,
}

/// Pattern definition for suggestions
struct PatternDef {
    keywords: &'static [&'static str],
    description: &'static str,
    command: &'static str,
}

/// Pattern library for command suggestions
const PATTERNS: &[PatternDef] = &[
    // File finding patterns
    PatternDef {
        keywords: &["find", "python", "files", "py"],
        description: "Find all Python files",
        command: "find . -name \"*.py\" -type f",
    },
    PatternDef {
        keywords: &["find", "files", "modified", "today"],
        description: "Find files modified today",
        command: "find . -type f -mtime 0",
    },
    PatternDef {
        keywords: &["find", "files", "modified", "yesterday"],
        description: "Find files modified yesterday",
        command: "find . -type f -mtime 1",
    },
    PatternDef {
        keywords: &["find", "files", "modified", "last", "week"],
        description: "Find files modified in the last week",
        command: "find . -type f -mtime -7",
    },
    PatternDef {
        keywords: &["find", "large", "files", "100mb", "100", "mb"],
        description: "Find large files over 100MB",
        command: "find . -type f -size +100M",
    },
    PatternDef {
        keywords: &["find", "files", "50mb", "50", "mb"],
        description: "Find files larger than 50MB",
        command: "find . -type f -size +50M",
    },
    PatternDef {
        keywords: &["find", "files", "1gb", "1", "gb"],
        description: "Find files larger than 1GB",
        command: "find . -type f -size +1G",
    },
    PatternDef {
        keywords: &["find", "files", "changed", "hour"],
        description: "Find files modified in the last hour",
        command: "find . -type f -mmin -60",
    },
    // Disk space patterns
    PatternDef {
        keywords: &["disk", "space", "usage", "directory", "folder"],
        description: "Show disk usage by directory",
        command: "du -h -d 1",
    },
    PatternDef {
        keywords: &["disk", "space", "usage", "sorted"],
        description: "Show disk usage by directory, sorted",
        command: "du -h -d 1 | sort -hr",
    },
    PatternDef {
        keywords: &["disk", "space", "free"],
        description: "Show free disk space",
        command: "df -h",
    },
    // List patterns
    PatternDef {
        keywords: &["list", "files", "all", "hidden"],
        description: "List all files including hidden",
        command: "ls -la",
    },
    PatternDef {
        keywords: &["list", "hidden", "files", "only"],
        description: "List only hidden files",
        command: "ls -d .*",
    },
    PatternDef {
        keywords: &["list", "directories", "folders", "only"],
        description: "List only directories",
        command: "ls -d */",
    },
    PatternDef {
        keywords: &["list", "files", "size", "sorted"],
        description: "List files sorted by size",
        command: "ls -lhS",
    },
    PatternDef {
        keywords: &["list", "files", "time", "recent", "sorted"],
        description: "List files sorted by modification time",
        command: "ls -lt",
    },
    // Process patterns
    PatternDef {
        keywords: &["process", "running", "list", "ps"],
        description: "List running processes",
        command: "ps aux",
    },
    PatternDef {
        keywords: &["process", "cpu", "top", "usage"],
        description: "Show top CPU-consuming processes",
        command: "top -o cpu",
    },
    PatternDef {
        keywords: &["process", "memory", "top", "usage"],
        description: "Show top memory-consuming processes",
        command: "top -o mem",
    },
    // Network patterns
    PatternDef {
        keywords: &["network", "connections", "active", "netstat"],
        description: "Show active network connections",
        command: "netstat -an",
    },
    PatternDef {
        keywords: &["port", "listening", "open"],
        description: "Show listening ports",
        command: "lsof -i -P -n | grep LISTEN",
    },
    PatternDef {
        keywords: &["ip", "address", "local"],
        description: "Show local IP address",
        command: "ifconfig | grep \"inet \" | grep -v 127.0.0.1",
    },
    // Git patterns
    PatternDef {
        keywords: &["git", "status"],
        description: "Show git status",
        command: "git status",
    },
    PatternDef {
        keywords: &["git", "log", "history", "commits"],
        description: "Show git commit history",
        command: "git log --oneline -10",
    },
    PatternDef {
        keywords: &["git", "branches", "list"],
        description: "List git branches",
        command: "git branch -a",
    },
    PatternDef {
        keywords: &["git", "diff", "changes"],
        description: "Show uncommitted changes",
        command: "git diff",
    },
    // Docker patterns
    PatternDef {
        keywords: &["docker", "containers", "running", "list"],
        description: "List running Docker containers",
        command: "docker ps",
    },
    PatternDef {
        keywords: &["docker", "containers", "all", "list"],
        description: "List all Docker containers",
        command: "docker ps -a",
    },
    PatternDef {
        keywords: &["docker", "images", "list"],
        description: "List Docker images",
        command: "docker images",
    },
    // Kubernetes patterns
    PatternDef {
        keywords: &["kubernetes", "k8s", "pods", "list"],
        description: "List Kubernetes pods",
        command: "kubectl get pods",
    },
    PatternDef {
        keywords: &["kubernetes", "k8s", "services", "list"],
        description: "List Kubernetes services",
        command: "kubectl get services",
    },
    PatternDef {
        keywords: &["kubernetes", "k8s", "deployments", "list"],
        description: "List Kubernetes deployments",
        command: "kubectl get deployments",
    },
    // Text search patterns
    PatternDef {
        keywords: &["grep", "search", "text", "files", "pattern"],
        description: "Search for text pattern in files",
        command: "grep -r \"<pattern>\" .",
    },
    PatternDef {
        keywords: &["count", "lines", "file"],
        description: "Count lines in a file",
        command: "wc -l <file>",
    },
    PatternDef {
        keywords: &["tail", "log", "follow", "watch"],
        description: "Follow a log file",
        command: "tail -f <logfile>",
    },
    // Archive patterns
    PatternDef {
        keywords: &["tar", "extract", "archive", "unpack"],
        description: "Extract a tar archive",
        command: "tar -xvf <archive.tar>",
    },
    PatternDef {
        keywords: &["tar", "create", "archive", "compress"],
        description: "Create a tar archive",
        command: "tar -cvf archive.tar <directory>",
    },
    PatternDef {
        keywords: &["zip", "compress", "directory"],
        description: "Zip a directory",
        command: "zip -r archive.zip <directory>",
    },
    PatternDef {
        keywords: &["unzip", "extract"],
        description: "Extract a zip file",
        command: "unzip <archive.zip>",
    },
];

/// Suggest commands matching a natural language query
///
/// Returns up to `limit` suggestions sorted by relevance.
pub fn suggest_commands(query: &str, limit: usize) -> Vec<CommandSuggestion> {
    if query.trim().is_empty() {
        return Vec::new();
    }

    let query_lower = query.to_lowercase();
    let query_words: Vec<&str> = query_lower.split_whitespace().collect();

    let mut suggestions: Vec<CommandSuggestion> = PATTERNS
        .iter()
        .filter_map(|pattern| {
            let score = calculate_match_score(&query_words, pattern);
            if score > 0.0 {
                Some(CommandSuggestion {
                    description: pattern.description.to_string(),
                    command: pattern.command.to_string(),
                    score,
                })
            } else {
                None
            }
        })
        .collect();

    // Sort by score descending
    suggestions.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    // Return top N results
    suggestions.truncate(limit);
    suggestions
}

/// Calculate match score between query words and a pattern
fn calculate_match_score(query_words: &[&str], pattern: &PatternDef) -> f64 {
    if query_words.is_empty() {
        return 0.0;
    }

    let description_lower = pattern.description.to_lowercase();
    let keywords = pattern.keywords;

    let mut score = 0.0;
    let mut matched_words = 0;

    for word in query_words {
        // Check if word matches any keyword
        let keyword_match = keywords
            .iter()
            .any(|k| k.contains(word) || word.contains(k));
        if keyword_match {
            score += 1.0;
            matched_words += 1;
        }

        // Check if word appears in description
        if description_lower.contains(word) {
            score += 0.5;
            matched_words += 1;
        }
    }

    // Normalize score based on query length and keyword coverage
    if matched_words > 0 {
        let coverage = matched_words as f64 / (query_words.len() as f64 * 2.0);
        score * coverage
    } else {
        0.0
    }
}
