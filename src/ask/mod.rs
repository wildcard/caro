//! Ask Mode Module
//!
//! This module provides functionality for detecting when users are asking
//! questions rather than requesting shell commands, and generating helpful
//! informational responses in Markdown format.
//!
//! # Features
//!
//! - **Question Detection**: Identifies questions vs command requests
//! - **Knowledge Base**: Built-in knowledge about shell commands and system topics
//! - **Markdown Responses**: Terminal-friendly formatted responses
//!
//! # Example
//!
//! ```rust,ignore
//! use caro::ask::{AskService, InputType};
//!
//! let service = AskService::new();
//! if let InputType::Question(topic) = service.detect_input_type("what does grep do?") {
//!     let response = service.generate_response(&topic);
//!     println!("{}", response.markdown);
//! }
//! ```

use std::collections::HashMap;

/// Service for detecting questions and generating informational responses
pub struct AskService {
    knowledge_base: KnowledgeBase,
}

/// Type of user input
#[derive(Debug, Clone, PartialEq)]
pub enum InputType {
    /// User is asking a question - contains the detected topic
    Question(QuestionTopic),
    /// User is requesting a command to be generated
    CommandRequest,
}

/// Detected question topic with context
#[derive(Debug, Clone, PartialEq)]
pub struct QuestionTopic {
    /// The primary topic/command being asked about
    pub subject: String,
    /// The type of question being asked
    pub question_type: QuestionType,
    /// Original input text
    pub original_input: String,
}

/// Types of questions users might ask
#[derive(Debug, Clone, PartialEq)]
pub enum QuestionType {
    /// "What does X do?" or "What is X?"
    WhatIs,
    /// "How does X work?" or "How to use X?"
    HowTo,
    /// "Why use X?" or "Why does X...?"
    Why,
    /// "What's the difference between X and Y?"
    Comparison,
    /// Asking about flags/options: "what does -r mean?"
    FlagExplanation,
    /// General explanation request
    Explain,
    /// Definition or concept question
    Definition,
}

/// Response generated for a question
#[derive(Debug, Clone)]
pub struct AskResponse {
    /// The formatted Markdown response
    pub markdown: String,
    /// Short summary for quick display
    pub summary: String,
    /// Related topics the user might want to explore
    pub related_topics: Vec<String>,
    /// Whether Caro has good knowledge of this topic
    pub confidence: ResponseConfidence,
}

/// How confident Caro is in the response
#[derive(Debug, Clone, PartialEq)]
pub enum ResponseConfidence {
    /// Caro has detailed knowledge about this topic
    High,
    /// Caro has some knowledge but may be incomplete
    Medium,
    /// Caro has limited knowledge - may suggest alternatives
    Low,
}

/// Knowledge base for common shell/system topics
struct KnowledgeBase {
    commands: HashMap<String, CommandKnowledge>,
    concepts: HashMap<String, ConceptKnowledge>,
}

/// Knowledge about a specific command
#[derive(Debug, Clone)]
struct CommandKnowledge {
    name: String,
    short_description: String,
    detailed_description: String,
    common_flags: Vec<(String, String)>, // (flag, description)
    examples: Vec<(String, String)>,     // (command, description)
    related_commands: Vec<String>,
    category: String,
}

/// Knowledge about a concept
#[derive(Debug, Clone)]
struct ConceptKnowledge {
    name: String,
    description: String,
    key_points: Vec<String>,
    related_topics: Vec<String>,
}

impl AskService {
    /// Create a new ask service
    pub fn new() -> Self {
        Self {
            knowledge_base: KnowledgeBase::new(),
        }
    }

    /// Detect whether the input is a question or a command request
    pub fn detect_input_type(&self, input: &str) -> InputType {
        let input_lower = input.to_lowercase().trim().to_string();

        // Check for explicit question indicators
        if let Some(topic) = self.detect_question(&input_lower, input) {
            return InputType::Question(topic);
        }

        // Default to command request
        InputType::CommandRequest
    }

    /// Detect if input is a question and extract the topic
    fn detect_question(&self, input_lower: &str, original: &str) -> Option<QuestionTopic> {
        // Pattern 1: Ends with question mark
        let has_question_mark = input_lower.ends_with('?');

        // Pattern 2: Starts with question words
        let question_starters = [
            ("what is ", QuestionType::WhatIs),
            ("what's ", QuestionType::WhatIs),
            ("what does ", QuestionType::WhatIs),
            ("what do ", QuestionType::WhatIs),
            ("how does ", QuestionType::HowTo),
            ("how do ", QuestionType::HowTo),
            ("how to ", QuestionType::HowTo),
            ("how can ", QuestionType::HowTo),
            ("why does ", QuestionType::Why),
            ("why is ", QuestionType::Why),
            ("why do ", QuestionType::Why),
            ("why use ", QuestionType::Why),
            ("explain ", QuestionType::Explain),
            ("tell me about ", QuestionType::Explain),
            ("describe ", QuestionType::Explain),
            ("what's the difference between ", QuestionType::Comparison),
            ("difference between ", QuestionType::Comparison),
            ("compare ", QuestionType::Comparison),
            ("vs ", QuestionType::Comparison),
            ("what flag ", QuestionType::FlagExplanation),
            ("what option ", QuestionType::FlagExplanation),
            ("what does the ", QuestionType::FlagExplanation),
            ("what does -", QuestionType::FlagExplanation),
            ("meaning of ", QuestionType::Definition),
            ("definition of ", QuestionType::Definition),
        ];

        for (starter, q_type) in question_starters.iter() {
            if input_lower.starts_with(starter) {
                let subject = input_lower
                    .strip_prefix(starter)
                    .unwrap_or("")
                    .trim_end_matches('?')
                    .trim()
                    .to_string();

                if !subject.is_empty() {
                    return Some(QuestionTopic {
                        subject,
                        question_type: q_type.clone(),
                        original_input: original.to_string(),
                    });
                }
            }
        }

        // Pattern 3: Contains "mean" or "meaning" (what does X mean?)
        if input_lower.contains(" mean") && has_question_mark {
            let subject = self.extract_subject_before_mean(input_lower);
            if !subject.is_empty() {
                return Some(QuestionTopic {
                    subject,
                    question_type: QuestionType::WhatIs,
                    original_input: original.to_string(),
                });
            }
        }

        // Pattern 4: Simple question with question mark and known command
        if has_question_mark {
            // Check if it's asking about a known command
            for word in input_lower.split_whitespace() {
                let clean_word = word.trim_matches(|c: char| !c.is_alphanumeric() && c != '-');
                if self.knowledge_base.commands.contains_key(clean_word) {
                    return Some(QuestionTopic {
                        subject: clean_word.to_string(),
                        question_type: QuestionType::WhatIs,
                        original_input: original.to_string(),
                    });
                }
            }
        }

        // Pattern 5: "can you explain" or "could you explain"
        if input_lower.contains("can you explain")
            || input_lower.contains("could you explain")
            || input_lower.contains("please explain")
        {
            let subject = self.extract_after_explain(input_lower);
            if !subject.is_empty() {
                return Some(QuestionTopic {
                    subject,
                    question_type: QuestionType::Explain,
                    original_input: original.to_string(),
                });
            }
        }

        None
    }

