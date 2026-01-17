//! Pro tips for effective command generation
//!
//! Provides helpful tips to users about how to get better results from caro,
//! displayed during inference or after command generation.

use rand::prelude::IndexedRandom;

/// A helpful tip for using caro more effectively
#[derive(Debug, Clone)]
pub struct Tip {
    /// Short category name
    pub category: &'static str,
    /// The tip content
    pub content: &'static str,
    /// Optional example input
    pub example: Option<&'static str>,
}

/// Collection of tips for different usage scenarios
pub struct TipCollection {
    tips: Vec<Tip>,
}

impl Default for TipCollection {
    fn default() -> Self {
        Self::new()
    }
}

impl TipCollection {
    /// Create a new tip collection with all built-in tips
    pub fn new() -> Self {
        Self {
            tips: vec![
                // Be specific tips
                Tip {
                    category: "Precision",
                    content: "Be specific about file types: 'find all .rs files' works better than 'find files'",
                    example: Some("find all rust files modified today"),
                },
                Tip {
                    category: "Precision",
                    content: "Include size limits when working with large datasets: 'show first 10 lines'",
                    example: Some("show first 20 lines of error.log"),
                },
                Tip {
                    category: "Precision",
                    content: "Specify the target explicitly: 'in current directory' vs 'recursively'",
                    example: Some("list files in current directory only"),
                },

                // Context tips
                Tip {
                    category: "Context",
                    content: "Mention the OS if you need platform-specific behavior: 'on macOS...'",
                    example: Some("on macOS, open this folder in finder"),
                },
                Tip {
                    category: "Context",
                    content: "Include file paths when relevant: 'in ~/projects/'",
                    example: Some("count lines in all python files in ~/projects/"),
                },
                Tip {
                    category: "Context",
                    content: "Mention the tool if you prefer it: 'using git...' or 'with ripgrep...'",
                    example: Some("using ripgrep, find TODO comments"),
                },

                // Safety tips
                Tip {
                    category: "Safety",
                    content: "Use --dry-run to see the command without executing: caro -n 'your request'",
                    example: None,
                },
                Tip {
                    category: "Safety",
                    content: "Add 'safely' or 'without deleting' to avoid destructive operations",
                    example: Some("safely clean up node_modules"),
                },
                Tip {
                    category: "Safety",
                    content: "Use -y/--confirm to auto-confirm safe commands in scripts",
                    example: None,
                },

                // Power user tips
                Tip {
                    category: "Power",
                    content: "Chain requests: 'find large files AND sort by size'",
                    example: Some("find files over 100MB and sort by size descending"),
                },
                Tip {
                    category: "Power",
                    content: "Use -x/--execute for immediate execution of generated commands",
                    example: None,
                },
                Tip {
                    category: "Power",
                    content: "Pipe output: caro 'list files' | other-command",
                    example: None,
                },
                Tip {
                    category: "Power",
                    content: "Request specific output formats: 'as JSON' or 'one per line'",
                    example: Some("list running processes as json"),
                },

                // Shell integration tips
                Tip {
                    category: "Shell",
                    content: "Set up shell integration for seamless editing: eval \"$(caro init zsh)\"",
                    example: None,
                },
                Tip {
                    category: "Shell",
                    content: "Use 'Edit' option to modify commands before running",
                    example: None,
                },

                // Query optimization tips
                Tip {
                    category: "Query",
                    content: "Describe the result, not the steps: 'files changed today' not 'use find with mtime'",
                    example: Some("files I modified in the last hour"),
                },
                Tip {
                    category: "Query",
                    content: "Include constraints: 'excluding hidden files' or 'only in src/'",
                    example: Some("find TODO comments excluding vendor/"),
                },
                Tip {
                    category: "Query",
                    content: "Specify units for sizes and times: 'files over 1GB', 'modified in last 2 hours'",
                    example: Some("delete files older than 30 days"),
                },
            ],
        }
    }

    /// Get a random tip
    pub fn random(&self) -> &Tip {
        self.tips
            .choose(&mut rand::rng())
            .expect("Tips collection should never be empty")
    }

    /// Get a random tip from a specific category
    pub fn random_from_category(&self, category: &str) -> Option<&Tip> {
        let category_tips: Vec<_> = self
            .tips
            .iter()
            .filter(|t| t.category.eq_ignore_ascii_case(category))
            .collect();

        category_tips.choose(&mut rand::rng()).copied()
    }

    /// Get all tips from a category
    pub fn by_category(&self, category: &str) -> Vec<&Tip> {
        self.tips
            .iter()
            .filter(|t| t.category.eq_ignore_ascii_case(category))
            .collect()
    }

    /// Get total number of tips
    pub fn len(&self) -> usize {
        self.tips.len()
    }

    /// Check if collection is empty
    pub fn is_empty(&self) -> bool {
        self.tips.is_empty()
    }

    /// Get all unique categories
    pub fn categories(&self) -> Vec<&str> {
        let mut cats: Vec<_> = self.tips.iter().map(|t| t.category).collect();
        cats.sort();
        cats.dedup();
        cats
    }
}

/// Format a tip for display
pub fn format_tip(tip: &Tip) -> String {
    let mut output = format!("[{}] {}", tip.category, tip.content);
    if let Some(example) = tip.example {
        output.push_str(&format!("\n  Example: caro '{}'", example));
    }
    output
}

/// Format a tip for short display (no example)
pub fn format_tip_short(tip: &Tip) -> String {
    tip.content.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tip_collection_not_empty() {
        let tips = TipCollection::new();
        assert!(!tips.is_empty());
        assert!(tips.len() > 10);
    }

    #[test]
    fn test_random_tip() {
        let tips = TipCollection::new();
        let tip = tips.random();
        assert!(!tip.content.is_empty());
    }

    #[test]
    fn test_category_tips() {
        let tips = TipCollection::new();
        let safety_tips = tips.by_category("Safety");
        assert!(!safety_tips.is_empty());
        for tip in safety_tips {
            assert_eq!(tip.category, "Safety");
        }
    }

    #[test]
    fn test_random_from_category() {
        let tips = TipCollection::new();
        let tip = tips.random_from_category("Power");
        assert!(tip.is_some());
        assert_eq!(tip.unwrap().category, "Power");
    }

    #[test]
    fn test_nonexistent_category() {
        let tips = TipCollection::new();
        let tip = tips.random_from_category("NonExistent");
        assert!(tip.is_none());
    }

    #[test]
    fn test_format_tip() {
        let tip = Tip {
            category: "Test",
            content: "This is a test tip",
            example: Some("test example"),
        };
        let formatted = format_tip(&tip);
        assert!(formatted.contains("[Test]"));
        assert!(formatted.contains("This is a test tip"));
        assert!(formatted.contains("Example:"));
    }

    #[test]
    fn test_format_tip_no_example() {
        let tip = Tip {
            category: "Test",
            content: "Tip without example",
            example: None,
        };
        let formatted = format_tip(&tip);
        assert!(formatted.contains("Tip without example"));
        assert!(!formatted.contains("Example:"));
    }

    #[test]
    fn test_categories() {
        let tips = TipCollection::new();
        let cats = tips.categories();
        assert!(cats.contains(&"Safety"));
        assert!(cats.contains(&"Power"));
        assert!(cats.contains(&"Precision"));
    }
}
