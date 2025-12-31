# Caro Personas & Jobs-To-Be-Done Framework

**Version**: 1.0.0
**Last Updated**: December 30, 2024
**Purpose**: Product messaging, onboarding, feature prioritization, and go-to-market strategy

---

## Executive Summary

This document defines 5 distinct personas and 12 Jobs-To-Be-Done for Caro, prioritized by pain severity, frequency, and differentiation potential. The analysis identifies **3 primary wedges** for MVP/Beta focus that leverage Caro's unique offline-first, safety-validated architecture.

**Core Product Truth**: Caro is the only production-ready AI shell assistant that works in air-gapped environments with independent, pattern-based safety validation—serving developers who can't use cloud-based AI tools.

---

## [PERSONAS]

### 1. The Production-Paranoid SRE

**Name**: Alex, Site Reliability Engineer

**Environment**:
- Production and staging Kubernetes clusters
- On-call rotation with high-stakes incident response
- Mix of cloud (AWS/GCP) and on-prem infrastructure
- Strict change management with audit requirements

**Core Goals**:
- Prevent outages caused by command mistakes
- Resolve incidents faster during on-call
- Maintain audit trail for compliance
- Reduce cognitive load when stressed/tired

**Constraints & Risks**:
- Commands in prod can cause cascading failures
- Sleep-deprived during incidents (higher error rate)
- Must justify every production change
- Limited time to verify command syntax

**Tools They Already Use**:
- kubectl, helm, terraform
- Datadog/Prometheus/Grafana
- PagerDuty, Slack
- bash/zsh with extensive aliases

**"Hire Caro When..."**:
"I need to run a complex command on a production system and can't afford a syntax error."

**Churn Triggers**:
- False positive safety blocks during incident response (blocking legitimate commands)
- Slow inference time (>3s) when under pressure
- Generated commands that don't work on their specific platform (BSD vs GNU)
- Requires network when they need it offline

---

### 2. The Air-Gapped Security Engineer

**Name**: Jordan, Security Engineer at Defense Contractor

**Environment**:
- Air-gapped secure development environment (SCIF)
- No internet access during work hours
- Strict approval process for new tools (6+ months)
- FedRAMP/NIST compliance requirements

**Core Goals**:
- Get AI productivity without security compromise
- Meet compliance requirements (no data exfiltration)
- Simplify tool approval process
- Work efficiently in restricted environment

**Constraints & Risks**:
- Zero tolerance for external network calls
- Every tool requires security review
- Binary must be auditable (no dynamic dependencies)
- Command history may be subject to audit

**Tools They Already Use**:
- Approved Linux utilities only
- Custom-compiled tools from source
- Air-gapped package mirrors
- Hardened shell environments

**"Hire Caro When..."**:
"I need AI command assistance in my air-gapped environment where nothing else works."

**Churn Triggers**:
- Any network call attempt (immediate disqualification)
- Telemetry that can't be disabled
- Requires dependencies not available in air-gapped environment
- Binary that can't pass security audit

---

### 3. The Cross-Platform DevOps Engineer

**Name**: Sam, Platform Engineer

**Environment**:
- Manages infrastructure across Mac (local dev), Linux (servers), and sometimes BSD
- Frequent context switching between environments
- CI/CD pipelines that must work everywhere
- Mix of cloud providers (AWS, GCP, Azure)

**Core Goals**:
- Write commands that work on any platform first time
- Stop fighting BSD vs GNU differences
- Build portable automation scripts
- Reduce time debugging platform-specific issues

**Constraints & Risks**:
- Commands that work on Mac fail on Linux servers
- CI pipeline failures from platform differences
- Time wasted on syntax differences (sed, find, xargs)
- Documentation assumes single platform

**Tools They Already Use**:
- Docker, Kubernetes
- Terraform, Ansible
- GitHub Actions, Jenkins
- Multiple shell types (bash, zsh, sh)

**"Hire Caro When..."**:
"I need a command that will work on both my Mac and our Linux production servers."

**Churn Triggers**:
- Generated commands don't account for platform differences
- No awareness of BSD vs GNU syntax variations
- Doesn't detect target environment correctly
- Commands fail in CI but work locally

