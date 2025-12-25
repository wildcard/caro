# Create cmdai Claude Skill for Plugin Marketplace

## Summary

Create an official Claude skill that helps developers effectively use cmdai (caro) for safe, efficient shell command generation directly within Claude Code sessions.

## Motivation

cmdai is a powerful CLI tool that converts natural language to safe POSIX shell commands. By creating a Claude skill, we can:

1. **Seamless Integration**: Users can generate safe shell commands directly in their Claude Code workflow without context-switching
2. **Safety First**: Guide users through cmdai's safety validation features
3. **Discoverability**: Increase cmdai adoption through the official Claude plugin marketplace
4. **Best Practices**: Educate users on proper usage patterns for LLM-generated commands

## Skill Specification

### Metadata
```yaml
---
name: "cmdai-shell-helper"
description: "Use when users need help generating safe, tested POSIX shell commands from natural language descriptions. Guides users through command generation, safety validation, and execution workflows using cmdai/caro best practices"
version: "1.0.0"
allowed-tools: "Bash, Read, Write"
dependencies: "cmdai (optional - provides installation guidance)"
license: "AGPL-3.0"
---
```

### Skill Purpose

The skill will:
- **Detect** when users need shell command generation assistance
- **Guide** users through describing their command needs effectively
- **Generate** safe commands using cmdai's safety-first approach
- **Validate** commands for POSIX compliance and safety patterns
- **Educate** about risk levels and command safety best practices
- **Integrate** with existing cmdai installations or guide setup

### Key Features

1. **Smart Detection**: Automatically activates when Claude detects:
   - "generate a command to..."
   - "how do I [shell operation]..."
   - "shell command for..."
   - File/directory operations discussed
   - System administration tasks

2. **Safety Guidance**:
   - Explain cmdai's 4-tier risk assessment (Safe, Moderate, High, Critical)
   - Show validation results before execution
   - Warn about dangerous patterns
   - Suggest safer alternatives

3. **Best Practices Education**:
   - POSIX compliance importance
   - Proper path quoting for spaces/special chars
   - Command verification workflows
   - When to use different backends (MLX, Ollama, vLLM)

4. **Installation Assistance**:
   - Check if cmdai/caro is installed
   - Provide installation commands if needed
   - Guide backend configuration

## Implementation Plan

### Phase 1: Skill Structure Setup
**Tasks:**
- [ ] Create `.claude/skills/cmdai-shell-helper/` directory structure
- [ ] Draft SKILL.md with frontmatter and core instructions
- [ ] Create `scripts/` directory for helper utilities
- [ ] Create `references/` directory for safety pattern documentation

**Deliverables:**
```
.claude/skills/cmdai-shell-helper/
├── SKILL.md                           # Main skill definition
├── scripts/
│   └── check-cmdai-installed.sh      # Verify cmdai availability
├── references/
│   ├── safety-patterns.md            # Common dangerous patterns
│   ├── posix-compliance.md           # POSIX vs bash-specific features
│   └── risk-levels.md                # Explanation of 4 risk tiers
└── examples/
    ├── basic-usage.md                # Simple command examples
    ├── safety-examples.md            # Safety validation examples
    └── backend-config.md             # Backend configuration examples
```

### Phase 2: SKILL.md Core Content
**Tasks:**
- [ ] Write activation triggers and detection patterns
- [ ] Document cmdai workflow integration
- [ ] Create safety validation guidance sections
- [ ] Add POSIX compliance education
- [ ] Include installation check procedures

**Key Sections:**
1. **What This Skill Does**
2. **When to Activate** (detection patterns)
3. **Workflow Integration**
   - Pre-flight check (is cmdai installed?)
   - Command generation workflow
   - Safety validation review
   - Execution guidance
4. **Safety Education**
   - Risk level explanations
   - Common dangerous patterns
   - Safe alternatives
5. **Best Practices**
   - Natural language prompt tips
   - POSIX compliance
   - Backend selection

### Phase 3: Helper Scripts
**Tasks:**
- [ ] Create `check-cmdai-installed.sh` - Verify cmdai/caro availability
- [ ] Create `show-risk-levels.sh` - Display safety tier information
- [ ] Create `validate-command.sh` - Pre-execution validation wrapper

