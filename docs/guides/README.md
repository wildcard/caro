# cmdai Community Guides

Welcome to the cmdai guides library - a curated collection of common developer tasks with natural language → command examples.

## Overview

Think of this as a living cookbook of terminal commands. Each guide shows:
- The natural language prompt you'd give cmdai
- The generated shell command
- Step-by-step explanations
- Safety notes and common pitfalls
- Community ratings and feedback

## Categories

### 🔀 Git (25 guides)
Version control operations, commit management, branch workflows
```bash
cmdai guides list --category git
```

### 🐳 Docker (18 guides)
Container management, image operations, cleanup tasks
```bash
cmdai guides list --category docker
```

### 📁 File Management (22 guides)
Finding files, disk usage, permissions, cleanup
```bash
cmdai guides list --category files
```

### 🌐 Networking (15 guides)
Port checking, connectivity testing, network debugging
```bash
cmdai guides list --category network
```

### ⚙️ System Administration (20 guides)
Service management, user operations, system monitoring
```bash
cmdai guides list --category system
```

### 💻 Development (16 guides)
Build tools, dependency management, environment setup
```bash
cmdai guides list --category dev
```

### 🗄️ Database (12 guides)
Database operations, backups, queries
```bash
cmdai guides list --category database
```

### ☸️ Kubernetes (10 guides)
Pod management, deployments, debugging
```bash
cmdai guides list --category kubernetes
```

### ☁️ Cloud (8 guides)
AWS, GCP, Azure CLI operations
```bash
cmdai guides list --category cloud
```

### 🔒 Security (11 guides)
Permissions, encryption, key management
```bash
cmdai guides list --category security
```

## Browse Guides

**List all guides:**
```bash
cmdai guides list
```

**By difficulty:**
```bash
cmdai guides list --difficulty beginner
cmdai guides list --difficulty intermediate
cmdai guides list --difficulty advanced
```

**By risk level:**
```bash
cmdai guides list --risk safe
cmdai guides list --risk moderate
cmdai guides list --risk high
```

**Search:**
```bash
cmdai guides search "undo commit"
cmdai guides search "docker cleanup"
cmdai guides search "find large files"
```

**View details:**
```bash
cmdai guides show guide-git-001
```

**Execute a guide:**
```bash
cmdai guides run guide-git-001
```

## Popular Guides

### Git
- **guide-git-001**: Undo last commit but keep changes
- **guide-git-007**: Squash last N commits
- **guide-git-015**: Amend commit message
- **guide-git-022**: Interactive rebase

### Docker
- **guide-docker-001**: Remove all stopped containers
- **guide-docker-005**: View container logs
- **guide-docker-012**: Clean up disk space
- **guide-docker-018**: Copy files from container

### Files
- **guide-files-001**: Find largest files in directory
- **guide-files-008**: Find files by name pattern
- **guide-files-014**: Change permissions recursively
- **guide-files-020**: Compress directory

## File Format

Each guide is a Markdown file with YAML frontmatter:

```markdown
---
id: "guide-XXX-###"
title: "Short descriptive title"
description: "One-line description"
category: CategoryName
difficulty: Beginner|Intermediate|Advanced
tags: [tag1, tag2, tag3]
natural_language_prompt: "what you'd ask cmdai"
generated_command: "the shell command"
shell_type: Bash|Zsh|Fish
risk_level: Safe|Moderate|High
author: "username"
created_at: "2024-01-10T10:00:00Z"
updated_at: "2024-11-28T14:30:00Z"
prerequisites:
  - "prerequisite 1"
  - "prerequisite 2"
expected_outcomes:
  - "outcome 1"
  - "outcome 2"
related_guides:
  - "guide-XXX-###"
related_guardrails:
  - "grd-###"
alternatives:
  - "alternative command 1"
  - "alternative command 2"
---

# Guide Title

## What it does
Brief explanation...

## When to use this
- ✅ Use case 1
- ✅ Use case 2
- ⚠️ Warning case

## The cmdai way
\`\`\`bash
cmdai "prompt"
\`\`\`

...rest of guide content...
```

## Difficulty Levels

**Beginner** 🟢
- Common tasks with simple commands
- Minimal risk of data loss
- Clear step-by-step instructions
- Examples: list files, create directory, basic git

**Intermediate** 🟡
- Multi-step operations
- Some risk if used incorrectly
- Requires understanding of concepts
- Examples: rebasing, Docker volumes, ssh keys

**Advanced** 🔴
- Complex workflows
- Potential for data loss or system issues
- Assumes deep knowledge
- Examples: kernel parameters, database migrations

## Risk Levels

**Safe** ✓
- Read-only operations
- No data modification
- No system changes
- Example: `ls`, `cat`, `git log`

