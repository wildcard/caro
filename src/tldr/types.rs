//! TLDR data types for representing parsed TLDR pages.
//!
//! These types model the structure of TLDR pages which provide
//! simplified, practical examples for command-line tools.

use serde::{Deserialize, Serialize};

/// A parsed TLDR page containing command documentation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TldrPage {
    /// The command name (e.g., "git", "curl", "find")
    pub name: String,

    /// Brief description of what the command does
    pub description: String,

    /// Additional context or notes about the command
    pub notes: Vec<String>,

    /// Practical examples demonstrating common use cases
    pub examples: Vec<TldrExample>,

    /// Platform this page is for (common, linux, osx, windows)
    pub platform: Platform,

    /// Language/locale of the page (e.g., "en", "es", "zh")
    pub language: String,
}

/// A single example from a TLDR page.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TldrExample {
    /// Description of what this example does
    pub description: String,

    /// The command template with placeholders
    pub command: String,

    /// Extracted placeholders from the command
    pub placeholders: Vec<Placeholder>,
}

/// A placeholder in a TLDR command template.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Placeholder {
    /// The full placeholder text including braces (e.g., "{{filename}}")
    pub raw: String,

    /// The placeholder name (e.g., "filename")
    pub name: String,

    /// Whether this is an optional placeholder
    pub optional: bool,

    /// Alternative options if this is a choice placeholder
    pub alternatives: Vec<String>,
}

/// Platform for TLDR pages.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum Platform {
    /// Platform-independent commands
    Common,
    /// Linux-specific commands
    Linux,
    /// macOS-specific commands (BSD-style)
    #[serde(alias = "macos")]
    Osx,
    /// Windows-specific commands
    Windows,
    /// Android-specific commands
    Android,
    /// SunOS-specific commands
    SunOs,
    /// FreeBSD-specific commands
    FreeBsd,
    /// NetBSD-specific commands
    NetBsd,
    /// OpenBSD-specific commands
    OpenBsd,
}

impl Platform {
    /// Get the directory name for this platform in TLDR archives.
    pub fn as_dir_name(&self) -> &'static str {
        match self {
            Platform::Common => "common",
            Platform::Linux => "linux",
            Platform::Osx => "osx",
            Platform::Windows => "windows",
            Platform::Android => "android",
            Platform::SunOs => "sunos",
            Platform::FreeBsd => "freebsd",
            Platform::NetBsd => "netbsd",
            Platform::OpenBsd => "openbsd",
        }
    }

    /// Get the current platform based on the OS.
    pub fn current() -> Self {
        #[cfg(target_os = "macos")]
        return Platform::Osx;

        #[cfg(target_os = "linux")]
        return Platform::Linux;

        #[cfg(target_os = "windows")]
        return Platform::Windows;

        #[cfg(target_os = "android")]
        return Platform::Android;

        #[cfg(target_os = "freebsd")]
        return Platform::FreeBsd;

        #[cfg(target_os = "netbsd")]
        return Platform::NetBsd;

        #[cfg(target_os = "openbsd")]
        return Platform::OpenBsd;

        #[cfg(not(any(
            target_os = "macos",
            target_os = "linux",
            target_os = "windows",
            target_os = "android",
            target_os = "freebsd",
            target_os = "netbsd",
            target_os = "openbsd"
        )))]
        Platform::Common
    }

    /// Get the search priority order for this platform.
    /// Returns platforms to search in order of preference.
    pub fn search_order(&self) -> Vec<Platform> {
        match self {
            Platform::Osx => vec![Platform::Osx, Platform::Common, Platform::Linux],
            Platform::Linux => vec![Platform::Linux, Platform::Common],
            Platform::Windows => vec![Platform::Windows, Platform::Common],
            Platform::FreeBsd | Platform::NetBsd | Platform::OpenBsd => {
                vec![*self, Platform::Common, Platform::Linux]
            }
            Platform::Android => vec![Platform::Android, Platform::Linux, Platform::Common],
            Platform::SunOs => vec![Platform::SunOs, Platform::Common, Platform::Linux],
            Platform::Common => vec![Platform::Common],
        }
    }

    /// Parse platform from directory name.
    pub fn from_dir_name(name: &str) -> Option<Self> {
        match name.to_lowercase().as_str() {
            "common" => Some(Platform::Common),
            "linux" => Some(Platform::Linux),
            "osx" | "macos" => Some(Platform::Osx),
            "windows" => Some(Platform::Windows),
            "android" => Some(Platform::Android),
            "sunos" => Some(Platform::SunOs),
            "freebsd" => Some(Platform::FreeBsd),
            "netbsd" => Some(Platform::NetBsd),
            "openbsd" => Some(Platform::OpenBsd),
            _ => None,
        }
    }
}

impl std::fmt::Display for Platform {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_dir_name())
    }
}

