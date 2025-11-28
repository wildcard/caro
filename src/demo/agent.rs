// Demo agent - Enhanced command generator for demonstration mode

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::backends::{BackendInfo, CommandGenerator, GeneratorError};
use crate::models::{BackendType, CommandRequest, GeneratedCommand, RiskLevel};

use super::prompts::{get_critique_for_request, get_showcase_suggestions};

/// Demo agent that showcases cmdai capabilities
pub struct DemoAgent {
    config: DemoConfig,
}

/// Configuration for demo mode behavior
#[derive(Debug, Clone)]
pub struct DemoConfig {
    /// Number of alternatives to generate (default: 3)
    pub num_alternatives: usize,
    /// Whether to be critical of inappropriate requests (default: true)
    pub enable_critique: bool,
    /// Whether to provide follow-up suggestions (default: true)
    pub enable_suggestions: bool,
}

impl Default for DemoConfig {
    fn default() -> Self {
        Self {
            num_alternatives: 3,
            enable_critique: true,
            enable_suggestions: true,
        }
    }
}

/// Demo response format with enhanced fields
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DemoResponse {
    pub command: String,
    pub explanation: String,
    pub alternatives: Vec<String>,
    pub suggestions: Vec<String>,
    pub critique: Option<String>,
}

impl DemoAgent {
    /// Create a new demo agent with default configuration
    pub fn new() -> Self {
        Self {
            config: DemoConfig::default(),
        }
    }

    /// Create a demo agent with custom configuration
    pub fn with_config(config: DemoConfig) -> Self {
        Self { config }
    }

    /// Parse demo response from generated text
    fn parse_demo_response(&self, text: &str) -> Result<DemoResponse, GeneratorError> {
        // Try to parse as JSON first
        if let Ok(response) = serde_json::from_str::<DemoResponse>(text) {
            return Ok(response);
        }

        // Fallback: extract JSON from markdown code blocks
        if let Some(json_start) = text.find('{') {
            if let Some(json_end) = text.rfind('}') {
                let json_str = &text[json_start..=json_end];
                if let Ok(response) = serde_json::from_str::<DemoResponse>(json_str) {
                    return Ok(response);
                }
            }
        }

        Err(GeneratorError::ParseError {
            content: text.to_string(),
        })
    }

    /// Generate enhanced demo command with multiple alternatives
    fn generate_demo_command(&self, request: &CommandRequest) -> DemoResponse {
        let input = &request.input;
        let input_lower = input.to_lowercase();

        // Check if request deserves critique
        let critique = if self.config.enable_critique {
            get_critique_for_request(input)
        } else {
            None
        };

        // Generate appropriate command based on input
        let (command, explanation, alternatives) = if input_lower.contains("list")
            && input_lower.contains("file")
        {
            self.generate_list_files_demo()
        } else if input_lower.contains("find") && input_lower.contains("large") {
            self.generate_find_large_files_demo()
        } else if input_lower.contains("process") || input_lower.contains("cpu") {
            self.generate_process_demo()
        } else if input_lower.contains("disk") || input_lower.contains("space") {
            self.generate_disk_space_demo()
        } else if critique.is_some() {
            // For critiqued requests, show impressive alternatives
            self.generate_showcase_demo()
        } else {
            // Generic but impressive command
            self.generate_generic_demo(input)
        };

        // Get suggestions if enabled
        let suggestions = if self.config.enable_suggestions {
            get_showcase_suggestions(input)
        } else {
            Vec::new()
        };

        DemoResponse {
            command,
            explanation,
            alternatives,
            suggestions,
            critique,
        }
    }