    /// Extract subject before "mean" in a question
    fn extract_subject_before_mean(&self, input: &str) -> String {
        // Pattern: "what does X mean"
        if let Some(idx) = input.find(" mean") {
            let before = &input[..idx];
            // Try to find the subject after "does" or "do"
            if let Some(does_idx) = before.rfind("does ") {
                return before[does_idx + 5..].trim().to_string();
            }
            if let Some(do_idx) = before.rfind("do ") {
                return before[do_idx + 3..].trim().to_string();
            }
        }
        String::new()
    }

    /// Extract subject after "explain"
    fn extract_after_explain(&self, input: &str) -> String {
        if let Some(idx) = input.find("explain") {
            let after = &input[idx + 7..];
            return after.trim().trim_end_matches('?').trim().to_string();
        }
        String::new()
    }

    /// Generate a response for a question topic
    pub fn generate_response(&self, topic: &QuestionTopic) -> AskResponse {
        // Try to find in knowledge base
        let subject_lower = topic.subject.to_lowercase();

        // Try exact match first
        if let Some(cmd_knowledge) = self.knowledge_base.commands.get(&subject_lower) {
            return self.format_command_response(cmd_knowledge, &topic.question_type);
        }

        // Try concept match
        if let Some(concept) = self.knowledge_base.concepts.get(&subject_lower) {
            return self.format_concept_response(concept, &topic.question_type);
        }

        // Try to extract known command from subject (e.g., "find work" -> "find")
        let primary_subject = self.extract_primary_subject(&subject_lower);
        if let Some(cmd_knowledge) = self.knowledge_base.commands.get(&primary_subject) {
            return self.format_command_response(cmd_knowledge, &topic.question_type);
        }
        if let Some(concept) = self.knowledge_base.concepts.get(&primary_subject) {
            return self.format_concept_response(concept, &topic.question_type);
        }

        // Try plural -> singular (pipes -> pipe)
        let singular = subject_lower.trim_end_matches('s');
        if let Some(concept) = self.knowledge_base.concepts.get(singular) {
            return self.format_concept_response(concept, &topic.question_type);
        }

        // Check for flag explanations
        if topic.question_type == QuestionType::FlagExplanation {
            return self.format_flag_response(&topic.subject);
        }

        // Generate a helpful "I don't know but here's what you can do" response
        self.format_unknown_topic_response(topic)
    }

    /// Extract the primary subject from a multi-word phrase
    fn extract_primary_subject(&self, subject: &str) -> String {
        // Common suffixes to strip: "work", "command", "tool", "utility"
        let strip_suffixes = ["work", "command", "tool", "utility", "works", "commands"];

        let words: Vec<&str> = subject.split_whitespace().collect();

        // If single word, return as-is
        if words.len() <= 1 {
            return subject.to_string();
        }

        // Check if last word is a suffix to strip
        if let Some(last) = words.last() {
            if strip_suffixes.contains(last) {
                return words[..words.len() - 1].join(" ");
            }
        }

        // Otherwise check if first word is a known command
        if let Some(first) = words.first() {
            if self.knowledge_base.commands.contains_key(*first)
                || self.knowledge_base.concepts.contains_key(*first)
            {
                return first.to_string();
            }
        }

        subject.to_string()
    }

    /// Format response for a known command
    fn format_command_response(
        &self,
        cmd: &CommandKnowledge,
        question_type: &QuestionType,
    ) -> AskResponse {
        let mut markdown = String::new();

        // Title
        markdown.push_str(&format!("## `{}`\n\n", cmd.name));

        // Short description
        markdown.push_str(&format!("{}\n\n", cmd.short_description));

        // Add more detail based on question type
        match question_type {
            QuestionType::WhatIs | QuestionType::Explain => {
                markdown.push_str(&format!("{}\n\n", cmd.detailed_description));

                if !cmd.common_flags.is_empty() {
                    markdown.push_str("### Common Flags\n\n");
                    for (flag, desc) in &cmd.common_flags {
                        markdown.push_str(&format!("- `{}` - {}\n", flag, desc));
                    }
                    markdown.push('\n');
                }
            }
            QuestionType::HowTo => {
                if !cmd.examples.is_empty() {
                    markdown.push_str("### Examples\n\n");
                    for (example, desc) in &cmd.examples {
                        markdown.push_str(&format!("```bash\n{}\n```\n{}\n\n", example, desc));
                    }
                }
            }
            _ => {
                markdown.push_str(&format!("{}\n\n", cmd.detailed_description));
            }
        }

        // Related commands
        if !cmd.related_commands.is_empty() {
            markdown.push_str("### See Also\n\n");
            markdown.push_str(&cmd.related_commands.join(", "));
            markdown.push('\n');
        }

        AskResponse {
            markdown,
            summary: cmd.short_description.clone(),
            related_topics: cmd.related_commands.clone(),
            confidence: ResponseConfidence::High,
        }
    }

