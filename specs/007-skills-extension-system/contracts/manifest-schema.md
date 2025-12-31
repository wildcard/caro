# Skill Manifest Schema Contract

**Version**: 1.0.0
**Last Updated**: 2025-12-31

## Overview

This document specifies the complete schema for `skill.toml` manifest files. All skills MUST include a valid manifest.

## Schema Definition

### Root Structure

```toml
[skill]           # Required: Skill metadata
[provides]        # Required: What the skill provides
[knowledge]       # Optional: Knowledge pack configuration
[recipes]         # Optional: Recipe configuration
[executable]      # Optional: WASM module configuration
[capabilities]    # Optional: Requested permissions
[dependencies]    # Optional: Required skills
[suggestions]     # Optional: Suggested skills
[features]        # Optional: Optional features
[platform]        # Optional: Platform requirements
```

---

## [skill] Section

**Required**. Core skill metadata.

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `id` | string | Yes | Unique skill identifier |
| `name` | string | Yes | Human-readable name |
| `version` | string | Yes | Semantic version (X.Y.Z) |
| `description` | string | Yes | Brief description |
| `api_version` | string | Yes | Manifest API version |
| `authors` | string[] | No | Author list |
| `license` | string | No | SPDX license identifier |
| `repository` | string | No | Repository URL |
| `documentation` | string | No | Documentation URL |
| `keywords` | string[] | No | Search keywords |
| `min_caro_version` | string | No | Minimum caro version |

### Skill ID Format

```
<namespace>.<name>

Namespaces:
  core.*      - Built-in skills (reserved)
  cloud.*     - Cloud provider skills
  tool.*      - CLI tool skills
  lang.*      - Programming language skills
  platform.*  - Platform/service skills
  custom.*    - Custom/private skills
```

**Examples**:
- `cloud.aws`
- `tool.kubernetes`
- `lang.rust`
- `platform.github`
- `custom.corp-internal`

### Version Format

Semantic versioning: `MAJOR.MINOR.PATCH`

- **MAJOR**: Breaking changes to skill behavior
- **MINOR**: New features, backward compatible
- **PATCH**: Bug fixes, no behavior change

### API Version

Current: `1.0`

Skills MUST specify the API version they're built for. Caro will:
- Load skills with matching major version
- Warn on minor version mismatch
- Reject incompatible major versions

---

## [provides] Section

**Required**. Declares what the skill contains.

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `knowledge` | bool | No | false | Has knowledge/ directory |
| `recipes` | bool | No | false | Has recipes/ directory |
| `executable` | bool | No | false | Has WASM module |

```toml
[provides]
knowledge = true
recipes = true
executable = false
```

---

## [knowledge] Section

**Optional**. Configuration for knowledge packs.

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `topics` | string[] | No | [] | Topics this skill enhances |
| `priority` | u32 | No | 50 | Injection priority (higher = first) |
| `max_context_tokens` | u32 | No | 1000 | Maximum tokens to inject |

```toml
[knowledge]
topics = ["aws", "s3", "ec2", "lambda"]
priority = 100
max_context_tokens = 2000
```

### Topic Matching

Topics are matched against user prompts using:
1. Exact keyword match
2. Fuzzy matching (levenshtein distance ≤ 2)
3. Synonym expansion (configured per skill)

---

## [recipes] Section

**Optional**. Configuration for recipe workflows.

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `files` | string[] | No | `["recipes/*.yaml"]` | Recipe file globs |
| `default_confirmation` | string | No | `"prompt"` | Default confirmation level |

```toml
[recipes]
files = ["recipes/*.yaml", "workflows/*.yaml"]
default_confirmation = "prompt"  # "auto" | "prompt" | "always"
```

### Confirmation Levels

| Level | Behavior |
|-------|----------|
| `auto` | Execute without confirmation |
| `prompt` | Prompt user before execution |
| `always` | Always require explicit confirmation |

---

## [executable] Section

**Optional**. WASM module configuration.

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `module` | string | Yes* | - | Path to WASM module |
| `exports` | string[] | No | [] | Exported function names |

*Required if `provides.executable = true`

```toml
[executable]
module = "bin/skill.wasm"
exports = ["discover_tools", "parse_output", "plan_workflow"]
```

### WASM Interface

Modules must export functions matching the skill contract:

```rust
// Required exports
#[no_mangle]
pub extern "C" fn init(config_ptr: *const u8, config_len: u32) -> i32;

#[no_mangle]
pub extern "C" fn shutdown() -> i32;

// Optional exports
#[no_mangle]
pub extern "C" fn discover_tools(env_ptr: *const u8, env_len: u32,
                                  out_ptr: *mut u8, out_len: u32) -> i32;

#[no_mangle]
pub extern "C" fn parse_output(cmd_ptr: *const u8, cmd_len: u32,
                                output_ptr: *const u8, output_len: u32,
                                out_ptr: *mut u8, out_len: u32) -> i32;

#[no_mangle]
pub extern "C" fn plan_workflow(intent_ptr: *const u8, intent_len: u32,
                                 ctx_ptr: *const u8, ctx_len: u32,
                                 out_ptr: *mut u8, out_len: u32) -> i32;
```