**Scripts:**

**`scripts/check-cmdai-installed.sh`:**
```bash
#!/bin/bash
# Check if cmdai or caro is installed
if command -v caro &> /dev/null; then
    echo "✓ caro found: $(which caro)"
    caro --version
    exit 0
elif command -v cmdai &> /dev/null; then
    echo "✓ cmdai found: $(which cmdai)"
    cmdai --version
    exit 0
else
    echo "✗ cmdai/caro not found"
    echo ""
    echo "Install with: cargo install cmdai"
    echo "Or run: bash <(curl -sSfL https://setup.caro.sh)"
    exit 1
fi
```

### Phase 4: Reference Documentation
**Tasks:**
- [ ] Document all 52 dangerous command patterns from safety validator
- [ ] Create POSIX compliance reference
- [ ] Document risk level matrix
- [ ] Add command validation flowchart

**`references/safety-patterns.md`:**
```markdown
# cmdai Safety Patterns

## Critical (Red) - Always Blocked
- System destruction: `rm -rf /`, `rm -rf ~`
- Disk operations: `mkfs.*`, `dd if=/dev/zero`
- Fork bombs: `:(){:|:&};:`
- Critical paths: Operations on `/bin`, `/usr`, `/etc`

## High (Orange) - Requires Explicit Confirmation
- Recursive deletions: `rm -rf`
- Mass operations: `chmod 777`
- Privilege escalation: `sudo su`

## Moderate (Yellow) - Confirmation in Strict Mode
- File modifications in system directories
- Network operations affecting firewall
- Package manager operations

## Safe (Green) - No Confirmation
- Read-only operations
- Standard file/directory listings
- Safe data transformations
```

### Phase 5: Plugin Marketplace Integration
**Tasks:**
- [ ] Create `.claude-plugin/marketplace.json` manifest
- [ ] Write comprehensive README.md for the skill repository
- [ ] Create GitHub repository: `wildcard/cmdai-claude-skill`
- [ ] Add installation instructions
- [ ] Submit to official marketplace listings

**`.claude-plugin/marketplace.json`:**
```json
{
  "name": "cmdai Shell Command Helper",
  "description": "Generate safe, POSIX-compliant shell commands using cmdai's LLM-powered command generation with comprehensive safety validation",
  "version": "1.0.0",
  "author": "wildcard",
  "repository": "https://github.com/wildcard/cmdai-claude-skill",
  "homepage": "https://caro.sh",
  "license": "AGPL-3.0",
  "skills": [
    {
      "name": "cmdai-shell-helper",
      "description": "Safe shell command generation with POSIX compliance and risk assessment",
      "path": "cmdai-shell-helper"
    }
  ],
  "installation": {
    "requires": {
      "cmdai": "optional"
    }
  }
}
```

### Phase 6: Testing & Validation
**Tasks:**
- [ ] Test skill activation in various scenarios
- [ ] Validate all helper scripts work cross-platform
- [ ] Test with cmdai installed vs not installed
- [ ] Verify safety guidance is accurate
- [ ] Test POSIX compliance recommendations

**Test Scenarios:**
1. User asks "how do I list all PDF files larger than 10MB?"
2. User requests dangerous operation (e.g., "delete all logs")
3. User needs POSIX-compliant alternative to bash-specific syntax
4. cmdai not installed - skill should guide installation
5. cmdai installed - skill should integrate seamlessly

### Phase 7: Documentation & Launch
**Tasks:**
- [ ] Create comprehensive README.md for skill repository
- [ ] Add usage examples and screenshots
- [ ] Write blog post about the skill
- [ ] Update cmdai website with skill information
- [ ] Announce on GitHub Discussions
- [ ] Submit to Claude plugin marketplace directories:
  - Official Anthropic skills repository
  - SkillsMP.com
  - claude-plugins.dev

## Success Metrics

1. **Functionality**:
   - ✅ Skill correctly detects shell command needs
   - ✅ Safety validation guidance is accurate
   - ✅ Works with and without cmdai installed
   - ✅ Cross-platform compatibility (macOS, Linux, Windows)