    /// Format response for a concept
    fn format_concept_response(
        &self,
        concept: &ConceptKnowledge,
        _question_type: &QuestionType,
    ) -> AskResponse {
        let mut markdown = String::new();

        markdown.push_str(&format!("## {}\n\n", concept.name));
        markdown.push_str(&format!("{}\n\n", concept.description));

        if !concept.key_points.is_empty() {
            markdown.push_str("### Key Points\n\n");
            for point in &concept.key_points {
                markdown.push_str(&format!("- {}\n", point));
            }
            markdown.push('\n');
        }

        AskResponse {
            markdown,
            summary: concept.description.clone(),
            related_topics: concept.related_topics.clone(),
            confidence: ResponseConfidence::High,
        }
    }

    /// Format response for flag explanation
    fn format_flag_response(&self, subject: &str) -> AskResponse {
        // Try to extract command and flag
        let (cmd, flag) = self.extract_command_and_flag(subject);

        if let Some(cmd_knowledge) = self.knowledge_base.commands.get(&cmd.to_lowercase()) {
            for (f, desc) in &cmd_knowledge.common_flags {
                if f.contains(&flag) || flag.contains(f.trim_start_matches('-')) {
                    return AskResponse {
                        markdown: format!(
                            "## `{}` flag `{}`\n\n{}\n",
                            cmd_knowledge.name, f, desc
                        ),
                        summary: desc.clone(),
                        related_topics: vec![cmd_knowledge.name.clone()],
                        confidence: ResponseConfidence::High,
                    };
                }
            }
        }

        // Generic flag explanation
        let markdown = format!(
            "## Flag `{}`\n\n\
            I don't have specific information about this flag.\n\n\
            **Tip:** Try `man {}` or `{} --help` to see all available options.\n",
            flag, cmd, cmd
        );

        AskResponse {
            markdown,
            summary: format!("Flag {} for {}", flag, cmd),
            related_topics: vec![cmd],
            confidence: ResponseConfidence::Low,
        }
    }

    /// Extract command and flag from a flag question
    fn extract_command_and_flag(&self, subject: &str) -> (String, String) {
        let words: Vec<&str> = subject.split_whitespace().collect();

        // Look for pattern like "grep -r" or "-r in grep"
        let mut cmd = String::new();
        let mut flag = String::new();

        for word in &words {
            if word.starts_with('-') {
                flag = word.to_string();
            } else if self.knowledge_base.commands.contains_key(&word.to_lowercase()) {
                cmd = word.to_string();
            }
        }

        if cmd.is_empty() && !words.is_empty() {
            // Try first non-flag word
            for word in &words {
                if !word.starts_with('-') && !["in", "for", "the", "flag", "option"].contains(word)
                {
                    cmd = word.to_string();
                    break;
                }
            }
        }

        (cmd, flag)
    }

    /// Format response when topic is unknown
    fn format_unknown_topic_response(&self, topic: &QuestionTopic) -> AskResponse {
        let markdown = format!(
            "## About \"{}\"\n\n\
            I don't have detailed information about this topic in my knowledge base.\n\n\
            ### Suggestions\n\n\
            - Try `man {}` for the manual page\n\
            - Try `{} --help` for usage information\n\
            - Search online for more details\n\n\
            If you'd like me to generate a command instead, \
            just describe what you want to do!\n",
            topic.subject, topic.subject, topic.subject
        );

        AskResponse {
            markdown,
            summary: format!("Limited information about {}", topic.subject),
            related_topics: vec![],
            confidence: ResponseConfidence::Low,
        }
    }
}

impl Default for AskService {
    fn default() -> Self {
        Self::new()
    }
}

impl KnowledgeBase {
    fn new() -> Self {
        let mut kb = Self {
            commands: HashMap::new(),
            concepts: HashMap::new(),
        };
        kb.populate_commands();
        kb.populate_concepts();
        kb
    }