**Moderate** ⚠️
- Modifies files/data
- Reversible with effort
- Could cause inconvenience
- Example: `git reset`, `docker rm`, `chmod`

**High** ✗
- Destructive operations
- Difficult/impossible to reverse
- Could cause data loss
- Example: `rm -rf`, `git push --force`, `DROP TABLE`

## Quality Metrics

Each guide has community-driven quality scores:

**Upvotes/Downvotes** - Community rating
**Execution Count** - How many times users tried it
**Success Rate** - Percentage of successful executions
**Quality Score** - Weighted combination of above

**High-quality guides:**
```bash
cmdai guides list --min-quality 0.8
```

**Popular guides:**
```bash
cmdai guides list --popular
```

**Guides needing review:**
```bash
cmdai guides list --needs-review
```

## Contributing

We welcome community contributions! See [CONTRIBUTING.md](../../CONTRIBUTING.md).

**How to contribute a guide:**
1. Check if guide already exists (search first!)
2. Copy template from `docs/guides/_template.md`
3. Fill in all sections thoroughly
4. Test the command works as described
5. Submit PR with clear description

**What makes a great guide:**
- ✓ Clear, specific title
- ✓ Real-world use case
- ✓ Step-by-step examples
- ✓ Common pitfalls documented
- ✓ Safety notes included
- ✓ Related guides linked
- ✓ Tested and verified

**What to avoid:**
- ✗ Duplicate existing guides
- ✗ Overly complex commands
- ✗ Platform-specific without noting
- ✗ Missing safety warnings
- ✗ Untested examples

## Using Guides

**In CLI:**
```bash
# List guides for current task
cmdai guides search "your task"

# View full guide
cmdai guides show <guide-id>

# Execute guide's command
cmdai guides run <guide-id>

# Rate a guide after using it
cmdai guides vote <guide-id> up|down
```

**On Web:**
- Browse: https://cmdai.dev/guides
- Categories: https://cmdai.dev/guides/<category>
- Individual: https://cmdai.dev/guides/<category>/<guide-id>

## Try in cmdai

Every guide has a "Try in cmdai" section:

```bash
# Option 1: Execute guide directly
cmdai guides run guide-git-001

# Option 2: Use the natural language prompt
cmdai "undo my last git commit but keep changes"

# Option 3: Execute command manually
git reset --soft HEAD~1
```

## Relationship to Guardrails

Guides may reference related guardrails:

```yaml
related_guardrails:
  - "grd-001"  # rm -rf / protection
```

This helps users understand:
- What safety patterns apply to this guide
- Why certain alternatives are safer
- How to work within safety constraints

## Learning Path

**New to terminal? Start here:**
1. File basics: `guide-files-001`, `guide-files-008`
2. Git basics: `guide-git-001`, `guide-git-015`
3. Docker basics: `guide-docker-001`, `guide-docker-005`

**Getting comfortable? Try these:**
1. Advanced Git: `guide-git-022` (interactive rebase)
2. Docker optimization: `guide-docker-012` (cleanup)
3. System admin: `guide-system-005` (service management)

**Power user? Dive deep:**
1. Custom workflows with multiple commands
2. Contribute your own guides
3. Propose new guardrails based on your learnings

## Statistics

```bash
# Guide analytics
cmdai guides stats

# Most popular by category
cmdai guides stats --category git

# Trending this week
cmdai guides stats --trending

# Your guide usage history
cmdai guides history
```

## Questions?

- **Can't find a guide?** - Request it: `cmdai guides request "your task"`
- **Guide doesn't work?** - Report issue with guide ID
- **Want to improve a guide?** - Submit PR with changes
- **Guide outdated?** - Flag it: `cmdai guides flag <id> outdated`

## Philosophy

cmdai guides follow these principles:

1. **Practical** - Real tasks developers actually need to do
2. **Educational** - Explain why, not just how
3. **Safe** - Highlight risks and safer alternatives
4. **Community-driven** - Improve through collective knowledge
5. **Always learning** - Guides evolve as tools evolve

## Comparison to Other Resources

**vs. Stack Overflow:**
- ✓ Curated and tested
- ✓ cmdai-specific prompts
- ✓ Safety-aware
- ✗ Smaller library (but growing!)

**vs. tldr-pages:**
- ✓ Natural language interface
- ✓ Community metrics
- ✓ Safety context
- ✗ Less command coverage (for now)

**vs. man pages:**
- ✓ Task-focused (not command-focused)
- ✓ Examples-first
- ✓ Plain language
- ✗ Less comprehensive per command

---

**Last updated:** 2024-11-28
**Total guides:** 150+
**Categories:** 12
**Community contributors:** 200+
**Average quality score:** 0.87

Happy learning! 🚀
