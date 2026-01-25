# Skills Extension System Specification

**Status**: Draft
**Version**: 0.1.0
**Last Updated**: 2025-12-31

## Overview

The Skills Extension System enables community contributions of domain knowledge and executable behaviors without modifying caro's core binary. Skills represent tool/ecosystem expertise that enhances command generation.

## Goals

1. **Community Contributions**: Enable non-Rust developers to contribute domain expertise
2. **Lean Core**: Keep caro binary <50MB, load skills on demand
3. **Offline Support**: Full functionality in air-gapped environments
4. **Security-First**: Sandboxed execution with explicit capability grants
5. **Flexibility**: Support install-time, runtime, and build-time inclusion

## Non-Goals

1. **Full Package Manager**: Not building npm/cargo-like dependency resolution
2. **Universal Plugin System**: Skills are caro-specific, not general plugins
3. **Backwards Compatibility Forever**: API versioning allows breaking changes
4. **Skill Marketplace**: Initial focus on local/git distribution only

## Skill Anatomy

### Skill Types

| Type | Contents | Execution | Friction |
|------|----------|-----------|----------|
| **Knowledge** | Markdown docs, prompt templates | None (context injection) | Lowest |
| **Recipe** | Declarative workflows with guardrails | Interpreted by caro | Medium |
| **Executable** | WASM modules | Sandboxed runtime | Highest |

### Directory Structure

```
skill-name/
├── skill.toml              # Manifest (required)
├── README.md               # Documentation (recommended)
├── LICENSE                 # License file (required for distribution)
├── knowledge/              # Knowledge assets
│   ├── overview.md         # High-level domain overview
│   ├── concepts/           # Core concepts
│   │   ├── services.md
│   │   └── workflows.md
│   ├── patterns/           # Common patterns
│   │   ├── deployment.md
│   │   └── troubleshooting.md
│   └── prompts/            # Prompt enhancement templates
│       ├── context.md      # Always-included context
│       └── specialized/    # Task-specific prompts
│           ├── debug.md
│           └── deploy.md
├── recipes/                # Declarative workflows
│   ├── deploy-app.yaml
│   ├── rollback.yaml
│   └── scale-service.yaml
├── bin/                    # Executable modules (optional)
│   └── skill.wasm          # WASM component
└── tests/                  # Skill tests
    ├── knowledge_test.md   # Knowledge validation
    └── recipe_test.yaml    # Recipe test cases
```

## Skill Manifest Format

The manifest uses TOML format for Rust ecosystem alignment.

### Complete Example

```toml
# skill.toml - AWS Cloud Skill

[skill]
id = "cloud.aws"
name = "AWS Cloud Skill"
version = "0.3.0"
description = "AWS workflows from terminal: awscli, sso, iam, cloudwatch, eks, terraform patterns"
authors = ["caro-community <community@caro.dev>"]
license = "MIT"
repository = "https://github.com/caro-skills/cloud-aws"
documentation = "https://caro.dev/skills/cloud-aws"
keywords = ["aws", "cloud", "devops", "infrastructure"]

# API compatibility
api_version = "1.0"
min_caro_version = "1.1.0"

# What this skill provides
[provides]
knowledge = true      # Has knowledge/ directory
recipes = true        # Has recipes/ directory
executable = false    # No WASM module (MVP: false)

# Knowledge pack configuration
[knowledge]
# Topics this skill enhances
topics = ["aws", "cloud", "s3", "ec2", "lambda", "eks", "iam", "cloudwatch"]

# Context injection priority (higher = injected first)
priority = 100

# Maximum tokens to inject (prevents bloat)
max_context_tokens = 2000

# Recipe configuration
[recipes]
# Recipe files to load
files = ["recipes/*.yaml"]

# Default confirmation level
default_confirmation = "prompt"  # "auto" | "prompt" | "always"

# Executable configuration (future)
[executable]
# WASM module path
module = "bin/skill.wasm"

# Exported functions
exports = ["discover_tools", "parse_output", "plan_workflow"]

# Capability requests (must be granted at enable time)
[capabilities]
# Terminal execution permission
terminal_exec = {
    allowed = true,
    commands = ["aws", "eksctl", "kubectl"],  # Allowlist (empty = all)
    blocked = ["rm", "dd", "mkfs"]            # Denylist
}

# Filesystem access
filesystem_read = [
    "~/.aws",
    "~/.kube",
    "./",
    "$CARO_PROJECT_ROOT"
]
filesystem_write = [
    "./.caro-cache/aws"
]

# Network access
network = [
    "*.amazonaws.com",
    "sts.*.amazonaws.com"
]

# Environment variable access
env_read = [
    "AWS_*",
    "KUBECONFIG"
]

# No secrets access by default
secrets_access = false

# Dependencies on other skills
[dependencies]
"core.shell" = ">=1.0"        # Built-in skill
"tool.kubectl" = { version = ">=0.2", optional = true }

# Suggested (not required) skills
[suggestions]
"tool.terraform" = "Works great with Terraform for IaC"
"tool.docker" = "Useful for ECR container workflows"

# Conditional features
[features]
eks = { description = "EKS cluster management", default = true }
lambda = { description = "Lambda function workflows", default = true }
iam = { description = "IAM policy management", default = true }

# Platform requirements
[platform]
# Supported platforms (empty = all)
os = []  # ["macos", "linux", "windows"]
arch = []  # ["aarch64", "x86_64"]

# Required external tools
required_tools = ["aws"]
optional_tools = ["eksctl", "sam", "cdk"]
```