    fn populate_commands(&mut self) {
        // grep
        self.commands.insert(
            "grep".to_string(),
            CommandKnowledge {
                name: "grep".to_string(),
                short_description: "Search for patterns in files or input.".to_string(),
                detailed_description: "grep (Global Regular Expression Print) searches for \
                    lines matching a pattern and prints them. It's one of the most commonly \
                    used Unix utilities for finding text in files."
                    .to_string(),
                common_flags: vec![
                    ("-i".to_string(), "Case-insensitive search".to_string()),
                    ("-r".to_string(), "Recursive search in directories".to_string()),
                    ("-n".to_string(), "Show line numbers".to_string()),
                    ("-v".to_string(), "Invert match (show non-matching lines)".to_string()),
                    ("-l".to_string(), "Show only filenames with matches".to_string()),
                    ("-c".to_string(), "Count matching lines".to_string()),
                    ("-E".to_string(), "Extended regex (same as egrep)".to_string()),
                    ("-w".to_string(), "Match whole words only".to_string()),
                ],
                examples: vec![
                    ("grep 'error' logfile.txt".to_string(), "Find lines containing 'error'".to_string()),
                    ("grep -ri 'TODO' .".to_string(), "Recursively find TODO comments".to_string()),
                    ("ps aux | grep nginx".to_string(), "Find nginx processes".to_string()),
                ],
                related_commands: vec!["awk".to_string(), "sed".to_string(), "find".to_string(), "rg".to_string()],
                category: "text processing".to_string(),
            },
        );

        // find
        self.commands.insert(
            "find".to_string(),
            CommandKnowledge {
                name: "find".to_string(),
                short_description: "Search for files in a directory hierarchy.".to_string(),
                detailed_description: "find walks through a directory tree and finds files \
                    matching specified criteria like name, type, size, or modification time. \
                    It can also execute commands on found files."
                    .to_string(),
                common_flags: vec![
                    ("-name".to_string(), "Search by filename pattern".to_string()),
                    ("-type f".to_string(), "Find only files".to_string()),
                    ("-type d".to_string(), "Find only directories".to_string()),
                    ("-mtime".to_string(), "Find by modification time".to_string()),
                    ("-size".to_string(), "Find by file size".to_string()),
                    ("-exec".to_string(), "Execute command on each file".to_string()),
                    ("-delete".to_string(), "Delete matching files".to_string()),
                ],
                examples: vec![
                    ("find . -name '*.txt'".to_string(), "Find all .txt files".to_string()),
                    ("find . -type f -size +10M".to_string(), "Find files larger than 10MB".to_string()),
                    ("find . -mtime -7".to_string(), "Find files modified in last 7 days".to_string()),
                ],
                related_commands: vec!["locate".to_string(), "fd".to_string(), "ls".to_string()],
                category: "file management".to_string(),
            },
        );

        // ls
        self.commands.insert(
            "ls".to_string(),
            CommandKnowledge {
                name: "ls".to_string(),
                short_description: "List directory contents.".to_string(),
                detailed_description: "ls lists information about files and directories. \
                    By default it lists the current directory, but can show detailed \
                    information including permissions, size, and timestamps."
                    .to_string(),
                common_flags: vec![
                    ("-l".to_string(), "Long format with details".to_string()),
                    ("-a".to_string(), "Show hidden files (starting with .)".to_string()),
                    ("-h".to_string(), "Human-readable sizes".to_string()),
                    ("-t".to_string(), "Sort by modification time".to_string()),
                    ("-r".to_string(), "Reverse sort order".to_string()),
                    ("-S".to_string(), "Sort by file size".to_string()),
                ],
                examples: vec![
                    ("ls -la".to_string(), "List all files with details".to_string()),
                    ("ls -lh".to_string(), "List with human-readable sizes".to_string()),
                    ("ls -lt".to_string(), "List sorted by time".to_string()),
                ],
                related_commands: vec!["exa".to_string(), "tree".to_string(), "dir".to_string()],
                category: "file management".to_string(),
            },
        );

        // cat
        self.commands.insert(
            "cat".to_string(),
            CommandKnowledge {
                name: "cat".to_string(),
                short_description: "Concatenate and display file contents.".to_string(),
                detailed_description: "cat reads files and writes them to standard output. \
                    While named for concatenating files, it's commonly used to display \
                    file contents or create small files."
                    .to_string(),
                common_flags: vec![
                    ("-n".to_string(), "Number all output lines".to_string()),
                    ("-b".to_string(), "Number non-empty lines only".to_string()),
                    ("-s".to_string(), "Squeeze multiple blank lines".to_string()),
                ],
                examples: vec![
                    ("cat file.txt".to_string(), "Display file contents".to_string()),
                    ("cat file1.txt file2.txt > combined.txt".to_string(), "Concatenate files".to_string()),
                    ("cat -n script.sh".to_string(), "Display with line numbers".to_string()),
                ],
                related_commands: vec!["less".to_string(), "head".to_string(), "tail".to_string(), "bat".to_string()],
                category: "file viewing".to_string(),
            },
        );

        // awk
        self.commands.insert(
            "awk".to_string(),
            CommandKnowledge {
                name: "awk".to_string(),
                short_description: "Pattern scanning and text processing language.".to_string(),
                detailed_description: "awk is a powerful text processing tool that works on \
                    a line-by-line basis. It's excellent for extracting columns, performing \
                    calculations, and transforming structured text data."
                    .to_string(),
                common_flags: vec![
                    ("-F".to_string(), "Set field separator".to_string()),
                    ("-v".to_string(), "Assign variable before execution".to_string()),
                ],
                examples: vec![
                    ("awk '{print $1}' file.txt".to_string(), "Print first column".to_string()),
                    ("awk -F: '{print $1}' /etc/passwd".to_string(), "Extract usernames".to_string()),
                    ("awk '{sum += $1} END {print sum}'".to_string(), "Sum first column".to_string()),
                ],
                related_commands: vec!["sed".to_string(), "cut".to_string(), "grep".to_string()],
                category: "text processing".to_string(),
            },
        );

        // sed
        self.commands.insert(
            "sed".to_string(),
            CommandKnowledge {
                name: "sed".to_string(),
                short_description: "Stream editor for text transformation.".to_string(),
                detailed_description: "sed (stream editor) performs text transformations on \
                    input streams. It's commonly used for find-and-replace operations, \
                    line deletion, and text substitution."
                    .to_string(),
                common_flags: vec![
                    ("-i".to_string(), "Edit files in-place".to_string()),
                    ("-e".to_string(), "Add editing commands".to_string()),
                    ("-n".to_string(), "Suppress automatic printing".to_string()),
                ],
                examples: vec![
                    ("sed 's/old/new/g' file.txt".to_string(), "Replace all occurrences".to_string()),
                    ("sed -i 's/foo/bar/g' *.txt".to_string(), "In-place replacement".to_string()),
                    ("sed '5d' file.txt".to_string(), "Delete line 5".to_string()),
                ],
                related_commands: vec!["awk".to_string(), "tr".to_string(), "grep".to_string()],
                category: "text processing".to_string(),
            },
        );

        // chmod
        self.commands.insert(
            "chmod".to_string(),
            CommandKnowledge {
                name: "chmod".to_string(),
                short_description: "Change file permissions.".to_string(),
                detailed_description: "chmod modifies file access permissions. Permissions \
                    control who can read, write, or execute a file. Can use symbolic \
                    (rwx) or numeric (755) notation."
                    .to_string(),
                common_flags: vec![
                    ("-R".to_string(), "Recursive (apply to directories)".to_string()),
                    ("+x".to_string(), "Add execute permission".to_string()),
                    ("-w".to_string(), "Remove write permission".to_string()),
                ],
                examples: vec![
                    ("chmod +x script.sh".to_string(), "Make file executable".to_string()),
                    ("chmod 755 file".to_string(), "rwxr-xr-x permissions".to_string()),
                    ("chmod -R 644 *.txt".to_string(), "Set read permissions recursively".to_string()),
                ],
                related_commands: vec!["chown".to_string(), "chgrp".to_string(), "ls -l".to_string()],
                category: "file permissions".to_string(),
            },
        );

        // ps
        self.commands.insert(
            "ps".to_string(),
            CommandKnowledge {
                name: "ps".to_string(),
                short_description: "Report process status.".to_string(),
                detailed_description: "ps displays information about running processes. \
                    It shows process IDs, CPU/memory usage, and command names. \
                    Essential for system monitoring and troubleshooting."
                    .to_string(),
                common_flags: vec![
                    ("aux".to_string(), "All processes, detailed format".to_string()),
                    ("-e".to_string(), "All processes".to_string()),
                    ("-f".to_string(), "Full format".to_string()),
                    ("--sort".to_string(), "Sort by column (Linux)".to_string()),
                ],
                examples: vec![
                    ("ps aux".to_string(), "Show all processes".to_string()),
                    ("ps aux | grep python".to_string(), "Find python processes".to_string()),
                    ("ps -ef".to_string(), "Full format listing".to_string()),
                ],
                related_commands: vec!["top".to_string(), "htop".to_string(), "pgrep".to_string(), "kill".to_string()],
                category: "process management".to_string(),
            },
        );

        // kill
        self.commands.insert(
            "kill".to_string(),
            CommandKnowledge {
                name: "kill".to_string(),
                short_description: "Send signals to processes.".to_string(),
                detailed_description: "kill sends signals to processes, typically to terminate \
                    them. Despite its name, it can send various signals for different purposes \
                    like stopping, continuing, or terminating processes."
                    .to_string(),
                common_flags: vec![
                    ("-9".to_string(), "SIGKILL - Force kill (cannot be caught)".to_string()),
                    ("-15".to_string(), "SIGTERM - Graceful termination (default)".to_string()),
                    ("-l".to_string(), "List all signal names".to_string()),
                ],
                examples: vec![
                    ("kill 1234".to_string(), "Terminate process 1234".to_string()),
                    ("kill -9 1234".to_string(), "Force kill process".to_string()),
                    ("killall nginx".to_string(), "Kill all nginx processes".to_string()),
                ],
                related_commands: vec!["ps".to_string(), "pkill".to_string(), "killall".to_string()],
                category: "process management".to_string(),
            },
        );

        // curl
        self.commands.insert(
            "curl".to_string(),
            CommandKnowledge {
                name: "curl".to_string(),
                short_description: "Transfer data from or to a server.".to_string(),
                detailed_description: "curl is a tool for transferring data using various \
                    protocols including HTTP, HTTPS, FTP. It's commonly used for API testing, \
                    downloading files, and web scraping."
                    .to_string(),
                common_flags: vec![
                    ("-X".to_string(), "Specify HTTP method (GET, POST, etc.)".to_string()),
                    ("-H".to_string(), "Add header".to_string()),
                    ("-d".to_string(), "Send data in request body".to_string()),
                    ("-o".to_string(), "Write output to file".to_string()),
                    ("-s".to_string(), "Silent mode".to_string()),
                    ("-v".to_string(), "Verbose output".to_string()),
                ],
                examples: vec![
                    ("curl https://api.example.com".to_string(), "GET request".to_string()),
                    ("curl -X POST -d '{\"key\":\"value\"}' -H 'Content-Type: application/json' url".to_string(), "POST JSON".to_string()),
                    ("curl -o file.zip https://example.com/file.zip".to_string(), "Download file".to_string()),
                ],
                related_commands: vec!["wget".to_string(), "httpie".to_string()],
                category: "networking".to_string(),
            },
        );

        // tar
        self.commands.insert(
            "tar".to_string(),
            CommandKnowledge {
                name: "tar".to_string(),
                short_description: "Archive files (tape archive).".to_string(),
                detailed_description: "tar creates and extracts archive files, commonly used \
                    for backups and file distribution. Often combined with gzip or bzip2 \
                    for compression."
                    .to_string(),
                common_flags: vec![
                    ("-c".to_string(), "Create archive".to_string()),
                    ("-x".to_string(), "Extract archive".to_string()),
                    ("-v".to_string(), "Verbose (show files)".to_string()),
                    ("-f".to_string(), "Specify archive filename".to_string()),
                    ("-z".to_string(), "Use gzip compression".to_string()),
                    ("-j".to_string(), "Use bzip2 compression".to_string()),
                ],
                examples: vec![
                    ("tar -czvf archive.tar.gz folder/".to_string(), "Create compressed archive".to_string()),
                    ("tar -xzvf archive.tar.gz".to_string(), "Extract compressed archive".to_string()),
                    ("tar -tvf archive.tar".to_string(), "List archive contents".to_string()),
                ],
                related_commands: vec!["gzip".to_string(), "zip".to_string(), "unzip".to_string()],
                category: "archiving".to_string(),
            },
        );

        // ssh
        self.commands.insert(
            "ssh".to_string(),
            CommandKnowledge {
                name: "ssh".to_string(),
                short_description: "Secure Shell - remote login and command execution.".to_string(),
                detailed_description: "ssh provides secure encrypted communication between \
                    two untrusted hosts over an insecure network. Used for remote login, \
                    file transfer, and tunneling."
                    .to_string(),
                common_flags: vec![
                    ("-p".to_string(), "Specify port".to_string()),
                    ("-i".to_string(), "Identity file (private key)".to_string()),
                    ("-L".to_string(), "Local port forwarding".to_string()),
                    ("-R".to_string(), "Remote port forwarding".to_string()),
                    ("-v".to_string(), "Verbose mode".to_string()),
                ],
                examples: vec![
                    ("ssh user@host".to_string(), "Connect to remote host".to_string()),
                    ("ssh -p 2222 user@host".to_string(), "Connect on custom port".to_string()),
                    ("ssh -L 8080:localhost:80 user@host".to_string(), "Port forwarding".to_string()),
                ],
                related_commands: vec!["scp".to_string(), "rsync".to_string(), "sftp".to_string()],
                category: "networking".to_string(),
            },
        );

        // git
        self.commands.insert(
            "git".to_string(),
            CommandKnowledge {
                name: "git".to_string(),
                short_description: "Distributed version control system.".to_string(),
                detailed_description: "git is a distributed version control system for \
                    tracking changes in source code. It allows multiple developers to \
                    work together and maintains complete history of changes."
                    .to_string(),
                common_flags: vec![
                    ("status".to_string(), "Show working tree status".to_string()),
                    ("add".to_string(), "Stage changes".to_string()),
                    ("commit".to_string(), "Record changes".to_string()),
                    ("push".to_string(), "Upload to remote".to_string()),
                    ("pull".to_string(), "Download from remote".to_string()),
                    ("log".to_string(), "Show commit history".to_string()),
                ],
                examples: vec![
                    ("git status".to_string(), "Check repository status".to_string()),
                    ("git add . && git commit -m 'message'".to_string(), "Stage and commit".to_string()),
                    ("git log --oneline".to_string(), "Compact history".to_string()),
                ],
                related_commands: vec!["gh".to_string(), "svn".to_string()],
                category: "version control".to_string(),
            },
        );

        // docker
        self.commands.insert(
            "docker".to_string(),
            CommandKnowledge {
                name: "docker".to_string(),
                short_description: "Container platform for building and running applications.".to_string(),
                detailed_description: "docker is a platform for developing, shipping, and \
                    running applications in containers. Containers package code with \
                    dependencies for consistent deployment across environments."
                    .to_string(),
                common_flags: vec![
                    ("run".to_string(), "Run a container".to_string()),
                    ("ps".to_string(), "List containers".to_string()),
                    ("images".to_string(), "List images".to_string()),
                    ("build".to_string(), "Build an image".to_string()),
                    ("exec".to_string(), "Execute command in container".to_string()),
                ],
                examples: vec![
                    ("docker run -it ubuntu bash".to_string(), "Run interactive Ubuntu".to_string()),
                    ("docker ps -a".to_string(), "List all containers".to_string()),
                    ("docker build -t myapp .".to_string(), "Build image from Dockerfile".to_string()),
                ],
                related_commands: vec!["docker-compose".to_string(), "podman".to_string(), "kubectl".to_string()],
                category: "containers".to_string(),
            },
        );

        // xargs
        self.commands.insert(
            "xargs".to_string(),
            CommandKnowledge {
                name: "xargs".to_string(),
                short_description: "Build and execute commands from standard input.".to_string(),
                detailed_description: "xargs reads items from standard input and executes \
                    a command using those items as arguments. It's essential for \
                    processing lists of files from find, grep, or other commands."
                    .to_string(),
                common_flags: vec![
                    ("-I {}".to_string(), "Replace {} with each input".to_string()),
                    ("-n".to_string(), "Max arguments per command".to_string()),
                    ("-p".to_string(), "Prompt before execution".to_string()),
                    ("-0".to_string(), "Null-separated input".to_string()),
                ],
                examples: vec![
                    ("find . -name '*.txt' | xargs grep 'pattern'".to_string(), "Search in found files".to_string()),
                    ("cat files.txt | xargs -I {} cp {} backup/".to_string(), "Copy listed files".to_string()),
                    ("find . -name '*.log' -print0 | xargs -0 rm".to_string(), "Safe deletion".to_string()),
                ],
                related_commands: vec!["find".to_string(), "parallel".to_string()],
                category: "text processing".to_string(),
            },
        );

        // head
        self.commands.insert(
            "head".to_string(),
            CommandKnowledge {
                name: "head".to_string(),
                short_description: "Output the first part of files.".to_string(),
                detailed_description: "head displays the first lines of a file. By default \
                    it shows the first 10 lines. Useful for previewing files or \
                    getting the beginning of logs."
                    .to_string(),
                common_flags: vec![
                    ("-n".to_string(), "Number of lines to show".to_string()),
                    ("-c".to_string(), "Number of bytes to show".to_string()),
                ],
                examples: vec![
                    ("head file.txt".to_string(), "Show first 10 lines".to_string()),
                    ("head -n 5 file.txt".to_string(), "Show first 5 lines".to_string()),
                    ("head -c 100 file.txt".to_string(), "Show first 100 bytes".to_string()),
                ],
                related_commands: vec!["tail".to_string(), "cat".to_string(), "less".to_string()],
                category: "file viewing".to_string(),
            },
        );

        // tail
        self.commands.insert(
            "tail".to_string(),
            CommandKnowledge {
                name: "tail".to_string(),
                short_description: "Output the last part of files.".to_string(),
                detailed_description: "tail displays the last lines of a file. Essential for \
                    viewing log files and monitoring file updates in real-time with -f."
                    .to_string(),
                common_flags: vec![
                    ("-n".to_string(), "Number of lines to show".to_string()),
                    ("-f".to_string(), "Follow file updates (live)".to_string()),
                    ("-F".to_string(), "Follow and retry if file rotates".to_string()),
                ],
                examples: vec![
                    ("tail file.txt".to_string(), "Show last 10 lines".to_string()),
                    ("tail -f /var/log/syslog".to_string(), "Follow log file".to_string()),
                    ("tail -n 100 access.log".to_string(), "Show last 100 lines".to_string()),
                ],
                related_commands: vec!["head".to_string(), "less".to_string(), "multitail".to_string()],
                category: "file viewing".to_string(),
            },
        );

        // sort
        self.commands.insert(
            "sort".to_string(),
            CommandKnowledge {
                name: "sort".to_string(),
                short_description: "Sort lines of text.".to_string(),
                detailed_description: "sort orders lines of text alphabetically or numerically. \
                    Can sort by specific fields and handle various data formats."
                    .to_string(),
                common_flags: vec![
                    ("-n".to_string(), "Numeric sort".to_string()),
                    ("-r".to_string(), "Reverse order".to_string()),
                    ("-k".to_string(), "Sort by field/column".to_string()),
                    ("-u".to_string(), "Unique (remove duplicates)".to_string()),
                    ("-h".to_string(), "Human-readable numbers".to_string()),
                ],
                examples: vec![
                    ("sort file.txt".to_string(), "Alphabetical sort".to_string()),
                    ("sort -n numbers.txt".to_string(), "Numeric sort".to_string()),
                    ("sort -k2 -n data.txt".to_string(), "Sort by second column".to_string()),
                ],
                related_commands: vec!["uniq".to_string(), "cut".to_string(), "awk".to_string()],
                category: "text processing".to_string(),
            },
        );

        // wc
        self.commands.insert(
            "wc".to_string(),
            CommandKnowledge {
                name: "wc".to_string(),
                short_description: "Count lines, words, and bytes.".to_string(),
                detailed_description: "wc (word count) counts lines, words, and bytes in files. \
                    Commonly used for getting line counts and file statistics."
                    .to_string(),
                common_flags: vec![
                    ("-l".to_string(), "Count lines only".to_string()),
                    ("-w".to_string(), "Count words only".to_string()),
                    ("-c".to_string(), "Count bytes only".to_string()),
                    ("-m".to_string(), "Count characters".to_string()),
                ],
                examples: vec![
                    ("wc -l file.txt".to_string(), "Count lines".to_string()),
                    ("find . -name '*.py' | wc -l".to_string(), "Count Python files".to_string()),
                    ("cat file.txt | wc -w".to_string(), "Count words".to_string()),
                ],
                related_commands: vec!["cat".to_string(), "grep -c".to_string()],
                category: "text processing".to_string(),
            },
        );

        // du
        self.commands.insert(
            "du".to_string(),
            CommandKnowledge {
                name: "du".to_string(),
                short_description: "Estimate file and directory space usage.".to_string(),
                detailed_description: "du (disk usage) reports the sizes of directory trees \
                    and files. Essential for finding what's consuming disk space."
                    .to_string(),
                common_flags: vec![
                    ("-h".to_string(), "Human-readable sizes".to_string()),
                    ("-s".to_string(), "Summary only (total)".to_string()),
                    ("-d".to_string(), "Max depth (macOS/BSD)".to_string()),
                    ("--max-depth".to_string(), "Max depth (Linux)".to_string()),
                ],
                examples: vec![
                    ("du -sh .".to_string(), "Total size of current directory".to_string()),
                    ("du -h -d 1 .".to_string(), "Size of subdirectories".to_string()),
                    ("du -sh * | sort -h".to_string(), "Sorted sizes".to_string()),
                ],
                related_commands: vec!["df".to_string(), "ncdu".to_string(), "ls -lh".to_string()],
                category: "disk management".to_string(),
            },
        );

        // df
        self.commands.insert(
            "df".to_string(),
            CommandKnowledge {
                name: "df".to_string(),
                short_description: "Report filesystem disk space usage.".to_string(),
                detailed_description: "df (disk free) shows the amount of disk space used \
                    and available on mounted filesystems."
                    .to_string(),
                common_flags: vec![
                    ("-h".to_string(), "Human-readable sizes".to_string()),
                    ("-T".to_string(), "Show filesystem type".to_string()),
                ],
                examples: vec![
                    ("df -h".to_string(), "Show disk usage".to_string()),
                    ("df -h /".to_string(), "Show root partition".to_string()),
                ],
                related_commands: vec!["du".to_string(), "mount".to_string()],
                category: "disk management".to_string(),
            },
        );
    }