impl TldrPage {
    /// Create a new empty TLDR page.
    pub fn new(name: impl Into<String>, platform: Platform) -> Self {
        Self {
            name: name.into(),
            description: String::new(),
            notes: Vec::new(),
            examples: Vec::new(),
            platform,
            language: "en".to_string(),
        }
    }

    /// Format the page as a concise context string for LLM prompts.
    pub fn as_context(&self) -> String {
        let mut context = format!("TLDR for `{}`:\n{}\n", self.name, self.description);

        if !self.notes.is_empty() {
            context.push_str("\nNotes:\n");
            for note in &self.notes {
                context.push_str(&format!("- {}\n", note));
            }
        }

        if !self.examples.is_empty() {
            context.push_str("\nExamples:\n");
            for example in &self.examples {
                context.push_str(&format!("- {}\n  `{}`\n", example.description, example.command));
            }
        }

        context
    }

    /// Get a summary suitable for display.
    pub fn summary(&self) -> String {
        format!(
            "{}: {} ({} examples)",
            self.name,
            self.description,
            self.examples.len()
        )
    }
}

impl TldrExample {
    /// Create a new example.
    pub fn new(description: impl Into<String>, command: impl Into<String>) -> Self {
        let command = command.into();
        let placeholders = Self::extract_placeholders(&command);
        Self {
            description: description.into(),
            command,
            placeholders,
        }
    }

    /// Extract placeholders from a command string.
    fn extract_placeholders(command: &str) -> Vec<Placeholder> {
        let mut placeholders = Vec::new();
        let mut remaining = command;

        while let Some(start) = remaining.find("{{") {
            if let Some(end) = remaining[start..].find("}}") {
                let raw = &remaining[start..start + end + 2];
                let inner = &remaining[start + 2..start + end];

                let (name, optional, alternatives) = if inner.starts_with('[') && inner.ends_with(']')
                {
                    // Optional with alternatives: {{[-s|--sort]}}
                    let inner_stripped = &inner[1..inner.len() - 1];
                    let alts: Vec<String> =
                        inner_stripped.split('|').map(|s| s.to_string()).collect();
                    let name = alts.first().cloned().unwrap_or_default();
                    (name, true, alts)
                } else if inner.contains('|') {
                    // Alternatives: {{option1|option2}}
                    let alts: Vec<String> = inner.split('|').map(|s| s.to_string()).collect();
                    let name = alts.first().cloned().unwrap_or_default();
                    (name, false, alts)
                } else {
                    // Simple placeholder: {{filename}}
                    (inner.to_string(), false, vec![])
                };

                placeholders.push(Placeholder {
                    raw: raw.to_string(),
                    name,
                    optional,
                    alternatives,
                });

                remaining = &remaining[start + end + 2..];
            } else {
                break;
            }
        }

        placeholders
    }

    /// Render the command with placeholders replaced by values.
    pub fn render(&self, values: &std::collections::HashMap<String, String>) -> String {
        let mut result = self.command.clone();
        for placeholder in &self.placeholders {
            if let Some(value) = values.get(&placeholder.name) {
                result = result.replace(&placeholder.raw, value);
            }
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_platform_current() {
        let platform = Platform::current();
        // Just verify it returns something valid
        assert!(!platform.as_dir_name().is_empty());
    }

    #[test]
    fn test_platform_search_order() {
        let order = Platform::Osx.search_order();
        assert_eq!(order[0], Platform::Osx);
        assert!(order.contains(&Platform::Common));
    }

    #[test]
    fn test_extract_simple_placeholder() {
        let example = TldrExample::new("List files", "ls {{directory}}");
        assert_eq!(example.placeholders.len(), 1);
        assert_eq!(example.placeholders[0].name, "directory");
        assert!(!example.placeholders[0].optional);
    }

    #[test]
    fn test_extract_optional_placeholder() {
        let example = TldrExample::new("Sort", "sort {{[-r|--reverse]}} {{file}}");
        assert_eq!(example.placeholders.len(), 2);
        assert!(example.placeholders[0].optional);
        assert_eq!(
            example.placeholders[0].alternatives,
            vec!["-r", "--reverse"]
        );
    }

    #[test]
    fn test_render_example() {
        let example = TldrExample::new("List directory", "ls {{directory}}");
        let mut values = std::collections::HashMap::new();
        values.insert("directory".to_string(), "/tmp".to_string());
        assert_eq!(example.render(&values), "ls /tmp");
    }

    #[test]
    fn test_page_as_context() {
        let mut page = TldrPage::new("git", Platform::Common);
        page.description = "Distributed version control system.".to_string();
        page.examples.push(TldrExample::new("Clone a repository", "git clone {{url}}"));

        let context = page.as_context();
        assert!(context.contains("TLDR for `git`"));
        assert!(context.contains("Clone a repository"));
    }
}