---

## [capabilities] Section

**Optional**. Requested permissions.

### terminal_exec

Permission to execute shell commands.

```toml
[capabilities.terminal_exec]
allowed = true
commands = ["aws", "kubectl"]     # Allowlist (empty = all)
blocked = ["rm", "dd", "mkfs"]    # Denylist
```

### filesystem_read

Permission to read files and directories.

```toml
[capabilities]
filesystem_read = [
    "~/.aws",
    "~/.kube",
    "./",
    "$CARO_PROJECT_ROOT"
]
```

**Path Variables**:
| Variable | Expansion |
|----------|-----------|
| `~` | User home directory |
| `./` | Current working directory |
| `$CARO_PROJECT_ROOT` | Detected project root |
| `$CARO_CACHE` | Caro cache directory |

### filesystem_write

Permission to write files and directories.

```toml
[capabilities]
filesystem_write = [
    "./.caro-cache/skill-name"
]
```

### network

Permission to make network requests.

```toml
[capabilities]
network = [
    "*.amazonaws.com",
    "api.github.com",
    "https://internal.corp:8443"
]
```

**Domain Patterns**:
- `*.example.com` - Any subdomain
- `example.com` - Exact domain only
- `https://example.com:8443` - Specific port

### env_read

Permission to read environment variables.

```toml
[capabilities]
env_read = [
    "AWS_*",
    "KUBECONFIG",
    "HOME"
]
```

**Pattern Syntax**:
- `VAR` - Exact variable
- `PREFIX_*` - Prefix match

### secrets_access

Permission to access secret storage.

```toml
[capabilities]
secrets_access = false  # Default: false
```

**Note**: This is a high-risk capability. When `true`, the skill can access:
- System keychain (macOS Keychain, GNOME Keyring)
- Caro's encrypted secret store

---

## [dependencies] Section

**Optional**. Required skills.

```toml
[dependencies]
"core.shell" = ">=1.0"
"tool.kubectl" = { version = ">=0.2", optional = true }
"cloud.aws" = { version = ">=0.3", features = ["eks"] }
```

### Dependency Specification

| Format | Meaning |
|--------|---------|
| `">=1.0"` | Version requirement |
| `{ version = "..." }` | Extended specification |
| `{ version = "...", optional = true }` | Optional dependency |
| `{ version = "...", features = [...] }` | With feature flags |

### Version Requirements

Uses semver-like syntax:
- `>=1.0` - At least version 1.0
- `<2.0` - Less than version 2.0
- `>=1.0,<2.0` - Range
- `1.0` - Exact version
- `*` - Any version

---

## [suggestions] Section

**Optional**. Suggested (not required) skills.

```toml
[suggestions]
"tool.terraform" = "Works great with Terraform for IaC"
"tool.docker" = "Useful for ECR container workflows"
```

Shown to users during installation.

---

## [features] Section

**Optional**. Optional skill features.

```toml
[features]
eks = { description = "EKS cluster management", default = true }
lambda = { description = "Lambda function workflows", default = true }
iam = { description = "IAM policy management", default = false }
```

### Feature Specification

| Field | Type | Required | Default |
|-------|------|----------|---------|
| `description` | string | Yes | - |
| `default` | bool | No | true |

Features can be enabled/disabled at install time:

```bash
caro skill add cloud.aws --features=eks,lambda --no-default-features
```

---

## [platform] Section

**Optional**. Platform requirements.

```toml
[platform]
os = ["linux", "macos"]           # Supported OS (empty = all)
arch = ["aarch64", "x86_64"]      # Supported architectures
required_tools = ["aws"]          # Must be installed
optional_tools = ["eksctl", "sam"] # Nice to have
```

### OS Values

- `linux`
- `macos`
- `windows`

### Architecture Values

- `x86_64`
- `aarch64`
- `arm`

---

## Validation Rules

### Manifest Validation

1. **Required fields**: All required fields must be present
2. **ID format**: Must match `<namespace>.<name>` pattern
3. **Version format**: Must be valid semver
4. **API version**: Must be supported by current caro
5. **Path validation**: All paths must be within skill directory
6. **Capability format**: All capabilities must use valid syntax

### Structure Validation

1. If `provides.knowledge = true`, `knowledge/` must exist
2. If `provides.recipes = true`, `recipes/` must exist
3. If `provides.executable = true`, WASM module must exist
4. All referenced files must exist

### Recipe Validation

1. All recipes must have valid YAML syntax
2. Required recipe fields must be present
3. Parameter references must be valid
4. Step commands must be non-empty

---

## Complete Example

