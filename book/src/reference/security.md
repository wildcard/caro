# Security Policy

## Overview

cmdai is a safety-first CLI tool that generates shell commands using local LLMs. Security is fundamental to our mission: we must prevent dangerous command execution while maintaining user trust in the tool.

This document outlines:
- Supported versions and security update policy
- How to report security vulnerabilities
- What qualifies as a security issue
- Our response timeline and process
- Recognition for security researchers

---

## Supported Versions

cmdai is currently in **early development** (v0.1.x). Security updates are provided for:

| Version | Supported | Notes |
|---------|-----------|-------|
| 0.1.x   | ✅ Yes    | Current development series |
| < 0.1.0 | ❌ No     | Pre-release, not recommended for use |

Once cmdai reaches v1.0.0, we will maintain security support for:
- **Latest stable release**: Full security support
- **Previous major version**: Critical security fixes only for 6 months after new major release
- **Older versions**: No security support (upgrade recommended)

---

## Reporting a Vulnerability

**DO NOT** report security vulnerabilities through public GitHub issues.

### Private Disclosure Process

We use GitHub's private security advisory system for vulnerability reports:

1. **Navigate to the Security tab**: Visit [github.com/wildcard/cmdai/security](https://github.com/wildcard/cmdai/security)

2. **Create a private security advisory**:
   - Click "Report a vulnerability"
   - Provide a clear description of the vulnerability
   - Include steps to reproduce
   - Describe the potential impact
   - Suggest a fix if possible

3. **Wait for acknowledgment**: We will respond within **48 hours** with:
   - Confirmation of receipt
   - Initial assessment of severity
   - Expected timeline for investigation

### Alternative Reporting Methods

If you cannot use GitHub's security advisory system:

- **Email maintainers** directly (email addresses listed in repository maintainer profiles)
- **Encrypted communication**: If needed, request a PGP key for sensitive disclosures

### What to Include in Your Report

A good security report includes:

- **Description**: What is the vulnerability?
- **Impact**: What can an attacker accomplish?
- **Reproduction steps**: How to trigger the vulnerability
- **Affected versions**: Which versions are vulnerable?
- **Environment**: OS, Rust version, configuration details
- **Proof of concept**: Code, commands, or screenshots demonstrating the issue
- **Suggested fix**: If you have ideas for remediation (optional)

**Example report structure**:

```
Title: Command Injection via Unescaped Model Output

Description:
The MLX backend does not properly escape special characters in model-generated
commands, allowing injection of additional commands via carefully crafted prompts.

Impact:
An attacker can execute arbitrary commands by providing a prompt that causes
the model to generate output containing shell metacharacters (;, &&, ||, etc.).

Reproduction:
1. Use MLX backend with default safety settings
2. Provide prompt: "list files" (but craft model to return "ls; rm -rf /")
3. Safety validator does not catch the injection
4. User executes the command, triggering unintended rm

Affected versions: 0.1.0 - 0.1.3

Environment:
- macOS 14.5, Apple M1
- Rust 1.75.0
- cmdai v0.1.2

Suggested fix:
Apply shell escaping using the `shell-escape` crate before passing commands
to the safety validator. Validate that model output contains only a single
command with no metacharacters outside of quoted strings.
```

---

## Response Timeline

Our commitment to security researchers:

| Timeframe | Action |
|-----------|--------|
| **48 hours** | Acknowledgment of your report |
| **7 days** | Initial assessment and severity classification |
| **14 days** | Regular status updates on investigation progress |
| **30 days** | Target fix release for critical vulnerabilities |
| **90 days** | Target fix release for moderate vulnerabilities |

For **critical** vulnerabilities (remote code execution, privilege escalation), we will:
- Prioritize investigation immediately
- Develop a fix within 7 days if possible
- Release an emergency patch version
- Coordinate disclosure timing with you

For **high/moderate** vulnerabilities, we will:
- Develop a fix for the next scheduled release
- Include the fix in the next minor version
- Coordinate disclosure after patch is available

---

## Vulnerability Scope

### In Scope - Security Issues We Address

We consider the following security vulnerabilities:

#### Safety Validation Bypasses

- **Command injection**: Ability to execute unintended commands
- **Pattern evasion**: Bypassing dangerous command detection
- **Metacharacter exploitation**: Using shell features to escape validation
- **Path traversal**: Accessing files outside intended scope
- **Privilege escalation**: Bypassing `--allow-dangerous` restrictions

**Examples**:
- Crafting prompts that cause model to generate commands that bypass safety checks
- Using shell quoting or escaping to hide dangerous patterns
- Exploiting race conditions in validation vs execution
- Unicode or encoding tricks that bypass regex patterns

#### Model Integrity and Trust

- **Model tampering**: Ability to replace cached models with malicious versions
- **Checksum bypass**: Circumventing model verification
- **Cache poisoning**: Injecting malicious models into the cache directory
- **Path substitution**: Redirecting model loading to attacker-controlled files

**Examples**:
- Modifying cached model files to generate malicious commands
- Bypassing SHA-256 checksums during model downloads
- Symlink attacks on cache directory
- TOCTOU (time-of-check-time-of-use) vulnerabilities in model loading

#### Command Execution Vulnerabilities

- **Shell injection**: Executing unintended shell code
- **Environment pollution**: Manipulating environment variables to change behavior
- **Subprocess exploitation**: Using subprocess features unsafely
- **Signal handling**: Race conditions in command execution

**Examples**:
- Injecting shell commands via `$()` or backticks
- Setting `LD_PRELOAD` or other dangerous env vars
- Exploiting `eval` or other dynamic code execution
- Race conditions between validation and execution

#### Information Disclosure

- **Sensitive data exposure**: Leaking user data, API keys, or system information
- **Prompt injection**: Extracting system prompts or internal configuration
- **Cache inspection**: Reading cached models or manifests to gain insights
- **Log leakage**: Exposing sensitive information in logs or error messages

**Examples**:
- Prompts that cause model to output configuration files
- Reading `.env` files or API keys from error messages
- Accessing other users' cached models on shared systems
- Timing attacks revealing model or system state

### Out of Scope - Issues We Do NOT Address

We do **not** consider the following as security vulnerabilities:

#### Theoretical or Impractical Attacks

- Attacks requiring physical access to the machine
- Social engineering or phishing attacks on users
- Vulnerabilities in third-party dependencies (report those upstream)
- Attacks requiring user to manually edit configuration with malicious intent

#### Model Output Quality

- Model generating incorrect or suboptimal commands (use GitHub Issues)
- Model failing to understand prompts (this is a model quality issue)
- Model output being slow or inefficient (performance issue, not security)

#### Expected Behavior

- Safety validator blocking commands you intended to run (use `--allow-dangerous`)
- Requiring confirmation for risky commands (this is intentional)
- Refusing to generate commands for destructive operations (by design)

#### Resource Exhaustion (DOS)

- Consuming disk space with large model downloads (expected usage)
- Using significant CPU/memory during inference (expected usage)
- Denial-of-service via repeated requests (rate limiting is not implemented)

**However**, if you can demonstrate **remote** or **persistent** resource exhaustion vulnerabilities, those may be in scope.

---

## Security Best Practices for Users

While using cmdai:

1. **Review generated commands** before execution, especially for sensitive operations
2. **Never use `--auto` mode** for destructive or privileged operations
3. **Keep cmdai updated** to get the latest security fixes
4. **Verify model checksums** when downloading from Hugging Face
5. **Limit `--allow-dangerous` usage** to specific, understood cases
6. **Check logs** regularly for unexpected model or command behavior
7. **Use restricted shells** if deploying cmdai in shared environments

---

## Security Development Practices

cmdai employs security-first development practices:

### Defense in Depth

We apply multiple layers of security:
1. **Input validation**: Sanitize prompts before sending to models
2. **Output validation**: Check generated commands against safety patterns
3. **Execution sandboxing**: Consider containerization for high-risk environments
4. **Least privilege**: Never run as root, minimize file permissions
5. **Fail-safe defaults**: Block dangerous operations unless explicitly allowed

### Safety Pattern Library

Our safety validator includes patterns for:
- **Filesystem destruction**: `rm -rf /`, `mkfs`, `dd if=/dev/zero`
- **Fork bombs**: `:(){ :|:& };:` and variants
- **Privilege escalation**: `sudo su`, `chmod 777 /`
- **System path tampering**: Operations on `/bin`, `/usr`, `/etc`
- **Device writes**: Direct device manipulation

See `src/safety/patterns.rs` for the complete pattern library.

### Secure Dependency Management

- **cargo-audit** runs on all PRs to detect vulnerable dependencies
- **deny.toml** configuration restricts dependency licenses and sources
- **Minimal dependencies** to reduce attack surface
- **Pinned versions** in Cargo.lock for reproducibility

### Code Review and Testing

- **Security-focused code review** for all PRs touching safety validation
- **Property-based testing** with proptest for safety validator
- **Integration tests** for command execution paths
- **Manual security testing** before releases

---

## Disclosure Policy

### Coordinated Disclosure

We practice **coordinated disclosure**:

1. **Private investigation**: We investigate reported vulnerabilities privately
2. **Fix development**: We develop and test a fix without public disclosure
3. **Coordinated release**: We release the fix and coordinate disclosure with reporter
4. **Public announcement**: We publish a security advisory with credit to reporter

### Disclosure Timeline

- **Day 0**: Vulnerability reported privately
- **Day 2**: Acknowledgment sent to reporter
- **Day 7**: Assessment and severity classification
- **Day 30**: Target fix release for critical issues
- **Day 30+**: Public disclosure after fix is released

We will work with reporters to:
- Coordinate disclosure timing
- Provide credit in security advisories
- Give advance notice of fix releases
- Respect reporter preferences for anonymity

### Public Disclosure

Once a fix is released, we will:
- Publish a **security advisory** on GitHub
- Update **CHANGELOG.md** with security fix details
- Release a **patch version** with the fix
- Notify users via **release notes** and discussions

---

## Security Hall of Fame

We recognize security researchers who help improve cmdai:

### 2025

*No security vulnerabilities have been reported yet. Be the first!*

### Recognition

Security researchers who responsibly disclose vulnerabilities receive:
- **Public acknowledgment** in security advisories (unless anonymity is preferred)
- **Credit in release notes** for the fix version
- **Hall of Fame listing** in this document
- **Priority support** for future security research

**Qualities we value**:
- **Responsible disclosure**: Following our private reporting process
- **Clear communication**: Providing detailed, actionable reports
- **Collaboration**: Working with us to verify fixes
- **Patience**: Understanding that fixes take time to develop properly

---

## Contact

For security concerns:

- **Report vulnerabilities**: [GitHub Security Advisories](https://github.com/wildcard/cmdai/security)
- **Security questions**: [GitHub Discussions](https://github.com/wildcard/cmdai/discussions) (for non-sensitive topics)
- **Maintainer contact**: See repository maintainer profiles for direct contact

---

**cmdai takes security seriously.** Thank you for helping us keep our users safe.

---

*Last updated: 2025-10-03*
*Security policy version: 1.0*
