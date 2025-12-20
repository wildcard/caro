//! Capabilities module - Caro's scope and boundary detection
//!
//! Caro knows what she does best: POSIX one-liners, Unix philosophy,
//! and using existing tools. She politely (and sassily) refuses tasks
//! outside her expertise and recommends better alternatives.
//!
//! # What Caro DOES:
//! - POSIX shell commands and one-liners
//! - File system operations (list, find, move, copy, etc.)
//! - Data manipulation with existing tools (grep, sed, awk, sort, etc.)
//! - Text processing and filtering
//! - Process management
//! - System information queries
//!
//! # What Caro REFUSES:
//! - Multi-line scripts (more than a simple pipe chain)
//! - Application development (weather apps, todo lists, etc.)
//! - Writing software/code in programming languages
//! - Package installation (that's what package managers are for!)
//! - Anything requiring complex programming logic

use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};

/// Result of capability boundary check
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoundaryCheckResult {
    /// Whether the request is within Caro's capabilities
    pub is_within_scope: bool,
    /// Category of rejection if out of scope
    pub rejection_category: Option<RejectionCategory>,
    /// Sassy rejection message if out of scope
    pub rejection_message: Option<String>,
    /// Recommended alternative tools
    pub alternatives: Vec<AlternativeTool>,
    /// Confidence score (0.0 to 1.0)
    pub confidence: f32,
}

/// Categories of requests Caro refuses
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RejectionCategory {
    /// User wants a multi-line script
    MultiLineScript,
    /// User wants to build an application
    ApplicationDevelopment,
    /// User wants to write code/software
    SoftwareDevelopment,
    /// User wants package installation
    PackageInstallation,
    /// Windows-specific request (not yet supported)
    WindowsNotSupported,
    /// General programming task
    ProgrammingTask,
    /// Web development
    WebDevelopment,
}

impl std::fmt::Display for RejectionCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MultiLineScript => write!(f, "multi-line script"),
            Self::ApplicationDevelopment => write!(f, "application development"),
            Self::SoftwareDevelopment => write!(f, "software development"),
            Self::PackageInstallation => write!(f, "package installation"),
            Self::WindowsNotSupported => write!(f, "Windows (not yet supported)"),
            Self::ProgrammingTask => write!(f, "programming task"),
            Self::WebDevelopment => write!(f, "web development"),
        }
    }
}

/// Alternative tools Caro recommends
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlternativeTool {
    pub name: String,
    pub description: String,
    pub url: Option<String>,
    pub is_favorite: bool,
}

impl AlternativeTool {
    fn crush() -> Self {
        Self {
            name: "Crush".to_string(),
            description: "Charm.land's powerful terminal coding agent".to_string(),
            url: Some("https://charm.sh/crush".to_string()),
            is_favorite: true,
        }
    }

    fn claude_code() -> Self {
        Self {
            name: "Claude Code".to_string(),
            description: "Anthropic's CLI for Claude - great for coding tasks".to_string(),
            url: Some("https://claude.ai/code".to_string()),
            is_favorite: true,
        }
    }

    fn cline() -> Self {
        Self {
            name: "Cline".to_string(),
            description: "AI coding assistant in your terminal".to_string(),
            url: Some("https://github.com/cline/cline".to_string()),
            is_favorite: true,
        }
    }

    fn codex() -> Self {
        Self {
            name: "Codex".to_string(),
            description: "OpenAI's code generation assistant".to_string(),
            url: Some("https://openai.com/codex".to_string()),
            is_favorite: false,
        }
    }

    fn gemini() -> Self {
        Self {
            name: "Gemini".to_string(),
            description: "Google's AI assistant for coding".to_string(),
            url: Some("https://gemini.google.com".to_string()),
            is_favorite: false,
        }
    }

    fn quantcoder() -> Self {
        Self {
            name: "QuantCoder".to_string(),
            description: "Specialized coding AI for quantitative work".to_string(),
            url: None,
            is_favorite: false,
        }
    }

    fn aider() -> Self {
        Self {
            name: "Aider".to_string(),
            description: "AI pair programming in your terminal".to_string(),
            url: Some("https://aider.chat".to_string()),
            is_favorite: false,
        }
    }

    fn homebrew() -> Self {
        Self {
            name: "Homebrew".to_string(),
            description: "The missing package manager for macOS".to_string(),
            url: Some("https://brew.sh".to_string()),
            is_favorite: false,
        }
    }