### Minimal Example (Knowledge-Only)

```toml
# skill.toml - Minimal knowledge skill

[skill]
id = "lang.python"
name = "Python Development"
version = "0.1.0"
description = "Python development patterns and tooling"
api_version = "1.0"

[provides]
knowledge = true
recipes = false
executable = false

[knowledge]
topics = ["python", "pip", "venv", "poetry", "pytest"]
priority = 50
max_context_tokens = 1000
```

## Recipe Format

Recipes define structured workflows with safety guardrails.

### Recipe Schema

```yaml
# recipes/deploy-app.yaml

id: deploy-app
name: Deploy Application to EKS
description: Deploy a containerized application to Amazon EKS cluster
version: "1.0"

# When to suggest this recipe
triggers:
  - keywords: ["deploy", "eks", "kubernetes", "k8s"]
  - intent: "deploy application"
  - context:
      has_files: ["Dockerfile", "k8s/"]

# Required parameters
parameters:
  - name: cluster_name
    type: string
    required: true
    description: "EKS cluster name"
    validation: "^[a-z0-9-]+$"

  - name: image_tag
    type: string
    required: true
    description: "Docker image tag to deploy"
    default: "latest"

  - name: namespace
    type: string
    required: false
    description: "Kubernetes namespace"
    default: "default"

# Preconditions that must be met
preconditions:
  - check: "command_exists"
    args: ["kubectl", "aws"]
    message: "kubectl and aws CLI must be installed"

  - check: "env_set"
    args: ["AWS_PROFILE"]
    message: "AWS_PROFILE must be set"
    fallback: "export AWS_PROFILE=default"

  - check: "custom"
    script: "aws sts get-caller-identity"
    message: "Must be authenticated to AWS"

# Workflow steps
steps:
  - id: update-kubeconfig
    name: "Update kubeconfig"
    command: "aws eks update-kubeconfig --name {{cluster_name}} --region {{aws_region}}"
    description: "Configure kubectl to use EKS cluster"
    confirmation: auto  # No confirmation needed

  - id: verify-cluster
    name: "Verify cluster access"
    command: "kubectl cluster-info"
    description: "Ensure cluster is accessible"
    confirmation: auto
    expect:
      exit_code: 0
      output_contains: "Kubernetes control plane"

  - id: apply-manifests
    name: "Apply Kubernetes manifests"
    command: "kubectl apply -f k8s/ -n {{namespace}}"
    description: "Deploy application resources"
    confirmation: prompt  # Require user confirmation
    risk_level: moderate

  - id: wait-rollout
    name: "Wait for rollout"
    command: "kubectl rollout status deployment/{{app_name}} -n {{namespace}} --timeout=300s"
    description: "Wait for deployment to complete"
    confirmation: auto
    timeout: 300

# Post-execution verification
verification:
  - check: "kubectl get pods -n {{namespace}} -l app={{app_name}} -o json"
    expect:
      json_path: ".items[*].status.phase"
      contains: "Running"
    message: "Pods should be in Running state"

# Rollback procedure (if verification fails)
rollback:
  - command: "kubectl rollout undo deployment/{{app_name}} -n {{namespace}}"
    description: "Rollback to previous version"
```

