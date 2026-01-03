# Feature Specification: FunctionGemma Intent Router

**Feature ID:** 008
**Status:** Draft
**Created:** 2026-01-01
**Last Updated:** 2026-01-01
**Owners:** Development Team

---

## Executive Summary

This specification defines an intelligent **Intent Router** system powered by Google's FunctionGemma model. FunctionGemma is a specialized 270M parameter model fine-tuned specifically for function calling tasks, enabling caro to accurately classify user intent and route requests to domain-specific command generation pipelines.

### Problem Statement

Current command generation in caro faces several challenges:

1. **Flat Prompt Structure**: All requests use the same generation prompt regardless of domain, leading to suboptimal outputs for specialized tasks (git, networking, package management)
2. **No Intent Classification**: The system treats "find large files" and "check network connectivity" identically, missing opportunities for domain-specific optimizations
3. **Context Bloat**: General-purpose prompts include examples from all domains, wasting context window and potentially confusing the model
4. **Safety Gaps**: Domain-agnostic safety checks miss domain-specific dangers (e.g., `git push --force` in git domain)

### Solution Overview

A two-stage architecture that:
- **Stage 1 (FunctionGemma)**: Classifies user intent and selects appropriate command domain(s)
- **Stage 2 (Primary LLM)**: Generates commands using domain-specific prompts, examples, and safety rules

FunctionGemma's strengths (small, fast, optimized for function selection) perfectly complement the primary LLM's strengths (complex reasoning, command generation).

---

## Goals & Success Metrics

### Primary Goals

1. **Accurate Intent Classification**: 95%+ accuracy in routing requests to correct domain
2. **Reduced Generation Time**: Specialized prompts are smaller and more focused
3. **Improved Command Quality**: Domain-specific examples and constraints
4. **Enhanced Safety**: Domain-aware safety validation
5. **Extensibility**: Easy addition of new command domains

### Success Metrics

| Metric | Current | Target | Measurement |
|--------|---------|--------|-------------|
| Intent classification accuracy | N/A | 95% | Correct domain selection |
| First-shot success rate | 60% | 80% | Domain-appropriate command |
| Average generation time | 2.5s | 2.0s | End-to-end latency |
| Safety coverage | 52 patterns | 80+ patterns | Domain-specific patterns |
| Context utilization | 4K tokens | 2K tokens | Average prompt size |

### Non-Goals