```toml
# skill.toml - Full Example

[skill]
id = "cloud.aws"
name = "AWS Cloud Skill"
version = "0.3.0"
description = "AWS workflows from terminal: awscli, sso, iam, cloudwatch, eks"
authors = ["caro-community <community@caro.dev>"]
license = "MIT"
repository = "https://github.com/caro-skills/cloud-aws"
documentation = "https://caro.dev/skills/cloud-aws"
keywords = ["aws", "cloud", "devops", "infrastructure"]
api_version = "1.0"
min_caro_version = "1.1.0"

[provides]
knowledge = true
recipes = true
executable = false

[knowledge]
topics = ["aws", "cloud", "s3", "ec2", "lambda", "eks", "iam", "cloudwatch"]
priority = 100
max_context_tokens = 2000

[recipes]
files = ["recipes/*.yaml"]
default_confirmation = "prompt"

[capabilities.terminal_exec]
allowed = true
commands = ["aws", "eksctl", "kubectl"]
blocked = ["rm", "dd", "mkfs"]

[capabilities]
filesystem_read = ["~/.aws", "~/.kube", "./"]
filesystem_write = ["./.caro-cache/aws"]
network = ["*.amazonaws.com"]
env_read = ["AWS_*", "KUBECONFIG"]
secrets_access = false

[dependencies]
"core.shell" = ">=1.0"

[dependencies."tool.kubectl"]
version = ">=0.2"
optional = true

[suggestions]
"tool.terraform" = "Works great with Terraform for IaC"
"tool.docker" = "Useful for ECR container workflows"

[features]
eks = { description = "EKS cluster management", default = true }
lambda = { description = "Lambda function workflows", default = true }
iam = { description = "IAM policy management", default = true }

[platform]
os = []
arch = []
required_tools = ["aws"]
optional_tools = ["eksctl", "sam", "cdk"]
```

---

## Rust Type Definitions

```rust
// src/skills/manifest.rs

use semver::Version;
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Deserialize, Serialize)]
pub struct SkillManifest {
    pub skill: SkillMetadata,
    pub provides: SkillProvides,
    #[serde(default)]
    pub knowledge: Option<KnowledgeConfig>,
    #[serde(default)]
    pub recipes: Option<RecipesConfig>,
    #[serde(default)]
    pub executable: Option<ExecutableConfig>,
    #[serde(default)]
    pub capabilities: CapabilityRequest,
    #[serde(default)]
    pub dependencies: HashMap<String, Dependency>,
    #[serde(default)]
    pub suggestions: HashMap<String, String>,
    #[serde(default)]
    pub features: HashMap<String, FeatureConfig>,
    #[serde(default)]
    pub platform: Option<PlatformConfig>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SkillMetadata {
    pub id: String,
    pub name: String,
    pub version: Version,
    pub description: String,
    pub api_version: String,
    #[serde(default)]
    pub authors: Vec<String>,
    pub license: Option<String>,
    pub repository: Option<String>,
    pub documentation: Option<String>,
    #[serde(default)]
    pub keywords: Vec<String>,
    pub min_caro_version: Option<Version>,
}

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct SkillProvides {
    #[serde(default)]
    pub knowledge: bool,
    #[serde(default)]
    pub recipes: bool,
    #[serde(default)]
    pub executable: bool,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct KnowledgeConfig {
    #[serde(default)]
    pub topics: Vec<String>,
    #[serde(default = "default_priority")]
    pub priority: u32,
    #[serde(default = "default_max_tokens")]
    pub max_context_tokens: u32,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RecipesConfig {
    #[serde(default = "default_recipe_files")]
    pub files: Vec<String>,
    #[serde(default)]
    pub default_confirmation: ConfirmationLevel,
}

#[derive(Debug, Deserialize, Serialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ConfirmationLevel {
    Auto,
    #[default]
    Prompt,
    Always,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ExecutableConfig {
    pub module: PathBuf,
    #[serde(default)]
    pub exports: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct CapabilityRequest {
    #[serde(default)]
    pub terminal_exec: Option<TerminalExecCapability>,
    #[serde(default)]
    pub filesystem_read: Vec<String>,
    #[serde(default)]
    pub filesystem_write: Vec<String>,
    #[serde(default)]
    pub network: Vec<String>,
    #[serde(default)]
    pub env_read: Vec<String>,
    #[serde(default)]
    pub secrets_access: bool,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TerminalExecCapability {
    #[serde(default = "default_true")]
    pub allowed: bool,
    #[serde(default)]
    pub commands: Vec<String>,
    #[serde(default)]
    pub blocked: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Dependency {
    Simple(String),
    Extended {
        version: String,
        #[serde(default)]
        optional: bool,
        #[serde(default)]
        features: Vec<String>,
    },
}

#[derive(Debug, Deserialize, Serialize)]
pub struct FeatureConfig {
    pub description: String,
    #[serde(default = "default_true")]
    pub default: bool,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PlatformConfig {
    #[serde(default)]
    pub os: Vec<String>,
    #[serde(default)]
    pub arch: Vec<String>,
    #[serde(default)]
    pub required_tools: Vec<String>,
    #[serde(default)]
    pub optional_tools: Vec<String>,
}

fn default_priority() -> u32 { 50 }
fn default_max_tokens() -> u32 { 1000 }
fn default_recipe_files() -> Vec<String> { vec!["recipes/*.yaml".to_string()] }
fn default_true() -> bool { true }
```