## Runtime Contract

### Skill API Interface

```rust
// src/skills/contract.rs

/// Describes what a skill provides
pub struct SkillDescriptor {
    pub id: String,
    pub name: String,
    pub version: semver::Version,
    pub api_version: semver::Version,
    pub provides: SkillProvides,
    pub capabilities: CapabilityRequest,
}

/// What capabilities a skill provides
pub struct SkillProvides {
    pub knowledge: bool,
    pub recipes: bool,
    pub executable: bool,
}

/// Context augmentation from knowledge packs
pub struct ContextAugmentation {
    /// Markdown content to inject into system prompt
    pub system_context: String,
    /// Maximum tokens this augmentation should use
    pub max_tokens: usize,
    /// Priority for ordering (higher = first)
    pub priority: u32,
}

/// A recipe that can be executed
pub struct Recipe {
    pub id: String,
    pub name: String,
    pub description: String,
    pub parameters: Vec<RecipeParameter>,
    pub preconditions: Vec<Precondition>,
    pub steps: Vec<RecipeStep>,
    pub verification: Vec<Verification>,
    pub rollback: Vec<RollbackStep>,
}

/// Result of recipe execution
pub enum RecipeResult {
    Success {
        steps_completed: usize,
        outputs: Vec<StepOutput>,
    },
    PartialSuccess {
        steps_completed: usize,
        failed_step: String,
        error: String,
    },
    Failed {
        step: String,
        error: String,
        rolled_back: bool,
    },
}

/// Interface for executable skills (WASM)
pub trait SkillExecutable: Send + Sync {
    /// Initialize the skill with configuration
    fn init(&mut self, config: &SkillConfig) -> Result<(), SkillError>;

    /// Discover available tools in environment
    fn discover_tools(&self, env: &EnvironmentContext) -> Result<Vec<ToolInfo>, SkillError>;

    /// Parse command output for structured data
    fn parse_output(&self, command: &str, output: &str) -> Result<ParsedOutput, SkillError>;

    /// Plan a workflow for a given intent
    fn plan_workflow(&self, intent: &str, context: &ExecutionContext)
        -> Result<WorkflowPlan, SkillError>;

    /// Shutdown and cleanup
    fn shutdown(&mut self) -> Result<(), SkillError>;
}
```

### Context Injection Flow

```
┌─────────────────────────────────────────────────────────────────┐
│                    USER REQUEST                                 │
│  "deploy my app to production EKS cluster"                      │
└─────────────────────────────────────────────────────────────────┘
                               │
                               ▼
┌─────────────────────────────────────────────────────────────────┐
│                   SKILL MATCHING                                │
│  1. Extract topics: [deploy, app, production, eks, cluster]     │
│  2. Match skills: [cloud.aws, tool.kubernetes]                  │
│  3. Sort by priority: cloud.aws (100) > tool.kubernetes (80)    │
└─────────────────────────────────────────────────────────────────┘
                               │
                               ▼
┌─────────────────────────────────────────────────────────────────┐
│                 CONTEXT AUGMENTATION                            │
│  1. Load knowledge/prompts/context.md from matched skills       │
│  2. Load relevant patterns (knowledge/patterns/deployment.md)   │
│  3. Inject into system prompt (respecting max_context_tokens)   │
│  4. Add available recipes to context                            │
└─────────────────────────────────────────────────────────────────┘
                               │
                               ▼
┌─────────────────────────────────────────────────────────────────┐
│                   LLM INFERENCE                                 │
│  System prompt now includes:                                    │
│  - AWS/EKS domain knowledge                                     │
│  - Kubernetes deployment patterns                               │
│  - Available recipes (deploy-app, scale-service, rollback)      │
│  - Safety constraints for cloud operations                      │
└─────────────────────────────────────────────────────────────────┘
                               │
                               ▼
┌─────────────────────────────────────────────────────────────────┐
│                  RECIPE MATCHING                                │
│  If LLM suggests recipe or user requests one:                   │
│  1. Match recipe by id or description                           │
│  2. Validate preconditions                                      │
│  3. Prompt for parameters                                       │
│  4. Execute with confirmation gates                             │
└─────────────────────────────────────────────────────────────────┘
```

