# Architecture

cmdai is designed as a modular, extensible Rust CLI application with a focus on safety, performance, and maintainability.

## System Overview

```
┌─────────────┐
│  CLI Entry  │  ← clap-based argument parsing
└──────┬──────┘
       │
       ▼
┌─────────────────┐
│  Command Router │  ← Request validation and routing
└────────┬────────┘
         │
         ├──────────────────┬──────────────────┐
         ▼                  ▼                  ▼
   ┌──────────┐      ┌──────────┐      ┌──────────┐
   │ Embedded │      │  Ollama  │      │   vLLM   │
   │ Backend  │      │ Backend  │      │ Backend  │
   └────┬─────┘      └────┬─────┘      └────┬─────┘
        │                 │                  │
        └─────────────────┴──────────────────┘
                          │
                          ▼
                   ┌──────────────┐
                   │   Safety     │  ← Pattern matching
                   │  Validator   │     Risk assessment
                   └──────┬───────┘
                          │
                          ▼
                   ┌──────────────┐
                   │ Confirmation │  ← User interaction
                   │    Flow      │
                   └──────┬───────┘
                          │
                          ▼
                   ┌──────────────┐
                   │  Execution   │  ← Shell command execution
                   └──────────────┘
```

## Module Structure

```
cmdai/
├── src/
│   ├── main.rs              # CLI entry point
│   ├── backends/            # LLM backend implementations
│   │   ├── mod.rs          # Backend trait system
│   │   ├── embedded.rs     # Embedded model backend
│   │   ├── mlx.rs          # Apple Silicon MLX backend
│   │   ├── vllm.rs         # vLLM remote backend
│   │   └── ollama.rs       # Ollama local backend
│   ├── safety/             # Command validation
│   │   └── mod.rs          # Safety validator
│   ├── cache/              # Model caching (planned)
│   ├── config/             # Configuration management
│   ├── cli/                # CLI interface
│   ├── models/             # Data models
│   └── execution/          # Command execution
└── tests/                  # Contract-based tests
```

## Core Components

### 1. Backend Trait System

All LLM backends implement the `CommandGenerator` trait:

```rust
#[async_trait]
pub trait CommandGenerator: Send + Sync {
    /// Generate a shell command from a natural language prompt
    async fn generate_command(
        &self,
        request: &CommandRequest,
    ) -> Result<GeneratedCommand, GeneratorError>;

    /// Check if the backend is available and healthy
    async fn is_available(&self) -> bool;

    /// Get backend metadata and capabilities
    fn backend_info(&self) -> BackendInfo;
}
```

**Key Design Decisions**:
- **Trait-based**: Enables polymorphism and easy backend swapping
- **Async**: All backends support async operations
- **Error handling**: Unified `GeneratorError` type
- **Availability checking**: Graceful degradation with fallbacks

### 2. Safety Validation System

The `SafetyValidator` analyzes commands before execution:

```rust
pub struct SafetyValidator {
    config: SafetyConfig,
    patterns: Vec<DangerousPattern>,
}

impl SafetyValidator {
    /// Validate a command and assess risk
    pub fn validate(&self, command: &str) -> ValidationResult {
        // Pattern matching
        // Risk assessment
        // Suggestion generation
    }
}
```

**Features**:
- Pattern-based detection (regex)
- Risk level classification
- Custom pattern support
- Path protection
- Shell metacharacter detection

### 3. Configuration Management

Hierarchical configuration with multiple sources:

```rust
pub struct Config {
    pub backend: BackendConfig,
    pub safety: SafetyConfig,
    pub output: OutputConfig,
    pub shell: ShellConfig,
}
```

**Configuration Priority**:
1. Built-in defaults (lowest)
2. Configuration file
3. Environment variables
4. Command-line flags (highest)

### 4. CLI Interface

Built with `clap` using derive macros:

```rust
#[derive(Parser)]
#[command(name = "cmdai")]
#[command(version, about, long_about = None)]
pub struct Cli {
    /// Natural language command description
    pub prompt: String,

    /// Target shell
    #[arg(short, long)]
    pub shell: Option<Shell>,

    /// Safety level
    #[arg(long)]
    pub safety: Option<SafetyLevel>,

    // ... more options
}
```

## Data Flow

### Command Generation Flow

```
1. User Input
   ↓
2. CLI Parsing (clap)
   ↓
3. Configuration Loading
   ↓
4. Backend Selection
   ↓
5. Command Generation (LLM)
   ↓
6. Safety Validation
   ↓
7. Risk Assessment
   ↓
8. User Confirmation
   ↓
9. Command Execution
   ↓
10. Result Display
```

### Backend Selection Algorithm

```rust
async fn select_backend(config: &Config) -> Result<Box<dyn CommandGenerator>> {
    // 1. Try primary backend
    if let Some(backend) = try_backend(&config.backend.primary).await {
        return Ok(backend);
    }

    // 2. Fall back to available backends (if enabled)
    if config.backend.enable_fallback {
        for backend_type in BACKEND_PRIORITY {
            if let Some(backend) = try_backend(backend_type).await {
                return Ok(backend);
            }
        }
    }

    // 3. No backend available
    Err(Error::NoBackendAvailable)
}
```

