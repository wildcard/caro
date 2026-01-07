---
title: Beta Testing Program
description: How to participate in caro beta testing and help improve the product
---

## Join the Beta Testing Program

We welcome community members to help test caro releases before they reach general availability. Beta testers play a crucial role in ensuring caro works reliably across different environments and use cases.

### What We're Looking For

Beta testing is most valuable when it represents **diverse real-world scenarios**:

- **Diverse environments**: Different operating systems, shells, network setups, and tool configurations
- **Real-world usage**: Testing actual workflows you'd use daily, not just toy examples
- **Honest feedback**: Document friction points, confusing error messages, and missing documentation
- **Systematic testing**: Follow the testing workflow to ensure comprehensive coverage

### Why Participate?

- **Early access**: Try new features before they're publicly released
- **Direct impact**: Your feedback directly shapes the product
- **Recognition**: Contributors are credited in release notes and the contributors list
- **Learn**: Understand how CLI tools are tested and validated
- **Community**: Join discussions with maintainers and other testers

## Testing Profiles

We use **persona-based testing** to ensure coverage across different user types. You can either:

1. **Adopt an existing profile** from our [Testing Profiles](/contributing/testing-profiles) page
2. **Create your own profile** representing your unique environment and use case

### Example Profiles

| Profile | Focus | Environment |
|---------|-------|-------------|
| **Terminal Novice** | First-time CLI users | macOS, basic tools, GUI preference |
| **Corporate Developer** | Restricted environments | Proxy, no sudo, security policies |
| **Data Scientist** | Data processing workflows | Python, conda, Jupyter, ML tools |
| **Fish Shell User** | Non-POSIX shells | macOS/Linux, fish shell, tmux |
| **SSH-Only Admin** | Remote/offline usage | CentOS, SSH-only, airgapped |

See [all available profiles →](/contributing/testing-profiles)

## Testing Workflow

### Phase 1: Setup

1. **Choose a profile** or define your own
2. **Set up your environment** to match the profile's specifications
3. **Review the current beta cycle** in the [ROADMAP](https://github.com/wildcard/caro/blob/main/ROADMAP.md#beta-testing-cycles)

### Phase 2: Documentation Discovery

**Important**: Approach testing as if you've never used caro before.

- Only use public documentation (website, README, docs site)
- Don't rely on internal knowledge or source code
- Document where you find information (or fail to find it)
- Note any confusing or unclear instructions

### Phase 3: Installation Testing

Follow the installation instructions **exactly as documented**:

1. Try the primary installation method for your OS
2. Document any errors, warnings, or unexpected behavior
3. Verify installation with `caro --version` and `caro doctor`
4. Test that `--help` provides useful guidance

### Phase 4: Feature Testing

Test the features and claims advertised on the website:

- **Command generation**: Try examples from the website
- **Safety validation**: Attempt dangerous commands to verify blocking
- **CLI flags**: Test `--execute`, `--output json`, `--verbose`, etc.
- **Backend detection**: Verify your backend (MLX, Ollama, etc.) is detected
- **Platform-specific behavior**: Test shell syntax and BSD/GNU awareness

### Phase 5: Issue Documentation

When you find a discrepancy:

1. **Verify it's reproducible**: Try the same steps 2-3 times
2. **File a GitHub issue** using our template (see below)
3. **Include all evidence**: Commands, outputs, environment details
4. **Categorize severity**: P0 (critical), P1 (high), P2 (medium), P3 (low)

## Filing Issues

Use this template when reporting issues:

```markdown
### [Beta Testing] <Brief Description>

**Tester Profile**: <Your profile name or bt_XXX>
**Environment**: <OS>, <Shell>, <Version>
**Severity**: P0/P1/P2/P3

**Website Claim**:
> <Quote from website or documentation>

**Expected Behavior**:
<What should happen based on documentation>

**Actual Behavior**:
<What actually happened>

**Steps to Reproduce**:
1. <Step 1>
2. <Step 2>
3. <Step 3>

**Evidence**:
```bash
$ <command>
<full output>
```

**Reproducibility**: Every time / Intermittent / Once

**Additional Context**:
<Any other relevant information>
```

### Severity Guidelines

| Severity | Description | Examples |
|----------|-------------|----------|
| **P0** | Critical - Blocks primary use case | Installation fails, crashes on startup |
| **P1** | High - Breaks common workflow | Documented flag doesn't work, safety bypass |
| **P2** | Medium - Degrades experience | Confusing error message, performance issue |
| **P3** | Low - Minor polish | Typo in output, cosmetic issue |

## Current Testing Cycle

Check the [ROADMAP Beta Testing Cycles](https://github.com/wildcard/caro/blob/main/ROADMAP.md#beta-testing-cycles) section for:

- Current release being tested
- Testing focus areas
- Profiles needed
- Timeline and deadlines

## Recognition

Beta testers who contribute are recognized in:

- **Release notes**: Acknowledged in the changelog for each release
- **Contributors list**: Added to CONTRIBUTORS.md
- **GitHub discussions**: Featured in community spotlight posts
- **Special badges**: Beta tester role in Discord (coming soon)

## Resources

- [Testing Profiles](/contributing/testing-profiles) - Available persona profiles
- [Beta Testing Playbook](https://github.com/wildcard/caro/tree/main/.claude/skills/quality-engineer-manager/references/beta-testing-playbook.md) - Internal methodology (advanced)
- [ROADMAP](https://github.com/wildcard/caro/blob/main/ROADMAP.md) - Current testing cycles
- [GitHub Issues](https://github.com/wildcard/caro/issues) - Report findings

## Questions?

- **GitHub Discussions**: [wildcard/caro/discussions](https://github.com/wildcard/caro/discussions)
- **Discord**: Coming soon
- **Email**: [Insert contact email]

---

Thank you for helping make caro better for everyone!
