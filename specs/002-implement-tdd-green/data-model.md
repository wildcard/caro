# Data Model: TDD GREEN Phase

**Feature**: 002-implement-tdd-green | **Date**: 2025-10-01 | **Phase**: 1

This document defines the core data model entities for caro's command generation system, extracted from contract tests and feature specifications.

## Overview

The data model consists of 7 core entities organized in three layers:
1. **Request/Response Models**: CommandRequest, GeneratedCommand
2. **Configuration Enums**: ShellType, RiskLevel, SafetyLevel, BackendType
3. **Metadata Structures**: BackendInfo

All entities use serde for serialization to support JSON/YAML output formats.

## State Flow Diagram

```
User Input (String)
  ↓
CommandRequest {input, shell_type, safety_level, context}
  ↓
[CommandGenerator::generate_command() - async]
  ↓
GeneratedCommand {command, explanation, safety_level, ...}
  ↓
[SafetyValidator::validate() - sync]
  ↓
ValidationResult {risk_level, explanation, patterns_matched}
  ↓
[User Confirmation if RiskLevel > Moderate]
  ↓
Approved Command (String) → [Future: Execution Module]
```

## Entity Definitions

### 1. CommandRequest

**Purpose**: Encapsulates user input and generation preferences for command generation.

**Structure**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandRequest {
    /// Natural language description of desired command
    pub input: String,

    /// Target shell type for command generation
    pub shell_type: ShellType,

    /// User's safety preference level
    pub safety_level: SafetyLevel,

    /// Optional additional context (current directory, environment info)
    pub context: Option<String>,
}
```

**Fields**:
- `input: String` - Natural language command description (e.g., "list all files")
  - Validation: Must be non-empty, trimmed
  - Example: `"find all PDF files modified in the last week"`

- `shell_type: ShellType` - Target shell for command syntax
  - Validation: Enum variant, defaults to detected shell
  - Example: `ShellType::Bash`

- `safety_level: SafetyLevel` - User's risk tolerance preference
  - Validation: Enum variant, defaults to Moderate
  - Example: `SafetyLevel::Strict`

- `context: Option<String>` - Additional execution context
  - Validation: Optional, trimmed if present
  - Example: `Some("/home/user/projects")` (current directory)

**Constructor**:
```rust
impl CommandRequest {
    pub fn new(input: impl Into<String>, shell_type: ShellType) -> Self {
        Self {
            input: input.into().trim().to_string(),
            shell_type,
            safety_level: SafetyLevel::default(),
            context: None,
        }
    }

    pub fn with_safety(mut self, level: SafetyLevel) -> Self {
        self.safety_level = level;
        self
    }

    pub fn with_context(mut self, ctx: impl Into<String>) -> Self {
        self.context = Some(ctx.into());
        self
    }
}
```

**Validation Rules**:
- Input must not be empty after trimming
- Shell type must be valid enum variant
- Safety level must be valid enum variant
- Context, if present, should be meaningful (not empty/whitespace)

**Relationships**:
- Input to `CommandGenerator::generate_command()`
- Created by CLI argument parser
- Immutable after creation (value object pattern)

**Example**:
```rust
let request = CommandRequest::new("list all files", ShellType::Bash)
    .with_safety(SafetyLevel::Strict)
    .with_context("/home/user");
```

---

### 2. GeneratedCommand

**Purpose**: Represents a generated command with metadata and safety assessment.

**Structure**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedCommand {
    /// The actual shell command string
    pub command: String,

    /// Human-readable explanation of what the command does
    pub explanation: String,

    /// Assessed safety risk level
    pub safety_level: RiskLevel,

    /// Description of command's potential impact
    pub estimated_impact: String,

    /// Alternative safer or equivalent commands
    pub alternatives: Vec<String>,

    /// Backend that generated this command
    pub backend_used: String,

    /// Time taken to generate (milliseconds)
    pub generation_time_ms: u64,

    /// Model's confidence in correctness (0.0-1.0)
    pub confidence_score: f64,
}
```

**Fields**:
- `command: String` - The generated shell command
  - Validation: Non-empty, POSIX-compliant syntax
  - Example: `"ls -la | grep .pdf"`

