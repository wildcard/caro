use crate::backends::{CommandGenerator, GeneratorError};
use crate::context::ExecutionContext;
use crate::models::{CommandRequest, GeneratedCommand, ShellType, SafetyLevel};
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use tracing::{debug, info};

/// Configuration for exploration behavior
#[derive(Debug, Clone)]
pub struct ExploreConfig {
    /// Whether exploration is enabled
    pub enabled: bool,
    
    /// Number of alternative commands to generate
    pub depth: usize,
    
    /// Whether to wait for exploration before prompting user
    pub wait: bool,
    
    /// File context inclusion mode
    pub files: ExploreFiles,
}

/// File context inclusion modes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExploreFiles {
    /// Always include ls output
    Always,
    
    /// Ask model if files are relevant
    Auto,
    
    /// Never include files
    Never,
}

impl Default for ExploreConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            depth: 3,
            wait: false,
            files: ExploreFiles::Auto,
        }
    }
}

/// Assessment of query complexity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplexityAssessment {
    /// Is this a complex query requiring exploration?
    pub is_complex: bool,
    
    /// Confidence in the assessment (0.0-1.0)
    pub confidence: f32,
    
    /// Reasoning for the assessment
    pub reasoning: String,
    
    /// Likely tools needed
    #[serde(default)]
    pub likely_tools: Vec<String>,
    
    /// Quick command for simple queries
    pub quick_command: Option<String>,
}

/// Tool suggestion from discovery phase
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolSuggestion {
    /// Tool name (ps, top, find, etc.)
    pub tool: String,
    
    /// Why this tool is relevant
    pub relevance: String,
    
    /// Confidence that this tool is appropriate (0.0-1.0)
    pub confidence: f32,
    
    /// Is this tool native to the platform?
    #[serde(default)]
    pub platform_native: bool,
}

/// Exploration agent that handles complexity assessment and tool discovery
pub struct ExplorationAgent {
    backend: Arc<dyn CommandGenerator>,
    context: ExecutionContext,
}

impl ExplorationAgent {
    /// Create a new exploration agent
    pub fn new(backend: Arc<dyn CommandGenerator>, context: ExecutionContext) -> Self {
        Self { backend, context }
    }
    
    /// Assess the complexity of a user query
    pub async fn assess_complexity(
        &self,
        prompt: &str,
    ) -> Result<ComplexityAssessment, GeneratorError> {
        info!("Assessing query complexity for: {}", prompt);
        
        let system_prompt = self.build_complexity_prompt(prompt);
        
        let request = CommandRequest {
            input: prompt.to_string(),
            shell: ShellType::Bash,
            safety_level: SafetyLevel::Moderate,
            context: Some(system_prompt),
            backend_preference: None,
        };
        
        let response = self.backend.generate_command(&request).await?;
        
        // Parse the assessment from response
        self.parse_complexity_assessment(&response.command)
    }
    
    /// Discover relevant command-line tools for a query
    pub async fn discover_tools(
        &self,
        prompt: &str,
        include_files: bool,
    ) -> Result<Vec<ToolSuggestion>, GeneratorError> {
        info!("Discovering tools for: {}", prompt);
        
        let system_prompt = self.build_discovery_prompt(prompt, include_files).await?;
        
        let request = CommandRequest {
            input: prompt.to_string(),
            shell: ShellType::Bash,
            safety_level: SafetyLevel::Moderate,
            context: Some(system_prompt),
            backend_preference: None,
        };
        
        let response = self.backend.generate_command(&request).await?;
        
        // Parse tool suggestions from response
        self.parse_tool_suggestions(&response.command)
    }
    
    /// Build system prompt for tool discovery
    async fn build_discovery_prompt(
        &self,
        prompt: &str,
        include_files: bool,
    ) -> Result<String, GeneratorError> {
        // Keep it short to avoid context errors
        let available_tools = self.context.available_commands
            .iter()
            .take(15)  // Only top 15 commands
            .map(|s| s.as_str())
            .collect::<Vec<_>>()
            .join(", ");
        
        let base_prompt = format!(
            r#"Query: "{}"
Platform: {}
Available: {}

List 2-3 tools to use. JSON format:
{{"tools": [{{"tool": "ps", "relevance": "monitors processes", "confidence": 0.9, "platform_native": true}}]}}"#,
            prompt,
            self.context.os,
            available_tools
        );
        
        Ok(base_prompt)
    }
    