    fn apt() -> Self {
        Self {
            name: "apt/apt-get".to_string(),
            description: "Debian/Ubuntu package manager".to_string(),
            url: None,
            is_favorite: false,
        }
    }
}

/// Capability boundary validator
#[derive(Debug)]
pub struct CapabilityValidator {
    /// Patterns that indicate out-of-scope requests
    patterns: Vec<(regex::Regex, RejectionCategory, f32)>,
}

impl CapabilityValidator {
    /// Create a new capability validator
    pub fn new() -> Result<Self, CapabilityError> {
        let patterns = Self::compile_patterns()?;
        Ok(Self { patterns })
    }

    /// Compile detection patterns
    fn compile_patterns() -> Result<Vec<(regex::Regex, RejectionCategory, f32)>, CapabilityError> {
        let pattern_defs: Vec<(&str, RejectionCategory, f32)> = vec![
            // Multi-line script detection
            (r"(?i)\b(write|create|make|build)\s+(me\s+)?(a\s+)?(multi[- ]?line|multiline)\s+(script|bash|shell)", RejectionCategory::MultiLineScript, 0.95),
            (r"(?i)\b(script|program)\s+(with|that)\s+(multiple|several)\s+(lines|steps|commands)", RejectionCategory::MultiLineScript, 0.90),
            (r"(?i)\bwrite\s+(me\s+)?(a\s+)?script\s+(that|to|for|which)", RejectionCategory::MultiLineScript, 0.85),
            (r"(?i)\bcreate\s+(me\s+)?(a\s+)?bash\s+script\s+(that|to|for)", RejectionCategory::MultiLineScript, 0.90),
            (r"(?i)\b(shell|bash)\s+script\s+file", RejectionCategory::MultiLineScript, 0.85),
            (r"(?i)\bmulti[- ]?line\s+script", RejectionCategory::MultiLineScript, 0.95),

            // Application development detection
            (r"(?i)\b(build|create|make|write|develop)\s+(a\s+|an\s+|me\s+)?(weather|todo|task|notes?|chat|calculator|game|blog|cms|ecommerce|e-commerce)\s+(app|application|program|software)", RejectionCategory::ApplicationDevelopment, 0.95),
            (r"(?i)\b(build|create|make)\s+(a\s+|an\s+|me\s+)?(web|mobile|desktop|gui|graphical)\s+(app|application)", RejectionCategory::ApplicationDevelopment, 0.95),
            (r"(?i)\b(build|create)\s+(a\s+)?full[- ]?(stack|fledged)\s+(app|application|website)", RejectionCategory::ApplicationDevelopment, 0.95),
            (r"(?i)\b(todo|task)\s*(list|manager|app)", RejectionCategory::ApplicationDevelopment, 0.90),
            (r"(?i)\b(weather|stock)\s*(tracker|app|widget)", RejectionCategory::ApplicationDevelopment, 0.90),

            // Software development detection
            (r"(?i)\b(write|create|code|implement|develop)\s+(a\s+)?(class|function|method|module|library|package|api|backend|frontend|microservice)", RejectionCategory::SoftwareDevelopment, 0.90),
            (r"(?i)\b(write|create)\s+(some\s+)?(python|rust|javascript|typescript|go|java|c\+\+|ruby|swift|kotlin)\s+(code|program|script)", RejectionCategory::SoftwareDevelopment, 0.95),
            (r"(?i)\bimplement\s+(a\s+)?(sorting|searching|encryption|compression|parsing)\s+(algorithm|logic)", RejectionCategory::SoftwareDevelopment, 0.90),
            (r"(?i)\b(write|code|implement)\s+(unit|integration|e2e|end-to-end)\s+tests?", RejectionCategory::SoftwareDevelopment, 0.85),
            (r"(?i)\b(build|create|implement)\s+(a\s+)?(rest|graphql|grpc)\s+api", RejectionCategory::SoftwareDevelopment, 0.95),
            (r"(?i)\bimplement\s+.{0,30}(rest|graphql|grpc)\s+api", RejectionCategory::SoftwareDevelopment, 0.90),
            (r"(?i)\brefactor\s+(the|my|this)\s+(code|codebase|project)", RejectionCategory::SoftwareDevelopment, 0.80),

            // Package installation detection
            (r"(?i)\b(install|setup|configure)\s+(a\s+)?(package|library|dependency|module)\s+(for|from|using)", RejectionCategory::PackageInstallation, 0.85),
            (r"(?i)\b(brew|apt|apt-get|yum|dnf|pacman|npm|pip|cargo)\s+install\b", RejectionCategory::PackageInstallation, 0.70), // Lower confidence - might be asking about the command
            (r"(?i)\bhow\s+(do\s+i|to|can\s+i)\s+install\s+\w+\s+(globally|system-wide)", RejectionCategory::PackageInstallation, 0.80),

            // Windows-specific detection (for future support message)
            (r"(?i)\b(powershell|cmd\.exe|batch|\.bat|\.ps1|windows\s+command)", RejectionCategory::WindowsNotSupported, 0.90),
            (r"(?i)\b(windows|win32|win64)\s+(command|script|batch)", RejectionCategory::WindowsNotSupported, 0.90),

            // General programming tasks
            (r"(?i)\b(debug|fix)\s+(the|this|my)\s+(bug|error|issue|problem)\s+(in|with)\s+(the|my)\s+(code|program|script)", RejectionCategory::ProgrammingTask, 0.85),
            (r"(?i)\b(optimize|improve)\s+(the|my)\s+(code|algorithm|performance)", RejectionCategory::ProgrammingTask, 0.80),
            (r"(?i)\badd\s+(a\s+)?(feature|functionality)\s+to\s+(the|my)\s+(code|app|application)", RejectionCategory::ProgrammingTask, 0.85),

            // Web development
            (r"(?i)\b(build|create|make)\s+(a\s+)?(website|webpage|web\s+page|landing\s+page|portfolio)", RejectionCategory::WebDevelopment, 0.95),
            (r"(?i)\b(write|create)\s+(html|css|javascript|react|vue|angular|svelte)\s+(code|component)", RejectionCategory::WebDevelopment, 0.95),
            (r"(?i)\b(setup|configure)\s+(a\s+)?(react|vue|angular|next\.?js|nuxt|svelte)\s+(project|app)", RejectionCategory::WebDevelopment, 0.90),
        ];

        let mut compiled = Vec::with_capacity(pattern_defs.len());
        for (pattern, category, confidence) in pattern_defs {
            let regex = regex::Regex::new(pattern).map_err(|e| CapabilityError::PatternError {
                pattern: pattern.to_string(),
                error: e.to_string(),
            })?;
            compiled.push((regex, category, confidence));
        }

        Ok(compiled)
    }