- `explanation: String` - Human-readable description
  - Validation: Non-empty, clear language
  - Example: `"Lists all files in long format and filters for PDF files"`

- `safety_level: RiskLevel` - Assessed risk
  - Validation: Enum variant from SafetyValidator
  - Example: `RiskLevel::Safe`

- `estimated_impact: String` - Impact description
  - Validation: Non-empty, describes potential effects
  - Example: `"Read-only operation, no system changes"`

- `alternatives: Vec<String>` - Alternative commands
  - Validation: Each alternative is valid command
  - Example: `vec!["find . -name '*.pdf'"]`

- `backend_used: String` - Backend identifier
  - Validation: Non-empty, matches BackendInfo.model_name
  - Example: `"mock-backend-v1"`

- `generation_time_ms: u64` - Generation latency
  - Validation: Positive number, typically <2000ms
  - Example: `142` (142ms)

- `confidence_score: f64` - Model confidence
  - Validation: Range [0.0, 1.0]
  - Example: `0.95` (95% confident)

**Validation Rules**:
- Command must be non-empty and syntactically valid
- Confidence score must be in [0.0, 1.0] range
- All string fields must be non-empty (no whitespace-only)
- Alternatives must be valid commands (no empty strings)

**Relationships**:
- Output from `CommandGenerator::generate_command()`
- Input to `SafetyValidator::validate()`
- Serialized for JSON/YAML output
- Immutable after creation

**Example**:
```rust
let command = GeneratedCommand {
    command: "ls -la".to_string(),
    explanation: "Lists all files in long format".to_string(),
    safety_level: RiskLevel::Safe,
    estimated_impact: "Read-only, no changes".to_string(),
    alternatives: vec!["find . -type f".to_string()],
    backend_used: "mock-v1".to_string(),
    generation_time_ms: 150,
    confidence_score: 0.92,
};
```

---

### 3. ShellType (Enum)

**Purpose**: Identifies the target shell for command syntax compatibility.

**Structure**:
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ShellType {
    Bash,
    Zsh,
    Fish,
    Sh,
    PowerShell,
    Cmd,
    Unknown,
}
```

**Variants**:
- `Bash` - GNU Bash (most common POSIX shell)
- `Zsh` - Z shell (macOS default, POSIX-compatible)
- `Fish` - Friendly interactive shell (different syntax)
- `Sh` - POSIX sh (minimal feature set)
- `PowerShell` - Windows PowerShell (object-based)
- `Cmd` - Windows Command Prompt (batch syntax)
- `Unknown` - Fallback for undetected shells

**Traits**:
```rust
impl Default for ShellType {
    fn default() -> Self {
        Self::detect()
    }
}

impl FromStr for ShellType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "bash" => Ok(Self::Bash),
            "zsh" => Ok(Self::Zsh),
            "fish" => Ok(Self::Fish),
            "sh" => Ok(Self::Sh),
            "powershell" | "pwsh" => Ok(Self::PowerShell),
            "cmd" => Ok(Self::Cmd),
            _ => Ok(Self::Unknown),
        }
    }
}

impl Display for ShellType {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let name = match self {
            Self::Bash => "bash",
            Self::Zsh => "zsh",
            Self::Fish => "fish",
            Self::Sh => "sh",
            Self::PowerShell => "powershell",
            Self::Cmd => "cmd",
            Self::Unknown => "unknown",
        };
        write!(f, "{}", name)
    }
}
```

**Shell Detection**:
```rust
impl ShellType {
    pub fn detect() -> Self {
        #[cfg(target_os = "windows")]
        {
            if std::env::var("PSModulePath").is_ok() {
                Self::PowerShell
            } else {
                Self::Cmd
            }
        }

        #[cfg(not(target_os = "windows"))]
        {
            std::env::var("SHELL")
                .ok()
                .and_then(|s| s.split('/').last())
                .and_then(|s| s.parse().ok())
                .unwrap_or(Self::Bash)
        }
    }

    pub fn is_posix(&self) -> bool {
        matches!(self, Self::Bash | Self::Zsh | Self::Fish | Self::Sh)
    }

