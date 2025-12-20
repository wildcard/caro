# Agent Discovery Pipeline

Caro maintains awareness of the coding agent ecosystem through a structured discovery and integration process. This document describes how new agents are discovered, evaluated, and added to the Caro companion documentation.

## Pipeline Overview

```
Discovery → Evaluation → Documentation → Integration → Maintenance
    ↑                                                      |
    └──────────────────── Feedback Loop ───────────────────┘
```

## 1. Discovery Phase

### Automated Monitoring

Set up monitoring for new coding agents:

```yaml
# .github/workflows/agent-discovery.yml
name: Agent Discovery

on:
  schedule:
    - cron: '0 0 * * 1'  # Weekly on Monday

jobs:
  discover:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Check for new agents
        run: |
          # Check GitHub trending in AI/ML categories
          # Monitor HuggingFace, ProductHunt, etc.
          # Compare against known agents list
          ./scripts/discover-agents.sh

      - name: Create issue for new agents
        if: steps.discover.outputs.new_agents
        uses: actions/github-script@v7
        with:
          script: |
            github.rest.issues.create({
              owner: context.repo.owner,
              repo: context.repo.repo,
              title: 'New Coding Agent Discovered',
              body: '${{ steps.discover.outputs.agents_list }}',
              labels: ['agent-discovery', 'documentation']
            })
```

### Manual Discovery Sources

| Source | Frequency | Action |
|--------|-----------|--------|
| GitHub Trending | Weekly | Check AI/ML repositories |
| HuggingFace | Weekly | Monitor new models/tools |
| ProductHunt | Weekly | AI tools category |
| Hacker News | Daily | "Show HN" AI tools |
| Reddit r/LocalLLaMA | Weekly | New CLI tools |
| X/Twitter | Daily | #CodingAI hashtag |
| Discord communities | Weekly | Announcements |

### Discovery Script

```bash
#!/bin/bash
# scripts/discover-agents.sh

KNOWN_AGENTS="docs/coding-agents/known-agents.txt"
NEW_AGENTS=""

# Check GitHub for trending AI coding tools
echo "Checking GitHub trending..."
gh api search/repositories \
  -f q="coding agent cli OR ide ai in:description" \
  -f sort=stars \
  -f order=desc \
  --jq '.items[].full_name' > /tmp/github-agents.txt

# Compare with known agents
while read agent; do
  if ! grep -q "$agent" "$KNOWN_AGENTS"; then
    NEW_AGENTS="$NEW_AGENTS\n$agent"
  fi
done < /tmp/github-agents.txt

if [ -n "$NEW_AGENTS" ]; then
  echo "new_agents=true" >> $GITHUB_OUTPUT
  echo "agents_list=$NEW_AGENTS" >> $GITHUB_OUTPUT
fi
```

## 2. Evaluation Phase

### Evaluation Criteria

Score each potential agent on these criteria (1-5):

| Criterion | Weight | Description |
|-----------|--------|-------------|
| Popularity | 20% | GitHub stars, downloads, community size |
| Maintenance | 20% | Recent commits, issue response, releases |
| Integration Potential | 25% | CLI support, terminal access, config options |
| Safety Focus | 15% | Command validation, confirmation workflows |
| Documentation | 10% | Quality of docs, examples, tutorials |
| Uniqueness | 10% | Distinct features from existing agents |

### Evaluation Template

```markdown
## Agent Evaluation: [Agent Name]

**Date**: YYYY-MM-DD
**Evaluator**: @username

### Basic Info
- **Repository**:
- **Website**:
- **License**:
- **Stars**:
- **Last Commit**:

### Scores

| Criterion | Score (1-5) | Notes |
|-----------|-------------|-------|
| Popularity | | |
| Maintenance | | |
| Integration Potential | | |
| Safety Focus | | |
| Documentation | | |
| Uniqueness | | |

**Total Score**: X/30
**Weighted Score**: X.XX/5.00

### Recommendation
- [ ] Add to Caro documentation
- [ ] Monitor for future inclusion
- [ ] Skip (reason: )

### Caro Integration Notes
- Terminal access: Yes/No
- Config file support:
- MCP support: Yes/No
- Key integration points:
```

### Minimum Thresholds

| Decision | Weighted Score |
|----------|---------------|
| Add immediately | ≥ 4.0 |
| Add with caveats | 3.0 - 3.9 |
| Monitor | 2.0 - 2.9 |
| Skip | < 2.0 |

## 3. Documentation Phase

### Documentation Template

Each agent documentation should follow this structure:

