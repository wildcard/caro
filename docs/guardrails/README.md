# cmdai Guardrails

This directory contains the complete catalog of cmdai's safety patterns that protect users from dangerous shell commands.

## Overview

cmdai implements **52+ safety patterns** organized into 9 categories. Each guardrail is documented in a YAML file with:
- Pattern regex and risk level
- Examples of blocked vs. safe commands
- Detailed explanations
- Community notes and usage statistics

## Categories

- **FilesystemDestruction** 🗑️ - `rm -rf /`, recursive deletions
- **DiskOperations** 💾 - `mkfs`, `dd`, disk formatting
- **PrivilegeEscalation** 🔐 - `sudo su`, setuid, root access
- **NetworkBackdoors** 🌐 - Netcat shells, reverse connections
- **ProcessManipulation** ⚙️ - Fork bombs, kill -9 -1
- **SystemModification** 🔧 - `/etc` writes, system file changes
- **EnvironmentManipulation** 🔤 - PATH override, alias hijacking
- **PackageManagement** 📦 - Force package removal, system package breakage
- **Containers** 🐳 - `--privileged` mode, container escapes

## Browse Guardrails

**By category:**
```bash
cmdai guardrails list --category filesystem
cmdai guardrails list --category privilege
```

**By risk level:**
```bash
cmdai guardrails list --risk critical
cmdai guardrails list --risk high
```

**Search:**
```bash
cmdai guardrails search "rm"
cmdai guardrails search "sudo"
cmdai guardrails search "network"
```

**View details:**
```bash
cmdai guardrails show grd-001
```

## Example Guardrails

### Critical Risk
- **grd-001**: `rm -rf /` - Recursive deletion of root directory
- **grd-014**: `curl url | bash` - Download and execute without inspection
- **grd-034**: `dd if=/dev/zero of=/dev/sda` - Overwrite disk with zeros

### High Risk
- **grd-025**: `sudo su` - Unrestricted root shell
- **grd-019**: `chmod 777 /` - Recursive permission change from root
- **grd-042**: `docker run --privileged` - Container with full host access

### Moderate Risk
- **grd-048**: `kill -9 -1` - Force kill all processes
- **grd-051**: `export PATH=` - Override PATH environment variable

## File Format

Each guardrail is a YAML file with this structure:

```yaml
id: "grd-XXX"
pattern:
  regex: "pattern_here"
  risk_level: Critical|High|Moderate
  description: "Short description"
  shell_specific: Bash|null

category: CategoryName

examples_blocked:
  - "dangerous command 1"
  - "dangerous command 2"

examples_safe:
  - "safe alternative 1"
  - "safe alternative 2"

explanation: |
  Multi-line explanation of why this is dangerous,
  what it does, and what could go wrong.

learn_more_url: "https://..."

tags:
  - tag1
  - tag2

community_notes:
  - author: "username"
    date: "2024-01-15T10:30:00Z"
    note: "Community insight"
    upvotes: 42

stats:
  times_triggered: 1247
  times_overridden: 3
  false_positive_reports: 0
  last_triggered: "2024-11-28T12:00:00Z"

created_at: "2024-01-01T00:00:00Z"
updated_at: "2024-11-28T00:00:00Z"
```

## Contributing

Want to propose a new guardrail? See [CONTRIBUTING.md](../../CONTRIBUTING.md) for guidelines.

**Before proposing:**
1. Search existing guardrails to avoid duplicates
2. Provide real-world examples of the danger
3. Include false positive testing
4. Document safe alternatives

**Proposal process:**
1. Create YAML file following format above
2. Test regex pattern thoroughly
3. Submit PR with clear description
4. Community review and discussion
5. Merge and auto-deploy

## Statistics

Track which guardrails are most important:

```bash
# Most triggered patterns
cmdai guardrails stats --sort triggers

# Patterns with high override rates (might need review)
cmdai guardrails stats --sort override-rate

# Patterns with false positives
cmdai guardrails stats --show-fp
```

## Testing

Test if a command would be blocked:

```bash
cmdai guardrails test "rm -rf /"
# Output: ✗ BLOCKED by grd-001 (Critical)

cmdai guardrails test "rm -rf ./temp"
# Output: ✓ SAFE
```

## Web Interface

Browse all guardrails at:
- **Web**: https://cmdai.dev/guardrails
- **CLI**: `cmdai guardrails list`

## Philosophy

cmdai's guardrails follow these principles:

1. **Transparency** - Users should understand what's blocked and why
2. **Education** - Explain dangers, teach safer alternatives
3. **Community-driven** - Safety rules evolve with user input
4. **Context-aware** - Distinguish between dangerous commands and safe documentation
5. **Reasonable defaults** - Block truly dangerous patterns, warn on risky ones

## Need to override?

If you have a legitimate use case for a blocked command:

```bash
# Learn why it's blocked first
cmdai guardrails show <id>

# Override with --override flag (requires confirmation)
cmdai --override "your dangerous command"

# Or adjust safety level
cmdai --safety permissive "command"
```

Remember: Guardrails are there to protect you. If you frequently need to override, consider whether there's a safer way to accomplish your goal.

## Questions?

- **Why is my command blocked?** - Run `cmdai guardrails search "your command"`
- **How do I disable a specific guardrail?** - See configuration docs
- **I found a false positive** - Report it: `cmdai guardrails report-fp <id>`
- **Suggest a new guardrail** - Submit PR or open issue with examples

---

**Last updated:** 2024-11-28
**Total guardrails:** 52+
**Community contributors:** 100+