    pub fn is_windows(&self) -> bool {
        matches!(self, Self::PowerShell | Self::Cmd)
    }
}
```

**Relationships**:
- Determines validation strategy (POSIX vs Windows)
- Influences command syntax generation
- Used in CommandRequest and safety validation
- Platform-specific default behavior

---

### 4. RiskLevel (Enum)

**Purpose**: Categorizes the safety risk of a generated command.

**Structure**:
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RiskLevel {
    Safe,
    Moderate,
    High,
    Critical,
}
```

**Variants**:
- `Safe` - No dangerous patterns, read-only operations
  - Examples: `ls`, `grep`, `find`, `cat`

- `Moderate` - Modifies files/system but recoverable
  - Examples: `touch`, `mkdir`, `cp`, `mv`

- `High` - Destructive operations with limited scope
  - Examples: `rm` (specific files), `chmod`, `chown`

- `Critical` - Potentially catastrophic operations
  - Examples: `rm -rf /`, `dd if=/dev/zero`, fork bombs

**Ordering**: Safe < Moderate < High < Critical (implements Ord)

**Display with Colors**:
```rust
impl Display for RiskLevel {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        use colored::Colorize;
        let text = match self {
            Self::Safe => "SAFE".green(),
            Self::Moderate => "MODERATE".yellow(),
            Self::High => "HIGH".truecolor(255, 165, 0), // Orange
            Self::Critical => "CRITICAL".red(),
        };
        write!(f, "{}", text)
    }
}
```

**Confirmation Logic**:
```rust
impl RiskLevel {
    pub fn requires_confirmation(&self, safety_level: SafetyLevel) -> bool {
        match safety_level {
            SafetyLevel::Strict => *self >= RiskLevel::Moderate,
            SafetyLevel::Moderate => *self >= RiskLevel::High,
            SafetyLevel::Permissive => *self == RiskLevel::Critical,
        }
    }

    pub fn is_blocked(&self, safety_level: SafetyLevel) -> bool {
        match safety_level {
            SafetyLevel::Strict => *self >= RiskLevel::High,
            SafetyLevel::Moderate => *self == RiskLevel::Critical,
            SafetyLevel::Permissive => false,
        }
    }
}
```

**Relationships**:
- Output from SafetyValidator
- Compared against SafetyLevel for action decisions
- Used in GeneratedCommand for display
- Determines user confirmation workflow

---

### 5. SafetyLevel (Enum)

**Purpose**: User preference for risk tolerance in command execution.

**Structure**:
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SafetyLevel {
    Strict,
    Moderate,
    Permissive,
}
```

**Variants**:
- `Strict` - Block High/Critical, confirm Moderate
  - Use case: Production systems, shared environments
  - Behavior: Maximum safety, minimal risk tolerance

- `Moderate` - Block Critical, confirm High, allow Moderate
  - Use case: Development environments, experienced users
  - Behavior: Balanced safety and productivity (default)

- `Permissive` - Warn Critical, allow all others
  - Use case: Testing, controlled environments
  - Behavior: User takes responsibility

**Default**:
```rust
impl Default for SafetyLevel {
    fn default() -> Self {
        Self::Moderate
    }
}
```

**Action Matrix**:

| Risk Level | Strict | Moderate | Permissive |
|------------|--------|----------|------------|
| Safe | Allow | Allow | Allow |
| Moderate | Confirm | Allow | Allow |
| High | Block | Confirm | Allow |
| Critical | Block | Block | Warn |

**Relationships**:
- User preference in CommandRequest
- Input to validation decision logic
- Stored in configuration file
- Influences RiskLevel.requires_confirmation()

---

### 6. BackendInfo (Struct)

**Purpose**: Metadata about a command generation backend for diagnostics and selection.

**Structure**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackendInfo {
    /// Type of backend (Mock, Ollama, vLLM, MLX)
    pub backend_type: BackendType,

    /// Model name or identifier
    pub model_name: String,

    /// Whether backend supports streaming responses
    pub supports_streaming: bool,

    /// Maximum tokens per generation
    pub max_tokens: u32,

    /// Typical latency in milliseconds
    pub typical_latency_ms: u64,

    /// Estimated memory usage in MB
    pub memory_usage_mb: u64,

    /// Backend version string
    pub version: String,
}
```

