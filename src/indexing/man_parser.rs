//! Man page parser for extracting structured content

use super::{CommandMetadata, ManPageDocument, ManSection};
use anyhow::{Context, Result};
use std::process::Command;
use std::path::{Path, PathBuf};
use chrono::Utc;
use tracing::{debug, warn};

/// Parse man page for a command into structured documents
pub fn parse_man_page(
    command: &str,
    os: &str,
    distro: &str,
) -> Result<Vec<ManPageDocument>> {
    debug!("Parsing man page for command: {}", command);

    // Get raw man page text
    let man_text = get_man_page_text(command)
        .with_context(|| format!("Failed to get man page for '{}'", command))?;

    // Extract sections
    let sections = extract_sections(&man_text)?;

    // Get command metadata
    let metadata = extract_metadata(command, os, distro)?;

    // Create documents for each section
    let documents: Vec<ManPageDocument> = sections
        .into_iter()
        .map(|(section, content)| ManPageDocument {
            command: command.to_string(),
            section,
            content,
            metadata: metadata.clone(),
        })
        .collect();

    Ok(documents)
}

/// Get raw man page text for a command
fn get_man_page_text(command: &str) -> Result<String> {
    // Try different methods to get man page

    // Method 1: man -P cat (works on most systems)
    if let Ok(output) = Command::new("man")
        .arg("-P")
        .arg("cat")
        .arg(command)
        .output()
    {
        if output.status.success() {
            let text = String::from_utf8_lossy(&output.stdout).to_string();
            if !text.trim().is_empty() {
                return Ok(text);
            }
        }
    }

    // Method 2: man command and capture output
    if let Ok(output) = Command::new("man").arg(command).output() {
        if output.status.success() {
            let text = String::from_utf8_lossy(&output.stdout).to_string();
            if !text.trim().is_empty() {
                return Ok(text);
            }
        }
    }

    // Method 3: Read from /usr/share/man (for offline/air-gapped systems)
    if let Some(text) = read_man_file_directly(command) {
        return Ok(text);
    }

    anyhow::bail!("Could not retrieve man page for '{}'", command)
}

/// Read man page directly from filesystem
fn read_man_file_directly(command: &str) -> Option<String> {
    let man_paths = vec![
        "/usr/share/man/man1",
        "/usr/share/man/man8",
        "/usr/local/share/man/man1",
        "/usr/local/share/man/man8",
    ];

    for base_path in man_paths {
        // Try uncompressed
        let path = PathBuf::from(base_path).join(format!("{}.1", command));
        if path.exists() {
            if let Ok(content) = std::fs::read_to_string(&path) {
                return Some(content);
            }
        }

        // Try gzipped
        let gz_path = PathBuf::from(base_path).join(format!("{}.1.gz", command));
        if gz_path.exists() {
            if let Some(content) = read_gzipped_file(&gz_path) {
                return Some(content);
            }
        }
    }

    None
}

/// Read gzipped man page
#[cfg(feature = "vector-store")]
fn read_gzipped_file(path: &Path) -> Option<String> {
    use flate2::read::GzDecoder;
    use std::io::Read;

    let file = std::fs::File::open(path).ok()?;
    let mut decoder = GzDecoder::new(file);
    let mut content = String::new();
    decoder.read_to_string(&mut content).ok()?;
    Some(content)
}

#[cfg(not(feature = "vector-store"))]
fn read_gzipped_file(_path: &Path) -> Option<String> {
    None
}

/// Extract sections from man page text
fn extract_sections(man_text: &str) -> Result<Vec<(ManSection, String)>> {
    let mut sections = Vec::new();
    let mut current_section: Option<ManSection> = None;
    let mut current_content = String::new();

    for line in man_text.lines() {
        // Check if this line is a section header
        // Section headers are typically all caps and at the start of a line
        let trimmed = line.trim();

        if is_section_header(trimmed) {
            // Save previous section
            if let Some(section) = current_section.take() {
                sections.push((section, current_content.trim().to_string()));
                current_content.clear();
            }

            // Start new section
            let section_name = trimmed.trim_end_matches(|c| c == ':' || c == '-');
            current_section = Some(ManSection::from_str(section_name));
        } else if current_section.is_some() {
            // Add to current section content
            current_content.push_str(line);
            current_content.push('\n');
        }
    }

    // Save last section
    if let Some(section) = current_section {
        sections.push((section, current_content.trim().to_string()));
    }

    if sections.is_empty() {
        // If no sections found, treat entire text as DESCRIPTION
        sections.push((ManSection::Description, man_text.trim().to_string()));
    }

    Ok(sections)
}