2. **User Experience**:
   - ✅ Clear activation triggers
   - ✅ Helpful error messages
   - ✅ Educational guidance without being verbose
   - ✅ Seamless workflow integration

3. **Distribution**:
   - ✅ Installable via `/plugin install wildcard/cmdai-claude-skill`
   - ✅ Listed on official marketplace directories
   - ✅ Comprehensive documentation
   - ✅ Active maintenance and updates

## Example Usage Flow

### Scenario: User Needs to Find Large Files

**User:** "I need to find all files larger than 100MB in my home directory"

**Claude (with skill activated):**
```
I'll help you generate a safe command for this using cmdai best practices.

Let me check if cmdai is available...
[Runs check-cmdai-installed.sh]

✓ cmdai is installed!

Based on your request, here's the cmdai approach:

$ caro "find all files larger than 100MB in home directory"

Generated command:
  find ~ -type f -size +100M -ls

Safety Assessment: ✅ Safe (Green)
- Read-only operation
- POSIX-compliant syntax
- Properly quoted paths

Execute? (y/N)

This command:
- Uses POSIX-standard `find` utility
- Searches from home directory (~)
- Filters for regular files (-type f)
- Finds files over 100MB (-size +100M)
- Lists results with details (-ls)

Would you like me to:
1. Execute this command?
2. Modify the search criteria?
3. Explain any part of the command?
```

## Website Integration

### Update cmdai Website (caro.sh)

**Tasks:**
- [ ] Add new "Integrations" or "Claude Skill" page
- [ ] Update homepage with Claude integration badge
- [ ] Add installation instructions
- [ ] Include usage examples with screenshots
- [ ] Link to skill repository

**Website Content Sections:**
1. **Hero Section**: "Use cmdai directly in Claude Code"
2. **Quick Install**: `/plugin install wildcard/cmdai-claude-skill`
3. **Features Highlight**:
   - Automatic activation
   - Safety-first guidance
   - POSIX compliance education
   - Risk assessment integration
4. **Usage Examples**: Screenshots and workflows
5. **Documentation Links**: Link to SKILL.md and references

### Blog Post

**Title**: "Introducing the cmdai Claude Skill: Safe Shell Command Generation in Your AI Workflow"

**Sections**:
- Why we built it
- How it works
- Safety features
- Installation guide
- Usage examples
- Community feedback invitation

## Timeline

### Week 1: Foundation
- Create skill directory structure
- Write SKILL.md core content
- Create helper scripts
- Write reference documentation

### Week 2: Testing & Refinement
- Test across multiple scenarios
- Refine activation triggers
- Validate safety guidance accuracy
- Cross-platform testing

### Week 3: Distribution & Launch
- Create GitHub repository
- Write comprehensive README
- Create marketplace.json
- Submit to marketplace directories
- Update cmdai website
- Publish blog post
- Community announcement

## Dependencies

- **cmdai**: Optional runtime dependency (skill guides installation if missing)
- **Claude Code**: Required platform
- **Bash**: For helper scripts (cross-platform)

## Related Resources

- [Official Claude Skills Documentation](https://code.claude.com/docs/en/skills)
- [Anthropic Skills Repository](https://github.com/anthropics/skills)
- [cmdai Repository](https://github.com/wildcard/cmdai)
- [cmdai Website](https://caro.sh)
- [Claude Plugin Marketplace Guide](https://code.claude.com/docs/en/plugin-marketplaces)

## Questions & Discussion

- Should the skill also support multi-step workflows (e.g., "find files, then compress them")?
- Should we include backend selection guidance (MLX vs Ollama vs vLLM)?
- Should the skill have different modes (beginner vs advanced)?
- How should we handle platform-specific commands (macOS vs Linux vs Windows)?

## Next Steps

1. **Review this plan** - Community feedback welcome
2. **Assign to developer** - Who will implement?
3. **Set milestone** - Target completion date?
4. **Create project board** - Track progress through phases
5. **Begin implementation** - Start with Phase 1

---

**Labels**: `enhancement`, `claude-skill`, `integration`, `documentation`
**Milestone**: v0.2.0
**Priority**: Medium
**Estimated Effort**: 2-3 weeks