    /// Parse tool suggestions from model response
    fn parse_tool_suggestions(
        &self,
        response: &str,
    ) -> Result<Vec<ToolSuggestion>, GeneratorError> {
        debug!("Parsing tool suggestions from: {}", response);
        
        // Try to parse as JSON first
        if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(response) {
            if let Some(tools_array) = parsed.get("tools").and_then(|v| v.as_array()) {
                let mut suggestions = Vec::new();
                
                for tool_val in tools_array {
                    if let Ok(tool) = serde_json::from_value::<ToolSuggestion>(tool_val.clone()) {
                        suggestions.push(tool);
                    }
                }
                
                if !suggestions.is_empty() {
                    return Ok(suggestions);
                }
            }
        }
        
        // Try to extract JSON from response
        if let Some(start) = response.find('{') {
            if let Some(end) = response.rfind('}') {
                let json_part = &response[start..=end];
                if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(json_part) {
                    if let Some(tools_array) = parsed.get("tools").and_then(|v| v.as_array()) {
                        let mut suggestions = Vec::new();
                        
                        for tool_val in tools_array {
                            if let Ok(tool) = serde_json::from_value::<ToolSuggestion>(tool_val.clone()) {
                                suggestions.push(tool);
                            }
                        }
                        
                        if !suggestions.is_empty() {
                            return Ok(suggestions);
                        }
                    }
                }
            }
        }
        
        // Fallback: extract tool names from text
        let tool_names = self.extract_tool_names_from_text(response);
        if !tool_names.is_empty() {
            return Ok(tool_names
                .into_iter()
                .map(|name| ToolSuggestion {
                    tool: name.clone(),
                    relevance: "Extracted from response".to_string(),
                    confidence: 0.5,
                    platform_native: self.context.available_commands.contains(&name),
                })
                .collect());
        }
        
        // No tools found
        Err(GeneratorError::ParseError {
            content: format!("Could not extract tool suggestions from: {}", response),
        })
    }
    
    /// Extract tool names from free-form text
    fn extract_tool_names_from_text(&self, text: &str) -> Vec<String> {
        let mut found_tools = Vec::new();
        
        // Look for known commands in the text
        for cmd in &self.context.available_commands {
            if text.contains(cmd) && !found_tools.contains(cmd) {
                found_tools.push(cmd.clone());
            }
        }
        
        found_tools
    }
    
    /// Build system prompt for complexity assessment
    fn build_complexity_prompt(&self, prompt: &str) -> String {
        format!(r#"Task: Assess query complexity

Query: "{}"
Platform: {}
Shell: {}
Available commands: {}

Determine if this query is SIMPLE or COMPLEX:

SIMPLE queries:
- Single obvious command (ls, pwd, date, whoami)
- No platform-specific concerns
- No multiple approaches needed
- Examples: "list files", "current directory", "what time"

COMPLEX queries:
- Multiple tools could work
- Platform-specific syntax required (BSD vs GNU)
- Needs careful flag selection
- User might benefit from alternatives
- Examples: "top CPU processes", "listening ports", "disk usage sorted"

Respond with JSON:
{{
  "complexity": "simple" or "complex",
  "confidence": 0.0-1.0,
  "reasoning": "brief explanation",
  "likely_tools": ["tool1", "tool2"],
  "quick_command": "command for simple queries only"
}}

IMPORTANT: 
- If simple, MUST provide quick_command
- If complex, quick_command should be null or empty
- Be conservative: when in doubt, mark as complex"#,
            prompt,
            self.context.os,
            self.context.shell,
            self.format_available_commands()
        )
    }
    
    /// Format available commands for prompt
    fn format_available_commands(&self) -> String {
        let commands: Vec<_> = self.context.available_commands
            .iter()
            .take(20)  // First 20 to keep prompt short
            .cloned()
            .collect();
        commands.join(", ")
    }
    
    /// Parse complexity assessment from model response
    fn parse_complexity_assessment(
        &self,
        response: &str,
    ) -> Result<ComplexityAssessment, GeneratorError> {
        debug!("Parsing complexity assessment from: {}", response);
        
        // Try to parse as JSON first
        if let Ok(assessment) = serde_json::from_str::<ComplexityAssessment>(response) {
            return Ok(assessment);
        }
        
        // Try to extract JSON from response
        if let Some(start) = response.find('{') {
            if let Some(end) = response.rfind('}') {
                let json_part = &response[start..=end];
                if let Ok(assessment) = serde_json::from_str::<ComplexityAssessment>(json_part) {
                    return Ok(assessment);
                }
            }
        }
        
        // Fallback: parse manually
        let is_complex = response.to_lowercase().contains("complex");
        let has_command = response.contains("cmd") || response.contains("command");
        
        // Try to extract command if it looks simple
        let quick_command = if !is_complex && has_command {
            self.extract_command_from_text(response)
        } else {
            None
        };
        
        Ok(ComplexityAssessment {
            is_complex,
            confidence: 0.5,  // Low confidence on fallback
            reasoning: "Fallback parsing used".to_string(),
            likely_tools: vec![],
            quick_command,
        })
    }
    
