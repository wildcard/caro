//! Command Template Library for Shell Command Generation
//!
//! This module provides a library of command templates organized by intent category
//! and filtered by platform capability profile. Templates are used by the SmolLM
//! prompt system to generate appropriate commands for each platform.
//!
//! # Template Categories
//!
//! - **listing**: Basic file/directory listing (ls)
//! - **filtering**: Finding files by criteria (find)
//! - **text_search**: Searching file contents (grep)
//! - **ranking**: Top N by size/time (find/ls + sort + head)
//! - **counting**: Counting files/lines (wc)
//! - **disk**: Disk usage analysis (du, df)
//! - **process**: Process management (ps, kill)
//! - **network**: Network operations (netstat, ss, curl)
//! - **archive**: Archive operations (tar, gzip)
//! - **permissions**: File permissions (chmod, chown)
//!
//! # Example
//!
//! ```rust
//! use caro::prompts::command_templates::TemplateLibrary;
//! use caro::prompts::capability_profile::CapabilityProfile;
//!
//! let profile = CapabilityProfile::ubuntu();
//! let library = TemplateLibrary::for_profile(&profile);
//!
//! // Get templates for listing files
//! let listing_templates = library.templates_for_category("listing");
//! ```

use super::capability_profile::{CapabilityProfile, ProfileType, StatFormat};

/// A command template with intent pattern and command
#[derive(Debug, Clone)]
pub struct CommandTemplate {
    /// Category of the command (listing, filtering, etc.)
    pub category: String,
    /// Natural language intent pattern this template matches
    pub intent_pattern: String,
    /// The command template (may contain placeholders like {path}, {pattern})
    pub command_template: String,
    /// Description of what this command does
    pub description: String,
    /// Whether this template requires confirmation (destructive)
    pub requires_confirmation: bool,
    /// Minimum profile requirements (features needed)
    pub required_features: Vec<String>,
}

impl CommandTemplate {
    fn new(
        category: impl Into<String>,
        intent: impl Into<String>,
        command: impl Into<String>,
        description: impl Into<String>,
    ) -> Self {
        Self {
            category: category.into(),
            intent_pattern: intent.into(),
            command_template: command.into(),
            description: description.into(),
            requires_confirmation: false,
            required_features: Vec::new(),
        }
    }

    fn destructive(mut self) -> Self {
        self.requires_confirmation = true;
        self
    }

    fn requires(mut self, feature: impl Into<String>) -> Self {
        self.required_features.push(feature.into());
        self
    }
}

/// Library of command templates organized by profile
pub struct TemplateLibrary {
    templates: Vec<CommandTemplate>,
}

impl TemplateLibrary {
    /// Create a template library for the given capability profile
    pub fn for_profile(profile: &CapabilityProfile) -> Self {
        let mut templates = Vec::new();

        // Add common templates that work on all platforms
        templates.extend(Self::common_templates());

        // Add profile-specific templates
        match profile.profile_type {
            ProfileType::GnuLinux => {
                templates.extend(Self::gnu_templates());
            }
            ProfileType::Bsd => {
                templates.extend(Self::bsd_templates());
            }
            ProfileType::Busybox => {
                templates.extend(Self::busybox_templates());
            }
            _ => {
                // Default to POSIX templates
                templates.extend(Self::posix_templates());
            }
        }

        // Filter templates based on available features
        templates = templates
            .into_iter()
            .filter(|t| Self::template_compatible(t, profile))
            .collect();

        Self { templates }
    }

    /// Get all templates
    pub fn all_templates(&self) -> &[CommandTemplate] {
        &self.templates
    }

    /// Get templates for a specific category
    pub fn templates_for_category(&self, category: &str) -> Vec<&CommandTemplate> {
        self.templates
            .iter()
            .filter(|t| t.category == category)
            .collect()
    }

    /// Find template matching an intent pattern
    pub fn find_template(&self, intent: &str) -> Option<&CommandTemplate> {
        let intent_lower = intent.to_lowercase();
        self.templates
            .iter()
            .find(|t| intent_lower.contains(&t.intent_pattern.to_lowercase()))
    }

