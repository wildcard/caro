# GitHub Repository Security Settings

This document provides a comprehensive guide for configuring GitHub repository settings to maintain BSD/GNU-level security standards for the caro project.

## Overview

As a security-critical CLI tool that generates and executes shell commands, caro requires strict repository security controls to prevent unauthorized releases, code injection, and supply chain attacks.

## Required Security Settings

### Branch Protection Rules

**Protected Branch**: `main`

Navigate to: **Settings → Branches → Add branch protection rule**

#### Required Settings

**Pattern**: `main`

- [x] **Require a pull request before merging**
  - [x] Require approvals: **1** (minimum)
  - [x] Dismiss stale pull request approvals when new commits are pushed
  - [x] Require review from Code Owners (when CODEOWNERS file exists)
  - [x] Restrict who can dismiss pull request reviews: **Maintainers only**
  - [ ] Allow specified actors to bypass required pull requests (leave unchecked)

- [x] **Require status checks to pass before merging**
  - [x] Require branches to be up to date before merging
  - Required status checks:
    - `test (ubuntu-latest)` - Linux tests
    - `test (macos-latest)` - macOS tests
    - `test (windows-latest)` - Windows tests
    - `clippy` - Linter checks
    - `fmt` - Format checks
    - `security-audit` - cargo audit

- [x] **Require conversation resolution before merging**
  - All review comments must be resolved

- [x] **Require signed commits**
  - All commits must be GPG signed

- [x] **Require linear history**
  - Prevent merge commits, enforce rebase/squash

- [x] **Require deployments to succeed before merging** (if using deployments)

- [x] **Lock branch**
  - [ ] Make the branch read-only (leave unchecked for normal development)

- [x] **Do not allow bypassing the above settings**
  - [x] Include administrators

- [x] **Restrict who can push to matching branches**
  - Add: **Maintainers** (verified maintainer accounts only)
  - Do NOT add individual contributor accounts

#### Additional Recommended Settings

- [x] **Require deployments to succeed before merging** (if applicable)
- [x] **Restrict creations** (prevent creation of matching branches)
- [x] **Restrict deletions** (prevent deletion of protected branch)
- [x] **Allow force pushes**: **Specify who can force push** → Nobody
- [x] **Allow deletions**: Disabled

### Tag Protection Rules

**Protected Tags**: `v*.*.*`

Navigate to: **Settings → Tags → Add tag protection rule**

- **Pattern**: `v*.*.*` (matches all version tags like v1.0.0)
- Only repository maintainers can create or delete tags matching this pattern
- Tags are immutable once created

**Additional Tag Patterns** (optional but recommended):
- `v*` - Protect all version-related tags
- `release-*` - Protect release-related tags

### Repository Security Settings

Navigate to: **Settings → Security**

#### Vulnerability Reporting

- [x] **Enable private vulnerability reporting**
  - Allows security researchers to privately report vulnerabilities
  - Notifications sent to repository maintainers
  - Creates private security advisories

#### Security Policies

- [x] **Add SECURITY.md** (already in repository)
  - Provides vulnerability disclosure guidelines
  - Lists security contact information
  - Defines security update policy

#### Dependency Graph

Navigate to: **Settings → Security & analysis**

- [x] **Dependency graph**: Enabled
  - Automatically tracks all dependencies
  - Shows dependency relationships
  - Required for Dependabot

#### Dependabot Alerts

- [x] **Dependabot alerts**: Enabled
  - Automatic detection of vulnerable dependencies
  - Email notifications to maintainers
  - Creates security advisories automatically

#### Dependabot Security Updates

- [x] **Dependabot security updates**: Enabled
  - Automatically creates PRs for vulnerable dependencies
  - Only patches security vulnerabilities
  - Respects semantic versioning

#### Dependabot Version Updates (Optional)

- [ ] **Dependabot version updates**: Consider enabling with config
  - Create `.github/dependabot.yml` to configure update schedule
  - Can be noisy for active development
  - Recommended for stable releases only

**Example `.github/dependabot.yml`**:
```yaml
version: 2
updates:
  - package-ecosystem: "cargo"
    directory: "/"
    schedule:
      interval: "weekly"
    open-pull-requests-limit: 5
    reviewers:
      - "maintainers"
    labels:
      - "dependencies"
      - "security"
```

#### Secret Scanning

- [x] **Secret scanning**: Enabled (automatic for public repos)
  - Scans for accidentally committed secrets
  - Alerts maintainers immediately
  - Supports 200+ token patterns

#### Code Scanning (GitHub Advanced Security)

For private repositories (requires GitHub Advanced Security):

- [x] **Code scanning**: Configure CodeQL
  - Automatic vulnerability detection
  - Security query packs
  - Pull request integration