**Backend Priority**:
1. User-specified primary backend
2. Embedded backend (MLX on Apple Silicon, CPU otherwise)
3. Ollama (if running)
4. vLLM (if configured)

## Error Handling

### Error Hierarchy

```rust
pub enum Error {
    Backend(BackendError),
    Safety(SafetyError),
    Config(ConfigError),
    Execution(ExecutionError),
    Io(io::Error),
}
```

**Design Principles**:
- Specific error types for each module
- Context-rich error messages
- Actionable suggestions
- Proper error propagation with `?`

### Result Types

```rust
pub type Result<T> = std::result::Result<T, Error>;
```

## Performance Optimizations

### Startup Performance

- **Lazy loading**: Backends loaded only when needed
- **Minimal dependencies**: Small dependency tree
- **Binary size**: Optimized with LTO and `opt-level = 'z'`
- **Target**: < 100ms cold start

### Inference Performance

- **MLX backend**: < 2s on Apple Silicon
- **HTTP backends**: Async with connection pooling
- **Caching**: Model weights cached locally
- **Streaming**: Partial response handling (planned)

### Memory Management

- **Zero-copy**: Where possible
- **Arena allocation**: For temporary data
- **Ref counting**: Minimal cloning
- **Smart pointers**: `Arc<>` for shared state

## Testing Strategy

### Test Pyramid

```
       ┌─────────────┐
       │   E2E Tests │  ← Full workflow tests
       └─────────────┘
      ┌───────────────┐
      │  Integration  │  ← Backend contracts
      └───────────────┘
    ┌─────────────────┐
    │   Unit Tests    │  ← Individual components
    └─────────────────┘
```

### Contract-Based Testing

All backends must pass the same contract tests:

```rust
#[cfg(test)]
mod backend_contract_tests {
    async fn test_basic_generation<T: CommandGenerator>(backend: T) {
        // Test implementation
    }

    async fn test_availability<T: CommandGenerator>(backend: T) {
        // Test implementation
    }
}
```

## Security Architecture

### Defense Layers

1. **Input Validation**: Sanitize user input
2. **LLM Generation**: Prompt engineering for safe output
3. **Pattern Matching**: Detect dangerous commands
4. **User Confirmation**: Human review required
5. **Execution Sandbox**: Limited permissions (planned)

### Threat Model

**In Scope**:
- Malicious LLM outputs
- User mistakes
- Configuration errors
- Injection attacks

**Out of Scope**:
- Compromised binaries
- Root-level exploits
- Hardware attacks

## Extension Points

### Adding New Backends

1. Implement `CommandGenerator` trait
2. Add backend configuration
3. Register in backend factory
4. Write contract tests

Example:

```rust
pub struct CustomBackend {
    config: CustomConfig,
}

#[async_trait]
impl CommandGenerator for CustomBackend {
    async fn generate_command(&self, request: &CommandRequest)
        -> Result<GeneratedCommand> {
        // Implementation
    }

    async fn is_available(&self) -> bool {
        // Check availability
    }

    fn backend_info(&self) -> BackendInfo {
        // Return metadata
    }
}
```

### Adding Safety Patterns

Add patterns to `safety/patterns.rs`:

```rust
pub const CUSTOM_PATTERNS: &[DangerousPattern] = &[
    DangerousPattern {
        name: "docker_force_remove",
        pattern: r"docker\s+rm\s+-f",
        severity: RiskLevel::High,
        message: "Force removing Docker containers",
    },
];
```

## Future Architecture

### Planned Enhancements

1. **Plugin System**: Dynamically loaded backends
2. **Execution Sandbox**: Restricted environment
3. **Multi-step Planning**: Complex goal completion
4. **Context Awareness**: Shell history and environment
5. **Distributed Backends**: Remote model serving

## Next Steps

**Developer Guides:**
- [Backend Development](./backends.md) - Implement new LLM backends
- [Testing Strategy](./testing.md) - Write comprehensive tests
- [TDD Workflow](./tdd-workflow.md) - Test-driven development process
- [Contributing](./contributing.md) - Contribute to the project

**Technical Deep Dives:**
- [Safety Validation](../technical/safety-validation.md) - Safety system implementation
- [MLX Integration](../technical/mlx-integration.md) - Apple Silicon backend details
- [Performance Optimization](../technical/performance.md) - Performance characteristics
- [Rust Learnings](../technical/rust-learnings.md) - Insights from implementation

**User Guides:**
- [Configuration](../user-guide/configuration.md) - Configure cmdai
- [Safety & Security](../user-guide/safety.md) - Understanding safety features

---

## See Also

**Architecture Resources:**
- [Backend Trait System](#backend-trait-system) - Interface design patterns
- [Error Handling](#error-handling) - Error hierarchy and propagation
- [Performance Optimizations](#performance-optimizations) - Speed and memory management

**Related Pages:**
- [Backend Development](./backends.md) - Implementing the `CommandGenerator` trait
- [Testing Strategy](./testing.md) - Contract-based testing for backends
- [Contributing](./contributing.md) - Development workflow and guidelines

**Community:**
- [Development Agents](../community/agents.md) - Specialized agents for different tasks
- [Project Roadmap](../community/roadmap.md) - Future architecture enhancements
- [Active Development](../community/active-development.md) - Current implementation work