    fn template_compatible(template: &CommandTemplate, profile: &CapabilityProfile) -> bool {
        for feature in &template.required_features {
            match feature.as_str() {
                "find_printf" if !profile.find_printf => return false,
                "sort_h" if !profile.sort_h => return false,
                "xargs_0" if !profile.xargs_0 => return false,
                "grep_r" if !profile.grep_r => return false,
                "grep_p" if !profile.grep_p => return false,
                "stat_gnu" if profile.stat_format != StatFormat::Gnu => return false,
                "stat_bsd" if profile.stat_format != StatFormat::Bsd => return false,
                "sed_gnu" if !profile.sed_inplace_gnu => return false,
                "du_max_depth" if !profile.du_max_depth => return false,
                "date_gnu" if !profile.date_gnu_format => return false,
                "ps_sort" if !profile.ps_sort => return false,
                "ls_sort" if !profile.ls_sort => return false,
                _ => {}
            }
        }
        true
    }

    /// Templates that work on all POSIX systems
    fn common_templates() -> Vec<CommandTemplate> {
        vec![
            // Listing
            CommandTemplate::new(
                "listing",
                "list files",
                "ls -la",
                "List all files with details",
            ),
            CommandTemplate::new(
                "listing",
                "list all files",
                "ls -a",
                "List all files including hidden",
            ),
            CommandTemplate::new(
                "listing",
                "list directories only",
                "ls -d */",
                "List only directories",
            ),
            // Basic find
            CommandTemplate::new(
                "filtering",
                "find files by name",
                "find . -name '{pattern}' -type f",
                "Find files matching pattern",
            ),
            CommandTemplate::new(
                "filtering",
                "find directories by name",
                "find . -name '{pattern}' -type d",
                "Find directories matching pattern",
            ),
            CommandTemplate::new(
                "filtering",
                "find files by extension",
                "find . -name '*.{ext}' -type f",
                "Find files with extension",
            ),
            CommandTemplate::new(
                "filtering",
                "find empty files",
                "find . -type f -empty",
                "Find empty files",
            ),
            CommandTemplate::new(
                "filtering",
                "find empty directories",
                "find . -type d -empty",
                "Find empty directories",
            ),
            CommandTemplate::new(
                "filtering",
                "find files larger than",
                "find . -type f -size +{size}",
                "Find files larger than size (e.g., +100M)",
            ),
            CommandTemplate::new(
                "filtering",
                "find files smaller than",
                "find . -type f -size -{size}",
                "Find files smaller than size",
            ),
            CommandTemplate::new(
                "filtering",
                "find files modified today",
                "find . -type f -mtime 0",
                "Find files modified in last 24 hours",
            ),
            CommandTemplate::new(
                "filtering",
                "find files modified in last N days",
                "find . -type f -mtime -{days}",
                "Find files modified within N days",
            ),
            // Text search
            CommandTemplate::new(
                "text_search",
                "search for text in file",
                "grep -n '{pattern}' {file}",
                "Search for pattern in file",
            ),
            CommandTemplate::new(
                "text_search",
                "search case insensitive",
                "grep -in '{pattern}' {file}",
                "Case-insensitive search",
            ),
            // Counting
            CommandTemplate::new(
                "counting",
                "count lines in file",
                "wc -l {file}",
                "Count lines in file",
            ),
            CommandTemplate::new(
                "counting",
                "count files",
                "find . -type f | wc -l",
                "Count files in directory",
            ),
            CommandTemplate::new(
                "counting",
                "count directories",
                "find . -type d | wc -l",
                "Count directories",
            ),
            // Disk
            CommandTemplate::new(
                "disk",
                "disk usage",
                "du -sh .",
                "Show disk usage of current directory",
            ),
            CommandTemplate::new("disk", "disk free space", "df -h", "Show disk free space"),
            // Process
            CommandTemplate::new("process", "list processes", "ps aux", "List all processes"),
            CommandTemplate::new(
                "process",
                "find process by name",
                "ps aux | grep '{name}'",
                "Find process by name",
            ),
            CommandTemplate::new(
                "process",
                "kill process by name",
                "pkill '{name}'",
                "Kill processes by name",
            )
            .destructive(),
            // Archive
            CommandTemplate::new(
                "archive",
                "create tar archive",
                "tar -cvf {archive}.tar {directory}",
                "Create tar archive",
            ),
            CommandTemplate::new(
                "archive",
                "extract tar archive",
                "tar -xvf {archive}.tar",
                "Extract tar archive",
            ),
            CommandTemplate::new(
                "archive",
                "create compressed archive",
                "tar -czvf {archive}.tar.gz {directory}",
                "Create gzipped tar archive",
            ),
            CommandTemplate::new(
                "archive",
                "extract compressed archive",
                "tar -xzvf {archive}.tar.gz",
                "Extract gzipped tar archive",
            ),
            // Network
            CommandTemplate::new(
                "network",
                "download file",
                "curl -O {url}",
                "Download file from URL",
            ),
            CommandTemplate::new("network", "fetch url", "curl -s {url}", "Fetch URL content"),
            // Permissions (destructive)
            CommandTemplate::new(
                "permissions",
                "make executable",
                "chmod +x {file}",
                "Make file executable",
            )
            .destructive(),
            CommandTemplate::new(
                "permissions",
                "change permissions",
                "chmod {mode} {file}",
                "Change file permissions",
            )
            .destructive(),
        ]
    }