**Setup CodeQL** (`.github/workflows/codeql.yml`):
```yaml
name: "CodeQL"

on:
  push:
    branches: [ "main" ]
  pull_request:
    branches: [ "main" ]
  schedule:
    - cron: '0 0 * * 1'  # Weekly on Monday

jobs:
  analyze:
    name: Analyze
    runs-on: ubuntu-latest
    permissions:
      security-events: write
    steps:
      - uses: actions/checkout@v4
      - uses: github/codeql-action/init@v3
        with:
          languages: 'rust'
      - uses: github/codeql-action/autobuild@v3
      - uses: github/codeql-action/analyze@v3
```

### Repository Access & Permissions

Navigate to: **Settings → Collaborators and teams**

#### Access Levels

**Maintainers (Admin)**:
- Full repository access
- Can modify settings
- Can manage releases
- Can add/remove collaborators
- **Requires**: 2FA enabled, GPG key on file, verified email

**Trusted Contributors (Write)**:
- Can push to non-protected branches
- Can create PRs
- Can review PRs
- Cannot merge to main directly
- **Requires**: 2FA enabled (recommended)

**General Contributors (Read)**:
- Can fork repository
- Can create issues and discussions
- Can submit PRs from forks
- No special requirements

#### Team Structure (if using GitHub Organizations)

**@caro/maintainers**:
- Core maintainers with release authority
- Admin access to repository
- Required 2FA and GPG signing

**@caro/trusted-contributors**:
- Regular contributors with proven track record
- Write access to repository
- Can approve non-security PRs

**@caro/security-team**:
- Handles security disclosures
- Access to private security advisories
- Can coordinate vulnerability responses

### GitHub Actions Settings

Navigate to: **Settings → Actions → General**

#### Actions Permissions

- [x] **Allow all actions and reusable workflows**
  - Required for CI/CD workflows
  - Alternative: "Allow select actions and reusable workflows" with allowlist

- [x] **Require approval for first-time contributors**
  - Prevents malicious PR workflow execution
  - Maintainers must approve workflow runs

#### Workflow Permissions

- **Default workflow permissions**: Read repository contents and packages
- [ ] **Read and write permissions**: Disabled
- [x] **Allow GitHub Actions to create and approve pull requests**: Only for Dependabot

**Token Permissions** (configure in each workflow):
```yaml
permissions:
  contents: read        # Read code
  pull-requests: write  # Comment on PRs
  security-events: write # CodeQL results
```

#### Artifact and Log Retention

- **Default retention**: 90 days (default)
- Adjust if needed for compliance requirements

### Secrets and Variables

Navigate to: **Settings → Secrets and variables → Actions**

#### Repository Secrets

**Required Secrets**:

1. **`CARGO_REGISTRY_TOKEN`**
   - crates.io publish token
   - Scope: `publish-update` only
   - Rotation: Every 90 days
   - Owner: Verified maintainer account with 2FA

**Optional Secrets** (if using remote backends):
- `VLLM_API_KEY` - For vLLM testing
- `OLLAMA_API_KEY` - For Ollama testing

#### Environment Secrets (Recommended)

Create environment: **production**

Navigate to: **Settings → Environments → New environment**

- Environment name: `production`
- Required reviewers: **Maintainers** (select 1-2 maintainers)
- Wait timer: 0 minutes (or set delay for critical releases)
- Deployment branches: `main` only

**Environment Secrets**:
- `CARGO_REGISTRY_TOKEN` (production token, separate from CI)

**Benefits**:
- Requires manual approval before publishing to crates.io
- Prevents accidental releases from CI
- Audit trail for all releases

### Webhooks and Integrations

Navigate to: **Settings → Webhooks**

#### Security Scanning Integrations (Optional)

Consider integrating:
- **Snyk**: Additional dependency scanning
- **Trivy**: Container and dependency scanning
- **Socket**: Supply chain attack detection

### Repository Settings

Navigate to: **Settings → General**

#### Features

- [x] **Issues**: Enabled
- [x] **Projects**: Enabled (optional)
- [x] **Discussions**: Enabled (for community questions)
- [x] **Wiki**: Disabled (use docs/ directory instead)
- [x] **Sponsorships**: Optional

#### Pull Requests

- [x] **Allow squash merging**: Enabled
  - Default message: Pull request title
- [x] **Allow merge commits**: Disabled
- [x] **Allow rebase merging**: Enabled
- [x] **Always suggest updating pull request branches**: Enabled
- [x] **Allow auto-merge**: Disabled (manual review required)
- [x] **Automatically delete head branches**: Enabled

#### Archives

