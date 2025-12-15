//! Command explainer - explains shell commands in plain English
//!
//! The explainer breaks down shell commands into understandable parts and provides
//! safety warnings, alternatives, and examples.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Command explanation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Explanation {
    pub command: String,
    pub breakdown: Vec<ExplanationPart>,
    pub safety_notes: Vec<String>,
    pub alternatives: Vec<Alternative>,
}

/// Part of a command explanation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExplanationPart {
    pub part: String,
    pub explanation: String,
    pub part_type: PartType,
}

/// Type of command part
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum PartType {
    Command,
    Flag,
    Argument,
    Pipe,
    Redirection,
    Operator,
}

/// Alternative command suggestion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alternative {
    pub command: String,
    pub reason: String,
    pub benefits: Vec<String>,
}

/// Command information in knowledge base
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandInfo {
    pub name: String,
    pub description: String,
    pub common_flags: HashMap<String, String>,
    pub examples: Vec<Example>,
    #[serde(default)]
    pub safety_notes: Vec<String>,
}

/// Example command
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Example {
    pub command: String,
    pub explanation: String,
}

/// Command explainer
pub struct CommandExplainer {
    knowledge_base: HashMap<String, CommandInfo>,
}

impl CommandExplainer {
    /// Create a new command explainer
    pub fn new() -> Result<Self> {
        let knowledge_base = Self::load_knowledge_base()?;
        Ok(Self { knowledge_base })
    }

    /// Load knowledge base from embedded JSON
    fn load_knowledge_base() -> Result<HashMap<String, CommandInfo>> {
        // Load from embedded JSON or file
        let json_data = include_str!("../../knowledge_base.json");
        let kb: HashMap<String, CommandInfo> = serde_json::from_str(json_data)
            .context("Failed to parse knowledge base JSON")?;

        Ok(kb)
    }

    /// Explain a shell command
    pub fn explain(&self, command: &str) -> Result<Explanation> {
        let command = command.trim();

        // Parse command into parts
        let parts = self.parse_command(command)?;

        // Build explanation
        let mut breakdown = Vec::new();
        let mut safety_notes = Vec::new();
        let mut alternatives = Vec::new();

        // Process each part
        for part in parts.iter() {
            match part {
                ParsedPart::Command(cmd) => {
                    if let Some(info) = self.knowledge_base.get(cmd.as_str()) {
                        breakdown.push(ExplanationPart {
                            part: cmd.clone(),
                            explanation: info.description.clone(),
                            part_type: PartType::Command,
                        });

                        safety_notes.extend(info.safety_notes.clone());
                    } else {
                        breakdown.push(ExplanationPart {
                            part: cmd.clone(),
                            explanation: format!("Execute '{}' command", cmd),
                            part_type: PartType::Command,
                        });
                    }
                }
                ParsedPart::Flag(flag) => {
                    let explanation = self.explain_flag(&parts[0], flag);
                    breakdown.push(ExplanationPart {
                        part: flag.clone(),
                        explanation,
                        part_type: PartType::Flag,
                    });
                }
                ParsedPart::Argument(arg) => {
                    breakdown.push(ExplanationPart {
                        part: arg.clone(),
                        explanation: format!("With argument: {}", arg),
                        part_type: PartType::Argument,
                    });
                }
                ParsedPart::Pipe => {
                    breakdown.push(ExplanationPart {
                        part: "|".to_string(),
                        explanation: "Pipe output to next command".to_string(),
                        part_type: PartType::Pipe,
                    });
                }
                ParsedPart::Redirection(redir) => {
                    let explanation = self.explain_redirection(redir);
                    breakdown.push(ExplanationPart {
                        part: redir.clone(),
                        explanation,
                        part_type: PartType::Redirection,
                    });
                }
                ParsedPart::Operator(op) => {
                    let explanation = self.explain_operator(op);
                    breakdown.push(ExplanationPart {
                        part: op.clone(),
                        explanation,
                        part_type: PartType::Operator,
                    });
                }
            }
        }

        // Add safety warnings for dangerous patterns
        if command.contains("rm -rf") {
            safety_notes.push(
                "WARNING: Recursive force deletion is dangerous and irreversible!".to_string(),
            );
        }

        if command.starts_with("sudo") {
            safety_notes.push(
                "Running with elevated privileges - be careful!".to_string(),
            );
        }

        // Suggest alternatives
        alternatives.extend(self.suggest_alternatives(command));

        Ok(Explanation {
            command: command.to_string(),
            breakdown,
            safety_notes,
            alternatives,
        })
    }