## Capability Model

### Permission Buckets

| Capability | Description | Risk Level | Default |
|------------|-------------|------------|---------|
| `terminal_exec` | Execute shell commands | High | Denied |
| `filesystem_read` | Read files/directories | Medium | Scoped |
| `filesystem_write` | Write files/directories | High | Scoped |
| `network` | HTTP/HTTPS requests | Medium | Scoped |
| `env_read` | Read environment variables | Low | Scoped |
| `secrets_access` | Access secret store | Critical | Denied |

### Capability Enforcement

```rust
// src/skills/capability.rs

pub struct CapabilityEnforcer {
    granted: GrantedCapabilities,
    audit_log: AuditLog,
}

impl CapabilityEnforcer {
    /// Check if skill can execute a command
    pub fn can_execute(&self, skill_id: &str, command: &str) -> CapabilityResult {
        let caps = self.granted.get(skill_id)?;

        // Check if terminal_exec is granted
        if !caps.terminal_exec.allowed {
            return CapabilityResult::Denied("terminal_exec not granted");
        }

        // Check command against allowlist/denylist
        let cmd_name = parse_command_name(command);

        if !caps.terminal_exec.blocked.is_empty()
            && caps.terminal_exec.blocked.contains(&cmd_name) {
            return CapabilityResult::Denied("command in denylist");
        }

        if !caps.terminal_exec.commands.is_empty()
            && !caps.terminal_exec.commands.contains(&cmd_name) {
            return CapabilityResult::Denied("command not in allowlist");
        }

        // Log the capability check
        self.audit_log.record(CapabilityCheck {
            skill_id: skill_id.to_string(),
            capability: "terminal_exec",
            resource: command.to_string(),
            result: "granted",
            timestamp: Utc::now(),
        });

        CapabilityResult::Granted
    }
}
```

### Audit Trail

All capability exercises are logged:

```json
{
  "timestamp": "2025-12-31T10:30:00Z",
  "skill_id": "cloud.aws",
  "capability": "terminal_exec",
  "resource": "aws s3 ls",
  "result": "granted",
  "user_confirmed": true,
  "execution_result": "success"
}
```

## Distribution Model

### Supported Sources

| Source | Format | Use Case | Air-Gap Support |
|--------|--------|----------|-----------------|
| Local Path | Directory | Development | Yes |
| Git URL | Repository | Community | Cache required |
| Tarball URL | .tar.gz/.tgz | Internal | Yes (downloaded) |
| OCI Registry | Container image | Enterprise | Mirror support |

### Resolution Flow

```
caro skill add cloud.aws

1. Parse source specification:
   - No scheme → check built-in registry
   - file:// → local path
   - git+https:// → git repository
   - https://*.tar.gz → tarball
   - oci:// → container registry

2. Resolve to concrete source:
   - Registry lookup: cloud.aws → git+https://github.com/caro-skills/cloud-aws

3. Fetch from source:
   - Clone/download to temp directory
   - Verify integrity (checksum/signature)

4. Validate manifest:
   - Parse skill.toml
   - Check api_version compatibility
   - Check min_caro_version

5. Install to skills directory:
   - Copy to ~/.caro/skills/cloud.aws/
   - Update skills.lock

6. Capability prompt:
   - Display requested capabilities
   - Require user confirmation for sensitive caps
   - Save granted capabilities
```

### Lockfile Format

```toml
# ~/.caro/skills.lock

version = 1

[[skills]]
id = "cloud.aws"
version = "0.3.0"
source = "git+https://github.com/caro-skills/cloud-aws#v0.3.0"
integrity = "sha256:abc123..."
installed = "2025-12-31T10:00:00Z"

[skills.capabilities]
terminal_exec = { granted = true, commands = ["aws", "eksctl"] }
filesystem_read = { granted = true, paths = ["~/.aws"] }
network = { granted = true, domains = ["*.amazonaws.com"] }

[[skills]]
id = "tool.kubernetes"
version = "0.2.0"
source = "file:///home/user/my-skills/kubernetes"
integrity = "sha256:def456..."
installed = "2025-12-31T09:00:00Z"
```

## CLI Commands

See `contracts/cli-interface.md` for complete CLI specification.

### Quick Reference

