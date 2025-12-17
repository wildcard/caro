//! Query interface for retrieving command documentation

use super::{VectorStoreClient, QueryResult};
use crate::indexing::{ManPageDocument, ManSection};
use anyhow::Result;
use std::collections::HashMap;
use tracing::{debug, info};

/// Command documentation retrieved from vector store
#[derive(Debug, Clone)]
pub struct CommandDoc {
    /// Command name
    pub command: String,

    /// Short description
    pub description: String,

    /// Command synopsis/usage
    pub synopsis: Option<String>,

    /// Relevant options
    pub options: Vec<String>,

    /// Example usage
    pub examples: Vec<String>,

    /// Version information
    pub version: Option<String>,

    /// Relevance score
    pub score: f32,
}

/// Query interface for command documentation
pub struct CommandDocQuery {
    client: VectorStoreClient,
}

impl CommandDocQuery {
    /// Create a new query interface
    pub fn new(client: VectorStoreClient) -> Self {
        Self { client }
    }

    /// Query for command documentation by user intent
    pub async fn query_by_intent(
        &self,
        user_intent: &str,
        limit: usize,
    ) -> Result<Vec<CommandDoc>> {
        info!("Querying commands for intent: '{}'", user_intent);

        let results = self.client.query(user_intent, limit).await?;

        // Group results by command
        let mut command_docs = HashMap::new();

        for result in results {
            let cmd = &result.document.command;

            let entry = command_docs.entry(cmd.clone()).or_insert_with(|| CommandDoc {
                command: cmd.clone(),
                description: String::new(),
                synopsis: None,
                options: Vec::new(),
                examples: Vec::new(),
                version: result.document.metadata.version.clone(),
                score: result.score,
            });

            // Populate from document section
            match result.document.section {
                ManSection::Name | ManSection::Description => {
                    if entry.description.is_empty() {
                        entry.description = result.document.content.clone();
                    }
                }
                ManSection::Synopsis => {
                    entry.synopsis = Some(result.document.content.clone());
                }
                ManSection::Options => {
                    // Parse options from content
                    let parsed_options = Self::parse_options(&result.document.content);
                    entry.options.extend(parsed_options);
                }
                ManSection::Examples => {
                    // Parse examples from content
                    let parsed_examples = Self::parse_examples(&result.document.content);
                    entry.examples.extend(parsed_examples);
                }
                _ => {}
            }
        }

        let mut docs: Vec<CommandDoc> = command_docs.into_values().collect();
        docs.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());

        Ok(docs)
    }

    /// Query for specific commands
    pub async fn query_commands(
        &self,
        command_names: &[String],
    ) -> Result<HashMap<String, CommandDoc>> {
        let mut docs = HashMap::new();

        for command in command_names {
            // Query with command name
            let query_text = format!("command: {}", command);
            let results = self.query_by_intent(&query_text, 5).await?;

            // Take the best match
            if let Some(doc) = results.into_iter().next() {
                docs.insert(command.clone(), doc);
            }
        }

        Ok(docs)
    }

    /// Parse options from OPTIONS section content
    fn parse_options(content: &str) -> Vec<String> {
        let mut options = Vec::new();

        for line in content.lines() {
            let trimmed = line.trim();

            // Look for lines starting with - or --
            if trimmed.starts_with('-') {
                // Extract just the option and brief description
                if let Some((opt, desc)) = trimmed.split_once(char::is_whitespace) {
                    let formatted = format!("{} - {}", opt.trim(), desc.trim());
                    options.push(formatted);
                } else {
                    options.push(trimmed.to_string());
                }
            }
        }

        // Limit to top 10 most relevant options
        options.truncate(10);
        options
    }

    /// Parse examples from EXAMPLES section content
    fn parse_examples(content: &str) -> Vec<String> {
        let mut examples = Vec::new();
        let mut current_example = String::new();

        for line in content.lines() {
            let trimmed = line.trim();

            // Look for command-like lines (start with command name or $)
            if trimmed.starts_with('$') || trimmed.starts_with("  ") {
                if !current_example.is_empty() {
                    examples.push(current_example.trim().to_string());
                    current_example.clear();
                }
                current_example = trimmed.trim_start_matches('$').trim().to_string();
            } else if !trimmed.is_empty() && !current_example.is_empty() {
                // Description line
                current_example.push_str(" # ");
                current_example.push_str(trimmed);
            }
        }

        if !current_example.is_empty() {
            examples.push(current_example);
        }

        // Limit to top 3 examples
        examples.truncate(3);
        examples
    }

    /// Format command documentation for prompt inclusion
    pub fn format_for_prompt(docs: &[CommandDoc]) -> String {
        let mut output = String::new();

        output.push_str("AVAILABLE COMMAND DOCUMENTATION:\n\n");

        for doc in docs {
            output.push_str(&format!("Command: {}\n", doc.command));

            if let Some(version) = &doc.version {
                output.push_str(&format!("Version: {}\n", version));
            }

            if !doc.description.is_empty() {
                output.push_str(&format!("Description: {}\n", doc.description));
            }

            if let Some(synopsis) = &doc.synopsis {
                output.push_str(&format!("\nUsage:\n{}\n", synopsis));
            }

            if !doc.options.is_empty() {
                output.push_str("\nRelevant Options:\n");
                for option in &doc.options {
                    output.push_str(&format!("  {}\n", option));
                }
            }

            if !doc.examples.is_empty() {
                output.push_str("\nExamples:\n");
                for example in &doc.examples {
                    output.push_str(&format!("  {}\n", example));
                }
            }

            output.push_str(&format!("\nRelevance Score: {:.2}\n", doc.score));
            output.push_str("\n---\n\n");
        }

        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_options() {
        let content = r#"
    -i, --ignore-case    Ignore case distinctions
    -r, --recursive      Read all files under each directory
    -n, --line-number    Prefix each line with line number
    -v, --invert-match   Invert the sense of matching
        "#;

        let options = CommandDocQuery::parse_options(content);

        assert!(options.len() > 0);
        assert!(options.iter().any(|opt| opt.contains("-i")));
        assert!(options.iter().any(|opt| opt.contains("recursive")));
    }

    #[test]
    fn test_parse_examples() {
        let content = r#"
$ grep "TODO" *.rs
    Search for TODO in all Rust files

$ grep -r "pattern" .
    Recursively search for pattern
        "#;

        let examples = CommandDocQuery::parse_examples(content);

        assert!(examples.len() > 0);
        assert!(examples.iter().any(|ex| ex.contains("grep")));
    }

    #[test]
    fn test_format_for_prompt() {
        let docs = vec![
            CommandDoc {
                command: "grep".to_string(),
                description: "Search for patterns in files".to_string(),
                synopsis: Some("grep [OPTIONS] PATTERN [FILE...]".to_string()),
                options: vec![
                    "-i - Ignore case".to_string(),
                    "-r - Recursive search".to_string(),
                ],
                examples: vec!["grep -r TODO .".to_string()],
                version: Some("grep 3.7".to_string()),
                score: 0.95,
            },
        ];

        let formatted = CommandDocQuery::format_for_prompt(&docs);

        assert!(formatted.contains("grep"));
        assert!(formatted.contains("Search for patterns"));
        assert!(formatted.contains("-i - Ignore case"));
        assert!(formatted.contains("grep -r TODO"));
    }
}