    /// Check if a request is within Caro's capabilities
    pub fn check(&self, request: &str) -> BoundaryCheckResult {
        // Find the highest-confidence match
        let mut best_match: Option<(RejectionCategory, f32)> = None;

        for (regex, category, confidence) in &self.patterns {
            if regex.is_match(request) {
                match &best_match {
                    None => best_match = Some((*category, *confidence)),
                    Some((_, existing_confidence)) if confidence > existing_confidence => {
                        best_match = Some((*category, *confidence));
                    }
                    _ => {}
                }
            }
        }

        match best_match {
            Some((category, confidence)) => {
                let rejection_message = Self::generate_sassy_rejection(category);
                let alternatives = Self::get_alternatives_for_category(category);

                BoundaryCheckResult {
                    is_within_scope: false,
                    rejection_category: Some(category),
                    rejection_message: Some(rejection_message),
                    alternatives,
                    confidence,
                }
            }
            None => BoundaryCheckResult {
                is_within_scope: true,
                rejection_category: None,
                rejection_message: None,
                alternatives: vec![],
                confidence: 0.95,
            },
        }
    }

    /// Generate a sassy rejection message based on category
    fn generate_sassy_rejection(category: RejectionCategory) -> String {
        let mut rng = rand::thread_rng();

        let messages: Vec<&str> = match category {
            RejectionCategory::MultiLineScript => vec![
                "Oh honey, multi-line scripts? That's not my jam. I'm a one-liner kinda gal. 💅",
                "Scripts with multiple lines? Sorry, I keep it simple - one line, one command, Unix philosophy!",
                "Look, I do one-liners. Like a haiku, but for the terminal. Multi-line scripts need a real coding agent.",
                "I'm flattered you think I can write scripts, but I'm more of a 'pipe it and forget it' type.",
                "Multi-line scripts are great! They're just... not what I do. I'm all about that single-line magic. ✨",
            ],
            RejectionCategory::ApplicationDevelopment => vec![
                "Build you an app? That's adorable. I'm here for shell commands, not software empires. 🏰",
                "An app? I'm Caro, not an app factory! I help you USE your terminal, not build the next unicorn.",
                "Apps are beautiful, complex creatures. I'm just a simple shell command assistant. Know thyself!",
                "Sweetie, I can help you list files, not build the next weather app. Let's be realistic here.",
                "Application development? That's way above my pay grade. I stick to the classics: ls, grep, find...",
            ],
            RejectionCategory::SoftwareDevelopment => vec![
                "Write code? In a programming language? Bestie, I speak POSIX, not Python. 🐍❌",
                "I'm not a code monkey - I'm a command whisperer. There's a difference!",
                "Software development sounds fancy, but I'm here for the simple things in life: pipes, redirects, and one-liners.",
                "Honey, I can help you FIND your code files, but I can't write them for you. That's a job for the big leagues.",
                "Code? Functions? Classes? I don't know her. I know 'cat', 'grep', and 'awk' though!",
            ],
            RejectionCategory::PackageInstallation => vec![
                "Package installation? I'm not a package manager, I'm Caro! Use brew, apt, or whatever your system prefers.",
                "Installing packages is a job for Homebrew, apt, or your friendly neighborhood package manager. Not me!",
                "I could tell you the COMMAND to install something, but actually installing? That's between you and your package manager.",
                "Look, I can help you USE tools, but installing them? That's a whole different relationship.",
                "Package installation is a trust exercise between you and your package manager. I'm just here for moral support. 💪",
            ],
            RejectionCategory::WindowsNotSupported => vec![
                "Windows? Oh sweetie, I'm a POSIX princess. Windows support is on my wish list, not my resume. 🪟❌",
                "PowerShell and cmd? I don't speak those languages... yet. Check back later!",
                "I'm all about that Unix life. Windows users, I see you, but I can't help you. Yet!",
                "Windows commands? That's like asking a fish to climb a tree. I'm built for Unix/Linux/macOS.",
                "Batch files? CMD? PowerShell? Future Caro might help, but current Caro only does POSIX. Sorry!",
            ],
            RejectionCategory::ProgrammingTask => vec![
                "Debug your code? Sweetie, I can help you FIND files with errors, but fixing bugs? That's programmer territory.",
                "Programming tasks require a real coding assistant. I'm just here to help you navigate your filesystem.",
                "I'm not a debugger or a code reviewer. I'm a command-line companion. Big difference!",
                "Optimizing algorithms? That sounds like a job for a coding AI. I optimize commands, not code.",
                "Look, I can grep for 'error' in your logs, but actually fixing the code? Way out of my league.",
            ],
            RejectionCategory::WebDevelopment => vec![
                "A website? Honey, I'm a terminal tool, not a web developer. I don't do HTML high fashion. 💻❌",
                "React? Vue? Angular? I only know one framework: the Unix command line. And I like it that way.",
                "Web development is art. What I do is... also art, but the kind that lives in a terminal.",
                "Creating websites is beautiful work! It's just not MY work. I stick to shell commands.",
                "Frontend? Backend? I'm more of a 'terminal-end' kind of assistant. Just commands, no components.",
            ],
        };

        let base_message = messages.choose(&mut rng).unwrap_or(&messages[0]);
        format!("{}\n\n📚 Visit https://caro.run to learn more about what I can do!", base_message)
    }