    fn populate_concepts(&mut self) {
        self.concepts.insert(
            "pipe".to_string(),
            ConceptKnowledge {
                name: "Pipe".to_string(),
                description: "A pipe (|) connects the output of one command to the input \
                    of another, allowing you to chain commands together."
                    .to_string(),
                key_points: vec![
                    "Syntax: command1 | command2".to_string(),
                    "First command's stdout becomes second command's stdin".to_string(),
                    "Can chain multiple commands: cmd1 | cmd2 | cmd3".to_string(),
                    "Essential for text processing and filtering".to_string(),
                ],
                related_topics: vec!["stdin".to_string(), "stdout".to_string(), "redirection".to_string()],
            },
        );

        self.concepts.insert(
            "redirection".to_string(),
            ConceptKnowledge {
                name: "I/O Redirection".to_string(),
                description: "Redirection changes where commands read input from or \
                    write output to, using operators like >, >>, <, and 2>."
                    .to_string(),
                key_points: vec![
                    "> file - Redirect stdout to file (overwrite)".to_string(),
                    ">> file - Redirect stdout to file (append)".to_string(),
                    "< file - Read stdin from file".to_string(),
                    "2> file - Redirect stderr to file".to_string(),
                    "&> file - Redirect both stdout and stderr".to_string(),
                ],
                related_topics: vec!["pipe".to_string(), "stdout".to_string(), "stderr".to_string()],
            },
        );

        self.concepts.insert(
            "stdin".to_string(),
            ConceptKnowledge {
                name: "Standard Input (stdin)".to_string(),
                description: "stdin is the default input stream for commands. It's typically \
                    the keyboard, but can be redirected from files or pipes."
                    .to_string(),
                key_points: vec![
                    "File descriptor 0".to_string(),
                    "Default source: keyboard".to_string(),
                    "Can read from files with < operator".to_string(),
                    "Receives piped output from previous command".to_string(),
                ],
                related_topics: vec!["stdout".to_string(), "stderr".to_string(), "pipe".to_string()],
            },
        );

        self.concepts.insert(
            "stdout".to_string(),
            ConceptKnowledge {
                name: "Standard Output (stdout)".to_string(),
                description: "stdout is the default output stream where commands write \
                    their results. Usually the terminal, but can be redirected."
                    .to_string(),
                key_points: vec![
                    "File descriptor 1".to_string(),
                    "Default destination: terminal".to_string(),
                    "Redirect with > or >> operators".to_string(),
                    "Can be piped to another command".to_string(),
                ],
                related_topics: vec!["stdin".to_string(), "stderr".to_string(), "redirection".to_string()],
            },
        );

        self.concepts.insert(
            "stderr".to_string(),
            ConceptKnowledge {
                name: "Standard Error (stderr)".to_string(),
                description: "stderr is the error output stream, separate from stdout. \
                    This allows error messages to be handled differently from normal output."
                    .to_string(),
                key_points: vec![
                    "File descriptor 2".to_string(),
                    "Separate from stdout for error messages".to_string(),
                    "Redirect with 2> operator".to_string(),
                    "Often useful to separate errors from output".to_string(),
                ],
                related_topics: vec!["stdout".to_string(), "redirection".to_string()],
            },
        );

        self.concepts.insert(
            "permissions".to_string(),
            ConceptKnowledge {
                name: "File Permissions".to_string(),
                description: "Unix file permissions control who can read, write, or \
                    execute a file. They're shown as rwxrwxrwx for user, group, others."
                    .to_string(),
                key_points: vec![
                    "r (4) - Read permission".to_string(),
                    "w (2) - Write permission".to_string(),
                    "x (1) - Execute permission".to_string(),
                    "First 3: owner, middle 3: group, last 3: others".to_string(),
                    "755 = rwxr-xr-x, 644 = rw-r--r--".to_string(),
                ],
                related_topics: vec!["chmod".to_string(), "chown".to_string(), "ls -l".to_string()],
            },
        );

        self.concepts.insert(
            "exit code".to_string(),
            ConceptKnowledge {
                name: "Exit Code".to_string(),
                description: "Exit codes are numeric values returned by commands to indicate \
                    success or failure. 0 means success, non-zero indicates an error."
                    .to_string(),
                key_points: vec![
                    "0 = success".to_string(),
                    "Non-zero = failure/error".to_string(),
                    "Check with $? variable".to_string(),
                    "Used by && and || operators".to_string(),
                ],
                related_topics: vec!["conditional execution".to_string()],
            },
        );

        self.concepts.insert(
            "glob".to_string(),
            ConceptKnowledge {
                name: "Glob Patterns".to_string(),
                description: "Glob patterns are wildcards used for filename matching. \
                    The shell expands them before executing commands."
                    .to_string(),
                key_points: vec![
                    "* - matches any characters".to_string(),
                    "? - matches single character".to_string(),
                    "[abc] - matches any of a, b, c".to_string(),
                    "** - recursive match (in some shells)".to_string(),
                ],
                related_topics: vec!["regex".to_string(), "find".to_string()],
            },
        );

        self.concepts.insert(
            "regex".to_string(),
            ConceptKnowledge {
                name: "Regular Expressions".to_string(),
                description: "Regular expressions (regex) are patterns for matching text. \
                    Used by grep, sed, awk, and many other tools."
                    .to_string(),
                key_points: vec![
                    ". - matches any character".to_string(),
                    "* - zero or more of previous".to_string(),
                    "+ - one or more of previous".to_string(),
                    "^ - start of line".to_string(),
                    "$ - end of line".to_string(),
                    "[a-z] - character class".to_string(),
                ],
                related_topics: vec!["grep".to_string(), "sed".to_string(), "awk".to_string()],
            },
        );

        self.concepts.insert(
            "shebang".to_string(),
            ConceptKnowledge {
                name: "Shebang".to_string(),
                description: "The shebang (#!) at the start of a script specifies which \
                    interpreter should execute the script."
                    .to_string(),
                key_points: vec![
                    "Must be first line of file".to_string(),
                    "#!/bin/bash - run with bash".to_string(),
                    "#!/usr/bin/env python - portable python".to_string(),
                    "File must be executable (chmod +x)".to_string(),
                ],
                related_topics: vec!["scripts".to_string(), "chmod".to_string()],
            },
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_what_is_question() {
        let service = AskService::new();
        let result = service.detect_input_type("what is grep?");
        match result {
            InputType::Question(topic) => {
                assert_eq!(topic.subject, "grep");
                assert_eq!(topic.question_type, QuestionType::WhatIs);
            }
            _ => panic!("Expected question"),
        }
    }

    #[test]
    fn test_detect_how_to_question() {
        let service = AskService::new();
        let result = service.detect_input_type("how to use find?");
        match result {
            InputType::Question(topic) => {
                assert_eq!(topic.subject, "use find");
                assert_eq!(topic.question_type, QuestionType::HowTo);
            }
            _ => panic!("Expected question"),
        }

        // Also test simpler "how does X work?" pattern
        let result2 = service.detect_input_type("how does grep work?");
        match result2 {
            InputType::Question(topic) => {
                assert_eq!(topic.subject, "grep work");
                assert_eq!(topic.question_type, QuestionType::HowTo);
            }
            _ => panic!("Expected question"),
        }
    }

    #[test]
    fn test_detect_explain_question() {
        let service = AskService::new();
        let result = service.detect_input_type("explain pipes");
        match result {
            InputType::Question(topic) => {
                assert_eq!(topic.subject, "pipes");
                assert_eq!(topic.question_type, QuestionType::Explain);
            }
            _ => panic!("Expected question"),
        }
    }

    #[test]
    fn test_detect_command_request() {
        let service = AskService::new();

        // These should be detected as command requests, not questions
        assert_eq!(
            service.detect_input_type("list all files"),
            InputType::CommandRequest
        );
        assert_eq!(
            service.detect_input_type("find large files"),
            InputType::CommandRequest
        );
        assert_eq!(
            service.detect_input_type("show disk usage"),
            InputType::CommandRequest
        );
    }

    #[test]
    fn test_generate_command_response() {
        let service = AskService::new();
        let topic = QuestionTopic {
            subject: "grep".to_string(),
            question_type: QuestionType::WhatIs,
            original_input: "what is grep?".to_string(),
        };
        let response = service.generate_response(&topic);

        assert_eq!(response.confidence, ResponseConfidence::High);
        assert!(response.markdown.contains("grep"));
        assert!(!response.related_topics.is_empty());
    }

    #[test]
    fn test_unknown_topic_response() {
        let service = AskService::new();
        let topic = QuestionTopic {
            subject: "unknowncommand".to_string(),
            question_type: QuestionType::WhatIs,
            original_input: "what is unknowncommand?".to_string(),
        };
        let response = service.generate_response(&topic);

        assert_eq!(response.confidence, ResponseConfidence::Low);
        assert!(response.markdown.contains("don't have detailed information"));
    }
}
