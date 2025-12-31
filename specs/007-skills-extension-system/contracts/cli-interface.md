# Skill CLI Interface Contract

**Version**: 1.0.0
**Last Updated**: 2025-12-31

## Overview

This document specifies the CLI commands for managing caro skills. All commands are subcommands of `caro skill`.

## Command Structure

```
caro skill <subcommand> [options] [arguments]
```

## Commands

### `caro skill list`

List all installed skills.

**Usage**:
```bash
caro skill list [--format <format>] [--enabled-only] [--verbose]
```

**Options**:
| Option | Short | Default | Description |
|--------|-------|---------|-------------|
| `--format` | `-f` | `table` | Output format: `table`, `json`, `yaml` |
| `--enabled-only` | `-e` | false | Show only enabled skills |
| `--verbose` | `-v` | false | Show additional details |

**Output (table)**:
```
ID              VERSION  STATUS   PROVIDES           CAPABILITIES
cloud.aws       0.3.0    enabled  knowledge,recipes  terminal,network
tool.kubernetes 0.2.0    enabled  knowledge,recipes  terminal,filesystem
lang.python     0.1.0    disabled knowledge          none
```

**Output (json)**:
```json
{
  "skills": [
    {
      "id": "cloud.aws",
      "version": "0.3.0",
      "status": "enabled",
      "provides": ["knowledge", "recipes"],
      "capabilities": ["terminal_exec", "network"]
    }
  ]
}
```

---

### `caro skill add`

Install a skill from various sources.

**Usage**:
```bash
caro skill add <skill-id-or-source> [options]
```

**Arguments**:
| Argument | Description |
|----------|-------------|
| `<skill-id-or-source>` | Skill ID from registry, or source specification |

**Options**:
| Option | Short | Default | Description |
|--------|-------|---------|-------------|
| `--path` | `-p` | - | Install from local directory |
| `--git` | `-g` | - | Install from git repository |
| `--url` | `-u` | - | Install from tarball URL |
| `--oci` | - | - | Install from OCI registry |
| `--ref` | `-r` | `HEAD` | Git ref (branch, tag, commit) |
| `--no-enable` | - | false | Install but don't enable |
| `--force` | `-f` | false | Overwrite existing installation |
| `--yes` | `-y` | false | Accept all capability prompts |

**Examples**:
```bash
# From registry (default)
caro skill add cloud.aws

# From local path
caro skill add --path ./my-skill

# From git with specific tag
caro skill add --git https://github.com/caro-skills/cloud-aws --ref v0.3.0

# From tarball
caro skill add --url https://internal.corp/skills/cloud-aws-0.3.0.tgz

# From OCI registry
caro skill add --oci registry.corp.local/caro/skills/cloud-aws:0.3.0
```

**Capability Prompt**:
```
Installing cloud.aws v0.3.0...

This skill requests the following capabilities:

  Terminal execution:
    Allowed commands: aws, eksctl, kubectl
    Blocked commands: rm, dd, mkfs

  Filesystem read:
    ~/.aws
    ~/.kube
    ./

  Network access:
    *.amazonaws.com

Grant these capabilities? [y/N/d(etails)]
```

**Exit Codes**:
| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | Skill not found |
| 2 | Version conflict |
| 3 | Capability denied |
| 4 | Network error |
| 5 | Validation error |

---

### `caro skill remove`

Uninstall a skill.

**Usage**:
```bash
caro skill remove <skill-id> [options]
```

**Options**:
| Option | Short | Default | Description |
|--------|-------|---------|-------------|
| `--force` | `-f` | false | Remove without confirmation |
| `--keep-config` | - | false | Keep skill configuration |

**Examples**:
```bash
caro skill remove cloud.aws
caro skill remove cloud.aws --force
```

---

### `caro skill update`

Update installed skills.

**Usage**:
```bash
caro skill update [skill-id] [options]
```

**Arguments**:
| Argument | Description |
|----------|-------------|
| `[skill-id]` | Specific skill to update (optional, updates all if omitted) |

**Options**:
| Option | Short | Default | Description |
|--------|-------|---------|-------------|
| `--check` | `-c` | false | Check for updates without installing |
| `--yes` | `-y` | false | Accept all updates |
| `--major` | - | false | Allow major version updates |

**Examples**:
```bash
# Update all skills
caro skill update

# Update specific skill
caro skill update cloud.aws

# Check for updates only
caro skill update --check
```

