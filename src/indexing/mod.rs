//! Man page indexing module for ChromaDB/Qdrant vector store
//!
//! This module provides functionality to:
//! - Scan system for installed commands
//! - Parse man pages into structured sections
//! - Extract metadata (version, installation path, etc.)
//! - Batch index commands for distribution-specific indexes

pub mod man_parser;
pub mod command_scanner;
pub mod batch_indexer;

use std::collections::HashMap;
use std::path::PathBuf;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use anyhow::Result;

/// Represents a parsed man page document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManPageDocument {
    /// Command name (e.g., "grep", "find", "sed")
    pub command: String,

    /// Man section (e.g., NAME, SYNOPSIS, DESCRIPTION, OPTIONS, EXAMPLES)
    pub section: ManSection,

    /// Text content of this section
    pub content: String,

    /// Metadata about the command
    pub metadata: CommandMetadata,
}

/// Man page sections
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ManSection {
    Name,
    Synopsis,
    Description,
    Options,
    Examples,
    SeeAlso,
    Author,
    Bugs,
    Copyright,
    Other(String),
}

impl ManSection {
    pub fn as_str(&self) -> &str {
        match self {
            ManSection::Name => "NAME",
            ManSection::Synopsis => "SYNOPSIS",
            ManSection::Description => "DESCRIPTION",
            ManSection::Options => "OPTIONS",
            ManSection::Examples => "EXAMPLES",
            ManSection::SeeAlso => "SEE ALSO",
            ManSection::Author => "AUTHOR",
            ManSection::Bugs => "BUGS",
            ManSection::Copyright => "COPYRIGHT",
            ManSection::Other(s) => s.as_str(),
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s.to_uppercase().as_str() {
            "NAME" => ManSection::Name,
            "SYNOPSIS" => ManSection::Synopsis,
            "DESCRIPTION" => ManSection::Description,
            "OPTIONS" => ManSection::Options,
            "EXAMPLES" | "EXAMPLE" => ManSection::Examples,
            "SEE ALSO" => ManSection::SeeAlso,
            "AUTHOR" | "AUTHORS" => ManSection::Author,
            "BUGS" => ManSection::Bugs,
            "COPYRIGHT" => ManSection::Copyright,
            other => ManSection::Other(other.to_string()),
        }
    }
}

/// Command metadata for indexing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandMetadata {
    /// Operating system (linux, macos, windows)
    pub os: String,

    /// Distribution identifier (ubuntu-22.04, macos-14, etc.)
    pub distro: String,

    /// Command version string
    pub version: Option<String>,

    /// Full path to command binary
    pub installed_path: Option<PathBuf>,

    /// When this was indexed
    pub indexed_at: DateTime<Utc>,

    /// Man page section number (1-8)
    pub man_section: Option<u8>,

    /// Whether this is a GNU utility
    pub is_gnu: bool,

    /// Command type classification
    pub command_type: CommandType,
}

/// Classification of command types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CommandType {
    CoreUtil,      // Core utilities (ls, cat, grep, etc.)
    NetworkTool,   // Network tools (curl, wget, netstat, etc.)
    SystemAdmin,   // System administration (systemctl, useradd, etc.)
    Development,   // Development tools (git, make, gcc, etc.)
    TextProcessing, // Text processing (sed, awk, tr, etc.)
    Archiving,     // Archive tools (tar, gzip, zip, etc.)
    Other,
}

impl CommandType {
    pub fn classify(command_name: &str) -> Self {
        match command_name {
            // Core utilities
            "ls" | "cat" | "cp" | "mv" | "rm" | "mkdir" | "rmdir" | "touch" | "chmod" | "chown" => {
                CommandType::CoreUtil
            }
            // Network tools
            "curl" | "wget" | "nc" | "telnet" | "ssh" | "scp" | "rsync" | "ping" | "traceroute"
            | "netstat" | "ss" | "lsof" | "ifconfig" | "ip" => CommandType::NetworkTool,
            // System admin
            "systemctl" | "service" | "useradd" | "usermod" | "userdel" | "groupadd" | "sudo"
            | "su" | "ps" | "top" | "kill" | "killall" => CommandType::SystemAdmin,
            // Development
            "git" | "make" | "gcc" | "g++" | "clang" | "cargo" | "npm" | "pip" | "mvn"
            | "gradle" => CommandType::Development,
            // Text processing
            "grep" | "egrep" | "fgrep" | "sed" | "awk" | "cut" | "tr" | "sort" | "uniq" | "wc"
            | "head" | "tail" => CommandType::TextProcessing,
            // Archiving
            "tar" | "gzip" | "gunzip" | "bzip2" | "bunzip2" | "zip" | "unzip" | "xz" => {
                CommandType::Archiving
            }
            _ => CommandType::Other,
        }
    }
}

/// Index statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexStatistics {
    pub total_commands: usize,
    pub total_documents: usize,
    pub by_type: HashMap<CommandType, usize>,
    pub index_size_bytes: u64,
    pub last_updated: DateTime<Utc>,
    pub os: String,
    pub distro: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_type_classification() {
        assert_eq!(CommandType::classify("ls"), CommandType::CoreUtil);
        assert_eq!(CommandType::classify("grep"), CommandType::TextProcessing);
        assert_eq!(CommandType::classify("curl"), CommandType::NetworkTool);
        assert_eq!(CommandType::classify("git"), CommandType::Development);
        assert_eq!(CommandType::classify("tar"), CommandType::Archiving);
        assert_eq!(CommandType::classify("systemctl"), CommandType::SystemAdmin);
    }

    #[test]
    fn test_man_section_parsing() {
        assert_eq!(ManSection::from_str("NAME"), ManSection::Name);
        assert_eq!(ManSection::from_str("SYNOPSIS"), ManSection::Synopsis);
        assert_eq!(ManSection::from_str("DESCRIPTION"), ManSection::Description);
        assert_eq!(ManSection::from_str("OPTIONS"), ManSection::Options);
        assert_eq!(ManSection::from_str("EXAMPLES"), ManSection::Examples);
    }
}