    fn generate_list_files_demo(&self) -> (String, String, Vec<String>) {
        let command = "ls -lAh --color=auto --group-directories-first".to_string();
        let explanation =
            "This command showcases multiple POSIX features:\n\
             • -l: Long format with permissions, owner, size, and timestamps\n\
             • -A: Show hidden files (except . and ..)\n\
             • -h: Human-readable sizes (KB, MB, GB)\n\
             • --color: Color-coded output for better visibility\n\
             • --group-directories-first: Directories listed before files\n\
             \nThis demonstrates cmdai's understanding of user intent and best practices!".to_string();
        let alternatives = vec![
            "find . -maxdepth 1 -type f -ls  # More detailed info with find".to_string(),
            "tree -L 1 -h --dirsfirst  # Visual tree structure".to_string(),
            "exa -la --group-directories-first --git  # Modern ls alternative with git status".to_string(),
        ];

        (command, explanation, alternatives)
    }

    fn generate_find_large_files_demo(&self) -> (String, String, Vec<String>) {
        let command = "find . -type f -size +100M -exec ls -lh {} \\; | sort -k5 -hr".to_string();
        let explanation =
            "Advanced file search demonstrating cmdai's capability:\n\
             • find: Recursively search current directory\n\
             • -type f: Files only (exclude directories)\n\
             • -size +100M: Larger than 100MB\n\
             • -exec ls -lh: Execute ls with human-readable sizes\n\
             • sort -k5 -hr: Sort by size (column 5), largest first\n\
             \nThis shows cmdai's ability to chain complex operations safely!".to_string();
        let alternatives = vec![
            "du -ah . | sort -rh | head -20  # Top 20 largest files/directories".to_string(),
            "find . -type f -printf '%s %p\\n' | sort -rn | head -10  # Fastest approach".to_string(),
            "ncdu .  # Interactive disk usage analyzer (if installed)".to_string(),
        ];

        (command, explanation, alternatives)
    }

    fn generate_process_demo(&self) -> (String, String, Vec<String>) {
        let command = "ps aux --sort=-%mem | head -11".to_string();
        let explanation =
            "Process monitoring showcasing cmdai's system administration knowledge:\n\
             • ps aux: All processes with detailed info\n\
             • --sort=-%mem: Sort by memory usage (descending)\n\
             • head -11: Top 10 processes (plus header)\n\
             \nDemonstrates understanding of system monitoring best practices!".to_string();
        let alternatives = vec![
            "top -o %MEM -n 1 | head -20  # Dynamic top snapshot".to_string(),
            "htop --sort-key PERCENT_MEM  # Interactive process viewer (if installed)".to_string(),
            "ps -eo pid,ppid,cmd,%mem,%cpu --sort=-%mem | head -11  # Custom columns".to_string(),
        ];

        (command, explanation, alternatives)
    }

    fn generate_disk_space_demo(&self) -> (String, String, Vec<String>) {
        let command = "df -h | grep -v tmpfs | grep -v loop".to_string();
        let explanation =
            "Filesystem usage analysis showcasing practical filtering:\n\
             • df -h: Disk free in human-readable format\n\
             • grep -v tmpfs: Exclude temporary filesystems\n\
             • grep -v loop: Exclude loop devices\n\
             \nShows cmdai's ability to provide clean, actionable output!".to_string();
        let alternatives = vec![
            "df -h --output=source,size,used,avail,pcent,target  # Custom columns".to_string(),
            "du -sh /* 2>/dev/null | sort -rh  # Directory-level usage".to_string(),
            "lsblk -o NAME,SIZE,USED,AVAIL,MOUNTPOINT  # Block device overview".to_string(),
        ];

        (command, explanation, alternatives)
    }

    fn generate_showcase_demo(&self) -> (String, String, Vec<String>) {
        let command = "find . -type f -name '*.log' -mtime +30 -exec gzip {} \\;".to_string();
        let explanation =
            "Instead of that request, let me showcase a REAL capability:\n\
             This command finds and compresses old log files:\n\
             • find: Recursively search current directory\n\
             • -name '*.log': Match log files\n\
             • -mtime +30: Modified more than 30 days ago\n\
             • -exec gzip: Compress each file\n\
             \nThis is the kind of powerful automation cmdai excels at!".to_string();
        let alternatives = vec![
            "find . -name '*.log' -mtime +30 -print0 | xargs -0 tar czf old_logs.tar.gz  # Archive to single file".to_string(),
            "find . -name '*.log' -mtime +30 -delete  # Remove old logs (careful!)".to_string(),
            "journalctl --vacuum-time=30d  # Clean systemd journal (if applicable)".to_string(),
        ];

        (command, explanation, alternatives)
    }