    /// Parse command into parts
    fn parse_command(&self, command: &str) -> Result<Vec<ParsedPart>> {
        let mut parts = Vec::new();
        let tokens = self.tokenize(command);

        let mut i = 0;
        while i < tokens.len() {
            let token = &tokens[i];

            if i == 0 {
                // First token is the command
                parts.push(ParsedPart::Command(token.clone()));
            } else if token == "|" {
                parts.push(ParsedPart::Pipe);
                // Next token after pipe is another command
                if i + 1 < tokens.len() {
                    i += 1;
                    parts.push(ParsedPart::Command(tokens[i].clone()));
                }
            } else if token.starts_with('-') {
                parts.push(ParsedPart::Flag(token.clone()));
            } else if token == ">" || token == ">>" || token == "<" || token == "2>" || token == "2>&1" {
                parts.push(ParsedPart::Redirection(token.clone()));
            } else if token == "&&" || token == "||" || token == ";" {
                parts.push(ParsedPart::Operator(token.clone()));
            } else {
                parts.push(ParsedPart::Argument(token.clone()));
            }

            i += 1;
        }

        Ok(parts)
    }

    /// Tokenize command (simple whitespace split)
    fn tokenize(&self, command: &str) -> Vec<String> {
        // TODO: Handle quoted strings properly
        command
            .split_whitespace()
            .map(|s| s.to_string())
            .collect()
    }

    /// Explain a flag
    fn explain_flag(&self, command: &ParsedPart, flag: &str) -> String {
        if let ParsedPart::Command(cmd) = command {
            if let Some(info) = self.knowledge_base.get(cmd.as_str()) {
                if let Some(explanation) = info.common_flags.get(flag) {
                    return explanation.clone();
                }
            }
        }

        format!("Flag: {}", flag)
    }

    /// Explain redirection
    fn explain_redirection(&self, redir: &str) -> String {
        match redir {
            ">" => "Redirect output to file (overwrite)".to_string(),
            ">>" => "Redirect output to file (append)".to_string(),
            "<" => "Read input from file".to_string(),
            "2>" => "Redirect error output to file".to_string(),
            "2>&1" => "Redirect errors to standard output".to_string(),
            _ => format!("Redirection: {}", redir),
        }
    }

    /// Explain operator
    fn explain_operator(&self, op: &str) -> String {
        match op {
            "&&" => "Execute next command only if this succeeds".to_string(),
            "||" => "Execute next command only if this fails".to_string(),
            ";" => "Execute next command regardless of result".to_string(),
            _ => format!("Operator: {}", op),
        }
    }

    /// Suggest safer or better alternatives
    fn suggest_alternatives(&self, command: &str) -> Vec<Alternative> {
        let mut alternatives = Vec::new();

        // Suggest trash instead of rm
        if command.contains("rm ") && !command.contains("trash") {
            alternatives.push(Alternative {
                command: command.replace("rm ", "trash "),
                reason: "Use trash instead of rm for safer deletion".to_string(),
                benefits: vec![
                    "Files can be recovered from trash".to_string(),
                    "Prevents accidental data loss".to_string(),
                ],
            });
        }

        // Suggest fd instead of find
        if command.starts_with("find ") {
            let fd_cmd = command.replace("find ", "fd ");
            alternatives.push(Alternative {
                command: fd_cmd,
                reason: "fd is a faster and more user-friendly alternative".to_string(),
                benefits: vec![
                    "Faster performance".to_string(),
                    "Simpler syntax".to_string(),
                    "Better defaults".to_string(),
                ],
            });
        }

        // Suggest rg instead of grep
        if command.contains("grep ") {
            let rg_cmd = command.replace("grep ", "rg ");
            alternatives.push(Alternative {
                command: rg_cmd,
                reason: "ripgrep (rg) is faster than grep".to_string(),
                benefits: vec![
                    "10-100x faster".to_string(),
                    "Respects .gitignore by default".to_string(),
                    "Better output formatting".to_string(),
                ],
            });
        }

        alternatives
    }
}

/// Parsed command part
#[derive(Debug, Clone)]
enum ParsedPart {
    Command(String),
    Flag(String),
    Argument(String),
    Pipe,
    Redirection(String),
    Operator(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize() {
        let explainer = CommandExplainer::new().unwrap();
        let tokens = explainer.tokenize("ls -la /tmp");

        assert_eq!(tokens, vec!["ls", "-la", "/tmp"]);
    }

    #[test]
    fn test_parse_simple_command() {
        let explainer = CommandExplainer::new().unwrap();
        let parts = explainer.parse_command("ls -la").unwrap();

        assert_eq!(parts.len(), 2);
    }

    #[test]
    fn test_explain_redirection() {
        let explainer = CommandExplainer::new().unwrap();

        assert_eq!(
            explainer.explain_redirection(">"),
            "Redirect output to file (overwrite)"
        );
        assert_eq!(
            explainer.explain_redirection(">>"),
            "Redirect output to file (append)"
        );
    }

    #[test]
    fn test_suggest_alternatives() {
        let explainer = CommandExplainer::new().unwrap();
        let alternatives = explainer.suggest_alternatives("find . -name '*.txt'");

        assert!(!alternatives.is_empty());
        assert!(alternatives[0].command.contains("fd"));
    }
}