**Fields**:
- `backend_type: BackendType` - Backend category
  - Validation: Valid enum variant
  - Example: `BackendType::Mock`

- `model_name: String` - Model identifier
  - Validation: Non-empty
  - Example: `"mock-backend-v1"` or `"llama-2-7b"`

- `supports_streaming: bool` - Streaming capability
  - Validation: Boolean
  - Example: `false` (Mock doesn't stream)

- `max_tokens: u32` - Token limit
  - Validation: Positive number
  - Example: `512`

- `typical_latency_ms: u64` - Expected latency
  - Validation: Positive number
  - Example: `150` (150ms typical)

- `memory_usage_mb: u64` - Memory footprint
  - Validation: Positive number
  - Example: `50` (50MB)

- `version: String` - Version identifier
  - Validation: Non-empty
  - Example: `"1.0.0"`

**Example**:
```rust
let info = BackendInfo {
    backend_type: BackendType::Mock,
    model_name: "mock-backend-v1".to_string(),
    supports_streaming: false,
    max_tokens: 512,
    typical_latency_ms: 150,
    memory_usage_mb: 50,
    version: "1.0.0".to_string(),
};
```

**Relationships**:
- Returned by `CommandGenerator::backend_info()`
- Used for backend selection logic
- Logged for diagnostics
- Displayed in verbose output mode

---

### 7. BackendType (Enum)

**Purpose**: Categorizes command generation backend implementations.

**Structure**:
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BackendType {
    Mock,
    Ollama,
    VLlm,
    Mlx,
}
```

**Variants**:
- `Mock` - In-memory mock for testing
  - Features: Instant responses, no external dependencies
  - Use: Testing, development, CI/CD

- `Ollama` - Local Ollama API
  - Features: Local models, HTTP API, multi-model support
  - Use: Development, offline environments

- `VLlm` - vLLM HTTP backend
  - Features: High throughput, OpenAI-compatible API
  - Use: Production, server deployments

- `Mlx` - Apple Silicon MLX backend
  - Features: Metal-optimized, low latency
  - Use: macOS Apple Silicon systems

**Display**:
```rust
impl Display for BackendType {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let name = match self {
            Self::Mock => "mock",
            Self::Ollama => "ollama",
            Self::VLlm => "vllm",
            Self::Mlx => "mlx",
        };
        write!(f, "{}", name)
    }
}
```

**Relationships**:
- Determines backend instantiation logic
- Used in configuration for backend selection
- Part of BackendInfo metadata
- Enables backend availability checking

---

## Validation Rules Summary

| Entity | Required Validations |
|--------|---------------------|
| CommandRequest | Non-empty input, valid shell_type, valid safety_level |
| GeneratedCommand | Non-empty command, confidence_score ∈ [0,1], positive generation_time |
| ShellType | Valid enum variant or Unknown fallback |
| RiskLevel | Valid enum variant, ordering preserved |
| SafetyLevel | Valid enum variant, default to Moderate |
| BackendInfo | Positive numbers for metrics, non-empty strings |
| BackendType | Valid enum variant |

## Serialization Examples

**JSON Output**:
```json
{
  "command": "ls -la",
  "explanation": "Lists all files in long format",
  "safety_level": "safe",
  "estimated_impact": "Read-only operation",
  "alternatives": ["find . -type f"],
  "backend_used": "mock-v1",
  "generation_time_ms": 150,
  "confidence_score": 0.92
}
```

**YAML Output**:
```yaml
command: ls -la
explanation: Lists all files in long format
safety_level: safe
estimated_impact: Read-only operation
alternatives:
  - find . -type f
backend_used: mock-v1
generation_time_ms: 150
confidence_score: 0.92
```

## Module Organization

```rust
// src/models/mod.rs
pub mod request;
pub mod response;
pub mod enums;
pub mod backend;

pub use request::CommandRequest;
pub use response::GeneratedCommand;
pub use enums::{ShellType, RiskLevel, SafetyLevel};
pub use backend::{BackendInfo, BackendType};
```

## Next Steps

1. Implement type definitions in `src/models/` directory
2. Add serde derives and validation logic
3. Write unit tests for each entity
4. Verify contract tests can import and use types
5. Proceed to safety validation implementation

---
*Data model design complete - Ready for implementation*