    fn generate_generic_demo(&self, input: &str) -> (String, String, Vec<String>) {
        let command = format!("echo 'Processing: {}'", input);
        let explanation = format!(
            "Generated command for: '{}'\n\
             \nWhile I can process this request, let me show you more impressive capabilities!\n\
             cmdai excels at:\n\
             • Complex file operations with safety validation\n\
             • System administration tasks\n\
             • Data processing pipelines\n\
             • Multi-step command chaining\n\
             \nTry asking for something more advanced!",
            input
        );
        let alternatives = vec![
            "# Try: 'find all PDF files larger than 10MB'".to_string(),
            "# Try: 'show top 5 memory-consuming processes'".to_string(),
            "# Try: 'archive logs older than 30 days'".to_string(),
        ];

        (command, explanation, alternatives)
    }
}

impl Default for DemoAgent {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl CommandGenerator for DemoAgent {
    async fn generate_command(
        &self,
        request: &CommandRequest,
    ) -> Result<GeneratedCommand, GeneratorError> {
        // Generate demo response
        let demo_response = self.generate_demo_command(request);

        // Build explanation with critique if present
        let mut full_explanation = if let Some(critique) = &demo_response.critique {
            format!("⚠️  DEMO MODE CRITIQUE:\n{}\n\n", critique)
        } else {
            String::new()
        };
        full_explanation.push_str(&demo_response.explanation);

        // Add suggestions
        if !demo_response.suggestions.is_empty() {
            full_explanation.push_str("\n\n💡 SUGGESTED NEXT STEPS:\n");
            for suggestion in &demo_response.suggestions {
                full_explanation.push_str(&format!("   • {}\n", suggestion));
            }
        }

        Ok(GeneratedCommand {
            command: demo_response.command,
            explanation: full_explanation,
            safety_level: RiskLevel::Safe, // Demo mode generates safe commands
            estimated_impact: Default::default(),
            alternatives: demo_response.alternatives,
            backend_used: "demo".to_string(),
            generation_time_ms: 10, // Instant generation
            confidence_score: 0.99, // High confidence in demo mode
        })
    }

    async fn is_available(&self) -> bool {
        true // Always available
    }

    fn backend_info(&self) -> BackendInfo {
        BackendInfo {
            backend_type: BackendType::Ollama, // Placeholder
            model_name: "demo-showcase-agent".to_string(),
            supports_streaming: false,
            max_tokens: 2000,
            typical_latency_ms: 10,
            memory_usage_mb: 10,
            version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }

    async fn shutdown(&self) -> Result<(), GeneratorError> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_demo_agent_list_files() {
        let agent = DemoAgent::new();
        let request = CommandRequest {
            input: "list all files".to_string(),
            context: None,
            shell: crate::models::ShellType::Bash,
            safety_level: crate::models::SafetyLevel::Moderate,
            backend_preference: None,
        };

        let result = agent.generate_command(&request).await.unwrap();
        assert!(result.command.contains("ls"));
        assert!(!result.alternatives.is_empty());
        assert!(result.explanation.contains("cmdai"));
    }

    #[tokio::test]
    async fn test_demo_agent_critique() {
        let agent = DemoAgent::new();
        let request = CommandRequest {
            input: "hello world".to_string(),
            context: None,
            shell: crate::models::ShellType::Bash,
            safety_level: crate::models::SafetyLevel::Moderate,
            backend_preference: None,
        };

        let result = agent.generate_command(&request).await.unwrap();
        assert!(result.explanation.contains("CRITIQUE"));
    }
}
