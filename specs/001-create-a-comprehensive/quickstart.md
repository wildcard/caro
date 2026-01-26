# Quickstart Guide

## Installation & First Run

### Prerequisites
- Rust 1.75+ (for building from source)
- macOS with Apple Silicon (M1/M2/M3/M4) for optimal performance
- Linux x86_64/ARM64 or Windows for alternative platforms

### Installation Options

#### Option 1: Pre-compiled Binary (Recommended)
```bash
# Download latest release for your platform
curl -L https://github.com/user/caro/releases/latest/download/caro-$(uname -s)-$(uname -m) -o caro
chmod +x caro

# Install to user directory (no sudo required)
mkdir -p ~/.local/bin
mv caro ~/.local/bin/

# Add to PATH (add to ~/.bashrc or ~/.zshrc for persistence)
export PATH="$HOME/.local/bin:$PATH"
```

#### Option 2: Package Manager
```bash
# macOS with Homebrew
brew install caro

# Rust developers
cargo install caro
```

#### Option 3: Build from Source
```bash
git clone https://github.com/user/caro.git
cd caro
cargo build --release
./target/release/caro --version
```

## First Time Setup

### 1. Verify Installation
```bash
# Check version and basic functionality
caro --version
caro --help
```

### 2. Initialize Configuration
```bash
# Create default configuration
caro config show

# Set your preferred safety level
caro config set safety_level moderate

# Configure default shell
caro config set default_shell zsh
```

### 3. Test Backend Availability
```bash
# Check which backends are available
caro backends status

# Test specific backend
caro backends test mlx
caro backends test ollama
```

## Basic Usage Examples

### 1. Simple File Operations
```bash
# Find files
caro "find all PDF files larger than 10MB in Downloads"

# List directory contents
caro "show me the 10 largest files in current directory"

# Archive operations
caro "create a tar.gz archive of all .txt files"
```

### 2. System Information
```bash
# Disk usage
caro "show disk usage by directory"

# Process information
caro "find processes using the most memory"

# Network information
caro "show all open network connections"
```

### 3. Text Processing
```bash
# Search and filter
caro "find lines containing 'error' in all log files"

# Sort and count
caro "count unique IP addresses in access.log"

# Format conversion
caro "convert CSV file to JSON"
```

## Safety Features Demo

### 1. Safe Command Generation
```bash
# This will generate a safe command with green indicator
caro "list files in current directory"
# Expected output: ls -la (displayed in green)
```

### 2. Moderate Risk Commands
```bash
# This will show yellow warning and ask for confirmation
caro "delete all .tmp files in current directory"
# Expected: find . -name "*.tmp" -delete (displayed in yellow with confirmation)
```

### 3. High Risk Commands
```bash
# This will show red warning and require explicit confirmation
caro "delete all files in this directory"
# Expected: Red warning, detailed explanation, explicit confirmation required
```

### 4. Safety Override
```bash
# Force execution of normally blocked commands (expert users only)
caro --allow-dangerous "remove everything in /tmp"
```

## Configuration Examples

### 1. Backend Preferences
```bash
# Set preferred backend order
caro config set preferred_backends '["mlx", "ollama", "vllm"]'

# Configure specific backend
caro config set backends.mlx.model_path "/Users/you/.caro/models/mlx"
caro config set backends.ollama.endpoint "http://localhost:11434"
```

### 2. Safety Settings
```bash
# Strict safety (blocks all moderate+ risk commands)
caro config set safety_level strict

# Permissive safety (allows more commands)
caro config set safety_level permissive

# Custom confirmation timeout
caro config set confirmation_timeout 30
```

### 3. Output Preferences
```bash
# Always use JSON output
caro config set default_format json

# Enable verbose logging
caro config set log_level debug

# Disable command history
caro config set log_commands false
```

## Model Management

### 1. List Available Models
```bash
# Show all cached models
caro models list

# Show cache usage
caro models cache
```

### 2. Download Models
```bash
# Download specific model
caro models download "microsoft/DialoGPT-medium"

# Download recommended model for backend
caro models download --backend mlx
```

### 3. Cache Management
```bash
# Clean up old models
caro models cache --clean

# Remove specific model
caro models remove "old-model-id"
```

## Advanced Usage

### 1. Custom Backends
```bash
# Use specific backend
caro --backend vllm "process this data"

# Use custom endpoint
caro --backend vllm --endpoint "http://my-server:8000" "analyze logs"
```

### 2. Batch Processing
```bash
# Process multiple commands from file
cat commands.txt | xargs -I {} caro "{}"

# Generate commands without execution
caro --explain "complex operation"
```

### 3. Integration with Scripts
```bash
#!/bin/bash
# Generate and execute command programmatically
COMMAND=$(caro --format json "find large files" | jq -r '.command')
echo "Executing: $COMMAND"
eval "$COMMAND"
```

## Performance Validation

### 1. Startup Time Test
```bash
# Measure cold start time (should be <100ms)
time caro --version
```

### 2. Generation Speed Test
```bash
# Measure inference time (should be <2s)
time caro "simple file listing command"
```

### 3. Memory Usage Test
```bash
# Monitor memory usage during operation
caro benchmark --count 10
```

## Troubleshooting

### 1. Backend Issues
```bash
# Check backend status
caro backends status

# Test connectivity
caro backends test --all

# Reset backend configuration
caro config reset backends
```

### 2. Model Problems
```bash
# Verify model integrity
caro models validate

# Re-download corrupted models
caro models update --force

# Clear model cache
caro models cache --clear
```

### 3. Configuration Issues
```bash
# Validate current configuration
caro config validate

# Reset to defaults
caro config reset

# Show configuration hierarchy
caro config show --verbose
```

## Integration Tests

### User Story Validation
Each acceptance scenario from the specification should work as described:

1. **File Finding**: `caro "find all PDF files larger than 10MB in Downloads folder"`
   - Should generate: `find ~/Downloads -name "*.pdf" -size +10M -type f`
   - Display with green safety indicator
   - Ask for confirmation before execution

2. **Dangerous Operation**: `caro "delete all files in this directory"`
   - Should generate appropriate rm command
   - Display RED warning indicator
   - Require explicit confirmation with safety prompts

3. **Ambiguous Request**: `caro "compress files"`
   - Should ask clarifying questions
   - Should not generate command until clarified

4. **Safety Refusal**: `caro "format my hard drive"`
   - Should refuse to generate command
   - Should suggest alternative approaches

5. **Exploration**: `caro "show me disk usage by directory"`
   - Should generate: `du -sh */ | sort -hr`
   - Should explain command purpose
   - Should execute after confirmation

## Performance Benchmarks

### Expected Performance Targets
- **Startup time**: <100ms cold start
- **Generation time**: <2s for typical requests
- **Memory usage**: <500MB during operation
- **Binary size**: <50MB
- **Accuracy**: >90% for common user intents

### Validation Commands
```bash
# Run comprehensive benchmark
caro benchmark --count 100 --backend all

# Measure startup performance
for i in {1..10}; do time caro --version; done

# Test accuracy with known commands
caro "list files" | grep -q "ls" && echo "PASS" || echo "FAIL"
```