    /// Extract command from free-form text
    fn extract_command_from_text(&self, text: &str) -> Option<String> {
        // Look for common command patterns
        for line in text.lines() {
            let line = line.trim();
            
            // Skip empty lines and comments
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            
            // Check if line looks like a shell command
            if self.looks_like_command(line) {
                return Some(line.to_string());
            }
        }
        
        None
    }
    
    /// Check if a string looks like a shell command
    fn looks_like_command(&self, s: &str) -> bool {
        // Must start with a known command
        let words: Vec<&str> = s.split_whitespace().collect();
        if words.is_empty() {
            return false;
        }
        
        let first_word = words[0];
        
        // Check against available commands
        self.context.available_commands.contains(&first_word.to_string())
    }
    
    /// Check if files context should be included for this query
    pub async fn should_include_files(
        &self,
        prompt: &str,
        mode: ExploreFiles,
    ) -> Result<bool, GeneratorError> {
        match mode {
            ExploreFiles::Always => Ok(true),
            ExploreFiles::Never => Ok(false),
            ExploreFiles::Auto => {
                debug!("Auto-detecting if files are relevant for: {}", prompt);
                
                // Ask model if prompt is file-related
                let request = CommandRequest {
                    input: format!(
                        "Is this query about local files or directories?\n\
                         Query: '{}'\n\n\
                         Answer with JSON: {{\"file_related\": true/false, \"reason\": \"why\"}}",
                        prompt
                    ),
                    shell: ShellType::Bash,
                    safety_level: SafetyLevel::Moderate,
                    context: None,
                    backend_preference: None,
                };
                
                let response = self.backend.generate_command(&request).await?;
                
                // Try to parse JSON response
                if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&response.command) {
                    if let Some(file_related) = parsed.get("file_related").and_then(|v| v.as_bool()) {
                        debug!("File relevance: {}", file_related);
                        return Ok(file_related);
                    }
                }
                
                // Fallback: simple keyword check
                let keywords = ["file", "directory", "folder", "path", "ls", "find", "du"];
                let prompt_lower = prompt.to_lowercase();
                let is_file_related = keywords.iter().any(|kw| prompt_lower.contains(kw));
                
                debug!("File relevance (fallback): {}", is_file_related);
                Ok(is_file_related)
            }
        }
    }
    
    /// Get file context (ls output) for current directory
    pub async fn get_file_context(&self) -> Result<String, GeneratorError> {
        use std::process::Command;
        
        let output = Command::new("ls")
            .arg("-lh")
            .current_dir(&self.context.cwd)
            .output()
            .map_err(|e| GeneratorError::BackendUnavailable {
                reason: format!("Failed to run ls: {}", e),
            })?;
        
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            Ok(format!("Current directory: {}\n{}", 
                self.context.cwd.display(), 
                stdout
            ))
        } else {
            Ok(format!("Current directory: {}", self.context.cwd.display()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_complexity_assessment_parsing() {
        let json = r#"{
            "complexity": "complex",
            "confidence": 0.9,
            "reasoning": "Multiple tools available",
            "likely_tools": ["ps", "top"],
            "quick_command": null
        }"#;
        
        let assessment: ComplexityAssessment = serde_json::from_str(json).unwrap();
        assert!(assessment.is_complex);
        assert_eq!(assessment.confidence, 0.9);
        assert_eq!(assessment.likely_tools, vec!["ps", "top"]);
    }
    
    #[test]
    fn test_simple_query_parsing() {
        let json = r#"{
            "complexity": "simple",
            "confidence": 0.95,
            "reasoning": "Single obvious command",
            "likely_tools": ["ls"],
            "quick_command": "ls -lh"
        }"#;
        
        let assessment: ComplexityAssessment = serde_json::from_str(json).unwrap();
        assert!(!assessment.is_complex);
        assert_eq!(assessment.quick_command, Some("ls -lh".to_string()));
    }
}
