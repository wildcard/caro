# Beta Testing Plan - v1.1.0-beta

**Version**: v1.1.0-beta
**Testing Period**: Jan 13-17, 2026 (5 days)
**Target**: 3-5 beta testers
**Status**: Ready to begin

---

## Objectives

### Primary Goals
1. **Validate telemetry collection** in real-world usage
2. **Verify command generation quality** across diverse use cases
3. **Test privacy guarantees** - confirm zero PII in collected data
4. **Assess performance impact** - measure actual overhead
5. **Collect user feedback** on UX and features

### Success Criteria
- ✅ Zero PII found in exported telemetry data
- ✅ Command generation works for 80%+ of user queries
- ✅ No performance complaints (users don't notice overhead)
- ✅ Positive feedback on command quality and safety
- ✅ No critical bugs (P0) discovered

---

## Beta Tester Profiles

### Profile 1: Novice macOS User (bt_001)
**Background**:
- Basic command-line experience (cd, ls, cat)
- Uses macOS Terminal with default bash
- Wants to learn more powerful commands
- Safety-conscious (prefers confirmation prompts)

**Use Cases**:
- File management (find files, check sizes)
- Basic text searching (grep patterns)
- Simple process monitoring (CPU/memory)

**Expected Behavior**:
- Relies heavily on static matcher
- Appreciates clear explanations
- Values safety validation

---

### Profile 2: Expert Linux User (bt_002)
**Background**:
- Advanced shell user (bash/zsh expert)
- Daily CLI workflow for DevOps
- Comfortable with complex commands
- Values speed over safety prompts

**Use Cases**:
- System monitoring (disk, CPU, network)
- Log analysis (grep, awk, sed)
- Process management (kill, systemctl)
- File operations (find with complex filters)

**Expected Behavior**:
- Tests edge cases and complex queries
- May trigger LLM fallback more often
- Provides detailed technical feedback

---

### Profile 3: Data Scientist (bt_003)
**Background**:
- Python/R user, moderate shell experience
- Works with large datasets
- Needs file filtering and data prep commands
- Uses Jupyter notebooks, terminal for file ops

**Use Cases**:
- Find CSV/JSON files by size or date
- Text processing (grep patterns in data files)
- Disk space management (large datasets)
- Batch file operations

**Expected Behavior**:
- Natural language queries (not shell expert)
- May request commands outside static patterns
- Values explanations and examples

---

### Profile 4: SRE/DevOps Engineer (bt_005)
**Background**:
- Kubernetes/Docker expert
- Multi-server management
- Heavy automation user
- Needs fast, reliable commands

**Use Cases**:
- Container operations (docker ps, logs)
- Kubernetes debugging (kubectl commands)
- Network diagnostics (netstat, ss, lsof)
- Log aggregation and analysis

**Expected Behavior**:
- Complex compound queries
- Tests DevOps-specific patterns
- Performance-sensitive

---

### Profile 5: Security Engineer (bt_007)
**Background**:
- Security-focused workflows
- Auditing and compliance
- Needs safe, validated commands
- Works with sensitive data

**Use Cases**:
- File permission audits
- User/group analysis
- Security scanning patterns
- Log analysis for anomalies

**Expected Behavior**:
- Tests safety validation extensively
- Provides feedback on risk levels
- Values privacy guarantees

---

## Testing Protocol

### Phase 1: Setup (Day 1 - Jan 13)

**Installation**:
```bash
# Install from binary
curl -fsSL https://caro.sh/install.sh | sh

# Or build from source
git clone https://github.com/anthropics/caro.git
cd caro
cargo build --release
sudo cp target/release/caro /usr/local/bin/
```

**Initial Configuration**:
```bash
# First run - consent prompt will appear
caro "show largest files"

# Configure preferences
caro config set safety_level paranoid
caro config set default_shell bash
caro config set log_level info

# Verify telemetry settings
caro telemetry status
```

**Baseline Test**:
```bash
# Test all website claim commands
caro "show largest files"
caro "files modified in the last 7 days"
caro "list all python files"
caro "files larger than 10MB"
```

**Expected**: All 4 commands should work via static matcher.

---

### Phase 2: Natural Usage (Days 2-4 - Jan 14-16)

**Instructions to Beta Testers**:

1. **Use caro naturally in your daily workflow**
   - Don't force usage - use when it feels natural
   - Try commands you'd normally look up or write manually
   - Experiment with natural language phrasings

2. **Track your experiences**
   - Note when commands worked perfectly
   - Note when commands failed or were wrong
   - Note when explanations were helpful or confusing

3. **Test edge cases** (optional, for advanced users)
   - Complex compound queries
   - Platform-specific commands
   - Unusual phrasings

4. **Export telemetry data** (end of day):
   ```bash
   caro telemetry export ~/caro-telemetry-$(date +%Y%m%d).json
   ```

---

### Phase 3: Feedback Collection (Day 5 - Jan 17)

**Feedback Form** (to be sent to beta testers):

#### 1. Command Generation Quality
- How often did caro generate the correct command? (0-100%)
- How often did you need to modify the generated command?
- Were the explanations clear and helpful?

#### 2. Safety & Privacy
- Did you feel safe executing the generated commands?
- Did you review the exported telemetry data? Any concerns?
- Were safety warnings appropriate (not too many/few)?

#### 3. Performance
- Did you notice any slowdown when using caro?
- How fast did commands generate? (instant/fast/slow)
- Any performance issues?

#### 4. User Experience
- What did you like most about caro?
- What frustrated you most?
- What features are missing?

#### 5. Specific Issues
- List any bugs or errors encountered
- List any commands that failed
- List any confusing behaviors

---

## Data Collection

### Telemetry Data
**From Beta Testers**:
```bash
# Testers export daily
caro telemetry export ~/caro-telemetry-$(date +%Y%m%d).json
```

**Analysis**:
1. **Privacy Validation**:
   - Manually inspect all exported JSON files
   - Verify zero PII present (no commands, paths, emails, IPs)
   - Confirm session IDs are hashed and rotating

2. **Performance Analysis**:
   - Calculate average duration_ms for static vs embedded
   - Identify timeout patterns
   - Check error categories

3. **Usage Patterns**:
   - Count backend usage (static vs embedded)
   - Track success/failure rates
   - Identify common error categories

---

### User Feedback
**Collection Methods**:
1. Daily check-ins (Slack/email)
2. End-of-week feedback form
3. Optional follow-up calls for detailed feedback

**Key Metrics**:
- Command success rate (self-reported)
- User satisfaction (1-5 scale)
- Performance perception (acceptable/unacceptable)
- Privacy concerns (yes/no/maybe)
- Likelihood to recommend (NPS score)

---

## Test Scenarios

### Scenario 1: File Management
**Commands to Test**:
```bash
caro "find all PDF files larger than 10MB"
caro "files modified today"
caro "show disk usage by folder"
caro "list python files from last week"
```

**Expected Results**:
- All should work via static matcher
- Fast generation (<50ms)
- Correct file filters applied

---

### Scenario 2: System Monitoring
**Commands to Test**:
```bash
caro "show top memory-consuming processes"
caro "check which process is using port 8080"
caro "show CPU usage"
caro "list all listening ports"
```

**Expected Results**:
- Mix of static matcher and LLM fallback
- Platform-appropriate commands (BSD vs GNU)
- Correct flags and options

---

### Scenario 3: Text Processing
**Commands to Test**:
```bash
caro "find all ERROR logs from today"
caro "search for TODO comments in python files"
caro "count lines in all javascript files"
caro "find files containing 'password'"
```

**Expected Results**:
- Some may need LLM fallback
- Correct grep/find patterns
- Proper quoting and escaping

---

### Scenario 4: Safety Edge Cases
**Commands to Test**:
```bash
caro "delete all temporary files"  # Should warn
caro "remove old backups"           # Should warn
caro "kill all python processes"    # Should warn
caro "change permissions on config" # Should warn
```

**Expected Results**:
- Safety validation triggers
- Clear risk explanations
- User confirmation required

---

## Privacy Audit Protocol

### Manual Inspection Checklist

For each exported telemetry file:

**Step 1: Load and Parse**:
```bash
cat caro-telemetry-20260114.json | jq '.'
```

**Step 2: Check for PII Patterns**:
```bash
# File paths
jq -r '.[] | select(.event_type | contains("/"))' telemetry.json

# Email addresses
jq -r '.[] | select(.event_type | contains("@"))' telemetry.json

# IP addresses (public)
jq -r '.[] | select(.event_type | test("\\b(?!192\\.168|10\\.|172\\.(1[6-9]|2[0-9]|3[01]))[0-9]{1,3}\\.[0-9]{1,3}\\.[0-9]{1,3}\\.[0-9]{1,3}\\b"))' telemetry.json

# Environment variables
jq -r '.[] | select(.event_type | contains("PATH=") or contains("HOME="))' telemetry.json

# API keys
jq -r '.[] | select(.event_type | test("api[_-]?key|token|secret"))' telemetry.json
```

**Step 3: Validate Session IDs**:
```bash
# All session IDs should be:
# - 16 characters
# - Hexadecimal
# - Change daily (not same across days)
jq -r '.[].session_id' telemetry.json | sort -u
```

**Step 4: Verify Event Structure**:
```bash
# CommandGeneration events should have:
# - backend (string)
# - duration_ms (number)
# - success (boolean)
# - error_category (string or null)
# NO command text, NO prompt text
jq '.[] | select(.event_type.type == "command_generation")' telemetry.json
```

**Expected**: Zero matches for PII patterns, all events have correct structure.

---

## Bug Reporting

### Bug Report Template

```markdown
**Title**: [Component] Brief description

**Severity**: P0 (critical) | P1 (high) | P2 (medium) | P3 (low)

**Environment**:
- OS: macOS 14.2 / Linux Ubuntu 22.04
- Shell: bash 5.2 / zsh 5.9
- Caro version: 1.1.0-beta

**Steps to Reproduce**:
1. Run command: `caro "..."`
2. Observe behavior
3. ...

**Expected Behavior**:
What should happen

**Actual Behavior**:
What actually happened

**Error Message** (if any):
```
paste error here
```

**Workaround** (if found):
How to work around the issue

**Telemetry Data** (if relevant):
```json
paste relevant event
```
```

---

## Success Metrics

### Quantitative Metrics

**From Telemetry Data**:
| Metric | Target | Notes |
|--------|--------|-------|
| Command success rate | 80%+ | From telemetry events |
| Static matcher hit rate | 60%+ | Website claims + variants |
| Average generation time | <100ms | Static: <10ms, LLM: <2s |
| Error rate | <20% | timeout, parse_error, etc. |
| Safety blocks | <5% | Commands blocked by safety |

**From User Feedback**:
| Metric | Target | Notes |
|--------|--------|-------|
| User satisfaction | 4.0/5.0 | Average rating |
| NPS score | 40+ | Likelihood to recommend |
| Privacy concerns | <10% | Users with PII concerns |
| Performance acceptable | 90%+ | Users find speed acceptable |

---

### Qualitative Feedback

**Key Questions**:
1. What use cases work really well?
2. What use cases fail or frustrate?
3. Is the safety validation helpful or annoying?
4. Are explanations clear?
5. Would you use this daily?

---

## Risk Mitigation

### Potential Issues & Mitigations

**Issue 1: PII Leak in Telemetry**
- **Mitigation**: Manual inspection of all exported files
- **Severity**: P0 - blocks release
- **Response**: Identify leak source, fix validation, re-test

**Issue 2: Command Generation Failures**
- **Mitigation**: Collect failed queries, expand static patterns
- **Severity**: P1 - affects UX but not blocking
- **Response**: Add patterns in v1.1.1

**Issue 3: Performance Complaints**
- **Mitigation**: Profile slow cases, optimize hot paths
- **Severity**: P1 - affects UX
- **Response**: Investigate with flamegraph, optimize

**Issue 4: Safety False Positives**
- **Mitigation**: Review blocked commands, tune patterns
- **Severity**: P2 - annoying but safe
- **Response**: Adjust safety patterns in v1.1.1

**Issue 5: Critical Bugs (Crashes, Data Loss)**
- **Mitigation**: Hotfix immediately
- **Severity**: P0 - blocks release
- **Response**: Fix, rebuild, re-deploy to testers

---

## Timeline

### Week of Jan 13-17, 2026

**Monday (Jan 13)** - Setup Day
- Recruit 3-5 beta testers
- Send installation instructions
- Conduct baseline tests
- Set up feedback channel (Slack/Discord)

**Tuesday-Thursday (Jan 14-16)** - Active Testing
- Testers use caro in daily workflow
- Daily check-ins for issues
- Collect telemetry exports daily
- Begin privacy inspection

**Friday (Jan 17)** - Feedback Collection
- Send feedback form to all testers
- Conduct follow-up calls (optional)
- Complete privacy audit of all data
- Analyze telemetry for patterns

**Weekend (Jan 18-19)** - Analysis & Planning
- Synthesize feedback
- Prioritize bugs (P0, P1, P2, P3)
- Create fix plan for P0/P1 issues
- Document lessons learned

---

## Post-Beta Actions

### Week of Jan 20-22 (Bug Fixes)

**P0 Bugs** (must fix before release):
- Privacy leaks
- Critical crashes
- Data corruption
- Security vulnerabilities

**P1 Bugs** (should fix before release):
- Command generation failures for common queries
- Performance issues
- Safety validation problems

**P2/P3 Bugs** (defer to v1.1.1):
- Minor UX issues
- Edge case failures
- Feature requests

---

### Release Decision (Jan 23)

**Go/No-Go Checklist**:
- [ ] Zero PII found in telemetry data
- [ ] All P0 bugs fixed and verified
- [ ] Command generation success rate >80%
- [ ] User satisfaction >4.0/5.0
- [ ] Performance acceptable to 90%+ users
- [ ] No critical security issues

**If GO**:
- Tag v1.1.0-beta on Jan 24
- Publish release notes
- Deploy to crates.io
- Announce to community

**If NO-GO**:
- Fix remaining P0 issues
- Conduct focused re-test
- Re-evaluate on Jan 27
- Adjust timeline if needed

---

## Appendix: Beta Tester Recruitment

### Ideal Candidates
- Active CLI users (daily terminal usage)
- Diverse platforms (macOS, Linux)
- Diverse skill levels (novice to expert)
- Privacy-conscious (will review telemetry)
- Good communicators (detailed feedback)

### Recruitment Channels
- GitHub issues/discussions
- Twitter/social media
- Developer communities
- Internal Anthropic team
- Friends of the project

### Incentives
- Early access to features
- Influence on roadmap
- Acknowledgment in release notes
- Potential beta tester badge/swag

---

## Success Definition

**Beta test is successful if**:
1. ✅ Zero PII found in exported telemetry
2. ✅ >80% command success rate
3. ✅ >4.0/5.0 user satisfaction
4. ✅ Performance acceptable to >90% users
5. ✅ All P0 bugs identified and fixed

**Result**: Confident to release v1.1.0-beta to wider audience.