    /// Get appropriate alternative tools for a category
    fn get_alternatives_for_category(category: RejectionCategory) -> Vec<AlternativeTool> {
        match category {
            RejectionCategory::MultiLineScript | RejectionCategory::SoftwareDevelopment | RejectionCategory::ProgrammingTask => vec![
                AlternativeTool::crush(),
                AlternativeTool::claude_code(),
                AlternativeTool::cline(),
                AlternativeTool::aider(),
                AlternativeTool::codex(),
            ],
            RejectionCategory::ApplicationDevelopment | RejectionCategory::WebDevelopment => vec![
                AlternativeTool::claude_code(),
                AlternativeTool::crush(),
                AlternativeTool::cline(),
                AlternativeTool::gemini(),
                AlternativeTool::codex(),
            ],
            RejectionCategory::PackageInstallation => vec![
                AlternativeTool::homebrew(),
                AlternativeTool::apt(),
            ],
            RejectionCategory::WindowsNotSupported => vec![
                AlternativeTool::claude_code(),
                AlternativeTool::crush(),
            ],
        }
    }
}

impl Default for CapabilityValidator {
    fn default() -> Self {
        Self::new().expect("Failed to create default CapabilityValidator")
    }
}

/// Errors that can occur during capability validation
#[derive(Debug, thiserror::Error, Serialize, Deserialize)]
pub enum CapabilityError {
    #[error("Pattern compilation failed for '{pattern}': {error}")]
    PatternError { pattern: String, error: String },