---

### 4. The AI-Skeptical Tech Lead

**Name**: Morgan, Engineering Manager / Tech Lead

**Environment**:
- Responsible for team's production systems
- Has junior engineers running commands in production
- Experienced multiple AI-caused incidents (Claude Code, Gemini CLI)
- Must balance productivity with safety

**Core Goals**:
- Provide safety rails for the team
- Prevent AI hallucination disasters
- Enable AI productivity without risk
- Reduce on-call incidents from human error

**Constraints & Risks**:
- AI tools have failed them before (real incidents)
- Junior devs trust AI output too much
- Can't monitor every command before execution
- Liable for team's production mistakes

**Tools They Already Use**:
- GitHub Copilot (with concerns)
- Claude/ChatGPT for research (not CLI)
- Traditional shell without AI
- Custom runbooks and documentation

**"Hire Caro When..."**:
"I want AI command help for my team but need a safety layer that works regardless of what the AI outputs."

**Churn Triggers**:
- Safety validation is AI-based (not deterministic)
- Can't customize blocked patterns for their org
- No way to audit what patterns caught
- Safety layer can be bypassed easily

---

### 5. The Terminal-Learning Developer

**Name**: Casey, Mid-Level Developer

**Environment**:
- Uses terminal daily but doesn't feel expert
- Frequently Googles command syntax
- Works mostly in application code, not infrastructure
- Uses Mac for development, deploys to Linux

**Core Goals**:
- Stop context-switching to browser for command help
- Build terminal confidence through practice
- Learn proper command patterns
- Avoid embarrassing mistakes in shared terminals

**Constraints & Risks**:
- Doesn't know what they don't know
- Copy-pasting from Stack Overflow without understanding
- Afraid to run commands that might break things
- Limited time to read man pages

**Tools They Already Use**:
- VS Code with integrated terminal
- GitHub Copilot for code
- Stack Overflow, tldr pages
- Basic bash/zsh

**"Hire Caro When..."**:
"I know what I want to do but can't remember the exact command syntax."

**Churn Triggers**:
- Generated commands are overly complex
- No explanation of what command does
- Safety warnings without education
- Slower than just Googling

---

## [JTBD CATALOG]

### Job 1: Safe Production Command Execution

**Job Statement**: When I need to run a command on a production system, I want to validate it against known dangerous patterns, so I can execute with confidence and avoid catastrophic mistakes.

**Primary Persona(s)**: Production-Paranoid SRE, AI-Skeptical Tech Lead

**Trigger Moments**:
- Middle of incident response, sleep-deprived
- Running unfamiliar command for first time in prod
- Executing AI-suggested command
- Performing maintenance on critical systems

**Current Workaround**:
- Manual review, hoping to catch mistakes
- Running in staging first (slow, not always available)
- Second pair of eyes (not always available)
- Avoiding certain commands entirely

**Pain Severity**: 5/5
**Frequency**: Daily (for SREs), Weekly (for DevOps)
**Risk Level**: High (production impact)