    /// Templates specific to GNU/Linux systems
    fn gnu_templates() -> Vec<CommandTemplate> {
        vec![
            // Listing with GNU extensions
            CommandTemplate::new(
                "listing",
                "list files by size",
                "ls -lhS",
                "List files sorted by size (largest first)",
            ),
            CommandTemplate::new(
                "listing",
                "list files by time",
                "ls -lht",
                "List files sorted by modification time",
            ),
            // Advanced find with -printf
            CommandTemplate::new(
                "ranking",
                "largest files",
                "find . -type f -printf '%s %p\\n' | sort -nr | head -n {count}",
                "Find N largest files",
            )
            .requires("find_printf"),
            CommandTemplate::new(
                "ranking",
                "newest files",
                "find . -type f -printf '%T@ %p\\n' | sort -nr | head -n {count} | cut -d' ' -f2-",
                "Find N most recently modified files",
            )
            .requires("find_printf"),
            CommandTemplate::new(
                "ranking",
                "oldest files",
                "find . -type f -printf '%T@ %p\\n' | sort -n | head -n {count} | cut -d' ' -f2-",
                "Find N oldest files",
            )
            .requires("find_printf"),
            // Recursive grep
            CommandTemplate::new(
                "text_search",
                "search recursively",
                "grep -Rn '{pattern}' .",
                "Recursive text search",
            )
            .requires("grep_r"),
            CommandTemplate::new(
                "text_search",
                "search with perl regex",
                "grep -Pn '{pattern}' {file}",
                "Search with Perl regex",
            )
            .requires("grep_p"),
            CommandTemplate::new(
                "text_search",
                "find files containing",
                "grep -Rl '{pattern}' .",
                "Find files containing pattern",
            )
            .requires("grep_r"),
            // Disk usage
            CommandTemplate::new(
                "disk",
                "directory sizes",
                "du -h --max-depth=1 | sort -hr",
                "Show directory sizes sorted",
            )
            .requires("du_max_depth")
            .requires("sort_h"),
            CommandTemplate::new(
                "disk",
                "largest directories",
                "du -h --max-depth=1 | sort -hr | head -n {count}",
                "Find N largest directories",
            )
            .requires("du_max_depth")
            .requires("sort_h"),
            // Date operations
            CommandTemplate::new(
                "filtering",
                "files modified in last week",
                "find . -type f -newermt \"$(date --date='7 days ago' +%Y-%m-%d)\"",
                "Find files modified in last 7 days",
            )
            .requires("date_gnu"),
            // Process with sort
            CommandTemplate::new(
                "process",
                "top cpu processes",
                "ps aux --sort=-%cpu | head -n {count}",
                "Show top N CPU-consuming processes",
            )
            .requires("ps_sort"),
            CommandTemplate::new(
                "process",
                "top memory processes",
                "ps aux --sort=-%mem | head -n {count}",
                "Show top N memory-consuming processes",
            )
            .requires("ps_sort"),
            // Network (Linux-specific)
            CommandTemplate::new(
                "network",
                "listening ports",
                "ss -tuln",
                "Show listening TCP/UDP ports",
            ),
            CommandTemplate::new(
                "network",
                "established connections",
                "ss -tun state established",
                "Show established connections",
            ),
            // Safe filename handling
            CommandTemplate::new(
                "filtering",
                "find and process files safely",
                "find . -type f -name '{pattern}' -print0 | xargs -0 {command}",
                "Find files and process with safe filename handling",
            )
            .requires("xargs_0"),
            // Stat
            CommandTemplate::new(
                "listing",
                "file details",
                "stat -c 'Name: %n\\nSize: %s\\nModified: %y' {file}",
                "Show detailed file information",
            )
            .requires("stat_gnu"),
            // Sed in-place
            CommandTemplate::new(
                "text_search",
                "replace in file",
                "sed -i 's/{old}/{new}/g' {file}",
                "Replace text in file",
            )
            .requires("sed_gnu")
            .destructive(),
        ]
    }

