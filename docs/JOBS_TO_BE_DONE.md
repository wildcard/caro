# Caro: Jobs to Be Done
## Product Requirements Document for Angel Investors

**Document Version:** 2.0
**Last Updated:** January 2025
**Product Status:** Production-Ready (v1.0.3 on crates.io)
**Website:** [caro.sh](https://caro.sh)

---

## Executive Summary

Caro is the **only production-ready AI shell assistant** that works in air-gapped environments with **independent, pattern-based safety validation**. We serve the 8 million developers who cannot use cloud-based AI tools due to security policies.

**Core Innovation:** While AI tools like Claude Code and GitHub Copilot CLI generate commands using probabilistic models, Caro adds a **deterministic safety layer** that operates independently of AI inference. This catches dangerous commands that AI models may approve—including patterns like `rm -rf /` that probabilistic systems can miss.

**Market Opportunity:** $1.44B serviceable market of enterprise developers in restricted environments, with a clear path to $2M ARR in 18 months.

---

## The Problem We Solve

### Market Pain Points

| Pain Point | Severity | Affected Users | Current Solution |
|------------|----------|----------------|------------------|
| AI hallucinations cause production disasters | Critical | 25M+ CLI users | Hope and manual review |
| Cloud AI tools blocked in secure environments | Critical | 8M+ enterprise devs | No AI assistance at all |
| Commands work on Mac, fail on Linux servers | High | 12M+ DevOps engineers | Trial and error, documentation |
| Junior devs run dangerous commands in prod | High | Teams with mixed experience | Code review, restricted access |
| Constant context-switching to look up syntax | Medium | All terminal users | Stack Overflow, man pages |

### Why This Matters Now

1. **AI adoption is accelerating** - Enterprise AI spend: $50B (2024) to $150B (2027)
2. **Security concerns persist** - 73% of enterprises cannot use cloud-based AI coding assistants
3. **AI tools lack safety layers** - Probabilistic models can approve dangerous commands
4. **Local AI is production-ready** - 1.5B parameter models achieve 87% task accuracy
5. **Terminal usage is increasing** - 78% of developers use terminal daily (vs 71% in 2020)

---

## Target Personas

### Persona 1: The Production-Paranoid SRE

**Profile:** Alex, Site Reliability Engineer at Series C Fintech

**Environment:**
- On-call rotation with high-stakes incident response
- Production Kubernetes clusters across cloud providers
- Strict change management with audit requirements

**Core Motivation:**
> "I need to run complex commands on production systems without syntax errors that could cause cascading failures."

**Key Jobs to Be Done:**
1. Validate commands against dangerous patterns before production execution
2. Generate diagnostic commands during 3 AM incidents
3. Catch AI-suggested commands that could cause damage

**Pain Quantification:**
- Pain Level: 5/5 (production impact)
- Frequency: Daily
- Current Cost: Average incident costs $10K-$100K; preventable command errors cause 15% of outages

**What Success Looks Like:**
- <50ms validation time (no perceived delay)
- Zero false negatives (never miss a dangerous command)
- <5% false positive rate
- Clear explanation of why commands are flagged

---

### Persona 2: The Air-Gapped Security Engineer

**Profile:** Jordan, Security Engineer at Defense Contractor

**Environment:**
- SCIF (Sensitive Compartmented Information Facility)
- Zero internet access during work hours
- 6+ month approval process for new tools
- FedRAMP/NIST compliance requirements

**Core Motivation:**
> "I need AI productivity without compromising security. Every other AI tool requires internet."

**Key Jobs to Be Done:**
1. Get AI-powered command generation in air-gapped environments
2. Pass security review with auditable, single-binary distribution
3. Maintain compliance with zero telemetry and data exfiltration

**Pain Quantification:**
- Pain Level: 5/5 (no alternative exists)
- Frequency: Daily
- Current Cost: 30% productivity loss vs. colleagues with AI access

**What Success Looks Like:**
- Zero network calls (verifiable with strace)
- Single binary, auditable source code
- Works identically online and offline
- Security review documentation included

**Unique Value:** Caro is the **only AI tool** that works in air-gapped environments. This is a market of ~8 million developers with zero competitive options.

---

### Persona 3: The Cross-Platform DevOps Engineer

**Profile:** Sam, Platform Engineer at SaaS Startup

**Environment:**
- Mac for local development
- Linux servers in production
- CI/CD pipelines that must work everywhere
- Constant context-switching between platforms

**Core Motivation:**
> "I write commands on Mac that break on Linux servers. The BSD vs GNU differences waste hours of my time."

**Key Jobs to Be Done:**
1. Generate commands that work on any platform first time
2. Stop fighting BSD vs GNU syntax differences
3. Build portable automation scripts

**Pain Quantification:**
- Pain Level: 4/5 (constant friction)
- Frequency: Weekly (every deploy)
- Current Cost: 2-3 hours/week debugging platform differences

**What Success Looks Like:**
- Detects target platform automatically
- Generates POSIX-compliant commands by default
- Warns about platform-specific syntax
- `--target linux` flag for explicit platform selection

**Common Platform Traps Caro Solves:**

| Task | macOS (BSD) | Linux (GNU) |
|------|-------------|-------------|
| Find files modified recently | `find . -mtime -1h` | `find . -mmin -60` |
| Extended regex in sed | `sed -E 's/pattern/replace/'` | `sed -r 's/pattern/replace/'` |
| Check listening ports | `lsof -i -P \| grep LISTEN` | `ss -tlnp` |
| Get file modification time | `stat -f %m file.txt` | `stat -c %Y file.txt` |

---

### Persona 4: The AI-Skeptical Tech Lead

**Profile:** Morgan, Engineering Manager responsible for team's production systems

**Environment:**
- Junior engineers running AI-generated commands in production
- Has experienced multiple AI-caused incidents
- Must balance productivity with safety
- Liable for team's production mistakes

**Core Motivation:**
> "AI tools have failed us before. I need safety rails for my team that work regardless of what the AI outputs."

**Key Jobs to Be Done:**
1. Deploy organization-wide safety patterns without micromanaging
2. Catch AI hallucinations with deterministic validation
3. Provide audit trails for compliance

**Pain Quantification:**
- Pain Level: 5/5 (team-wide risk)
- Frequency: Daily (any time AI tools are used)
- Current Cost: Real documented incidents costing $10K-$500K each

**What Success Looks Like:**
- Pattern-based validation (not another AI)
- Custom patterns configurable per organization
- Audit logs showing what patterns matched
- Can't be bypassed by clever prompting

**The Math Problem:**
If AI is 99.9% accurate and team runs 1,000 commands/day:
- 1,000 commands x 0.1% failure = **1 dangerous command/day**
- Over a year: **365 potential incidents**
- Caro provides deterministic layer: 50 patterns x 0 hallucination = **0 bypasses**

---

### Persona 5: The Terminal-Learning Developer

**Profile:** Casey, Mid-Level Developer expanding terminal skills

**Environment:**
- Uses terminal daily but doesn't feel expert
- Frequently Googles command syntax
- Works mostly in application code
- Uses Mac, deploys to Linux

**Core Motivation:**
> "I know what I want to do but can't remember the exact command syntax. I want to stop context-switching to Stack Overflow."

**Key Jobs to Be Done:**
1. Describe intent in natural language, get working command
2. Build terminal confidence through practice
3. Learn proper command patterns

**Pain Quantification:**
- Pain Level: 3/5 (productivity friction)
- Frequency: Multiple times daily
- Current Cost: 15-20 minutes/day context-switching to documentation

**What Success Looks Like:**
- <2s response time
- Command works first try >90% of time
- Understands context (shell type, OS, CWD)
- Optional explanations with `--explain` flag

---

## Jobs to Be Done Catalog

### Job 1: Safe Production Command Execution

**Job Statement:**
> When I need to run a command on a production system, I want to validate it against known dangerous patterns, so I can execute with confidence and avoid catastrophic mistakes.

**Trigger Moments:**
- Middle of incident response, sleep-deprived
- Running unfamiliar command for first time in prod
- Executing AI-suggested command
- Performing maintenance on critical systems

**Current Workaround:**
- Manual review (hoping to catch mistakes)
- Running in staging first (slow, not always available)
- Second pair of eyes (not always available at 3 AM)

**Pain Severity:** 5/5
**Frequency:** Daily for SREs, weekly for DevOps
**Risk Level:** High (production impact)

**Product Solution:**
- 50 predefined dangerous patterns with <50ms validation
- Risk level gradation: Critical (red), High (orange), Moderate (yellow), Safe (green)
- Bypass option with explicit confirmation for legitimate use
- Clear explanation of why command was flagged

**Acceptance Criteria:**
```
GIVEN a production engineer running a command
WHEN the command matches a dangerous pattern
THEN Caro blocks execution with specific warning
AND suggests safer alternative
AND allows explicit override with confirmation
```

**Example Scenario:**

```bash
$ caro "delete all log files to free disk space"

Generated command:
  find /var/log -name "*.log" -type f -delete

CRITICAL: Recursive delete in system directory without age filter
Suggestion: Use -mtime +7 to only remove logs older than 7 days

Safer alternative:
  find /var/log -name "*.log" -type f -mtime +7 -delete

Execute original? (requires explicit 'yes-i-understand'): _
```

---

### Job 2: Offline AI Assistance

**Job Statement:**
> When I'm working in an air-gapped or restricted network environment, I want AI-powered command generation, so I can have modern productivity tools without compromising security.

**Trigger Moments:**
- Starting work in SCIF or secure environment
- Working on classified systems
- During network outages
- Traveling without reliable internet

**Current Workaround:**
- Pre-downloading documentation
- Maintaining personal command cheatsheets
- Memorizing common patterns
- Going without AI assistance entirely

**Pain Severity:** 5/5 (no alternative exists)
**Frequency:** Daily for air-gapped workers
**Risk Level:** Medium (productivity)

**Product Solution:**
- Bundled Qwen2.5-Coder-1.5B-Instruct model
- Zero network calls after initial install
- Sub-2s inference on Apple Silicon (GPU)
- Single binary distribution (~50MB with model)

**Acceptance Criteria:**
```
GIVEN an engineer in an air-gapped environment
WHEN they run caro with any prompt
THEN command generates without network access
AND strace shows zero network syscalls
AND response time is <5s on standard hardware
```

**Verification Commands:**
```bash
# Verify zero network calls
strace -e network ./caro "list files"
# Result: No network syscalls

# Verify offline operation
sudo iptables -A OUTPUT -j DROP  # Block all network
./caro "find large files"
# Result: Works perfectly
```

---

### Job 3: Cross-Platform Command Generation

**Job Statement:**
> When I need to run a command on a different OS than my dev machine, I want platform-aware command generation, so I can avoid the BSD vs GNU syntax trap.

**Trigger Moments:**
- Writing CI pipeline that runs on Linux
- SSHing to production server from Mac
- Creating automation that works everywhere
- Debugging command that works locally but fails remotely

**Current Workaround:**
- Testing on each platform manually
- Using Docker for local Linux testing
- Memorizing platform differences
- Avoiding platform-specific features

**Pain Severity:** 4/5
**Frequency:** Weekly
**Risk Level:** Medium (CI failures, debugging time)

**Product Solution:**
- Automatic platform detection (OS, arch, shell)
- Platform-specific rules for 10+ common command differences
- POSIX compliance mode for maximum portability
- `--target` flag for explicit platform selection

**Acceptance Criteria:**
```
GIVEN a developer on macOS targeting Linux
WHEN they run `caro --target linux "find files changed today"`
THEN output uses GNU syntax (mmin) not BSD (mtime)
AND includes platform annotation
```

**Example:**
```bash
# On macOS
$ caro "find files changed in last hour"
Generated (macOS): find . -mtime -1h -type f

$ caro --target linux "find files changed in last hour"
Generated (Linux): find . -mmin -60 -type f
Note: Using GNU find syntax for Linux compatibility
```

---

### Job 4: AI Hallucination Protection

**Job Statement:**
> When an AI tool suggests a shell command, I want deterministic validation independent of the AI, so I can catch hallucinated dangerous commands before they execute.

**Trigger Moments:**
- After Claude Code suggests a command
- When AI output looks plausible but uncertain
- Before executing any AI-generated command
- When reviewing junior dev's AI-assisted work

**Current Workaround:**
- Manual review (error-prone, time-consuming)
- Not using AI for commands at all
- Only using AI for non-destructive operations
- Hoping permission flags work (they don't always)

**Pain Severity:** 5/5
**Frequency:** Daily for AI users
**Risk Level:** High (real incidents documented)

**Product Solution:**
- Pattern-based validation (regex, not AI)
- Operates after AI generation, before execution
- Catches patterns AI models miss
- MCP server integration for Claude/other agents

**Why Pattern-Based Beats Permission-Based:**

| Aspect | Permission Flags | Pattern Matching |
|--------|-----------------|------------------|
| Determinism | AI can convince itself | Same input = same result |
| Auditability | "AI said it was safe" | Exact pattern that matched |
| Hallucination risk | Can be jailbroken | Regex doesn't make things up |
| Speed | API round-trip | <50ms local validation |

**Command Patterns Caro Catches:**

| Risk Category | Example Pattern | Caro Response |
|---------------|-----------------|---------------|
| Recursive deletion | `rm -rf` in project/system dirs | Block + suggest safer alternative |
| Piped remote execution | `curl \| bash` from URLs | Block + warn about untrusted sources |
| Privilege escalation | `chmod -R 777 /` | Block + require explicit confirmation |

---

### Job 5: Natural Language to Command Translation

**Job Statement:**
> When I know what I want to do but not the exact syntax, I want to describe it in natural language, so I can get a working command without Googling.

**Trigger Moments:**
- Can't remember flag combinations
- Needs complex pipe chain
- Unfamiliar with a specific tool
- Time pressure (doesn't want to read docs)

**Current Workaround:**
- Stack Overflow search
- ChatGPT/Claude (then copy-paste)
- `tldr` or `man` pages
- Ask colleague

**Pain Severity:** 3/5
**Frequency:** Multiple times daily
**Risk Level:** Low

**Product Solution:**
- Embedded LLM for natural language understanding
- Context awareness (shell type, OS, CWD, available commands)
- Multiple model options (82MB to 3.5GB)
- Sub-2s response on Apple Silicon

**Natural Language Examples:**

| Intent | Generated Command |
|--------|-------------------|
| "find large files" | `find . -type f -size +100M -exec ls -lh {} \;` |
| "show disk usage by directory" | `du -sh */ \| sort -rh \| head -10` |
| "list all open ports" | `ss -tlnp` (Linux) / `lsof -i -P` (macOS) |
| "find duplicate files" | `find . -type f -exec md5sum {} \; \| sort \| uniq -d -w32` |
| "compress old logs" | `find /var/log -name "*.log" -mtime +7 -exec gzip {} \;` |

---

### Job 6: Team Safety Standardization

**Job Statement:**
> When I'm responsible for a team's production access, I want to deploy standardized safety patterns, so I can prevent mistakes without micromanaging individuals.

**Trigger Moments:**
- Onboarding new team members
- After a production incident
- Security audit preparation
- Implementing compliance requirements

**Current Workaround:**
- Written runbooks (often outdated)
- Pre-commit hooks (limited scope)
- Peer review requirements (slow)
- Restricting production access (reduces productivity)

**Pain Severity:** 4/5
**Frequency:** Monthly setup, daily benefit
**Risk Level:** High (team-wide impact)

**Product Solution:**
- Custom patterns via config file
- Organization-wide deployment via shared config
- Audit logging for compliance
- Risk level configuration per team

**Team Configuration Example:**
```toml
# ~/.config/caro/config.toml (shared via repo)
[safety]
level = "strict"

# Organization-specific patterns
[[safety.custom_patterns]]
pattern = "deploy.*production.*--force"
risk_level = "Critical"
description = "Force deploy to production"

[[safety.custom_patterns]]
pattern = "kubectl delete namespace production"
risk_level = "Critical"
description = "Delete production namespace"

# Allowlist safe operations
[safety.allowlist]
patterns = ["kubectl get", "docker ps", "terraform plan"]

[logging]
enabled = true
path = "/var/log/caro/commands.log"
format = "json"
```

---

### Job 7: Incident Response Acceleration

**Job Statement:**
> When I'm responding to a production incident at 3 AM, I want to quickly generate diagnostic commands, so I can reduce MTTR without making sleep-deprived mistakes.

**Trigger Moments:**
- PagerDuty alert fires
- Under time pressure during outage
- Investigating unfamiliar system
- Need to run commands on multiple hosts

**Current Workaround:**
- Personal runbook documents
- Bash history search
- Googling under pressure (risky)
- Calling more senior colleague

**Pain Severity:** 5/5
**Frequency:** Weekly for SREs
**Risk Level:** High (time-critical, error-prone)

**Product Solution:**
- Instant startup (<100ms)
- Fast inference (<2s)
- Works offline (network might be the problem)
- Context-aware generation

**Incident Response Examples:**

**Alert: API Response Time > 5s**
```bash
$ caro "check current load"
uptime && free -h && df -h

$ caro "find slow processes"
ps aux --sort=-%cpu | head -20

$ caro "check recent error logs"
journalctl -u api-server --since "5 minutes ago" --priority=err
```

**Alert: Disk Usage > 95%**
```bash
$ caro "what's using disk space"
du -sh /* 2>/dev/null | sort -rh | head -10

$ caro "find large files"
find / -type f -size +100M -exec ls -lh {} \; 2>/dev/null | sort -k5 -rh

$ caro "find old temp files to clean"
find /tmp -type f -atime +7 -exec ls -lh {} \;
```

---

## Technical Implementation Status

### Completed Features (v1.0.3)

| Feature | Status | Details |
|---------|--------|---------|
| CLI with flexible argument parsing | SHIPPED | `-p/--prompt`, stdin, trailing args |
| Natural language to shell commands | SHIPPED | 2-iteration agentic loop |
| Platform detection | SHIPPED | OS, arch, shell, distribution |
| Safety validation | SHIPPED | 50 patterns, 4 risk levels |
| Embedded MLX backend | SHIPPED | Apple Silicon GPU acceleration |
| Embedded CPU backend | SHIPPED | Cross-platform via Candle |
| Ollama backend | SHIPPED | Remote inference support |
| vLLM backend | SHIPPED | Enterprise inference |
| Configuration management | SHIPPED | TOML-based |
| Multiple output formats | SHIPPED | JSON, YAML, Plain |
| Command execution | SHIPPED | Shell-aware with confirmation |
| Published to crates.io | SHIPPED | `cargo install caro` |
| Pre-built binaries | SHIPPED | All major platforms |
| Website | SHIPPED | caro.sh |

### Model Options

| Model | Size | Use Case | Inference Time |
|-------|------|----------|----------------|
| SmolLM 135M | 82MB | CI/Testing | <1s |
| Qwen2.5-Coder 0.5B | 352MB | Fast inference | <1s |
| TinyLlama 1.1B | 669MB | Balanced | 1-2s |
| Qwen2.5-Coder 1.5B | 1.1GB | Default (recommended) | 1-2s |
| Phi-2 2.7B | 1.5GB | Higher quality | 2-3s |
| Mistral 7B | 3.5GB | Maximum quality | 3-5s |

### Safety Patterns (50)

**Critical (Red) - Blocked by Default:**
- `rm -rf /` - System destruction
- `:(){:|:&};:` - Fork bomb
- `dd if=/dev/zero of=/dev/sda` - Disk wipe
- `mkfs.ext4 /dev/sda1` - Filesystem format
- `> /etc/passwd` - Critical file truncation

**High (Orange) - Requires Confirmation:**
- `chmod -R 777 /` - System-wide permission change
- `rm -rf ~/*` - Home directory wipe
- `curl | bash` - Piped remote execution
- Privilege escalation patterns

**Moderate (Yellow) - Warning:**
- Package removal operations
- Network configuration changes
- Service restarts

---

## Competitive Positioning

| Feature | Caro | GitHub Copilot CLI | Warp AI | Generic LLM |
|---------|------|-------------------|---------|-------------|
| Works Offline | YES | No | No | No |
| Independent Safety Layer | YES (pattern-based) | No | No | No |
| Air-gap Compatible | YES | No | No | No |
| Platform Detection | YES | Partial | No | No |
| BSD/GNU Awareness | YES | No | No | Limited |
| Single Binary | YES | No | No | No |
| Open Source | YES (AGPL) | No | No | Varies |
| MCP Integration | YES | No | No | No |
| GPU Acceleration | YES (MLX) | Cloud | Cloud | Varies |

**Key Differentiators:**

1. **Only Offline AI CLI Tool** - First-mover advantage in local inference CLI space
2. **Deterministic Safety Layer** - Pattern-based, not AI-based validation
3. **Cross-Platform Intelligence** - BSD vs GNU awareness unique in market
4. **Enterprise Distribution** - Single binary, no dependencies, audit-ready

---

## Market Sizing

### TAM/SAM/SOM

| Metric | Calculation | Value |
|--------|-------------|-------|
| TAM (Total Addressable Market) | 25M+ terminal users x $120/year | $3B |
| SAM (Serviceable Available Market) | 12M+ enterprise/restricted devs x $120/year | $1.44B |
| SOM (3-year target) | 240K users x $50 avg | $12M ARR |

### Target Segments

| Segment | Size | Priority | Entry Strategy |
|---------|------|----------|----------------|
| Air-gapped enterprise devs | 8M | Primary | Only tool that works |
| Security-conscious teams | 4M | Primary | Independent safety layer |
| Cross-platform DevOps | 12M | Secondary | Platform intelligence |
| Terminal learners | 10M | Secondary | Productivity improvement |

---

## Business Model

### Three-Tier Pricing

| Tier | Price | Features |
|------|-------|----------|
| **Community** | Free | Basic generation, embedded model, community patterns |
| **Pro** | $14.99/month | Multiple models, custom workflows, priority updates |
| **Enterprise** | $79/seat/month (min 50) | Custom deployment, SSO, compliance, dedicated support |

### Unit Economics (Target)

| Metric | Community | Pro | Enterprise |
|--------|-----------|-----|------------|
| CAC | $0 (organic) | $25 | $500 |
| LTV | $0 | $180 (1yr) | $2,844 (3yr) |
| LTV:CAC | N/A | 7:1 | 5.7:1 |
| Gross Margin | N/A | 90% | 85% |

---

## Investment Ask

### Seeking: $2.5M Seed Round

**Use of Funds:**

| Category | Amount | Allocation |
|----------|--------|------------|
| Engineering & Product | $1.375M (55%) | 3 senior Rust engineers, ML engineer, security engineer |
| Sales & Marketing | $625K (25%) | VP Sales, 2 Enterprise AEs, marketing |
| Compliance & Legal | $300K (12%) | SOC2 Type II, FedRAMP prep, legal |
| Operations | $200K (8%) | Finance, HR, buffer |

### 18-Month Milestones

| Month | Target |
|-------|--------|
| 6 | 5,000 MAU, 100 Pro subscribers, 3 enterprise pilots, SOC2 Type I |
| 12 | 10,000 MAU, 300 Pro, 10 enterprise ($40K ACV), $500K ARR, SOC2 Type II |
| 18 | 25,000 MAU, 500 Pro, 30 enterprise ($50K ACV), $2M ARR, Series A ready |

---

## Appendix: User Stories by Persona

### SRE User Stories

```
As an SRE on-call at 3 AM,
I want to generate diagnostic commands quickly,
So that I can identify the root cause before my MTTR target is exceeded.

Acceptance Criteria:
- Command generation in <2s
- Works offline (network might be down)
- Safety validation prevents making incident worse
- History of commands for post-incident review
```

```
As an SRE running commands in production,
I want dangerous patterns blocked with clear explanations,
So that I don't accidentally cause a cascading failure.

Acceptance Criteria:
- Pattern matching in <50ms
- Clear explanation of why blocked
- Override path for legitimate commands
- Audit log for compliance
```

### Security Engineer User Stories

```
As a security engineer in an air-gapped environment,
I want AI command generation without network access,
So that I can have productivity tools without security compromise.

Acceptance Criteria:
- Zero network syscalls (verifiable with strace)
- Single binary distribution
- Works immediately after file copy
- No telemetry or data exfiltration
```

```
As a security engineer evaluating tools,
I want clear audit documentation,
So that I can fast-track security review and approval.

Acceptance Criteria:
- Open source code (auditable)
- SBOM available
- Security policy documented
- SHA256 checksums for all binaries
```

### DevOps User Stories

```
As a DevOps engineer working across platforms,
I want platform-aware command generation,
So that commands work in CI pipelines without modification.

Acceptance Criteria:
- Detects current platform automatically
- --target flag for explicit platform
- Warns about platform-specific syntax
- POSIX compliance mode available
```

```
As a DevOps engineer writing automation,
I want to generate safe script sequences,
So that I can build reliable automation without introducing risk.

Acceptance Criteria:
- Validates entire script
- Suggests idempotent alternatives
- POSIX-compliant output
- Platform-specific variants
```

### Tech Lead User Stories

```
As a tech lead responsible for team safety,
I want to deploy org-wide safety patterns,
So that I can prevent mistakes without micromanaging.

Acceptance Criteria:
- Shared config file via repo
- Custom patterns per organization
- Audit logging for compliance
- Risk level configuration
```

```
As a tech lead evaluating AI tools,
I want deterministic safety validation,
So that I can trust the safety layer won't hallucinate.

Acceptance Criteria:
- Pattern-based (not AI-based)
- Same input = same output
- Explicit pattern matching audit
- Can't be prompt-jailbroken
```

---

## Document History

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 1.0 | Dec 2024 | Product Team | Initial JTBD framework |
| 2.0 | Jan 2025 | Product Team | Investor-focused PRD format, updated metrics |

---

*This document is confidential and intended for potential investors. For product documentation, see [caro.sh](https://caro.sh).*