**What "Done Well" Looks Like**:
- <50ms validation time, no perceived delay
- Zero false negatives (never miss a dangerous command)
- <5% false positive rate (don't block legitimate commands)
- Clear explanation of why command was flagged

**Product Implications**:
- Pattern database must cover edge cases (52 patterns minimum)
- Risk level gradation (warn vs block)
- Bypass option with confirmation for legitimate use
- Explanation text that educates, not just blocks

**Messaging Angle**: "The last line of defense between you and `rm -rf /`—independent of AI, always on."

---

### Job 2: Offline AI Assistance

**Job Statement**: When I'm working in an air-gapped or restricted network environment, I want AI-powered command generation, so I can have modern productivity tools without compromising security.

**Primary Persona(s)**: Air-Gapped Security Engineer

**Trigger Moments**:
- Starting work in SCIF or secure environment
- Working on classified systems
- During network outages
- Traveling without reliable internet

**Current Workaround**:
- Pre-downloading documentation
- Maintaining personal command cheatsheets
- Memorizing common patterns
- Going without AI assistance entirely

**Pain Severity**: 5/5
**Frequency**: Daily (for air-gapped workers)
**Risk Level**: Medium (productivity, not safety)

**What "Done Well" Looks Like**:
- Zero network calls after initial install
- Sub-2s inference on Apple Silicon
- Single binary with bundled model
- Works identically online and offline

**Product Implications**:
- Model must be embedded in binary or local cache
- No fallback to network (not even for telemetry)
- Offline-first architecture decisions
- Clear documentation for security auditors

**Messaging Angle**: "AI shell commands that work where nothing else can—100% local, 100% private."

---

### Job 3: Cross-Platform Command Generation

**Job Statement**: When I need to run a command on a different OS than my dev machine, I want platform-aware command generation, so I can avoid the BSD vs GNU syntax trap.

**Primary Persona(s)**: Cross-Platform DevOps Engineer, Production-Paranoid SRE

**Trigger Moments**:
- Writing CI pipeline that runs on Linux
- SSHing to production server from Mac
- Creating automation that works everywhere
- Debugging command that works locally but fails remotely

**Current Workaround**:
- Testing on each platform manually
- Using Docker for local Linux testing
- Memorizing platform differences
- Avoiding platform-specific features

**Pain Severity**: 4/5
**Frequency**: Weekly
**Risk Level**: Medium (CI failures, debugging time)

**What "Done Well" Looks Like**:
- Detects target platform (local vs specified)
- Generates POSIX-compliant commands by default
- Warns about platform-specific syntax
- Explains BSD vs GNU differences when relevant

**Product Implications**:
- Platform detection system (OS, shell, available commands)
- POSIX compliance checker
- Platform-specific command variants
- `--target` flag for explicit platform selection

**Messaging Angle**: "Commands that work everywhere—Mac, Linux, BSD—first time, every time."

---

### Job 4: AI Hallucination Protection

**Job Statement**: When an AI tool suggests a shell command, I want deterministic validation independent of the AI, so I can catch hallucinated dangerous commands before they execute.

**Primary Persona(s)**: AI-Skeptical Tech Lead, Production-Paranoid SRE

**Trigger Moments**:
- After Claude Code suggests a command
- When AI output looks plausible but uncertain
- Before executing any AI-generated command
- When reviewing junior dev's AI-assisted work

**Current Workaround**:
- Manual review (error-prone, time-consuming)
- Not using AI for commands at all
- Only using AI for non-destructive operations
- Hoping permission flags work (they don't always)

**Pain Severity**: 5/5
**Frequency**: Daily (for AI users)
**Risk Level**: High (real incidents documented)

**What "Done Well" Looks Like**:
- Pattern-based validation (not AI-based)
- Catches `rm -rf /` even when AI marks it "safe"
- Works as integration layer with other AI tools
- Clear audit trail of what was caught

**Product Implications**:
- Safety layer must be deterministic (regex/pattern-based)
- Integration with Claude Code, Cursor, etc.
- MCP server capability
- Logging/audit functionality

**Messaging Angle**: "AI tools will fail. Caro catches what AI hallucinations and permission flags can't."

---

### Job 5: Natural Language to Command Translation

**Job Statement**: When I know what I want to do but not the exact syntax, I want to describe it in natural language, so I can get a working command without Googling.

**Primary Persona(s)**: Terminal-Learning Developer, Cross-Platform DevOps

**Trigger Moments**:
- Can't remember flag combinations
- Needs complex pipe chain
- Unfamiliar with a specific tool
- Time pressure (doesn't want to read docs)

**Current Workaround**:
- Stack Overflow search
- ChatGPT/Claude (then copy-paste)
- `tldr` or `man` pages
- Ask colleague

**Pain Severity**: 3/5
**Frequency**: Daily
**Risk Level**: Low

**What "Done Well" Looks Like**:
- <2s response time
- Command works first try >90%
- Understands context (CWD, available tools)
- Suggests corrections if first attempt fails

**Product Implications**:
- Quality model for command generation
- Context awareness (shell type, OS, CWD)
- Feedback loop for correction
- Clear, copyable output

**Messaging Angle**: "Describe what you want. Get the command that works. No Googling required."

---

### Job 6: Team Safety Standardization

**Job Statement**: When I'm responsible for a team's production access, I want to deploy standardized safety patterns, so I can prevent mistakes without micromanaging individuals.

**Primary Persona(s)**: AI-Skeptical Tech Lead

**Trigger Moments**:
- Onboarding new team members
- After a production incident
- Security audit preparation
- Implementing compliance requirements

**Current Workaround**:
- Written runbooks (often outdated)
- Pre-commit hooks (limited scope)
- Peer review requirements (slow)
- Restricting production access (reduces productivity)

**Pain Severity**: 4/5
**Frequency**: Monthly (setup), Daily (benefit)
**Risk Level**: High (team-wide impact)

**What "Done Well" Looks Like**:
- Custom pattern configuration
- Organization-wide deployment
- Audit logs for compliance
- Easy to update patterns

**Product Implications**:
- Custom patterns via config file
- Enterprise deployment options
- Audit logging functionality
- Pattern distribution mechanism

**Messaging Angle**: "Safety rails for your whole team—no micromanagement required."

---

### Job 7: Incident Response Acceleration

**Job Statement**: When I'm responding to a production incident at 3 AM, I want to quickly generate diagnostic commands, so I can reduce MTTR without making sleep-deprived mistakes.

**Primary Persona(s)**: Production-Paranoid SRE

**Trigger Moments**:
- PagerDuty alert fires
- Under time pressure during outage
- Investigating unfamiliar system
- Need to run commands on multiple hosts

**Current Workaround**:
- Personal runbook documents
- Bash history search
- Googling under pressure (risky)
- Calling more senior colleague

**Pain Severity**: 5/5
**Frequency**: Weekly (for SREs on call)
**Risk Level**: High (time-critical, error-prone)

**What "Done Well" Looks Like**:
- Instant startup (<100ms)
- Fast inference (<2s)
- Works offline (network might be the problem)
- Context-aware (knows what cluster/env)

**Product Implications**:
- Performance optimization critical
- Local-first architecture
- Context retention between commands
- Integration with k8s/cloud context

**Messaging Angle**: "Your 3 AM incident companion—fast, local, safe."

---

### Job 8: Compliance-Ready Tool Approval

**Job Statement**: When I need to get a new tool approved for my secure environment, I want clear audit documentation, so I can fast-track security review and start using it sooner.

**Primary Persona(s)**: Air-Gapped Security Engineer

**Trigger Moments**:
- Evaluating new tools for secure environment
- Preparing security review documentation
- Responding to audit questions
- Justifying tool to security team

**Current Workaround**:
- Manual code review (time-consuming)
- Running tool in sandbox for analysis
- Writing justification documents
- Often just going without the tool

**Pain Severity**: 4/5
**Frequency**: Monthly
**Risk Level**: Medium (approval blockers)

**What "Done Well" Looks Like**:
- Clear network behavior documentation
- Open source code (auditable)
- No telemetry by default
- Security-focused documentation

**Product Implications**:
- Security whitepaper/documentation
- Clear telemetry controls
- SBOM (Software Bill of Materials)
- Audit-ready logging options

**Messaging Angle**: "Designed for security review—open source, no telemetry, audit-ready."

---

### Job 9: Command Learning and Education

**Job Statement**: When I run a generated command, I want to understand what it does, so I can learn terminal skills and make better decisions next time.

**Primary Persona(s)**: Terminal-Learning Developer

**Trigger Moments**:
- Running unfamiliar command
- Learning new tool
- Reviewing generated command before execution
- Teaching junior team member

**Current Workaround**:
- `man` pages (verbose, unfriendly)
- `tldr` (limited coverage)
- ChatGPT explanations (requires context switch)
- Trial and error

**Pain Severity**: 2/5
**Frequency**: Daily
**Risk Level**: Low

**What "Done Well" Looks Like**:
- Explain flag with `--explain` option
- Progressive disclosure (summary → details)
- Links to relevant documentation
- Examples of variations

**Product Implications**:
- `--explain` or `--verbose` mode
- Man page integration
- Help text generation
- Learning-oriented output mode

**Messaging Angle**: "Don't just get commands—understand them."

---

### Job 10: Safe Automation Script Generation

**Job Statement**: When I need to write a shell script for automation, I want safety-validated multi-command sequences, so I can create reliable automation without introducing risk.

**Primary Persona(s)**: Cross-Platform DevOps, Production-Paranoid SRE

**Trigger Moments**:
- Creating CI/CD pipeline scripts
- Writing maintenance automation
- Building deployment scripts
- Creating backup procedures

**Current Workaround**:
- Manual script writing with manual review
- Copy-paste from documentation
- Adapting existing scripts
- Using higher-level tools (Ansible, etc.)

**Pain Severity**: 3/5
**Frequency**: Weekly
**Risk Level**: Medium (automation amplifies mistakes)

**What "Done Well" Looks Like**:
- Multi-step command generation
- Validates entire script, not just individual commands
- Generates idempotent commands where possible
- POSIX-compliant by default

**Product Implications**:
- Script generation mode
- Batch validation API
- Idempotency detection/suggestions
- Script output format option

**Messaging Angle**: "Automation scripts that don't break things—validated before they run."

---

### Job 11: MCP Integration for AI Agents

**Job Statement**: When I'm using Claude or another AI agent, I want it to have safe shell command capabilities, so I can leverage AI for terminal tasks without risking disasters.

**Primary Persona(s)**: AI-Skeptical Tech Lead, Terminal-Learning Developer

**Trigger Moments**:
- Setting up Claude Code for team
- Configuring AI coding assistant
- Building AI-powered automation
- Wanting AI to execute commands safely

**Current Workaround**:
- Not allowing AI to run commands
- Manual copy-paste with review
- Accepting risk (and consequences)
- Using AI only for read-only operations

**Pain Severity**: 4/5
**Frequency**: Daily (for AI power users)
**Risk Level**: High (AI execution)

**What "Done Well" Looks Like**:
- MCP server that works with Claude Desktop
- Seamless integration with Claude Code
- Safety validation before any execution
- Clear logs of AI-executed commands

**Product Implications**:
- MCP server implementation
- Claude Code integration guide
- API for programmatic validation
- Execution sandboxing options

**Messaging Angle**: "Give Claude safe shell superpowers—with guardrails."

---

### Job 12: Quick Tool Installation

**Job Statement**: When I want to try Caro, I want one-command installation, so I can evaluate it immediately without complex setup.

**Primary Persona(s)**: All personas (critical for adoption)

**Trigger Moments**:
- First hearing about Caro
- Colleague recommendation
- Reading blog post/documentation
- Evaluating tools for team

**Current Workaround**:
- Complex multi-step installations
- Building from source
- Docker containers (overhead)
- Giving up on evaluation

**Pain Severity**: 4/5
**Frequency**: Once per user (but critical)
**Risk Level**: Low

**What "Done Well" Looks Like**:
- `curl ... | bash` installs in <30s
- Single binary, no dependencies
- Works immediately after install
- Clear next steps after installation

**Product Implications**:
- Install script with smart defaults
- Pre-built binaries for all platforms
- Zero configuration required for basic use
- Immediate "aha moment" example

**Messaging Angle**: "One command to install. One command to be amazed."

---

## [TOP 3 WEDGES]

### Wedge 1: AI Safety Layer (Hallucination Defense)

**Target Job**: Job 4 - AI Hallucination Protection
**Target Personas**: AI-Skeptical Tech Lead, Production-Paranoid SRE

**Why This Wins**:
- **Pain**: 5/5 (real documented incidents with Claude Code, Gemini CLI)
- **Frequency**: Daily (anyone using AI for commands)
- **Differentiation**: 5/5 (no competitor has pattern-based, deterministic safety independent of AI)

**The Pitch**: "AI tools will fail. Claude marked `rm -rf /` as 'safe' in our testing. Caro's independent safety layer caught it. We're the only tool where safety validation isn't another AI—it's deterministic patterns that can't hallucinate."

**Feature Focus**:
- 52+ pre-compiled dangerous patterns
- Works as MCP server/integration layer
- Catches what permission flags miss
- Audit trail for enterprise

**What to Defer**:
- Advanced natural language generation (good enough wins)
- Complex multi-step automation
- Fancy UI/output formatting

---

### Wedge 2: Air-Gapped/Offline Operation

**Target Job**: Job 2 - Offline AI Assistance
**Target Personas**: Air-Gapped Security Engineer

**Why This Wins**:
- **Pain**: 5/5 (no alternative exists for these users)
- **Frequency**: Daily (for target segment)
- **Differentiation**: 5/5 (only production-ready offline AI CLI tool)

**The Pitch**: "8 million developers can't use cloud AI due to security policies. Caro is the only AI shell assistant that works in air-gapped environments—bundled model, zero network calls, designed for security audit."

**Feature Focus**:
- Embedded model with binary
- Zero telemetry by default
- Security audit documentation
- SBOM and compliance docs

**What to Defer**:
- Cloud backend options
- Telemetry and analytics
- Features requiring network

---

### Wedge 3: Cross-Platform Command Reliability

**Target Job**: Job 3 - Cross-Platform Command Generation
**Target Personas**: Cross-Platform DevOps Engineer

**Why This Wins**:
- **Pain**: 4/5 (constant friction, time drain)
- **Frequency**: Weekly/Daily (every deploy)
- **Differentiation**: 4/5 (most AI tools ignore BSD vs GNU)

**The Pitch**: "The command that works on your Mac will work on your Linux server. Caro detects your target platform and generates POSIX-compliant commands that don't break in CI."

**Feature Focus**:
- Platform detection (OS, arch, shell)
- BSD vs GNU awareness
- POSIX compliance by default
- `--target` flag for explicit platforms

**What to Defer**:
- Windows-specific features
- Exotic shell support (fish, nushell)
- Platform-specific optimizations

---

## [ONBOARDING PATH]

### SRE/DevOps Persona (Wedge 1: Safety)

**Day 1 Flow**:
1. Install: `curl -fsSL https://caro.sh/install | bash`
2. Try danger detection: `caro "delete all files in root"` → see safety block
3. Try legitimate command: `caro "list pods restarting in last hour"`
4. Execute with confirmation: press `y` to run

**Example Session**:
```bash
$ curl -fsSL https://caro.sh/install | bash
$ caro "delete everything in this directory recursively"
# [BLOCKED] Dangerous pattern detected: recursive deletion
# Risk Level: CRITICAL
# Matched: "rm -rf" with root path

$ caro "find log files larger than 100MB modified today"
Generated command:
  find . -name "*.log" -size +100M -mtime 0 -ls
Execute? (y/N) y
```

**Aha Moment**: "It actually blocked that dangerous command before I could run it."

---

### Air-Gapped Engineer Persona (Wedge 2: Offline)

**Day 1 Flow**:
1. Download binary to USB: `curl -O https://github.com/wildcard/caro/releases/...`
2. Transfer to air-gapped machine
3. Run immediately (no network): `./caro "list all users with shell access"`
4. Verify no network calls: `strace -e network ./caro ...` (nothing)

**Example Session**:
```bash
# On internet-connected machine:
$ curl -fsSL https://github.com/.../caro-linux-amd64 > caro
$ sha256sum caro  # Verify checksum

# Transfer to air-gapped system via approved method
# On air-gapped machine:
$ chmod +x ./caro
$ ./caro "show all listening ports"
Generated command:
  ss -tlnp
# Works immediately, no network required
```

**Aha Moment**: "It actually works offline—no waiting for network timeouts, no errors."

---

### DevOps Engineer Persona (Wedge 3: Cross-Platform)

**Day 1 Flow**:
1. Install: `curl -fsSL https://caro.sh/install | bash`
2. Generate Mac command: `caro "find files modified in last hour"`
3. Specify target: `caro --target linux "find files modified in last hour"`
4. Notice the difference (BSD vs GNU syntax)

**Example Session**:
```bash
$ caro "find files changed in the last hour"
Generated command (macOS detected):
  find . -mtime -1h -type f
# Note: Uses BSD find syntax

$ caro --target linux "find files changed in the last hour"
Generated command (Linux target):
  find . -mmin -60 -type f
# Note: Uses GNU find syntax

Execute locally? (y/N) n
# Copy to your Linux server
```

**Aha Moment**: "It actually knows BSD vs GNU differences—this would have broken my CI."

---

### Terminal Learner Persona

**Day 1 Flow**:
1. Install: `curl -fsSL https://caro.sh/install | bash`
2. Ask for help: `caro "compress all images in downloads folder"`
3. Review command before running
4. Execute and see result

**Example Session**:
```bash
$ caro "compress all jpg images in downloads to 50% quality"
Generated command:
  find ~/Downloads -name "*.jpg" -exec convert {} -quality 50 {} \;

Requires: ImageMagick (convert command)
Risk Level: MODERATE (modifies files)
Execute? (y/N)
```

**Aha Moment**: "It told me what tool I need and warned me it modifies files."

---

## [OPEN QUESTIONS]

### Questions to Validate in Beta

1. **Safety False Positive Rate**
   - Q: What's the acceptable false positive rate before users disable safety?
   - Validation: Track bypass rate, exit surveys when users disable safety
   - Hypothesis: <5% false positive rate is tolerable

2. **Inference Latency Tolerance**
   - Q: What's the maximum acceptable wait time before users abandon?
   - Validation: A/B test latencies, measure abandonment
   - Hypothesis: <2s on Apple Silicon, <5s on CPU is acceptable

3. **Model Quality vs Size Tradeoff**
   - Q: Is 1.5B parameter model sufficient for command accuracy?
   - Validation: Track "first command works" rate, user corrections
   - Hypothesis: >85% accuracy on first try is sufficient

4. **Air-Gapped User Volume**
   - Q: How large is the air-gapped/restricted market really?
   - Validation: Download sources, survey, security-focused community outreach
   - Hypothesis: 10-15% of enterprise developers work in restricted environments

5. **Cross-Platform Value Perception**
   - Q: Do users recognize platform detection as a differentiator?
   - Validation: Feature usage tracking, survey questions
   - Hypothesis: DevOps users value this more than developers

6. **MCP Integration Demand**
   - Q: How many users want Caro as Claude/AI integration vs standalone?
   - Validation: Feature requests, integration usage vs CLI usage
   - Hypothesis: 30% will primarily use via MCP integration

7. **Enterprise Adoption Path**
   - Q: What's the path from individual user to team deployment?
   - Validation: Track multiple installs from same organization, enterprise inquiries
   - Hypothesis: Bottom-up adoption (individual → team → enterprise)

8. **Explanation Feature Usage**
   - Q: Do users want command explanations or just commands?
   - Validation: `--explain` flag usage, time spent reading output
   - Hypothesis: Learning developers want explanations, experienced users want speed

### Beta Program Design

**Cohorts to Recruit**:
1. **Production SREs** (10 users) - Safety validation feedback
2. **Air-gapped workers** (5 users) - Offline capability validation
3. **Cross-platform DevOps** (10 users) - Platform detection feedback
4. **AI tool users** (10 users) - MCP/integration interest validation
5. **Terminal learners** (15 users) - UX and education feedback

**Metrics to Track**:
- Commands generated per session
- Safety block rate (and bypass rate)
- Platform detection accuracy (user corrections)
- Time from install to first successful command
- Day 7 retention rate by persona
- NPS score by persona

**Feedback Channels**:
- In-CLI feedback command (`caro feedback`)
- Weekly survey for beta cohort
- GitHub issues for bugs
- Discord/Slack for community discussion

---

## Appendix: Competitive Positioning

| Feature | Caro | GitHub Copilot CLI | Warp AI | Generic LLM |
|---------|------|-------------------|---------|-------------|
| Works Offline | ✅ Full | ❌ | ❌ | ❌ |
| Independent Safety Layer | ✅ Pattern-based | ❌ | ❌ | ❌ |
| Air-gap Compatible | ✅ | ❌ | ❌ | ❌ |
| Platform Detection | ✅ | Partial | ❌ | ❌ |
| BSD/GNU Awareness | ✅ | ❌ | ❌ | Limited |
| Single Binary | ✅ | ❌ | ❌ | ❌ |
| Open Source | ✅ AGPL | ❌ | ❌ | Varies |
| MCP Integration | ✅ | ❌ | ❌ | ❌ |

---

*This document should be reviewed quarterly and updated based on beta feedback and market changes.*
