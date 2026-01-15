//! TLDR page parser for converting markdown to structured data.
//!
//! Parses TLDR pages following the tldr-pages specification:
//! - Title: `# command_name`
//! - Description: `> One-line description`
//! - Notes: Additional `>` lines
//! - Examples: `- Description:` followed by `` `command` ``

use super::types::{Platform, TldrExample, TldrPage};
use thiserror::Error;

/// Errors that can occur during TLDR page parsing.
#[derive(Debug, Error)]
pub enum ParseError {
    #[error("Missing command title (expected '# command')")]
    MissingTitle,

    #[error("Missing description (expected '> description')")]
    MissingDescription,

    #[error("Invalid page format: {0}")]
    InvalidFormat(String),

    #[error("Empty page content")]
    EmptyContent,
}

/// Parser for TLDR markdown pages.
pub struct TldrParser;

impl TldrParser {
    /// Parse a TLDR markdown page into a structured TldrPage.
    ///
    /// # Arguments
    /// * `content` - The raw markdown content of the TLDR page
    /// * `platform` - The platform this page is for
    /// * `language` - The language/locale of the page (default: "en")
    ///
    /// # Returns
    /// A parsed TldrPage or a ParseError if the format is invalid.
    pub fn parse(content: &str, platform: Platform, language: &str) -> Result<TldrPage, ParseError> {
        let content = content.trim();
        if content.is_empty() {
            return Err(ParseError::EmptyContent);
        }

        let mut lines = content.lines().peekable();
        let mut page = TldrPage::new("", platform);
        page.language = language.to_string();

        // Parse title: # command_name
        while let Some(line) = lines.peek() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                lines.next();
                continue;
            }
            if let Some(title) = trimmed.strip_prefix("# ") {
                page.name = title.trim().to_string();
                lines.next();
                break;
            } else {
                return Err(ParseError::MissingTitle);
            }
        }

        if page.name.is_empty() {
            return Err(ParseError::MissingTitle);
        }

        // Skip empty lines
        while let Some(line) = lines.peek() {
            if line.trim().is_empty() {
                lines.next();
            } else {
                break;
            }
        }

        // Parse description and notes: > lines
        let mut found_description = false;
        while let Some(line) = lines.peek() {
            let trimmed = line.trim();
            if let Some(note) = trimmed.strip_prefix("> ") {
                if !found_description {
                    page.description = Self::clean_description(note);
                    found_description = true;
                } else {
                    // Additional notes
                    let cleaned = Self::clean_description(note);
                    if !cleaned.is_empty() {
                        page.notes.push(cleaned);
                    }
                }
                lines.next();
            } else if trimmed.starts_with('>') && trimmed.len() == 1 {
                // Empty blockquote line, skip
                lines.next();
            } else {
                break;
            }
        }

        if !found_description {
            return Err(ParseError::MissingDescription);
        }

        // Parse examples
        let mut current_description: Option<String> = None;

        for line in lines {
            let trimmed = line.trim();

            if trimmed.is_empty() {
                continue;
            }

            // Example description: - Description:
            if let Some(desc) = trimmed.strip_prefix("- ") {
                current_description = Some(Self::clean_example_description(desc));
            }
            // Command: `command here`
            else if trimmed.starts_with('`') && trimmed.ends_with('`') && trimmed.len() > 2 {
                let command = &trimmed[1..trimmed.len() - 1];
                let description = current_description.take().unwrap_or_default();
                page.examples.push(TldrExample::new(description, command));
            }
            // Inline code might span multiple lines in rare cases, handle single backtick start
            else if trimmed.starts_with('`') {
                // Single backtick start without end - treat as command anyway
                let command = trimmed.trim_start_matches('`').trim_end_matches('`');
                let description = current_description.take().unwrap_or_default();
                if !command.is_empty() {
                    page.examples.push(TldrExample::new(description, command));
                }
            }
        }

        Ok(page)
    }

    /// Parse a page from a file path, extracting metadata from the path.
    ///
    /// Expected path format: `pages[.lang]/platform/command.md`
    pub fn parse_from_path(
        content: &str,
        path: &std::path::Path,
    ) -> Result<TldrPage, ParseError> {
        // Extract platform from parent directory
        let platform = path
            .parent()
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str())
            .and_then(Platform::from_dir_name)
            .unwrap_or(Platform::Common);

        // Extract language from pages directory
        let language = path
            .iter()
            .find_map(|component| {
                let s = component.to_str()?;
                if s.starts_with("pages.") {
                    Some(s.strip_prefix("pages.")?.to_string())
                } else if s == "pages" {
                    Some("en".to_string())
                } else {
                    None
                }
            })
            .unwrap_or_else(|| "en".to_string());

        Self::parse(content, platform, &language)
    }

    /// Clean up a description line.
    fn clean_description(desc: &str) -> String {
        let desc = desc.trim();

        // Remove trailing "More information: <url>" links
        if desc.starts_with("More information:") {
            return String::new();
        }

        // Remove markdown links but keep the text
        let mut result = String::new();
        let mut chars = desc.chars().peekable();

        while let Some(c) = chars.next() {
            if c == '[' {
                // Collect link text
                let mut link_text = String::new();
                while let Some(&next) = chars.peek() {
                    if next == ']' {
                        chars.next();
                        break;
                    }
                    link_text.push(chars.next().unwrap());
                }
                // Skip the URL part (...)
                if chars.peek() == Some(&'(') {
                    chars.next();
                    while let Some(&next) = chars.peek() {
                        if next == ')' {
                            chars.next();
                            break;
                        }
                        chars.next();
                    }
                }
                result.push_str(&link_text);
            } else if c == '<' {
                // Skip bare URLs in angle brackets
                while let Some(&next) = chars.peek() {
                    if next == '>' {
                        chars.next();
                        break;
                    }
                    chars.next();
                }
            } else {
                result.push(c);
            }
        }

        result.trim().to_string()
    }

    /// Clean up an example description.
    fn clean_example_description(desc: &str) -> String {
        let desc = desc.trim();
        // Remove trailing colon
        desc.trim_end_matches(':').trim().to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_PAGE: &str = r#"# git

> Distributed version control system.
> More information: <https://git-scm.com>.

- Clone a repository:

`git clone {{repository_url}}`

- Initialize a new repository:

`git init`

- Show the working tree status:

`git status`

- Add files to staging:

`git add {{file1}} {{file2}}`
"#;

    #[test]
    fn test_parse_sample_page() {
        let page = TldrParser::parse(SAMPLE_PAGE, Platform::Common, "en").unwrap();

        assert_eq!(page.name, "git");
        assert_eq!(page.description, "Distributed version control system.");
        assert_eq!(page.examples.len(), 4);

        assert_eq!(page.examples[0].description, "Clone a repository");
        assert_eq!(page.examples[0].command, "git clone {{repository_url}}");

        assert_eq!(page.examples[1].description, "Initialize a new repository");
        assert_eq!(page.examples[1].command, "git init");
    }

    #[test]
    fn test_parse_extracts_placeholders() {
        let page = TldrParser::parse(SAMPLE_PAGE, Platform::Common, "en").unwrap();

        let clone_example = &page.examples[0];
        assert_eq!(clone_example.placeholders.len(), 1);
        assert_eq!(clone_example.placeholders[0].name, "repository_url");

        let add_example = &page.examples[3];
        assert_eq!(add_example.placeholders.len(), 2);
    }

    #[test]
    fn test_parse_missing_title() {
        let content = "> Some description\n\n- Example:\n\n`command`";
        let result = TldrParser::parse(content, Platform::Common, "en");
        assert!(matches!(result, Err(ParseError::MissingTitle)));
    }

    #[test]
    fn test_parse_missing_description() {
        let content = "# command\n\n- Example:\n\n`command`";
        let result = TldrParser::parse(content, Platform::Common, "en");
        assert!(matches!(result, Err(ParseError::MissingDescription)));
    }

    #[test]
    fn test_parse_empty_content() {
        let result = TldrParser::parse("", Platform::Common, "en");
        assert!(matches!(result, Err(ParseError::EmptyContent)));
    }

    #[test]
    fn test_clean_description_with_link() {
        let desc = "See [mnemonic](https://example.com) for reference.";
        let cleaned = TldrParser::clean_description(desc);
        assert_eq!(cleaned, "See mnemonic for reference.");
    }

    #[test]
    fn test_parse_from_path() {
        use std::path::Path;

        let content = "# curl\n\n> Transfer data.\n\n- Fetch URL:\n\n`curl {{url}}`";

        let path = Path::new("pages.es/linux/curl.md");
        let page = TldrParser::parse_from_path(content, path).unwrap();

        assert_eq!(page.name, "curl");
        assert_eq!(page.platform, Platform::Linux);
        assert_eq!(page.language, "es");
    }
}
