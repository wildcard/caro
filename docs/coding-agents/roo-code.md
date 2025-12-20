# Roo Code

**AI Coding Assistant for VS Code**

Roo Code (Roo-Cline) is a VS Code extension that provides AI-powered coding assistance with autonomous capabilities.

## Overview

| Attribute | Value |
|-----------|-------|
| **Developer** | Roo Code Team |
| **Type** | VS Code Extension |
| **License** | Apache 2.0 |
| **Repository** | [github.com/RooVetGit/Roo-Code](https://github.com/RooVetGit/Roo-Code) |
| **Marketplace** | [VS Code Marketplace](https://marketplace.visualstudio.com/items?itemName=RooVetGit.roo-cline) |

## Installation

### VS Code Marketplace

```bash
code --install-extension RooVetGit.roo-cline
```

Or search "Roo Code" in VS Code Extensions.

### Configuration

1. Open Roo Code panel
2. Configure API provider
3. Set custom instructions

## Configuration

### Extension Settings

```json
// settings.json
{
  "roo-cline.apiProvider": "anthropic",
  "roo-cline.apiKey": "...",
  "roo-cline.model": "claude-sonnet-4-20250514",
  "roo-cline.customInstructions": "Use caro for shell command safety",
  "roo-cline.autoApprove": false
}
```

### Project Configuration

Create `.roo/` directory:

```
.roo/
├── instructions.md    # Project-specific instructions
├── context/          # Additional context files
└── tools.json        # Custom tool definitions
```

## Key Features

### AI Capabilities
- Code generation
- Refactoring
- Bug fixing
- Documentation

### Autonomous Features
- Multi-step tasks
- File operations
- Terminal commands
- Error recovery

### Safety Features
- Approval workflows
- Diff previews
- Command confirmation
- Undo support

## Integration with Caro

### Method 1: Custom Instructions

```json
// settings.json
{
  "roo-cline.customInstructions": "Shell Command Safety:\n1. Generate commands using caro\n2. Validate with caro --validate\n3. Execute with caro --execute\n4. Never bypass safety checks"
}
```

### Method 2: Project Instructions

```markdown
<!-- .roo/instructions.md -->
# Project: cmdai (caro)

## Overview
This is a Rust CLI tool for safe shell command generation.

## Shell Commands
All shell commands must use caro:

### Generation
```bash
caro "<what you want to do>"
```

### Validation
```bash
caro --validate "<command>"
```

### Execution
```bash
caro --execute "<command>"
```

## Safety Rules
1. Never run rm -rf without validation
2. Always show risk level before execution
3. Require confirmation for MODERATE+ risk
4. Block CRITICAL risk commands
```

### Method 3: Tool Configuration

```json
// .roo/tools.json
{
  "tools": [
    {
      "name": "safe_command",
      "description": "Generate a safe shell command",
      "exec": "caro --json \"{{prompt}}\""
    },
    {
      "name": "validate_command",
      "description": "Check if a command is safe",
      "exec": "caro --validate --json \"{{command}}\""
    },
    {
      "name": "execute_safe",
      "description": "Execute command with safety confirmation",
      "exec": "caro --execute \"{{command}}\""
    }
  ]
}
```

### Method 4: Context Files

Add Caro documentation to context:

```markdown
<!-- .roo/context/caro.md -->
# Caro - Shell Command Safety Tool

Caro (cmdai) is the project you're working on. It provides:

## Features
- Natural language to shell command conversion
- 52+ dangerous command patterns detection
- Platform-aware generation (macOS, Linux, Windows)
- Risk level assessment (SAFE, MODERATE, HIGH, CRITICAL)

## Usage
- `caro "description"` - Generate command
- `caro --validate "cmd"` - Validate safety
- `caro --execute "cmd"` - Execute with confirmation

## When to Use
- Any shell command generation
- Before running potentially dangerous commands
- For cross-platform compatibility
```

## Best Practices with Caro

### 1. Autonomous Safety

Configure Roo Code to use Caro automatically:

```json
{
  "roo-cline.customInstructions": "Before ANY terminal command:\n1. Run: caro --validate \"<command>\"\n2. Check risk level\n3. If SAFE: proceed\n4. If MODERATE: ask user\n5. If HIGH/CRITICAL: explain and suggest alternative"
}
```

### 2. Development Workflow

```
User: Build and test this Rust project

Roo Code: I'll build and test with safety validation.

Step 1: Validate build command
$ caro --validate "cargo build --release"
Result: SAFE

Step 2: Execute build
$ caro --execute "cargo build --release"
[build output]

Step 3: Validate test command
$ caro --validate "cargo test"
Result: SAFE

Step 4: Execute tests
$ caro --execute "cargo test"
[test output]

All steps completed safely!
```

### 3. Error Recovery

```
User: Clean up old builds

Roo Code: Let me generate a safe cleanup command.

$ caro "delete old build artifacts"
Generated: rm -rf target/debug target/release

$ caro --validate "rm -rf target/debug target/release"
Result: MODERATE - Removes build directories

This is a moderate risk operation. Proceed? [y/N]
```

## Comparison with Cline

| Feature | Roo Code | Cline |
|---------|----------|-------|
| Base | Roo-Cline | Claude Dev |
| Model Support | Multi-provider | Multi-provider |
| Browser | Limited | Yes |
| Custom Tools | Yes | Yes |
| Community | Growing | Established |

## Troubleshooting

### Common Issues

**Issue**: Extension not loading
```bash
# Reload VS Code window
# Cmd+Shift+P > Developer: Reload Window
```

**Issue**: Caro not found
```json
// Add to terminal environment
{
  "terminal.integrated.env.linux": {
    "PATH": "${env:HOME}/.cargo/bin:${env:PATH}"
  }
}
```

**Issue**: API errors
```json
// Check API configuration
{
  "roo-cline.apiProvider": "anthropic",
  "roo-cline.apiKey": "sk-ant-..."
}
```

## Resources

- [Roo Code GitHub](https://github.com/RooVetGit/Roo-Code)
- [VS Code Marketplace](https://marketplace.visualstudio.com/items?itemName=RooVetGit.roo-cline)
- [Documentation](https://github.com/RooVetGit/Roo-Code#readme)

## See Also

- [Cline](./cline.md) - Original Claude Dev extension
- [Continue](./continue.md) - Open-source alternative
- [Cursor](./cursor.md) - AI-first IDE
