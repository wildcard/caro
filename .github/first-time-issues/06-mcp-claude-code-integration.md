# ⚡ Create MCP (Model Context Protocol) Skill for Claude Code Integration

**Labels**: `good-first-issue`, `first-time-contributor`, `mcp`, `claude-code`, `integration`, `cloud-skills`
**Difficulty**: Medium ⭐⭐⭐
**Skills**: MCP knowledge, JSON schemas, Claude Code experience, integration skills
**Perfect for**: Claude Code power users, MCP enthusiasts, cloud skills builders, integration specialists

## The Vision

cmdai + Claude Code = 🔥 **The ultimate developer workflow!**

Imagine working in Claude Code and being able to:
1. Generate safe terminal commands from natural language
2. Validate commands before execution
3. Get safety explanations inline
4. Access cmdai's decision tree from Claude

We'll create an **MCP skill** that brings cmdai into Claude Code workflows seamlessly.

## What You'll Build

An MCP (Model Context Protocol) skill that exposes cmdai functionality to Claude Code:

### MCP Skill Capabilities

```json
{
  "name": "cmdai",
  "description": "Safe command generation with AI-powered NLP",
  "version": "0.1.0",
  "tools": [
    {
      "name": "generate_command",
      "description": "Generate a safe shell command from natural language",
      "input_schema": {
        "type": "object",
        "properties": {
          "prompt": { "type": "string" },
          "shell": { "type": "string", "enum": ["bash", "zsh", "fish"] },
          "safety_level": { "type": "string", "enum": ["strict", "moderate"] }
        },
        "required": ["prompt"]
      }
    },
    {
      "name": "validate_command",
      "description": "Check if a command is safe to execute",
      "input_schema": {
        "type": "object",
        "properties": {
          "command": { "type": "string" }
        },
        "required": ["command"]
      }
    },
    {
      "name": "explain_safety",
      "description": "Explain why a command was flagged as dangerous",
      "input_schema": {
        "type": "object",
        "properties": {
          "command": { "type": "string" }
        },
        "required": ["command"]
      }
    }
  ]
}
```

### Example Usage in Claude Code

```typescript
// In Claude Code, using the MCP skill:

// Generate a safe command
const result = await mcp.cmdai.generate_command({
  prompt: "find all Python files modified in the last week",
  shell: "bash"
});

// Validate before execution
const validation = await mcp.cmdai.validate_command({
  command: "rm -rf ./temp"
});

if (validation.safe) {
  // Execute the command
} else {
  // Show safety warning
  console.warn(validation.reason);
}
```

## Implementation Guide

### Step 1: Create MCP Skill Structure

Create a new directory: `integrations/mcp/`

```
integrations/mcp/
├── skill.json          # MCP skill definition
├── server.rs           # MCP server implementation
├── handlers.rs         # Tool handlers
└── README.md          # Integration docs
```

### Step 2: Define the Skill Schema

`integrations/mcp/skill.json`:

```json
{
  "$schema": "https://github.com/anthropics/mcp-specification/blob/main/schemas/skill.schema.json",
  "name": "cmdai",
  "version": "0.1.0",
  "description": "Safe AI-powered command generation for terminals",
  "author": "cmdai contributors",
  "license": "AGPL-3.0",

  "tools": [
    {
      "name": "generate_command",
      "description": "Generate a safe shell command from natural language prompt",
      "input_schema": {
        "type": "object",
        "properties": {
          "prompt": {
            "type": "string",
            "description": "Natural language description of desired command"
          },
          "shell": {
            "type": "string",
            "description": "Target shell environment",
            "enum": ["bash", "zsh", "fish", "sh"],
            "default": "bash"
          },
          "safety_level": {
            "type": "string",
            "description": "Safety validation strictness",
            "enum": ["strict", "moderate", "permissive"],
            "default": "moderate"
          }
        },
        "required": ["prompt"]
      }
    },
    {
      "name": "validate_command",
      "description": "Validate a shell command for safety issues",
      "input_schema": {
        "type": "object",
        "properties": {
          "command": {
            "type": "string",
            "description": "Shell command to validate"
          }
        },
        "required": ["command"]
      }
    },
    {
      "name": "explain_safety",
      "description": "Get detailed explanation of why a command is dangerous",
      "input_schema": {
        "type": "object",
        "properties": {
          "command": {
            "type": "string",
            "description": "Command to explain"
          }
        },
        "required": ["command"]
      }
    },
    {
      "name": "show_decision_tree",
      "description": "Visualize the decision tree for command generation",
      "input_schema": {
        "type": "object",
        "properties": {
          "prompt": {
            "type": "string",
            "description": "Natural language prompt"
          }
        },
        "required": ["prompt"]
      }
    }
  ],

  "resources": [
    {
      "uri": "cmdai://history",
      "name": "Command History",
      "description": "Access recent command generation history"
    },
    {
      "uri": "cmdai://stats",
      "name": "Usage Statistics",
      "description": "Get cmdai usage and safety statistics"
    }
  ]
}
```

### Step 3: Implement MCP Server

`integrations/mcp/server.rs`:

```rust
use serde::{Deserialize, Serialize};
use std::io::{self, BufRead, Write};

#[derive(Debug, Deserialize)]
struct McpRequest {
    id: String,
    method: String,
    params: serde_json::Value,
}

#[derive(Debug, Serialize)]
struct McpResponse {
    id: String,
    result: Option<serde_json::Value>,
    error: Option<McpError>,
}

#[derive(Debug, Serialize)]
struct McpError {
    code: i32,
    message: String,
}

pub struct McpServer {
    // Use existing cmdai components
    backend: Arc<dyn CommandGenerator>,
    safety_validator: SafetyValidator,
}

impl McpServer {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        // Initialize with embedded backend
        Ok(Self {
            backend: Arc::new(EmbeddedBackend::new()?),
            safety_validator: SafetyValidator::new()?,
        })
    }

    pub async fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        let stdin = io::stdin();
        let mut stdout = io::stdout();

        for line in stdin.lock().lines() {
            let line = line?;
            let request: McpRequest = serde_json::from_str(&line)?;

            let response = match request.method.as_str() {
                "generate_command" => self.handle_generate_command(request).await,
                "validate_command" => self.handle_validate_command(request).await,
                "explain_safety" => self.handle_explain_safety(request).await,
                "show_decision_tree" => self.handle_decision_tree(request).await,
                _ => Err(McpError {
                    code: -32601,
                    message: format!("Method not found: {}", request.method),
                }),
            };

            let mcp_response = McpResponse {
                id: request.id,
                result: response.ok(),
                error: response.err(),
            };

            writeln!(stdout, "{}", serde_json::to_string(&mcp_response)?)?;
            stdout.flush()?;
        }

        Ok(())
    }

    async fn handle_generate_command(&self, request: McpRequest)
        -> Result<serde_json::Value, McpError> {
        // Implementation using cmdai's existing backend
        todo!()
    }

    async fn handle_validate_command(&self, request: McpRequest)
        -> Result<serde_json::Value, McpError> {
        // Implementation using cmdai's safety validator
        todo!()
    }
}
```

### Step 4: Add MCP Server Binary

In `Cargo.toml`, add a new binary:

```toml
[[bin]]
name = "cmdai-mcp-server"
path = "integrations/mcp/main.rs"
```

`integrations/mcp/main.rs`:

```rust
use cmdai::mcp::McpServer;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let server = McpServer::new()?;
    server.run().await?;
    Ok(())
}
```

### Step 5: Create Claude Code Integration Docs

`integrations/mcp/README.md`:

```markdown
# cmdai MCP Integration for Claude Code

## Installation

1. Build the MCP server:
   ```bash
   cargo build --release --bin cmdai-mcp-server
   ```

2. Add to Claude Code MCP configuration (`~/.config/claude-code/mcp.json`):
   ```json
   {
     "skills": {
       "cmdai": {
         "command": "/path/to/cmdai-mcp-server",
         "args": [],
         "env": {}
       }
     }
   }
   ```

3. Restart Claude Code

## Usage Examples

### Generate a Safe Command
```typescript
const cmd = await mcp.cmdai.generate_command({
  prompt: "compress all .log files older than 30 days"
});
console.log(cmd.command); // tar -czf old-logs.tar.gz $(find . -name "*.log" -mtime +30)
```

### Validate Before Execution
```typescript
const validation = await mcp.cmdai.validate_command({
  command: user_provided_command
});

if (validation.safety_level === "dangerous") {
  throw new Error(`Unsafe command: ${validation.reason}`);
}
```
```

## Acceptance Criteria

- [ ] MCP skill definition (`skill.json`) is complete and valid
- [ ] MCP server binary (`cmdai-mcp-server`) builds successfully
- [ ] Server implements `generate_command` tool
- [ ] Server implements `validate_command` tool
- [ ] Server implements `explain_safety` tool
- [ ] JSON-RPC 2.0 protocol is correctly implemented
- [ ] Integration works with Claude Code
- [ ] Documentation explains installation and usage
- [ ] Tests cover MCP request/response handling
- [ ] Code passes `cargo fmt` and `cargo clippy`

## Testing Your Integration

### Manual Testing

1. **Start the MCP server**:
   ```bash
   cargo run --bin cmdai-mcp-server
   ```

2. **Send test request** (in another terminal):
   ```bash
   echo '{"id":"1","method":"generate_command","params":{"prompt":"list files"}}' | nc localhost 8080
   ```

3. **Verify response**:
   ```json
   {
     "id": "1",
     "result": {
       "command": "ls -la",
       "safety_level": "safe"
     }
   }
   ```

### Integration Testing with Claude Code

Create test cases in Claude Code that exercise the MCP skill.

## Why This Matters

1. **Seamless Workflow**: Use cmdai directly in Claude Code
2. **Safety in IDE**: Validate commands before execution
3. **Power User Tool**: MCP makes cmdai more accessible
4. **Community Growth**: Attract Claude Code users to cmdai
5. **Ecosystem Building**: First-class MCP support

## Resources

- [MCP Specification](https://github.com/anthropics/mcp-specification)
- [Claude Code MCP Docs](https://docs.anthropic.com/claude/docs/mcp)
- [JSON-RPC 2.0 Spec](https://www.jsonrpc.org/specification)
- cmdai's existing backend system in `src/backends/`

## Questions?

We'll help you with:
- Understanding MCP protocol
- JSON-RPC implementation in Rust
- Claude Code integration testing
- Skill schema design

**Ready to bring cmdai to Claude Code? Let's build the ultimate integration! ⚡**