**Output**:
```
Checking for updates...

SKILL           CURRENT  AVAILABLE  ACTION
cloud.aws       0.3.0    0.4.0      update available
tool.kubernetes 0.2.0    0.2.0      up to date
lang.python     0.1.0    0.2.0      update available (major)

Update cloud.aws and lang.python? [y/N]
```

---

### `caro skill info`

Show detailed information about a skill.

**Usage**:
```bash
caro skill info <skill-id> [options]
```

**Options**:
| Option | Short | Default | Description |
|--------|-------|---------|-------------|
| `--format` | `-f` | `text` | Output format: `text`, `json`, `yaml` |
| `--remote` | `-r` | false | Show info from registry (not installed) |

**Output**:
```
Skill: cloud.aws
Name: AWS Cloud Skill
Version: 0.3.0
Status: enabled
Source: git+https://github.com/caro-skills/cloud-aws#v0.3.0
Installed: 2025-12-31T10:00:00Z

Description:
  AWS workflows from terminal: awscli, sso, iam, cloudwatch, eks,
  terraform patterns.

Provides:
  - Knowledge: 12 documents, 8 prompts
  - Recipes: 5 workflows
  - Executable: none

Topics:
  aws, cloud, s3, ec2, lambda, eks, iam, cloudwatch

Dependencies:
  - core.shell (>=1.0) [satisfied]
  - tool.kubectl (>=0.2) [optional, not installed]

Capabilities (granted):
  - terminal_exec: aws, eksctl, kubectl
  - filesystem_read: ~/.aws, ~/.kube
  - network: *.amazonaws.com
```

---

### `caro skill capabilities`

Manage skill capabilities.

**Usage**:
```bash
caro skill capabilities <skill-id> [options]
caro skill capabilities <skill-id> grant <capability>
caro skill capabilities <skill-id> revoke <capability>
```

**Subcommands**:
| Subcommand | Description |
|------------|-------------|
| (none) | Show current capabilities |
| `grant` | Grant a specific capability |
| `revoke` | Revoke a specific capability |

**Examples**:
```bash
# Show capabilities
caro skill capabilities cloud.aws

# Grant additional capability
caro skill capabilities cloud.aws grant filesystem_write

# Revoke capability
caro skill capabilities cloud.aws revoke network
```

**Output**:
```
Capabilities for cloud.aws:

CAPABILITY       STATUS    SCOPE
terminal_exec    granted   aws, eksctl, kubectl
filesystem_read  granted   ~/.aws, ~/.kube, ./
filesystem_write denied    (not requested)
network          granted   *.amazonaws.com
env_read         granted   AWS_*, KUBECONFIG
secrets_access   denied    (not requested)

Requested but denied:
  (none)

To modify: caro skill capabilities cloud.aws grant/revoke <capability>
```

---

### `caro skill enable`

Enable a disabled skill.

**Usage**:
```bash
caro skill enable <skill-id>
```

---

### `caro skill disable`

Disable a skill without removing it.

**Usage**:
```bash
caro skill disable <skill-id>
```

---

### `caro skill init`

Create a new skill scaffold.

**Usage**:
```bash
caro skill init <skill-id> [options]
```

**Options**:
| Option | Short | Default | Description |
|--------|-------|---------|-------------|
| `--type` | `-t` | `knowledge` | Skill type: `knowledge`, `recipe`, `full` |
| `--dir` | `-d` | `./<skill-id>` | Output directory |

**Examples**:
```bash
# Create knowledge-only skill
caro skill init my-tool

# Create full skill with recipes
caro skill init my-tool --type recipe

# Create in specific directory
caro skill init my-tool --dir ./skills/my-tool
```

**Generated Structure**:
```
my-tool/
├── skill.toml
├── README.md
├── LICENSE
├── knowledge/
│   ├── overview.md
│   └── prompts/
│       └── context.md
├── recipes/              # (if --type=recipe or full)
│   └── example.yaml
└── tests/
    └── knowledge_test.md
```

---

### `caro skill validate`

Validate a skill manifest and structure.

**Usage**:
```bash
caro skill validate [path] [options]
```

**Arguments**:
| Argument | Description |
|----------|-------------|
| `[path]` | Path to skill directory (default: current directory) |

**Options**:
| Option | Short | Default | Description |
|--------|-------|---------|-------------|
| `--strict` | `-s` | false | Fail on warnings |
| `--format` | `-f` | `text` | Output format: `text`, `json` |

