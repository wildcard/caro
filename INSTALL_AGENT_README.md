# Agent-Assisted Installation

The `install-agent.sh` script provides structured JSON output designed for consumption by AI agents, LLMs, and automation tools.

## Usage

### Basic Usage

```bash
curl -fsSL https://raw.githubusercontent.com/wildcard/caro/main/install-agent.sh | bash
```

### With wget

```bash
wget -qO- https://raw.githubusercontent.com/wildcard/caro/main/install-agent.sh | bash
```

### Save Output to File

```bash
curl -fsSL https://raw.githubusercontent.com/wildcard/caro/main/install-agent.sh | bash > caro-install-report.json
```

## Output Format

The script outputs a comprehensive JSON report containing:

### System Information
- Operating system and architecture
- Shell type and version
- Available tools (cargo, curl, wget)
- Network connectivity status
- Proxy settings

### Installation Analysis
- Current installation status
- Recommended installation method with reasoning
- Step-by-step installation instructions
- Troubleshooting guidance

### Example Output

```json
{
  "version": "1.0.0",
  "timestamp": "2024-01-06T12:00:00Z",
  "system": {
    "os": "macos",
    "architecture": "arm64",
    "platform": "macos-arm64",
    "shell": "zsh"
  },
  "environment": {
    "has_cargo": true,
    "has_curl": true,
    "has_wget": false,
    "network_available": true,
    "proxy": "none",
    "install_dir": "/Users/example/.local/bin"
  },
  "existing_installation": {
    "installed": false
  },
  "recommendation": {
    "method": "cargo",
    "reason": "Building from source enables MLX optimization for Apple Silicon"
  },
  "installation_steps": [
    {
      "step": 1,
      "description": "Install caro via cargo with MLX optimization",
      "command": "cargo install caro --features embedded-mlx",
      "expected_output": "Installing caro",
      "success_indicator": "Installed package `caro"
    },
    {
      "step": 2,
      "description": "Verify installation",
      "command": "caro --version",
      "expected_output": "caro",
      "success_indicator": "caro"
    }
  ],
  "troubleshooting": {
    "no_network": "Check internet connection and proxy settings (HTTP_PROXY, HTTPS_PROXY)",
    "no_cargo": "Install Rust from https://rustup.rs or download pre-built binary",
    "no_download_tools": "Install curl or wget to download pre-built binaries",
    "permission_denied": "Ensure /Users/example/.local/bin is writable or set CARO_INSTALL_DIR to a writable location"
  },
  "links": {
    "documentation": "https://caro.sh",
    "repository": "https://github.com/wildcard/caro",
    "releases": "https://github.com/wildcard/caro/releases/latest",
    "issues": "https://github.com/wildcard/caro/issues"
  }
}
```

## Integration with AI Agents

### LLM Integration Example

An AI agent can use this script to:

1. **Detect Environment**: Parse the JSON to understand system capabilities
2. **Choose Strategy**: Use the recommended method based on available tools
3. **Execute Steps**: Follow the installation_steps array sequentially
4. **Verify Success**: Check for success_indicator in command output
5. **Handle Errors**: Use troubleshooting guidance when steps fail

### Example Agent Workflow

```python
import json
import subprocess

# Get installation report
result = subprocess.run(
    ["curl", "-fsSL", "https://raw.githubusercontent.com/wildcard/caro/main/install-agent.sh"],
    capture_output=True,
    text=True
)
report = json.loads(result.stdout)

# Check if already installed
if report["existing_installation"]["installed"]:
    print(f"caro is already installed: {report['existing_installation']['version']}")
    exit(0)

# Execute recommended installation steps
for step in report["installation_steps"]:
    print(f"Step {step['step']}: {step['description']}")
    result = subprocess.run(
        step["command"],
        shell=True,
        capture_output=True,
        text=True
    )

    if step["success_indicator"] in result.stdout:
        print(f"✓ Step {step['step']} succeeded")
    else:
        print(f"✗ Step {step['step']} failed")
        print(report["troubleshooting"]["no_cargo"])  # Show relevant troubleshooting
        break
```

## Non-Interactive Installation

For automated installations without user interaction, use the standard `install.sh` with the `CARO_INTERACTIVE=false` environment variable:

```bash
curl -fsSL https://raw.githubusercontent.com/wildcard/caro/main/install.sh | CARO_INTERACTIVE=false bash
```

Or via piped execution (auto-detected):

```bash
curl -fsSL https://raw.githubusercontent.com/wildcard/caro/main/install.sh | bash
```

The installer automatically detects piped execution and disables interactive prompts.

## Environment Variables

Both `install.sh` and `install-agent.sh` respect these environment variables:

- `CARO_INSTALL_DIR`: Custom installation directory (default: `$HOME/.local/bin`)
- `CARO_INTERACTIVE`: Force interactive mode on/off (default: auto-detect from TTY)

## Use Cases

### CI/CD Pipelines

```yaml
# .github/workflows/test.yml
- name: Get installation report
  run: |
    curl -fsSL https://raw.githubusercontent.com/wildcard/caro/main/install-agent.sh | bash > install-report.json
    cat install-report.json

- name: Install caro (non-interactive)
  run: |
    curl -fsSL https://raw.githubusercontent.com/wildcard/caro/main/install.sh | bash
```

### AI-Assisted Setup

An AI agent helping a user install caro can:
1. Run `install-agent.sh` to understand the environment
2. Explain the recommended approach in natural language
3. Guide the user through manual steps if needed
4. Troubleshoot issues using the structured error guidance

### Docker Containers

```dockerfile
FROM ubuntu:22.04

# Install caro non-interactively
RUN curl -fsSL https://raw.githubusercontent.com/wildcard/caro/main/install.sh | CARO_INTERACTIVE=false bash

# Verify installation
RUN caro --version
```

## Related Documentation

- [Main Installation Guide](https://github.com/wildcard/caro#installation)
- [Troubleshooting](https://github.com/wildcard/caro/wiki/Troubleshooting)
- [Contributing](https://github.com/wildcard/caro/blob/main/CONTRIBUTING.md)
