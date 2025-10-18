//! Basic command generation tests following TDD principles
//!
//! This demonstrates the Red-Green-Refactor cycle of TDD

// Simple domain model for command generation
#[derive(Debug, Clone)]
struct CommandRequest {
    natural_language: String,
    shell_type: String,
    safety_level: String,
}

#[derive(Debug, Clone, PartialEq)]
struct GeneratedCommand {
    command: String,
    explanation: String,
    is_safe: bool,
    confidence: f64,
}

/// Command generator that converts natural language to shell commands
struct CommandGenerator {
    safety_checker: SafetyChecker,
}

impl CommandGenerator {
    fn new() -> Self {
        Self {
            safety_checker: SafetyChecker::new(),
        }
    }

    fn generate(&self, request: CommandRequest) -> GeneratedCommand {
        // Check for safety first
        if self.safety_checker.is_dangerous(&request.natural_language) {
            return GeneratedCommand {
                command: String::new(),
                explanation: "Command blocked due to safety concerns".to_string(),
                is_safe: false,
                confidence: 1.0,
            };
        }

        // Pattern matching for common commands
        let command = match request.natural_language.to_lowercase() {
            s if s.contains("list") && s.contains("files") => GeneratedCommand {
                command: "ls -la".to_string(),
                explanation: "List all files including hidden ones".to_string(),
                is_safe: true,
                confidence: 0.95,
            },
            s if s.contains("current") && s.contains("directory") => GeneratedCommand {
                command: "pwd".to_string(),
                explanation: "Print working directory".to_string(),
                is_safe: true,
                confidence: 0.98,
            },
            s if s.contains("disk") && s.contains("usage") => GeneratedCommand {
                command: "df -h".to_string(),
                explanation: "Display disk usage in human-readable format".to_string(),
                is_safe: true,
                confidence: 0.90,
            },
            _ => GeneratedCommand {
                command: String::new(),
                explanation: "Unable to understand the request".to_string(),
                is_safe: false,
                confidence: 0.0,
            },
        };

        command
    }
}

/// Safety checker to prevent dangerous commands
struct SafetyChecker {
    dangerous_patterns: Vec<&'static str>,
}

impl SafetyChecker {
    fn new() -> Self {
        Self {
            dangerous_patterns: vec![
                "rm -rf /",
                "delete everything",
                "format",
                "root",
                "sudo rm",
                ":(){ :|:& };:", // Fork bomb
            ],
        }
    }

    fn is_dangerous(&self, input: &str) -> bool {
        let lower = input.to_lowercase();
        self.dangerous_patterns
            .iter()
            .any(|pattern| lower.contains(pattern))
    }
}

// ============= TESTS =============

#[test]
fn test_simple_ls_command() {
    let generator = CommandGenerator::new();
    let request = CommandRequest {
        natural_language: "list all files".to_string(),
        shell_type: "bash".to_string(),
        safety_level: "moderate".to_string(),
    };

    let result = generator.generate(request);

    assert_eq!(result.command, "ls -la");
    assert!(result.is_safe);
    assert!(result.confidence > 0.9);
}

#[test]
fn test_pwd_command() {
    let generator = CommandGenerator::new();
    let request = CommandRequest {
        natural_language: "show current directory".to_string(),
        shell_type: "bash".to_string(),
        safety_level: "moderate".to_string(),
    };

    let result = generator.generate(request);

    assert_eq!(result.command, "pwd");
    assert!(result.is_safe);
}

#[test]
fn test_disk_usage_command() {
    let generator = CommandGenerator::new();
    let request = CommandRequest {
        natural_language: "check disk usage".to_string(),
        shell_type: "bash".to_string(),
        safety_level: "moderate".to_string(),
    };

    let result = generator.generate(request);

    assert_eq!(result.command, "df -h");
    assert!(result.is_safe);
}

#[test]
fn test_dangerous_command_detection() {
    let generator = CommandGenerator::new();
    let request = CommandRequest {
        natural_language: "delete everything in root".to_string(),
        shell_type: "bash".to_string(),
        safety_level: "strict".to_string(),
    };

    let result = generator.generate(request);

    assert!(!result.is_safe);
    assert!(result.command.is_empty());
    assert_eq!(result.explanation, "Command blocked due to safety concerns");
}

#[test]
fn test_fork_bomb_detection() {
    let generator = CommandGenerator::new();
    let request = CommandRequest {
        natural_language: "run :(){ :|:& };:".to_string(),
        shell_type: "bash".to_string(),
        safety_level: "strict".to_string(),
    };

    let result = generator.generate(request);

    assert!(!result.is_safe);
    assert!(result.command.is_empty());
}

#[test]
fn test_unknown_command() {
    let generator = CommandGenerator::new();
    let request = CommandRequest {
        natural_language: "do something random".to_string(),
        shell_type: "bash".to_string(),
        safety_level: "moderate".to_string(),
    };

    let result = generator.generate(request);

    assert!(!result.is_safe);
    assert!(result.command.is_empty());
    assert_eq!(result.confidence, 0.0);
}