```markdown
# [Agent Name]

**[One-line description]**

[2-3 sentence description of what the agent does]

## Overview

| Attribute | Value |
|-----------|-------|
| **Developer** | |
| **Type** | CLI/IDE/Extension |
| **Language** | |
| **License** | |
| **Website** | |
| **Repository** | |
| **Platforms** | |

## Installation

### [Method 1]
### [Method 2]
### Verify Installation

## Configuration

### [Config type]
### Project Configuration

## Key Features

### [Feature 1]
### [Feature 2]

## Integration with Caro

### Method 1: [Integration type]
### Method 2: [Integration type]
### Method 3: [Integration type]

## [Agent] Commands

| Command | Description |
|---------|-------------|

## Best Practices with Caro

### 1. [Practice]
### 2. [Practice]

## Comparison with [Related Agents]

| Feature | [Agent] | [Other] |
|---------|---------|---------|

## Troubleshooting

### Common Issues

**Issue**:
**Issue**:

## Resources

- [Documentation]()
- [GitHub]()
- [Community]()

## See Also

- [Related Agent](./related.md)
```

### Checklist for New Agent Documentation

- [ ] Complete overview table
- [ ] At least 2 installation methods
- [ ] Configuration examples
- [ ] 3+ Caro integration methods
- [ ] Command reference table
- [ ] 2+ best practices
- [ ] Troubleshooting section
- [ ] Resource links
- [ ] Cross-references to related agents
- [ ] Added to README.md index

## 4. Integration Phase

### Update Registry

Add to `docs/coding-agents/README.md`:

```markdown
| [New Agent](./new-agent.md) | Company | Type | Integration Type |
```

### Update Known Agents

```bash
# Add to known agents list
echo "company/agent-repo" >> docs/coding-agents/known-agents.txt
sort -u docs/coding-agents/known-agents.txt -o docs/coding-agents/known-agents.txt
```

### Integration Testing

```bash
# Test that Caro works with the agent
./scripts/test-agent-integration.sh new-agent

# Verify documentation links
./scripts/check-doc-links.sh docs/coding-agents/new-agent.md
```

## 5. Maintenance Phase

### Quarterly Review

Every quarter, review all documented agents:

```yaml
# .github/workflows/agent-review.yml
name: Quarterly Agent Review

on:
  schedule:
    - cron: '0 0 1 */3 *'  # First of every third month

jobs:
  review:
    runs-on: ubuntu-latest
    steps:
      - name: Check agent status
        run: |
          # For each documented agent:
          # - Check if repository still exists
          # - Check last commit date
          # - Check if still maintained
          # - Verify installation instructions work
          ./scripts/review-agents.sh
```

### Update Triggers

Update agent documentation when:
- Major version release
- New integration method available
- Installation process changes
- New safety features added
- Deprecation announced

### Deprecation Process

When an agent is no longer maintained:

1. Add deprecation notice to documentation
2. Move to "Historical Agents" section
3. Suggest alternatives
4. Keep for 6 months, then archive

```markdown
> **Deprecated**: This agent is no longer actively maintained.
> Consider using [Alternative](./alternative.md) instead.
```

## Contributing New Agents

### Community Submissions

Community members can submit new agents via:

1. **GitHub Issue**: Use the "New Agent" issue template
2. **Pull Request**: Add documentation following the template
3. **Discussion**: Start a discussion for evaluation

### Issue Template

```markdown
---
name: New Coding Agent
about: Suggest a new coding agent for Caro documentation
labels: agent-discovery
---

## Agent Information

**Name**:
**Repository**:
**Website**:
**Type**: CLI / IDE / Extension

## Why Include This Agent?

[Describe why this agent should be documented]

## Caro Integration Ideas

[How could this agent integrate with Caro?]

## Initial Evaluation

- Stars:
- Last Commit:
- Maintenance Status: Active / Sporadic / Unknown
```

### PR Requirements

For PRs adding new agent documentation:

1. Complete documentation following template
2. At least 2 working integration methods tested
3. Screenshots/examples where applicable
4. Added to README.md index
5. Added to known-agents.txt
6. All links verified

## Known Agents List

Maintained in `docs/coding-agents/known-agents.txt`:

```
# CLI Agents
anthropics/claude-code
charmbracelet/crush
openai/codex-cli
aug-dev/augie
ovh/shai
paul-gauthier/aider

# IDE Agents
getcursor/cursor
codeium/windsurf
voideditor/void
zed-industries/zed

# VS Code Extensions
continuedev/continue
sourcegraph/cody
cline/cline
RooVetGit/Roo-Code

# IDE Integrations
github/copilot
```

## Metrics & Tracking

Track agent ecosystem growth:

```json
// docs/coding-agents/metrics.json
{
  "last_updated": "2025-01-15",
  "total_agents": 15,
  "agents_by_type": {
    "cli": 6,
    "ide": 4,
    "extension": 5
  },
  "most_integrated": "claude-code",
  "newest_addition": "roo-code",
  "pending_evaluation": 2
}
```

## Resources

- [Agent Evaluation Template](./templates/evaluation.md)
- [Documentation Template](./templates/agent-doc.md)
- [Known Agents List](./known-agents.txt)
- [Integration Test Script](../../scripts/test-agent-integration.sh)

---

**Maintainers**: @wildcard, @caro-team

**Last Updated**: 2025-01-15