/// Check if a line is a section header
fn is_section_header(line: &str) -> bool {
    if line.is_empty() {
        return false;
    }

    // Section headers are typically:
    // 1. All uppercase
    // 2. 2-20 characters long
    // 3. At the start of the line
    // 4. May end with : or -

    let cleaned = line.trim_end_matches(|c| c == ':' || c == '-').trim();

    cleaned.len() >= 2
        && cleaned.len() <= 30
        && cleaned.chars().all(|c| c.is_uppercase() || c.is_whitespace())
        && cleaned.chars().any(|c| c.is_alphabetic())
}

/// Extract metadata for a command
fn extract_metadata(command: &str, os: &str, distro: &str) -> Result<CommandMetadata> {
    let version = get_command_version(command);
    let installed_path = get_command_path(command);
    let is_gnu = detect_gnu_command(command);
    let command_type = super::CommandType::classify(command);

    Ok(CommandMetadata {
        os: os.to_string(),
        distro: distro.to_string(),
        version,
        installed_path,
        indexed_at: Utc::now(),
        man_section: Some(1), // Most commands are section 1
        is_gnu,
        command_type,
    })
}

/// Get command version
fn get_command_version(command: &str) -> Option<String> {
    // Try common version flags
    for flag in &["--version", "-v", "-V", "version"] {
        if let Ok(output) = Command::new(command).arg(flag).output() {
            if output.status.success() {
                let version_text = String::from_utf8_lossy(&output.stdout);
                if let Some(first_line) = version_text.lines().next() {
                    if !first_line.trim().is_empty() {
                        return Some(first_line.trim().to_string());
                    }
                }
            }
        }
    }

    None
}

/// Get full path to command
fn get_command_path(command: &str) -> Option<PathBuf> {
    if let Ok(output) = Command::new("which").arg(command).output() {
        if output.status.success() {
            let path_str = String::from_utf8_lossy(&output.stdout);
            let path = path_str.trim();
            if !path.is_empty() {
                return Some(PathBuf::from(path));
            }
        }
    }

    None
}

/// Detect if command is a GNU utility
fn detect_gnu_command(command: &str) -> bool {
    if let Some(version) = get_command_version(command) {
        version.to_lowercase().contains("gnu")
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_section_header() {
        assert!(is_section_header("NAME"));
        assert!(is_section_header("SYNOPSIS"));
        assert!(is_section_header("DESCRIPTION"));
        assert!(is_section_header("OPTIONS:"));
        assert!(is_section_header("SEE ALSO-"));

        assert!(!is_section_header(""));
        assert!(!is_section_header("not a header"));
        assert!(!is_section_header("123"));
        assert!(!is_section_header("This is a sentence."));
    }

    #[test]
    fn test_extract_sections() {
        let man_text = r#"
NAME
       grep - file pattern searcher

SYNOPSIS
       grep [options] pattern [file...]

DESCRIPTION
       The grep utility searches files for patterns.

OPTIONS
       -i      Ignore case
       -r      Recursive search
"#;

        let sections = extract_sections(man_text).unwrap();
        assert!(sections.len() >= 4);

        let section_names: Vec<_> = sections.iter().map(|(s, _)| s.as_str()).collect();
        assert!(section_names.contains(&"NAME"));
        assert!(section_names.contains(&"SYNOPSIS"));
        assert!(section_names.contains(&"DESCRIPTION"));
        assert!(section_names.contains(&"OPTIONS"));
    }

    #[test]
    fn test_get_command_path() {
        // Test with a command that should exist on most systems
        let path = get_command_path("ls");
        assert!(path.is_some());
        if let Some(p) = path {
            assert!(p.to_string_lossy().contains("ls"));
        }
    }
}