- [ ] **Include Git LFS objects in archives**: Disabled (not using LFS)

#### Dangerous Zone

- [ ] **Allow forking**: Enabled (open source project)
- [ ] **Transfer ownership**: Disabled (protect against accidental transfer)
- [ ] **Archive repository**: Disabled
- [ ] **Delete repository**: Disabled

## Security Audit Checklist

Use this checklist to verify security settings are correctly configured:

### Branch Protection
- [ ] Main branch has protection enabled
- [ ] Require 1+ PR reviews
- [ ] Require status checks to pass
- [ ] Require signed commits
- [ ] Require linear history
- [ ] Include administrators in restrictions
- [ ] Restrict who can push (maintainers only)

### Tag Protection
- [ ] Version tags (v*.*.*) are protected
- [ ] Only maintainers can create/delete tags

### Security Features
- [ ] Private vulnerability reporting enabled
- [ ] SECURITY.md file present
- [ ] Dependabot alerts enabled
- [ ] Dependabot security updates enabled
- [ ] Secret scanning enabled (automatic)
- [ ] CodeQL scanning configured (if private repo)

### Access Control
- [ ] Repository has defined maintainer team
- [ ] All maintainers have 2FA enabled
- [ ] Maintainers have GPG keys configured
- [ ] Contributor access follows least privilege

### GitHub Actions
- [ ] Require approval for first-time contributors
- [ ] Workflow permissions are minimal (read-only default)
- [ ] CARGO_REGISTRY_TOKEN stored as secret
- [ ] Production environment configured with approvals

### Repository Settings
- [ ] Merge commits disabled (squash/rebase only)
- [ ] Auto-delete head branches enabled
- [ ] Auto-merge disabled (manual approval required)

## Monitoring and Maintenance

### Daily
- Monitor Dependabot alerts
- Review security scanning results
- Check for unusual repository activity

### Weekly
- Review open PRs for security implications
- Check CI/CD workflow status
- Verify no secrets in recent commits

### Monthly
- Audit repository access list
- Review and update dependencies
- Check for new security advisories

### Quarterly
- Rotate CARGO_REGISTRY_TOKEN
- Review and update security settings
- Audit GitHub Actions workflows
- Test incident response procedures

### Annually
- Comprehensive security audit
- Review all third-party integrations
- Update security documentation
- Train new maintainers on security procedures

## Incident Response

### If Secret is Leaked

1. **Immediate Actions** (within 1 hour):
   - Revoke the leaked secret (crates.io, API keys, etc.)
   - Rotate the secret with a new value
   - Update GitHub secret with new value
   - Check git history for secret exposure

2. **Investigation** (within 24 hours):
   - Identify when secret was committed
   - Determine if secret was used maliciously
   - Review access logs for unauthorized usage
   - Document timeline and impact

3. **Remediation** (within 48 hours):
   - Remove secret from git history (git filter-branch or BFG)
   - Force push cleaned history (if necessary)
   - Notify affected users if secret was used
   - Update security documentation

4. **Prevention**:
   - Add pattern to .gitignore
   - Configure pre-commit hooks
   - Update developer documentation
   - Conduct security training

### If Unauthorized Code is Merged

1. **Immediate Actions**:
   - Revert the malicious commit
   - Lock the affected branch
   - Audit all PRs from same author
   - Revoke contributor access

2. **Investigation**:
   - Analyze malicious code intent
   - Check if code was executed in CI/CD
   - Review all releases since merge
   - Identify compromised accounts

3. **Remediation**:
   - Yank affected releases from crates.io
   - Publish security advisory
   - Release patched version
   - Notify users of compromise

4. **Prevention**:
   - Increase PR review requirements
   - Enable CodeQL scanning
   - Add additional branch protections
   - Review contributor permissions

## Additional Resources

- [GitHub Security Best Practices](https://docs.github.com/en/code-security)
- [crates.io Security Policy](https://crates.io/policies)
- [Rust Security Response WG](https://www.rust-lang.org/governance/wgs/wg-security-response)
- [OWASP Software Component Verification Standard](https://owasp.org/www-project-software-component-verification-standard/)

## Verification

To verify all security settings are correctly configured, maintainers should:

1. Review this document quarterly
2. Run security audit checklist
3. Test branch protection (try to push to main directly - should fail)
4. Test tag protection (try to delete a version tag - should fail)
5. Verify PR approval requirements (create test PR - should require review)
6. Check secret scanning (commit test secret - should be detected)

## Questions

For questions about repository security settings:
- Open a discussion on GitHub
- Contact maintainers via private channel
- Email security contact for sensitive matters

**Remember**: Security is everyone's responsibility, but configuration is limited to verified maintainers only.