```bash
# Skill management
caro skill list                    # List installed skills
caro skill add cloud.aws           # Add from registry
caro skill add --git <url>         # Add from git
caro skill add --path ./my-skill   # Add from local path
caro skill remove cloud.aws        # Remove skill
caro skill update                  # Update all skills
caro skill update cloud.aws        # Update specific skill

# Skill information
caro skill info cloud.aws          # Show skill details
caro skill capabilities cloud.aws  # Show granted capabilities

# Skill development
caro skill init my-skill           # Create new skill scaffold
caro skill validate                # Validate current skill
caro skill test                    # Run skill tests
caro skill pack                    # Create distribution package

# Runtime control
caro skill enable cloud.aws        # Enable skill
caro skill disable cloud.aws       # Disable without removing
caro --skills=none "command"       # Run without skills
caro --skills=cloud.aws "command"  # Run with specific skill(s)
```

## Security Considerations

### Threat Model

1. **Malicious Skills**: Skill attempts to exfiltrate data or damage system
   - **Mitigation**: Capability enforcement, code review for registry, signature verification

2. **Supply Chain Attack**: Legitimate skill is compromised
   - **Mitigation**: Lockfile with integrity hashes, update review, signed releases

3. **Capability Escalation**: Skill requests more than needed
   - **Mitigation**: Least-privilege prompting, capability audit, periodic review

4. **WASM Sandbox Escape**: Executable module exploits runtime
   - **Mitigation**: Use audited WASM runtime (wasmtime), limit host functions

### Security Best Practices

1. **Principle of Least Privilege**: Request only needed capabilities
2. **Transparency**: Document what each capability is used for
3. **Audit Trail**: Log all capability exercises
4. **Periodic Review**: Users prompted to review capabilities periodically
5. **Revocation**: Easy capability revocation without removing skill

## Testing Strategy

### Skill Testing

```yaml
# tests/recipe_test.yaml

test_cases:
  - name: "Deploy app with valid inputs"
    recipe: deploy-app
    parameters:
      cluster_name: "test-cluster"
      image_tag: "v1.0.0"
    mock_commands:
      - command: "aws eks update-kubeconfig*"
        exit_code: 0
        stdout: "Updated context"
      - command: "kubectl cluster-info"
        exit_code: 0
        stdout: "Kubernetes control plane is running"
    expected_result: success

  - name: "Deploy fails when not authenticated"
    recipe: deploy-app
    parameters:
      cluster_name: "test-cluster"
    mock_commands:
      - command: "aws sts get-caller-identity"
        exit_code: 1
        stderr: "Unable to locate credentials"
    expected_result: precondition_failed
    expected_message: "Must be authenticated to AWS"
```

### Knowledge Testing

```markdown
<!-- tests/knowledge_test.md -->

# Knowledge Pack Test Cases

## Test: Context injection for S3 topic
- **Input topic**: "s3"
- **Expected files loaded**:
  - knowledge/concepts/s3.md
  - knowledge/patterns/storage.md
- **Expected context contains**:
  - "S3 bucket"
  - "aws s3"
  - Object storage concepts

## Test: No context for unrelated topic
- **Input topic**: "docker"
- **Expected files loaded**: (none)
- **Expected context**: (empty)
```

## Integration Points

### With Existing Systems

1. **Backend System**: Skills add context to `CommandRequest`
2. **Safety System**: Recipes integrate with `SafetyValidator`
3. **Agent Loop**: Skills can influence refinement iterations
4. **Configuration**: Skills respect `UserConfiguration`
5. **Cache System**: Skills use similar caching patterns

### Extension Points for Enterprise

1. **Policy Override**: Enterprise can mandate/block skills
2. **Audit Integration**: Skill actions flow to enterprise audit
3. **Centralized Registry**: Enterprise can host private registry
4. **Capability Governance**: Enterprise can pre-approve capabilities

## Migration Path

### From Current System

1. **Phase 1**: Extract built-in knowledge to `core.shell` skill
2. **Phase 2**: Add skill loader with local path support
3. **Phase 3**: Add git/tarball source support
4. **Phase 4**: Add recipe execution
5. **Phase 5**: Add WASM execution (experimental)

### Backward Compatibility

- Caro without skills works exactly as today
- Skills are additive, never breaking
- Old configs continue to work
- Gradual adoption path