- Replacing the primary LLM (FunctionGemma only routes, doesn't generate)
- Supporting Windows PowerShell in Phase 1
- Fine-tuning FunctionGemma on custom domains
- Multi-step command orchestration (future feature)

---

## FunctionGemma Overview

### Model Specifications

| Property | Value |
|----------|-------|
| **Base Model** | Gemma 3 270M |
| **Size** | 301MB |
| **Parameters** | 270 million |
| **Context Window** | 32K tokens |
| **Training** | 6 trillion tokens |
| **Knowledge Cutoff** | August 2024 |
| **Ollama Requirement** | v0.13.5+ |

### Strengths

- **Fast inference**: Small model enables sub-100ms routing decisions
- **Function calling optimized**: Trained specifically for tool selection
- **JSON output**: Structured function call responses
- **Low resource usage**: Can run alongside primary LLM

### Limitations

- **Simple scenarios only**: Best for single-function, single-turn calls
- **Limited complex reasoning**: Not suitable for actual command generation
- **Domain-specific fine-tuning needed**: May require prompt engineering for custom tools

### Benchmark Performance (BFCL)

| Benchmark | Score |
|-----------|-------|
| BFCL Irrelevance | 70.6% |
| BFCL Parallel | 63.5% |
| BFCL Live Parallel Multiple | 20.8% |

**Interpretation**: Strong at recognizing when NOT to call a function (irrelevance) and handling single parallel calls. Weaker at complex multi-function scenarios, which is acceptable since we use it only for routing.

---

## User Stories

### US-001: Automatic Domain Detection

**As a** caro user running general commands
**I want** caro to automatically detect the domain of my request
**So that** I get more accurate, domain-specific commands

**Acceptance Criteria:**
- System detects "find all rust files" → file_operations domain
- System detects "show my git branches" → git_operations domain
- System detects "what's my IP address" → network_diagnostics domain
- Domain-specific prompt used for generation
- No user action required for routing

**Example:**
```bash
$ caro "find all rust files larger than 1MB"

[Intent Router] Domain: file_operations (confidence: 0.94)
[Generator] Using file operations context

Command: find . -name "*.rs" -size +1M
```

---

### US-002: Multi-Domain Requests

**As a** power user with complex requests
**I want** caro to handle requests spanning multiple domains
**So that** I can describe compound operations naturally

**Acceptance Criteria:**
- System detects multiple applicable domains
- Primary domain selected for generation
- Related domains inform context
- Clear indication of multi-domain detection

**Example:**
```bash
$ caro "find large log files and compress them"

[Intent Router] Domains detected:
  - Primary: file_operations (0.87)
  - Secondary: archive_operations (0.72)

[Generator] Using file + archive context

Command: find . -name "*.log" -size +100M -exec gzip {} \;
```

---

### US-003: Domain-Specific Safety

**As a** developer using git commands
**I want** domain-aware safety warnings
**So that** I'm protected from domain-specific dangerous operations

**Acceptance Criteria:**
- Git domain warns about `--force` operations
- Network domain warns about binding to privileged ports
- File domain warns about recursive deletions
- Package domain warns about system-wide changes

**Example:**
```bash
$ caro "force push my changes"

[Intent Router] Domain: git_operations (confidence: 0.96)

⚠️  High Risk - Git Force Push Detected

This command rewrites remote history:
  git push --force origin main

Risks:
- Overwrites collaborators' commits
- Cannot be easily undone
- May break CI/CD pipelines

Proceed? (y/N):
```

---

### US-004: Fallback to General Mode

**As a** user with unusual requests
**I want** graceful fallback when domain detection fails
**So that** I still get useful commands

**Acceptance Criteria:**
- Low-confidence routing falls back to general mode
- User notified of fallback behavior
- General prompt still produces valid commands
- Option to specify domain manually

**Example:**
```bash
$ caro "do that thing we discussed"

[Intent Router] No confident domain match (max: 0.32)
[Fallback] Using general-purpose generation

Could not determine specific domain. Generating general command...
Please provide more details for better results.
```

---

### US-005: Manual Domain Override

**As a** power user who knows their intent
**I want** to manually specify the command domain
**So that** I can bypass automatic detection when needed

**Acceptance Criteria:**
- `--domain` flag accepts domain name
- Skips FunctionGemma routing
- Uses specified domain's prompt directly
- Lists available domains with `--list-domains`

**Example:**
```bash
$ caro --domain git "show recent activity"

[Manual Override] Domain: git_operations
[Generator] Using git context

Command: git log --oneline -n 10 --graph
```

---

## Technical Architecture

### System Overview

```
┌─────────────────────────────────────────────────────────────────────────┐
│                            CLI Interface                                 │
│                         (main.rs, cli/mod.rs)                           │
└────────────────────────────────┬────────────────────────────────────────┘
                                 │
                                 ▼
┌─────────────────────────────────────────────────────────────────────────┐
│                         Intent Router Layer                              │
│                                                                          │
│  ┌──────────────────────┐    ┌──────────────────────────────────────┐  │
│  │   FunctionGemma      │    │        Domain Registry               │  │
│  │   (Ollama Backend)   │───▶│  - file_operations                   │  │
│  │                      │    │  - git_operations                    │  │
│  │  Input: User prompt  │    │  - network_diagnostics               │  │
│  │  Output: Domain(s)   │    │  - process_management                │  │
│  └──────────────────────┘    │  - text_processing                   │  │
│                              │  - package_management                │  │
│                              │  - archive_operations                │  │
│                              │  - system_info                       │  │
│                              └──────────────────────────────────────┘  │
└────────────────────────────────┬────────────────────────────────────────┘
                                 │
                                 ▼
┌─────────────────────────────────────────────────────────────────────────┐
│                      Domain Context Loader                               │
│                                                                          │
│  ┌────────────────┐  ┌────────────────┐  ┌────────────────────────┐    │
│  │ Domain Prompt  │  │ Domain Safety  │  │   Domain Examples      │    │
│  │   Templates    │  │   Patterns     │  │   (few-shot)           │    │
│  └────────────────┘  └────────────────┘  └────────────────────────┘    │
└────────────────────────────────┬────────────────────────────────────────┘
                                 │
                                 ▼
┌─────────────────────────────────────────────────────────────────────────┐
│                      Command Generation Pipeline                         │
│                      (Existing Agentic Loop)                            │
│                                                                          │
│  ┌──────────────────┐    ┌──────────────────┐    ┌─────────────────┐   │
│  │ Domain-Specific  │    │   Refinement     │    │ Domain-Specific │   │
│  │    Generation    │───▶│   Iteration      │───▶│    Safety       │   │
│  └──────────────────┘    └──────────────────┘    └─────────────────┘   │
└────────────────────────────────┬────────────────────────────────────────┘
                                 │
                                 ▼
                          Generated Command
```

### Component Details

#### 1. Intent Router (`src/router/mod.rs`)

**Responsibilities:**
- Initialize FunctionGemma connection via Ollama
- Build tool definitions for each domain
- Route user requests to appropriate domain(s)
- Handle fallback for low-confidence matches
- Cache routing decisions for similar queries

**Key Types:**
```rust
pub struct IntentRouter {
    ollama_client: OllamaClient,
    domain_registry: DomainRegistry,
    cache: RoutingCache,
    config: RouterConfig,
}

pub struct RoutingResult {
    pub primary_domain: Domain,
    pub secondary_domains: Vec<Domain>,
    pub confidence: f32,
    pub raw_response: FunctionCall,
}

pub struct RouterConfig {
    pub model: String,              // "functiongemma"
    pub confidence_threshold: f32,  // 0.6
    pub fallback_domain: Domain,    // Domain::General
    pub enable_caching: bool,
    pub cache_ttl_seconds: u64,
}

impl IntentRouter {
    pub async fn new(config: RouterConfig) -> Result<Self>;
    pub async fn route(&self, user_input: &str) -> Result<RoutingResult>;
    pub fn is_available(&self) -> bool;
}
```

---

#### 2. Domain Registry (`src/router/domains.rs`)

**Responsibilities:**
- Define all supported command domains
- Provide tool definitions for FunctionGemma
- Map domains to prompt templates and safety rules
- Support domain discovery and listing

**Domains Defined:**

| Domain | Description | Key Commands |
|--------|-------------|--------------|
| `file_operations` | File and directory manipulation | find, ls, cp, mv, rm, mkdir, touch |
| `git_operations` | Version control operations | git (all subcommands) |
| `network_diagnostics` | Network testing and info | ping, curl, wget, netstat, ss, dig, nslookup |
| `process_management` | Process control and monitoring | ps, kill, top, htop, pgrep, pkill |
| `text_processing` | Text manipulation and search | grep, sed, awk, cat, head, tail, sort, uniq |
| `package_management` | Package installation/removal | brew, apt, yum, npm, pip, cargo |
| `archive_operations` | Compression and archiving | tar, zip, unzip, gzip, gunzip, 7z |
| `system_info` | System information queries | df, du, uname, uptime, whoami, id |
| `permission_management` | File permissions and ownership | chmod, chown, chgrp, umask |
| `general` | Fallback for unclassified requests | (all commands) |

**Key Types:**
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Domain {
    FileOperations,
    GitOperations,
    NetworkDiagnostics,
    ProcessManagement,
    TextProcessing,
    PackageManagement,
    ArchiveOperations,
    SystemInfo,
    PermissionManagement,
    General,
}

pub struct DomainDefinition {
    pub domain: Domain,
    pub name: String,
    pub description: String,
    pub keywords: Vec<String>,
    pub primary_commands: Vec<String>,
    pub prompt_template: String,
    pub safety_patterns: Vec<SafetyPattern>,
    pub examples: Vec<DomainExample>,
}

pub struct DomainRegistry {
    domains: HashMap<Domain, DomainDefinition>,
}

impl DomainRegistry {
    pub fn new() -> Self;
    pub fn get(&self, domain: Domain) -> &DomainDefinition;
    pub fn all_domains(&self) -> Vec<&DomainDefinition>;
    pub fn to_function_definitions(&self) -> Vec<FunctionDefinition>;
}
```

---

#### 3. FunctionGemma Client (`src/router/functiongemma.rs`)

**Responsibilities:**
- Format tool definitions for FunctionGemma's expected format
- Send routing requests to Ollama
- Parse function call responses
- Handle errors and timeouts

**Key Types:**
```rust
pub struct FunctionGemmaClient {
    ollama_url: String,
    model: String,
    timeout: Duration,
}

#[derive(Debug, Serialize)]
pub struct FunctionDefinition {
    pub name: String,
    pub description: String,
    pub parameters: ParameterSchema,
}

#[derive(Debug, Serialize)]
pub struct ParameterSchema {
    pub r#type: String,
    pub properties: HashMap<String, PropertySchema>,
    pub required: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct FunctionCall {
    pub name: String,
    pub arguments: HashMap<String, serde_json::Value>,
}

impl FunctionGemmaClient {
    pub async fn call_function(
        &self,
        user_prompt: &str,
        functions: &[FunctionDefinition],
    ) -> Result<FunctionCall>;
}
```

**FunctionGemma Request Format:**
```json
{
  "model": "functiongemma",
  "messages": [
    {
      "role": "user",
      "content": "find all rust files larger than 1MB"
    }
  ],
  "tools": [
    {
      "type": "function",
      "function": {
        "name": "file_operations",
        "description": "File and directory operations: finding, listing, copying, moving, removing files and directories",
        "parameters": {
          "type": "object",
          "properties": {
            "operation_type": {
              "type": "string",
              "enum": ["find", "list", "copy", "move", "remove", "create"],
              "description": "The type of file operation"
            },
            "target_description": {
              "type": "string",
              "description": "Description of files/directories to operate on"
            }
          },
          "required": ["operation_type", "target_description"]
        }
      }
    },
    {
      "type": "function",
      "function": {
        "name": "git_operations",
        "description": "Git version control operations: commits, branches, merges, status, history"
        // ... parameters
      }
    }
    // ... other domain functions
  ]
}
```

**FunctionGemma Response Format:**
```json
{
  "message": {
    "role": "assistant",
    "content": null,
    "tool_calls": [
      {
        "function": {
          "name": "file_operations",
          "arguments": "{\"operation_type\": \"find\", \"target_description\": \"rust files larger than 1MB\"}"
        }
      }
    ]
  }
}
```

---

#### 4. Domain Context Loader (`src/router/context.rs`)

**Responsibilities:**
- Load domain-specific prompt templates
- Inject domain examples into generation context
- Merge domain safety patterns with global patterns
- Handle multi-domain context merging

**Key Types:**
```rust
pub struct DomainContext {
    pub domain: Domain,
    pub system_prompt: String,
    pub examples: Vec<Example>,
    pub safety_patterns: Vec<SafetyPattern>,
    pub platform_notes: String,
}

pub struct DomainContextLoader {
    registry: Arc<DomainRegistry>,
    platform: PlatformInfo,
}

impl DomainContextLoader {
    pub fn load(&self, result: &RoutingResult) -> DomainContext;
    pub fn merge_domains(&self, domains: &[Domain]) -> DomainContext;
}
```

---

#### 5. Domain Safety Patterns (`src/router/safety.rs`)

**Responsibilities:**
- Define domain-specific dangerous patterns
- Extend existing safety validation
- Provide domain-aware warning messages
- Handle domain-specific risk levels

**Domain-Specific Patterns:**

```rust
// Git Operations - High Risk Patterns
pub const GIT_DANGEROUS_PATTERNS: &[SafetyPattern] = &[
    SafetyPattern {
        pattern: r"git\s+push\s+.*--force",
        risk_level: RiskLevel::High,
        message: "Force push overwrites remote history",
        domain: Domain::GitOperations,
    },
    SafetyPattern {
        pattern: r"git\s+reset\s+--hard",
        risk_level: RiskLevel::High,
        message: "Hard reset discards all uncommitted changes",
        domain: Domain::GitOperations,
    },
    SafetyPattern {
        pattern: r"git\s+clean\s+-fd",
        risk_level: RiskLevel::Moderate,
        message: "Removes all untracked files and directories",
        domain: Domain::GitOperations,
    },
    SafetyPattern {
        pattern: r"git\s+rebase\s+.*--force",
        risk_level: RiskLevel::High,
        message: "Force rebase rewrites commit history",
        domain: Domain::GitOperations,
    },
];

// Network Diagnostics - Moderate Risk Patterns
pub const NETWORK_DANGEROUS_PATTERNS: &[SafetyPattern] = &[
    SafetyPattern {
        pattern: r"nc\s+.*-l\s+.*-e",
        risk_level: RiskLevel::Critical,
        message: "Creates a network backdoor",
        domain: Domain::NetworkDiagnostics,
    },
    SafetyPattern {
        pattern: r"curl\s+.*\|\s*(sudo\s+)?bash",
        risk_level: RiskLevel::Critical,
        message: "Executes untrusted remote code",
        domain: Domain::NetworkDiagnostics,
    },
    SafetyPattern {
        pattern: r"bind.*:(80|443|22)\b",
        risk_level: RiskLevel::Moderate,
        message: "Binds to privileged port",
        domain: Domain::NetworkDiagnostics,
    },
];

// Package Management - Moderate Risk Patterns
pub const PACKAGE_DANGEROUS_PATTERNS: &[SafetyPattern] = &[
    SafetyPattern {
        pattern: r"(apt|yum|dnf)\s+.*--force",
        risk_level: RiskLevel::Moderate,
        message: "Force installing may break dependencies",
        domain: Domain::PackageManagement,
    },
    SafetyPattern {
        pattern: r"pip\s+install\s+--user.*--break-system",
        risk_level: RiskLevel::High,
        message: "May corrupt system Python installation",
        domain: Domain::PackageManagement,
    },
    SafetyPattern {
        pattern: r"npm\s+.*--unsafe-perm",
        risk_level: RiskLevel::Moderate,
        message: "Runs scripts with elevated permissions",
        domain: Domain::PackageManagement,
    },
];

// Permission Management - High Risk Patterns
pub const PERMISSION_DANGEROUS_PATTERNS: &[SafetyPattern] = &[
    SafetyPattern {
        pattern: r"chmod\s+.*777\s+/",
        risk_level: RiskLevel::Critical,
        message: "Makes system files world-writable",
        domain: Domain::PermissionManagement,
    },
    SafetyPattern {
        pattern: r"chown\s+.*-R\s+.*:\s+/",
        risk_level: RiskLevel::Critical,
        message: "Recursively changes system file ownership",
        domain: Domain::PermissionManagement,
    },
    SafetyPattern {
        pattern: r"chmod\s+.*\+s",
        risk_level: RiskLevel::High,
        message: "Sets setuid/setgid bit - potential privilege escalation",
        domain: Domain::PermissionManagement,
    },
];
```

---

### Integration with Existing Architecture

#### Modified Command Generation Flow

```
┌─────────────────────────────────────────────────────────────────┐
│                     EXISTING FLOW (Modified)                     │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  User Input                                                      │
│      │                                                           │
│      ▼                                                           │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │ NEW: Intent Router (FunctionGemma)                       │    │
│  │  - If Ollama available with functiongemma: Route         │    │
│  │  - If not available: Skip to general generation          │    │
│  └───────────────────────────┬─────────────────────────────┘    │
│                              │                                   │
│                              ▼                                   │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │ NEW: Domain Context Loader                               │    │
│  │  - Load domain-specific prompt template                  │    │
│  │  - Load domain-specific examples                         │    │
│  │  - Merge domain safety patterns                          │    │
│  └───────────────────────────┬─────────────────────────────┘    │
│                              │                                   │
│                              ▼                                   │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │ EXISTING: Agentic Generation Loop (agent/mod.rs)        │    │
│  │  - Iteration 1: Initial generation with domain context  │    │
│  │  - Iteration 2: Refinement with command introspection   │    │
│  └───────────────────────────┬─────────────────────────────┘    │
│                              │                                   │
│                              ▼                                   │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │ MODIFIED: Safety Validation                              │    │
│  │  - Global patterns (existing 52)                         │    │
│  │  - + Domain-specific patterns (new ~30)                  │    │
│  └───────────────────────────┬─────────────────────────────┘    │
│                              │                                   │
│                              ▼                                   │
│                     Generated Command                            │
└─────────────────────────────────────────────────────────────────┘
```

#### Backend Integration

FunctionGemma runs via Ollama, leveraging the existing `OllamaBackend`:

```rust
// src/backends/remote/ollama.rs (Modified)

impl OllamaBackend {
    /// Check if a specific model is available
    pub async fn has_model(&self, model: &str) -> bool {
        // Call /api/tags and check for model
    }

    /// Call model with function definitions (for FunctionGemma)
    pub async fn call_with_tools(
        &self,
        model: &str,
        prompt: &str,
        tools: &[FunctionDefinition],
    ) -> Result<ToolCallResponse> {
        // POST /api/chat with tools parameter
    }
}
```

---

### Domain Prompt Templates

#### File Operations Domain

```toml
# domains/file_operations.toml

[meta]
domain = "file_operations"
name = "File Operations"
description = "File and directory manipulation commands"
primary_commands = ["find", "ls", "cp", "mv", "rm", "mkdir", "touch", "ln"]

[prompt]
system = """
You are a shell command expert specializing in FILE OPERATIONS for {{os}}/{{unix_flavor}}.

DOMAIN FOCUS: File and directory manipulation only.
Available commands: find, ls, cp, mv, rm, mkdir, touch, ln, readlink

PLATFORM CONSTRAINTS:
{{platform_notes}}

SAFETY REQUIREMENTS:
- NEVER generate rm -rf without explicit user confirmation context
- NEVER use wildcards with rm on system directories
- Prefer relative paths (. or ~/) over absolute paths
- Use -i flag for interactive prompts on destructive operations

Response format (JSON only):
{"cmd": "your_command_here"}

User request: {{user_input}}
"""

[examples]
find_by_name = { input = "find rust files", output = "find . -name '*.rs'" }
find_by_size = { input = "find large files", output = "find . -type f -size +100M" }
find_recent = { input = "find files modified today", output = "find . -type f -mtime 0" }
list_sorted = { input = "list by size", output = "ls -lhS" }
copy_recursive = { input = "copy directory", output = "cp -r src/ dest/" }
```

#### Git Operations Domain

```toml
# domains/git_operations.toml

[meta]
domain = "git_operations"
name = "Git Operations"
description = "Git version control commands"
primary_commands = ["git"]

[prompt]
system = """
You are a shell command expert specializing in GIT OPERATIONS.

DOMAIN FOCUS: Git version control only.
Available commands: git (all subcommands)

SAFETY REQUIREMENTS:
- NEVER use --force without explicit user intent
- Warn about history-rewriting commands (rebase, reset --hard)
- Prefer git stash over git reset for temporary changes
- Always specify remote and branch explicitly

Response format (JSON only):
{"cmd": "your_command_here"}

User request: {{user_input}}
"""

[examples]
recent_commits = { input = "show recent commits", output = "git log --oneline -n 10" }
current_branch = { input = "what branch am I on", output = "git branch --show-current" }
uncommitted = { input = "show changes", output = "git status -s" }
create_branch = { input = "new feature branch", output = "git checkout -b feature/new-feature" }
```

---

## Configuration

### New Configuration Options

```toml
# ~/.config/caro/config.toml

[router]
# Enable intent routing via FunctionGemma
enabled = true

# FunctionGemma model name in Ollama
model = "functiongemma"

# Minimum confidence for domain selection (0.0-1.0)
confidence_threshold = 0.6

# Fallback domain when confidence is low
fallback_domain = "general"

# Cache routing decisions for similar queries
enable_caching = true
cache_ttl_seconds = 3600

# Connection timeout for FunctionGemma calls
timeout_ms = 5000

[router.domains]
# Enable/disable specific domains
file_operations = true
git_operations = true
network_diagnostics = true
process_management = true
text_processing = true
package_management = true
archive_operations = true
system_info = true
permission_management = true

[router.custom_domains]
# Future: Allow user-defined domains
# See docs for domain definition format
```

### CLI Arguments

```bash
# Route to specific domain (skips FunctionGemma)
caro --domain git "show my branches"

# List available domains
caro --list-domains

# Disable routing for this request
caro --no-route "just do the thing"

# Show routing decision (verbose)
caro -v "find rust files"
# Output includes: [Router] Domain: file_operations (0.94)
```

---

## Implementation Plan

### Phase 1: Core Router Infrastructure

**Deliverables:**
1. IntentRouter struct with Ollama integration
2. Domain registry with 10 domains
3. FunctionGemma client with tool definitions
4. Basic routing logic

**Files to Create:**
```
src/router/
├── mod.rs              # Module exports
├── router.rs           # IntentRouter implementation
├── domains.rs          # Domain enum and registry
├── functiongemma.rs    # FunctionGemma client
└── config.rs           # Router configuration
```

**Tests:**
- Unit tests for each domain definition
- Integration test with mock Ollama
- Routing accuracy tests with sample prompts

---

### Phase 2: Domain Context System

**Deliverables:**
1. Domain-specific prompt templates
2. Domain context loader
3. Integration with agentic loop
4. Domain-specific examples

**Files to Create:**
```
src/router/
├── context.rs          # Domain context loader
└── templates/          # Domain TOML templates
    ├── file_operations.toml
    ├── git_operations.toml
    ├── network_diagnostics.toml
    └── ... (other domains)
```

**Tests:**
- Template loading and variable substitution
- Context merging for multi-domain requests
- Integration with generation pipeline

---

### Phase 3: Domain Safety Patterns

**Deliverables:**
1. Domain-specific safety patterns (~30 new)
2. Integration with safety validator
3. Domain-aware warning messages
4. Risk level adjustments per domain

**Files to Modify:**
```
src/safety/
├── mod.rs              # Add domain parameter
├── patterns.rs         # Add domain patterns
└── validator.rs        # Domain-aware validation
```

**Files to Create:**
```
src/router/
└── safety.rs           # Domain safety definitions
```

**Tests:**
- Domain pattern detection
- Warning message accuracy
- Integration with existing safety validation

---

### Phase 4: Polish & Optimization

**Deliverables:**
1. Routing cache implementation
2. Performance optimization
3. CLI argument handling
4. Documentation and examples

**Tests:**
- Cache hit/miss scenarios
- Performance benchmarks
- End-to-end integration tests

---

## Performance Requirements

| Operation | Target | Measurement |
|-----------|--------|-------------|
| FunctionGemma cold start | <500ms | First routing call |
| FunctionGemma routing | <100ms | Per request (warm) |
| Domain context loading | <10ms | Template + examples |
| Total routing overhead | <150ms | End-to-end |
| Cache lookup | <1ms | Cached routing |

### Optimization Strategies

1. **Lazy Loading**: Only load FunctionGemma when routing enabled
2. **Connection Pooling**: Reuse Ollama HTTP connection
3. **Template Caching**: Cache parsed domain templates
4. **Routing Cache**: Cache domain decisions for similar queries
5. **Parallel Loading**: Load domain context while generating

---

## Security Considerations

### 1. FunctionGemma Isolation

**Risk:** Malicious prompts could manipulate FunctionGemma
**Mitigation:**
- FunctionGemma only selects domains, never generates commands
- Output validated against known domain names
- Fallback to general mode on unexpected output

### 2. Domain Escalation Prevention

**Risk:** Attacker tricks router into wrong domain for weaker safety
**Mitigation:**
- All commands pass through global safety validation
- Domain patterns are additive, not replacing global patterns
- Critical patterns (rm -rf /, fork bombs) always apply

### 3. Prompt Injection in Routing

**Risk:** User input manipulates domain selection
**Mitigation:**
- User input isolated in function call parameters
- Tool definitions are static, not user-modifiable
- Low confidence triggers fallback, not bypass

---

## Testing Strategy

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_domain_registry_all_domains() {
        let registry = DomainRegistry::new();
        assert_eq!(registry.all_domains().len(), 10);
    }

    #[test]
    fn test_file_operations_keywords() {
        let registry = DomainRegistry::new();
        let domain = registry.get(Domain::FileOperations);
        assert!(domain.keywords.contains(&"find".to_string()));
    }

    #[tokio::test]
    async fn test_router_fallback_no_ollama() {
        let router = IntentRouter::new(RouterConfig::default()).await;
        // Should not fail, just disable routing
        assert!(!router.is_available());
    }
}
```

### Integration Tests

```rust
#[tokio::test]
async fn test_file_operations_routing() {
    let router = create_test_router().await;
    let result = router.route("find all rust files").await.unwrap();
    assert_eq!(result.primary_domain, Domain::FileOperations);
    assert!(result.confidence > 0.8);
}

#[tokio::test]
async fn test_multi_domain_detection() {
    let router = create_test_router().await;
    let result = router.route("find large logs and compress them").await.unwrap();
    assert_eq!(result.primary_domain, Domain::FileOperations);
    assert!(result.secondary_domains.contains(&Domain::ArchiveOperations));
}
```

### Accuracy Benchmarks

```rust
const ROUTING_TEST_CASES: &[(&str, Domain)] = &[
    ("find all rust files", Domain::FileOperations),
    ("show git branches", Domain::GitOperations),
    ("ping google.com", Domain::NetworkDiagnostics),
    ("kill process 1234", Domain::ProcessManagement),
    ("search for TODO comments", Domain::TextProcessing),
    ("install nodejs", Domain::PackageManagement),
    ("create a tarball", Domain::ArchiveOperations),
    ("show disk usage", Domain::SystemInfo),
    ("make file executable", Domain::PermissionManagement),
];

#[tokio::test]
async fn test_routing_accuracy() {
    let router = create_test_router().await;
    let mut correct = 0;

    for (input, expected) in ROUTING_TEST_CASES {
        let result = router.route(input).await.unwrap();
        if result.primary_domain == *expected {
            correct += 1;
        }
    }

    let accuracy = correct as f32 / ROUTING_TEST_CASES.len() as f32;
    assert!(accuracy >= 0.95, "Routing accuracy: {:.1}%", accuracy * 100.0);
}
```

---

## Dependencies

### Required

- **Ollama v0.13.5+**: Required for FunctionGemma model
- **functiongemma model**: Must be pulled: `ollama pull functiongemma`

### Optional

- None (FunctionGemma is optional enhancement; caro works without it)

### Graceful Degradation

When FunctionGemma is unavailable:
1. Router detects missing Ollama or model
2. Routing is transparently disabled
3. All requests use general-purpose generation
4. No user-facing errors or warnings

---

## Success Criteria

### Launch Criteria

- [ ] All 10 domains defined with templates
- [ ] Routing accuracy ≥95% on test set
- [ ] Routing latency <150ms (warm)
- [ ] Graceful fallback when Ollama unavailable
- [ ] Domain-specific safety patterns implemented
- [ ] CLI arguments working (--domain, --list-domains)
- [ ] Documentation complete

### Post-Launch Metrics (30 days)

- Routing accuracy in production: ≥90%
- User-reported wrong domain: <5%
- Performance regression: None
- Crashes/errors: 0

---

## Open Questions

1. **Q:** Should we support custom user-defined domains?
   **A:** Phase 2 feature - allow users to add domains via config

2. **Q:** What if user wants to combine domains explicitly?
   **A:** Support `--domain file,archive` syntax for multi-domain

3. **Q:** How to handle ambiguous requests like "show files"?
   **A:** Use confidence threshold; low confidence triggers clarification

4. **Q:** Should routing decisions be visible by default?
   **A:** Only in verbose mode (`-v`) to reduce noise

5. **Q:** What if FunctionGemma gives wrong domain consistently?
   **A:** Allow domain blocklist in config; report issues for prompt improvement

---

## Parallel Pre-Processing Architecture

### Core Philosophy: Protect the Inference

The decoder model (Qwen2.5-Coder) is our bottleneck—every call costs ~1500ms. Our goal is to:

1. **Prepare maximum context** before hitting the model
2. **Hit the model once** with a high-quality, domain-specific prompt
3. **Minimize post-processing** because needing it means we failed at pre-processing

### Three-Phase Model

```
┌─────────────────────────────────────────────────────────────────────────┐
│                           THE THREE PHASES                               │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  PHASE 1: PRE-PROCESSING              PHASE 2: INFERENCE                │
│  (Parallelizable, <200ms)             (Protected, ~1500ms)              │
│  ┌─────────────────────────┐          ┌──────────────────────┐          │
│  │ • Intent Classification │          │ • Single LLM Call    │          │
│  │ • Context Gathering     │    ──►   │ • Domain-Optimized   │          │
│  │ • Rule Selection        │          │ • One-Shot Goal      │          │
│  │ • Early Exit Check      │          └──────────────────────┘          │
│  └─────────────────────────┘                    │                        │
│          │                                      │                        │
│          ▼                                      ▼                        │
│  ┌─────────────────────────┐          ┌──────────────────────┐          │
│  │ EARLY EXIT (if safe)    │          │  PHASE 3: POST       │          │
│  │ • Block dangerous cmd   │          │  (Minimize This)     │          │
│  │ • Cached response       │          │ • Safety validation  │          │
│  │ • Clarification needed  │          │ • Format output      │          │
│  └─────────────────────────┘          └──────────────────────┘          │
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘
```

### Tokio-Based Parallelization

```rust
use tokio::try_join;

pub async fn pre_process(input: &str) -> Result<GenerationContext> {
    // STAGE 1: Parallel analysis (all run concurrently)
    let (intent_result, platform_context, early_safety) = try_join!(
        intent_router.classify(input),
        context_scanner.scan(),
        async { Ok(early_safety_checker.check(input)) },
    )?;

    // STAGE 2: Early exit gate
    match early_safety {
        EarlySafetyResult::Block { reason, severity } => {
            return Err(Error::SafetyBlock { reason, severity });
        }
        EarlySafetyResult::Continue => {}
    }

    // Check cache before context assembly
    if let Some(cached) = cache.get(input, &intent_result.primary_domain) {
        return Ok(cached);
    }

    // STAGE 3: Parallel context assembly (domain-aware)
    let (prompt_template, safety_rules, examples) = try_join!(
        domain_loader.load_prompt(&intent_result),
        async { Ok(rule_selector.select(&intent_result)) },
        example_selector.select(&intent_result),
    )?;

    Ok(GenerationContext { /* assembled context */ })
}
```

### Lazy Rule Evaluation

Good programs break early. If we can answer without full processing:

- **Critical danger detected** → Block immediately, don't generate
- **Cached response exists** → Return immediately, don't regenerate
- **Clarification needed** → Ask user, don't guess

```rust
pub struct LazyRuleEngine {
    // Rules organized by domain for fast lookup
    domain_rules: HashMap<Domain, Vec<CompiledPattern>>,

    // Global critical rules that always apply
    global_critical: Vec<CompiledPattern>,
}

impl LazyRuleEngine {
    /// Validate with early termination
    pub fn validate(&self, command: &str, domain: Domain) -> ValidationResult {
        // Level 1: Critical patterns only (very fast)
        for pattern in &self.global_critical {
            if pattern.is_match(command) {
                return ValidationResult::critical_block(pattern);
            }
        }

        // Level 2: Domain-specific patterns (if we know domain)
        if let Some(rules) = self.domain_rules.get(&domain) {
            for pattern in rules {
                if pattern.is_match(command) {
                    return ValidationResult::domain_match(pattern);
                }
            }
        }

        ValidationResult::safe()
    }
}
```

### Performance Budget

| Phase | Budget | Components |
|-------|--------|------------|
| **Pre-Processing** | 200ms | Intent + Context + Safety (parallel) |
| **Context Assembly** | 50ms | Domain loading + Rule selection |
| **Inference** | 1500ms | Coder model generation |
| **Post-Processing** | 50ms | Final safety + formatting |
| **Total** | **1800ms** | End-to-end target |

### Early Exit Performance

| Scenario | Time | Savings |
|----------|------|---------|
| Critical safety block | <15ms | 1785ms (99%) |
| Cache hit | <5ms | 1795ms (99.7%) |
| Clarification prompt | <50ms | 1750ms (97%) |
| Normal flow | 1800ms | 0ms |

### Memory Model Integration

The pre-processing pipeline integrates with Caro's planned memory model:

```
┌─────────────────────────────────────────────────────────────────┐
│                        MEMORY MODEL                              │
├─────────────────────────────────────────────────────────────────┤
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────────────┐  │
│  │ Session      │  │ Tool         │  │ Pattern              │  │
│  │ Memory       │  │ Memory       │  │ Memory               │  │
│  │              │  │              │  │                      │  │
│  │ • Recent     │  │ • Which      │  │ • User's cmd prefs   │  │
│  │   commands   │  │   tools work │  │ • Common tasks       │  │
│  │ • Domain     │  │   for intent │  │ • Corrections        │  │
│  │   preferences│  │ • Flags by   │  │                      │  │
│  │              │  │   platform   │  │                      │  │
│  └──────────────┘  └──────────────┘  └──────────────────────┘  │
│         │                  │                    │               │
│         └──────────────────┼────────────────────┘               │
│                            ▼                                     │
│              ┌─────────────────────────────┐                    │
│              │   Pre-Processing Pipeline   │                    │
│              │                             │                    │
│              │  Uses memory for:           │                    │
│              │  • Smarter routing          │                    │
│              │  • Better context           │                    │
│              │  • Personalized rules       │                    │
│              └─────────────────────────────┘                    │
└─────────────────────────────────────────────────────────────────┘
```

---

## Related Documents

- [ADR-004: Pre-Processing Pipeline Architecture](../../docs/adr/ADR-004-pre-processing-pipeline.md)
- [PRD: FunctionGemma Intent Router](./PRD.md)
- [Research: FunctionGemma Integration](./research.md)

---

## References

- [FunctionGemma on Ollama](https://ollama.com/library/functiongemma)
- [Gemma 3 Technical Report](https://ai.google.dev/gemma)
- [Berkeley Function Calling Leaderboard](https://gorilla.cs.berkeley.edu/blogs/8_berkeley_function_calling_leaderboard.html)
- [Caro Spec 004: Remote LLM Backends](./004-implement-ollama-and/spec.md)
- [Caro Spec 006: Intelligent Prompt Generation](./006-intelligent-prompt-generation/spec.md)

---

## Appendix A: Domain Tool Definitions

Complete FunctionGemma tool definitions for all domains:

```json
{
  "tools": [
    {
      "type": "function",
      "function": {
        "name": "file_operations",
        "description": "File and directory operations: finding files by name/size/date, listing directory contents, copying, moving, removing, creating files and directories, creating symbolic links",
        "parameters": {
          "type": "object",
          "properties": {
            "operation": {
              "type": "string",
              "enum": ["find", "list", "copy", "move", "remove", "create", "link"],
              "description": "The type of file operation to perform"
            },
            "target": {
              "type": "string",
              "description": "Description of files or directories to operate on"
            }
          },
          "required": ["operation", "target"]
        }
      }
    },
    {
      "type": "function",
      "function": {
        "name": "git_operations",
        "description": "Git version control: viewing history/status/branches, committing changes, creating/switching branches, merging, rebasing, pushing/pulling",
        "parameters": {
          "type": "object",
          "properties": {
            "operation": {
              "type": "string",
              "enum": ["status", "log", "branch", "commit", "merge", "rebase", "push", "pull", "stash", "diff"],
              "description": "The git operation to perform"
            },
            "details": {
              "type": "string",
              "description": "Additional details about the operation"
            }
          },
          "required": ["operation"]
        }
      }
    },
    {
      "type": "function",
      "function": {
        "name": "network_diagnostics",
        "description": "Network testing and information: pinging hosts, checking connectivity, downloading files, viewing network connections, DNS lookups",
        "parameters": {
          "type": "object",
          "properties": {
            "operation": {
              "type": "string",
              "enum": ["ping", "download", "connections", "dns", "http_request", "ports"],
              "description": "The network diagnostic operation"
            },
            "target": {
              "type": "string",
              "description": "Target host, URL, or network resource"
            }
          },
          "required": ["operation"]
        }
      }
    },
    {
      "type": "function",
      "function": {
        "name": "process_management",
        "description": "Process control: listing running processes, killing/stopping processes, monitoring system resources, finding processes by name",
        "parameters": {
          "type": "object",
          "properties": {
            "operation": {
              "type": "string",
              "enum": ["list", "kill", "find", "monitor", "background"],
              "description": "The process management operation"
            },
            "target": {
              "type": "string",
              "description": "Process name, PID, or search criteria"
            }
          },
          "required": ["operation"]
        }
      }
    },
    {
      "type": "function",
      "function": {
        "name": "text_processing",
        "description": "Text manipulation and search: searching file contents, filtering/transforming text, sorting, removing duplicates, extracting patterns",
        "parameters": {
          "type": "object",
          "properties": {
            "operation": {
              "type": "string",
              "enum": ["search", "filter", "transform", "sort", "count", "extract"],
              "description": "The text processing operation"
            },
            "pattern": {
              "type": "string",
              "description": "Search pattern or text transformation rule"
            }
          },
          "required": ["operation"]
        }
      }
    },
    {
      "type": "function",
      "function": {
        "name": "package_management",
        "description": "Package installation and management: installing/removing packages via brew, apt, npm, pip, cargo, updating packages, searching packages",
        "parameters": {
          "type": "object",
          "properties": {
            "operation": {
              "type": "string",
              "enum": ["install", "remove", "update", "search", "list", "info"],
              "description": "The package management operation"
            },
            "package_manager": {
              "type": "string",
              "enum": ["brew", "apt", "yum", "npm", "pip", "cargo", "auto"],
              "description": "Which package manager to use (auto = detect)"
            },
            "package": {
              "type": "string",
              "description": "Package name or search term"
            }
          },
          "required": ["operation"]
        }
      }
    },
    {
      "type": "function",
      "function": {
        "name": "archive_operations",
        "description": "Compression and archiving: creating/extracting tar archives, zip files, gzip compression, viewing archive contents",
        "parameters": {
          "type": "object",
          "properties": {
            "operation": {
              "type": "string",
              "enum": ["create", "extract", "list", "compress", "decompress"],
              "description": "The archive operation"
            },
            "format": {
              "type": "string",
              "enum": ["tar", "tar.gz", "zip", "gzip", "auto"],
              "description": "Archive format"
            },
            "target": {
              "type": "string",
              "description": "Files to archive or archive to extract"
            }
          },
          "required": ["operation"]
        }
      }
    },
    {
      "type": "function",
      "function": {
        "name": "system_info",
        "description": "System information queries: disk usage, directory sizes, system uptime, user info, OS details, environment variables",
        "parameters": {
          "type": "object",
          "properties": {
            "query": {
              "type": "string",
              "enum": ["disk", "memory", "uptime", "user", "os", "env", "hardware"],
              "description": "Type of system information to query"
            },
            "target": {
              "type": "string",
              "description": "Specific path or resource to query (optional)"
            }
          },
          "required": ["query"]
        }
      }
    },
    {
      "type": "function",
      "function": {
        "name": "permission_management",
        "description": "File permissions and ownership: changing file permissions, changing ownership, viewing permissions, managing access control",
        "parameters": {
          "type": "object",
          "properties": {
            "operation": {
              "type": "string",
              "enum": ["chmod", "chown", "view", "set_executable"],
              "description": "The permission operation"
            },
            "target": {
              "type": "string",
              "description": "File or directory to modify"
            },
            "permissions": {
              "type": "string",
              "description": "Permission specification (e.g., 755, u+x)"
            }
          },
          "required": ["operation", "target"]
        }
      }
    },
    {
      "type": "function",
      "function": {
        "name": "general",
        "description": "General shell commands that don't fit other categories, or when the request is unclear or spans multiple domains",
        "parameters": {
          "type": "object",
          "properties": {
            "description": {
              "type": "string",
              "description": "Description of what the user wants to accomplish"
            }
          },
          "required": ["description"]
        }
      }
    }
  ]
}
```

---

**Document Status:** Ready for Review
**Next Steps:** Architecture review → Implementation planning → TDD development