    /// Templates specific to BSD/macOS systems
    fn bsd_templates() -> Vec<CommandTemplate> {
        vec![
            // Listing
            CommandTemplate::new(
                "listing",
                "list files by size",
                "ls -lhS",
                "List files sorted by size",
            ),
            CommandTemplate::new(
                "listing",
                "list files by time",
                "ls -lht",
                "List files sorted by time",
            ),
            // Find with stat for metadata (no -printf)
            CommandTemplate::new(
                "ranking",
                "largest files",
                "find . -type f -exec stat -f '%z %N' {} + | sort -nr | head -n {count}",
                "Find N largest files",
            )
            .requires("stat_bsd"),
            CommandTemplate::new(
                "ranking",
                "newest files",
                "find . -type f -exec stat -f '%m %N' {} + | sort -nr | head -n {count}",
                "Find N newest files",
            )
            .requires("stat_bsd"),
            // Disk usage (BSD syntax)
            CommandTemplate::new(
                "disk",
                "directory sizes",
                "du -h -d 1 | sort -nr",
                "Show directory sizes (depth 1)",
            ),
            CommandTemplate::new(
                "disk",
                "largest directories",
                "du -h -d 1 | sort -nr | head -n {count}",
                "Find N largest directories",
            ),
            // Date (BSD syntax)
            CommandTemplate::new(
                "filtering",
                "files modified in last week",
                "find . -type f -mtime -7",
                "Find files modified in last 7 days",
            ),
            // Recursive grep
            CommandTemplate::new(
                "text_search",
                "search recursively",
                "grep -Rn '{pattern}' .",
                "Recursive text search",
            )
            .requires("grep_r"),
            CommandTemplate::new(
                "text_search",
                "find files containing",
                "grep -Rl '{pattern}' .",
                "Find files containing pattern",
            )
            .requires("grep_r"),
            // Process (no --sort)
            CommandTemplate::new(
                "process",
                "top cpu processes",
                "ps aux | sort -nrk 3 | head -n {count}",
                "Show top N CPU processes",
            ),
            CommandTemplate::new(
                "process",
                "top memory processes",
                "ps aux | sort -nrk 4 | head -n {count}",
                "Show top N memory processes",
            ),
            // Network (BSD/macOS)
            CommandTemplate::new(
                "network",
                "listening ports",
                "lsof -iTCP -sTCP:LISTEN -P",
                "Show listening TCP ports",
            ),
            CommandTemplate::new(
                "network",
                "network connections",
                "netstat -an",
                "Show all network connections",
            ),
            // Stat (BSD format)
            CommandTemplate::new(
                "listing",
                "file details",
                "stat -f 'Name: %N%nSize: %z%nModified: %Sm' {file}",
                "Show detailed file information",
            )
            .requires("stat_bsd"),
            // Sed in-place (BSD requires '')
            CommandTemplate::new(
                "text_search",
                "replace in file",
                "sed -i '' 's/{old}/{new}/g' {file}",
                "Replace text in file (BSD)",
            )
            .destructive(),
            // Safe filename handling
            CommandTemplate::new(
                "filtering",
                "find and process files safely",
                "find . -type f -name '{pattern}' -print0 | xargs -0 {command}",
                "Find files and process safely",
            )
            .requires("xargs_0"),
        ]
    }