**Output**:
```
Validating skill at ./my-skill...

  Manifest (skill.toml)
  - id: valid
  - version: valid (0.1.0)
  - api_version: valid (1.0)
  - provides: valid

  Structure
  - knowledge/: found (3 files)
  - recipes/: found (2 files)
  - tests/: found (1 file)

  Recipes
  - recipes/deploy.yaml: valid
  - recipes/rollback.yaml: valid

  Warnings:
  - README.md not found (recommended)
  - LICENSE not found (required for distribution)

Validation: PASSED with 2 warnings
```

---

### `caro skill test`

Run skill tests.

**Usage**:
```bash
caro skill test [path] [options]
```

**Options**:
| Option | Short | Default | Description |
|--------|-------|---------|-------------|
| `--filter` | `-f` | - | Run only matching tests |
| `--verbose` | `-v` | false | Show detailed output |

**Output**:
```
Running skill tests...

  Knowledge tests (tests/knowledge_test.md)
  - Context injection for S3 topic: PASSED
  - No context for unrelated topic: PASSED

  Recipe tests (tests/recipe_test.yaml)
  - Deploy app with valid inputs: PASSED
  - Deploy fails when not authenticated: PASSED

4 tests passed, 0 failed
```

---

### `caro skill pack`

Create a distributable package.

**Usage**:
```bash
caro skill pack [path] [options]
```

**Options**:
| Option | Short | Default | Description |
|--------|-------|---------|-------------|
| `--output` | `-o` | `./<id>-<version>.tgz` | Output path |
| `--sign` | `-s` | false | Sign package with GPG |

**Output**:
```
Packing skill: my-tool v0.1.0

  Including:
  - skill.toml
  - README.md
  - knowledge/ (3 files)
  - recipes/ (2 files)

  Excluding:
  - tests/
  - .git/

Created: my-tool-0.1.0.tgz (12KB)
SHA256: abc123...

To install: caro skill add --url file://./my-tool-0.1.0.tgz
```

---

### `caro skill search`

Search for skills in registries.

**Usage**:
```bash
caro skill search <query> [options]
```

**Options**:
| Option | Short | Default | Description |
|--------|-------|---------|-------------|
| `--registry` | `-r` | `default` | Registry to search |
| `--limit` | `-l` | `20` | Maximum results |

**Output**:
```
Searching for "aws"...

ID              VERSION  DESCRIPTION
cloud.aws       0.3.0    AWS workflows: awscli, sso, iam, eks...
cloud.aws-lite  0.1.0    Minimal AWS knowledge pack
tool.terraform  0.2.0    Terraform with AWS provider patterns

3 skills found. Install with: caro skill add <skill-id>
```

---

## Runtime Flags

Skills can be controlled at runtime without modifying configuration.

### `--skills`

Control which skills are active for a command.

**Usage**:
```bash
caro [--skills=<spec>] "<prompt>"
```

**Spec Formats**:
| Format | Meaning |
|--------|---------|
| `none` | Disable all skills |
| `all` | Enable all installed skills (default) |
| `skill1,skill2` | Enable only specified skills |
| `!skill1` | Enable all except specified |

**Examples**:
```bash
# Disable all skills
caro --skills=none "list files"

# Use only AWS skill
caro --skills=cloud.aws "create s3 bucket"

# Use AWS and Kubernetes skills
caro --skills=cloud.aws,tool.kubernetes "deploy to eks"

# Use all except Python skill
caro --skills=!lang.python "run script"
```

---

## Configuration

Skills can be configured in `~/.caro/config.toml`:

```toml
[skills]
# Default skills to enable
default = ["cloud.aws", "tool.kubernetes"]

# Auto-update settings
auto_update = false
update_check_interval = "7d"

# Capability defaults
[skills.capabilities]
# Always require confirmation for these
always_confirm = ["secrets_access"]

# Never grant these automatically
never_grant = ["filesystem_write"]

# Registry configuration
[skills.registry]
default = "https://skills.caro.dev"
mirrors = ["https://mirror.corp.local/caro-skills"]
```

---

## Error Messages

Standard error message formats:

```
Error: Skill 'cloud.gcp' not found

  The skill 'cloud.gcp' is not installed.

  To install: caro skill add cloud.gcp
  To search:  caro skill search gcp
```

```
Error: Capability denied

  The skill 'cloud.aws' requires 'terminal_exec' capability,
  which is blocked by your configuration.

  To grant: caro skill capabilities cloud.aws grant terminal_exec
  To configure: edit ~/.caro/config.toml [skills.capabilities]
```

```
Error: API version mismatch

  The skill 'old-skill' requires API version 0.5,
  but caro only supports version 1.0.

  Please update the skill or use an older caro version.
```