    #[error("Validation error: {message}")]
    ValidationError { message: String },
}

/// Format a rejection result for display
pub fn format_rejection(result: &BoundaryCheckResult) -> String {
    let mut output = String::new();

    if let Some(ref message) = result.rejection_message {
        output.push_str(message);
        output.push_str("\n\n");
    }

    if !result.alternatives.is_empty() {
        output.push_str("🔧 Try these instead:\n");

        // Show favorites first
        for alt in &result.alternatives {
            if alt.is_favorite {
                output.push_str(&format!(
                    "  ⭐ {} - {}\n",
                    alt.name, alt.description
                ));
                if let Some(ref url) = alt.url {
                    output.push_str(&format!("     {}\n", url));
                }
            }
        }

        // Then show others
        for alt in &result.alternatives {
            if !alt.is_favorite {
                output.push_str(&format!(
                    "  • {} - {}\n",
                    alt.name, alt.description
                ));
                if let Some(ref url) = alt.url {
                    output.push_str(&format!("     {}\n", url));
                }
            }
        }
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_multiline_script_detection() {
        let validator = CapabilityValidator::new().unwrap();

        let test_cases = vec![
            "write me a multi-line script to backup my files",
            "create a bash script that monitors CPU usage",
            "write a shell script file for deployment",
        ];

        for case in test_cases {
            let result = validator.check(case);
            assert!(
                !result.is_within_scope,
                "Should reject multi-line script request: '{}'",
                case
            );
            assert_eq!(
                result.rejection_category,
                Some(RejectionCategory::MultiLineScript),
                "Wrong category for: '{}'",
                case
            );
        }
    }

    #[test]
    fn test_application_development_detection() {
        let validator = CapabilityValidator::new().unwrap();

        let test_cases = vec![
            "build me a weather app",
            "create a todo list application",
            "make a web app for tracking expenses",
        ];

        for case in test_cases {
            let result = validator.check(case);
            assert!(
                !result.is_within_scope,
                "Should reject app development request: '{}'",
                case
            );
            assert_eq!(
                result.rejection_category,
                Some(RejectionCategory::ApplicationDevelopment),
                "Wrong category for: '{}'",
                case
            );
        }
    }

    #[test]
    fn test_software_development_detection() {
        let validator = CapabilityValidator::new().unwrap();

        let test_cases = vec![
            "write some Python code to parse JSON",
            "create a function to sort arrays",
            "implement a REST API for users",
        ];

        for case in test_cases {
            let result = validator.check(case);
            assert!(
                !result.is_within_scope,
                "Should reject software development request: '{}'",
                case
            );
        }
    }

    #[test]
    fn test_valid_posix_requests() {
        let validator = CapabilityValidator::new().unwrap();

        let test_cases = vec![
            "list all files in current directory",
            "find files larger than 100MB",
            "count lines in all Python files",
            "search for 'error' in log files",
            "show disk usage",
            "copy files from src to dest",
            "show running processes",
        ];

        for case in test_cases {
            let result = validator.check(case);
            assert!(
                result.is_within_scope,
                "Should accept valid POSIX request: '{}'",
                case
            );
        }
    }

    #[test]
    fn test_rejection_has_alternatives() {
        let validator = CapabilityValidator::new().unwrap();
        let result = validator.check("build me a weather application");

        assert!(!result.is_within_scope);
        assert!(!result.alternatives.is_empty());

        // Should recommend favorites
        let favorites: Vec<_> = result.alternatives.iter().filter(|a| a.is_favorite).collect();
        assert!(!favorites.is_empty(), "Should have favorite alternatives");
    }

    #[test]
    fn test_rejection_message_format() {
        let validator = CapabilityValidator::new().unwrap();
        let result = validator.check("create a todo list app");

        assert!(!result.is_within_scope);
        assert!(result.rejection_message.is_some());

        let message = result.rejection_message.unwrap();
        assert!(message.contains("caro.run"), "Should include website reference");
    }
}