    /// Templates for BusyBox environments (minimal feature set)
    fn busybox_templates() -> Vec<CommandTemplate> {
        vec![
            // Simplified listings
            CommandTemplate::new(
                "listing",
                "list files by size",
                "ls -lS",
                "List files sorted by size",
            ),
            // Simple find
            CommandTemplate::new(
                "ranking",
                "largest files",
                "find . -type f | xargs ls -lS | head -n {count}",
                "Find largest files",
            ),
            // Recursive grep (if available)
            CommandTemplate::new(
                "text_search",
                "search recursively",
                "grep -r '{pattern}' .",
                "Recursive search",
            ),
            // Disk usage (simple)
            CommandTemplate::new(
                "disk",
                "directory sizes",
                "du -h -d 1",
                "Show directory sizes",
            ),
            // Process (simple)
            CommandTemplate::new(
                "process",
                "top processes",
                "ps | head -n {count}",
                "Show processes",
            ),
        ]
    }

    /// POSIX-compliant templates for unknown environments
    fn posix_templates() -> Vec<CommandTemplate> {
        vec![
            CommandTemplate::new("listing", "list files", "ls -la", "List files"),
            CommandTemplate::new(
                "filtering",
                "find files",
                "find . -name '{pattern}'",
                "Find files by name",
            ),
            CommandTemplate::new(
                "text_search",
                "search in file",
                "grep '{pattern}' {file}",
                "Search for pattern",
            ),
            CommandTemplate::new("counting", "count lines", "wc -l {file}", "Count lines"),
            CommandTemplate::new("disk", "disk usage", "du -s", "Show disk usage"),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gnu_library() {
        let profile = CapabilityProfile::ubuntu();
        let library = TemplateLibrary::for_profile(&profile);

        // Should have find_printf templates
        let ranking = library.templates_for_category("ranking");
        assert!(!ranking.is_empty());
        assert!(ranking
            .iter()
            .any(|t| t.command_template.contains("-printf")));
    }

    #[test]
    fn test_bsd_library() {
        let profile = CapabilityProfile::for_platform(ProfileType::Bsd);
        let library = TemplateLibrary::for_profile(&profile);

        // Should NOT have find_printf templates
        let ranking = library.templates_for_category("ranking");
        assert!(!ranking.is_empty());
        assert!(ranking
            .iter()
            .all(|t| !t.command_template.contains("-printf")));
    }

    #[test]
    fn test_template_filtering() {
        let mut profile = CapabilityProfile::ubuntu();
        profile.find_printf = false;

        let library = TemplateLibrary::for_profile(&profile);

        // Should not include templates requiring find_printf
        let ranking = library.templates_for_category("ranking");
        for template in ranking {
            assert!(
                !template
                    .required_features
                    .contains(&"find_printf".to_string()),
                "Should not include find_printf template"
            );
        }
    }

    #[test]
    fn test_destructive_templates() {
        let profile = CapabilityProfile::ubuntu();
        let library = TemplateLibrary::for_profile(&profile);

        let permissions = library.templates_for_category("permissions");
        assert!(permissions.iter().all(|t| t.requires_confirmation));
    }

    #[test]
    fn test_find_template() {
        let profile = CapabilityProfile::ubuntu();
        let library = TemplateLibrary::for_profile(&profile);

        let template = library.find_template("list all files");
        assert!(template.is_some());
        assert!(template.unwrap().command_template.contains("ls"));
    }
}
