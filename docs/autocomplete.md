# Command Autocomplete Inference

The autocomplete module provides intelligent command-line completion using LLM-powered inference combined with static completion metadata. It helps users complete commands by suggesting valid arguments, flags, and values based on context.

## Architecture

The autocomplete system consists of four main components:

```
┌──────────────┐
│ User Input   │
└──────┬───────┘
       │
       ▼
┌──────────────────────┐
│ CompletionContext    │  Parse command, extract metadata
└──────┬───────────────┘
       │
       ▼
┌──────────────────────┐
│ InferenceAgent       │  LLM generates suggestions
└──────┬───────────────┘
       │
       ▼
┌──────────────────────┐
│ ValidatorAgent       │  Verify suggestions are valid
└──────┬───────────────┘
       │
       ▼
┌──────────────────────┐
│ Ranked Suggestions   │
└──────────────────────┘
```

### 1. Completion Context (`context.rs`)

Manages command signatures and provides context for autocomplete:

- **CommandSignature**: Defines a command's structure (subcommands, flags, arguments)
- **CompletionContext**: Stores all command metadata
- **CompletionType**: Identifies what's being completed (command, flag, value, etc.)

Built-in support for common commands:
- `git` (commit, add, status, etc.)
- `cargo` (build, test, etc.)
- Custom commands can be added dynamically

### 2. Inference Agent (`inference.rs`)

Uses LLM backend to generate context-aware suggestions:

- Builds prompts based on completion context
- Extracts relevant metadata (available flags, expected types)
- Parses LLM responses into structured candidates
- Assigns confidence scores to each suggestion

### 3. Validator Agent (`validator.rs`)

Validates suggested arguments against specifications:

- **Type Checking**: Files, directories, enums, integers, strings
- **Pattern Matching**: Regex validation for string arguments
- **Range Validation**: Min/max for numeric arguments
- **Existence Checks**: Verify files/directories exist (configurable)

### 4. Autocomplete Engine (`mod.rs`)

Orchestrates the complete autocomplete workflow:

- Configurable suggestion limits and confidence thresholds
- Filtering and ranking of candidates
- Optional validation of suggestions
- Performance metrics (generation time)

## Usage

### Basic Example

```rust
use cmdai::autocomplete::{AutocompleteEngine, AutocompleteConfig};
use cmdai::backends::CommandGenerator;

// Create engine with a backend
let config = AutocompleteConfig::default();
let engine = AutocompleteEngine::new(config, backend)?;

// Get suggestions for partial command
let result = engine.suggest("git commit -m ", 14).await?;

// Display suggestions
for candidate in result.candidates {
    println!("{}: {} (confidence: {:.2})",
        candidate.value,
        candidate.description,
        candidate.confidence
    );
}
```

### Custom Command Signatures

```rust
use cmdai::autocomplete::{CommandSignature, SubcommandSpec, FlagSpec, ArgumentSpec};

let signature = CommandSignature {
    command: "deploy".to_string(),
    description: "Deploy application".to_string(),
    subcommands: vec![
        SubcommandSpec {
            name: "production".to_string(),
            description: "Deploy to production".to_string(),
            flags: vec![
                FlagSpec {
                    short: Some('v'),
                    long: Some("version".to_string()),
                    description: "Version to deploy".to_string(),
                    takes_value: true,
                    value_spec: Some(ArgumentSpec::String {
                        pattern: Some(r"^v\d+\.\d+\.\d+$".to_string()),
                        examples: vec!["v1.0.0".to_string()],
                    }),
                }
            ],
            arguments: vec![],
        }
    ],
    global_flags: vec![],
};

engine.add_command_signature(signature);
```

### Loading from File

```rust
// Load completion definitions from JSON file
engine.load_completions_from_file("completions.json")?;
```

### Configuration Options

```rust
use cmdai::autocomplete::AutocompleteConfig;
use cmdai::models::ShellType;

let config = AutocompleteConfig {
    max_suggestions: 10,        // Maximum number to return
    min_confidence: 0.3,         // Minimum confidence threshold
    enable_validation: true,     // Validate suggestions
    shell_type: ShellType::Bash, // Shell context
};
```

## Argument Specifications

### String Arguments

```rust
ArgumentSpec::String {
    pattern: Some(r"^[a-z0-9-]+$".to_string()),
    examples: vec!["my-branch".to_string()],
}
```

### File Arguments

```rust
ArgumentSpec::File {
    must_exist: true,
    extensions: Some(vec!["rs".to_string(), "toml".to_string()]),
}
```

### Directory Arguments

```rust
ArgumentSpec::Directory {
    must_exist: true,
}
```

### Enum Arguments

```rust
ArgumentSpec::Enum {
    values: vec!["debug".to_string(), "info".to_string(), "warn".to_string()],
}
```

### Integer Arguments

```rust
ArgumentSpec::Integer {
    min: Some(1),
    max: Some(100),
}
```

### Boolean Arguments

```rust
ArgumentSpec::Boolean
```

## Validation

The validator checks:

1. **Type Conformance**: Value matches expected type
2. **Pattern Matching**: String values match regex patterns
3. **Range Checking**: Numeric values within bounds
4. **Existence Verification**: Files/directories exist (optional)
5. **Extension Matching**: Files have correct extensions

### Validation Configuration

```rust
use cmdai::autocomplete::ValidatorConfig;

let config = ValidatorConfig {
    check_file_existence: true,
    check_directory_existence: true,
    strict_pattern_matching: false,
};

let validator = ArgumentValidator::new(config)?;
```

## Integration with argc-completions

The autocomplete system is inspired by [argc-completions](https://github.com/sigoden/argc-completions) and shares similar concepts:

- **Multi-part completion**: Support for complex argument patterns
- **Dynamic value fetching**: Can integrate with remote data sources
- **Shell-agnostic**: Works across different shell environments
- **Parallel completion**: Efficient concurrent suggestion generation

Key differences:
- **LLM-powered**: Uses language models for intelligent suggestions
- **Validation layer**: Built-in argument verification
- **Confidence scoring**: Ranks suggestions by confidence
- **Type safety**: Strong typing for argument specifications

## Performance

The autocomplete system is designed for low latency:

- **Cached signatures**: Command metadata loaded once
- **Async operations**: Non-blocking I/O
- **Efficient parsing**: Minimal allocations
- **Parallel validation**: Concurrent suggestion checks

Typical performance metrics:
- Context parsing: < 1ms
- LLM inference: 10-500ms (backend dependent)
- Validation: < 5ms per suggestion
- Total latency: 10-510ms

## Testing

The module includes comprehensive tests:

- **Unit tests**: In each module (`context`, `inference`, `validator`)
- **Integration tests**: End-to-end autocomplete workflows
- **Mock backends**: For testing without LLM dependencies

Run tests:

```bash
# Unit tests
cargo test --lib autocomplete

# Integration tests
cargo test --test autocomplete_integration

# All autocomplete tests
cargo test autocomplete
```

## Future Enhancements

Planned improvements:

1. **Man page parsing**: Extract completion data from man pages
2. **Learning system**: Improve suggestions based on usage patterns
3. **Shell integration**: Native completion scripts for bash/zsh/fish
4. **Remote completion**: Fetch dynamic values from APIs
5. **Caching**: Cache LLM responses for common completions
6. **Streaming**: Stream suggestions as they're generated

## Related Documentation

- [argc-completions](https://github.com/sigoden/argc-completions) - Inspiration for completion system
- [Command Generator Trait](../src/backends/mod.rs) - Backend interface
- [Safety Validation](../src/safety/mod.rs) - Command safety checks
- [Models](../src/models/mod.rs) - Core data types
