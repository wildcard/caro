# Telemetry Testing & QA Plan

**Date**: 2026-01-08
**Status**: Ready for Execution
**Estimated Time**: 3-4 hours

---

## Test Environment Setup

### Prerequisites
```bash
# 1. Build latest code
cargo build --release

# 2. Clear any existing telemetry data
rm -rf ~/Library/Application\ Support/caro/telemetry/

# 3. Remove existing config (for first-run testing)
mv ~/.config/caro/config.toml ~/.config/caro/config.toml.backup

# 4. Verify binary location
which caro
# Or use: ./target/release/caro
```

---

## Test Suite 1: First-Run Experience (30 min)

### Test 1.1: First-Run Consent Prompt
**Expected**: User sees telemetry consent prompt on first run

```bash
caro "list files"
```

**Verify**:
- [ ] Consent prompt appears
- [ ] Prompt explains what is collected
- [ ] Prompt explains what is NOT collected
- [ ] Clear accept/decline options
- [ ] Default behavior (accept/decline)

**Manual Check**:
- Read prompt text carefully
- Verify clarity and transparency
- Note any confusing language

### Test 1.2: Accept Telemetry
**Action**: Run and accept consent

```bash
caro "list files"
# Accept when prompted
```

**Verify**:
- [ ] Command executes normally
- [ ] Config file created with telemetry.enabled = true
- [ ] Database file created at expected location

```bash
# Check config
cat ~/.config/caro/config.toml | grep -A 5 telemetry

# Check database
ls -lh ~/Library/Application\ Support/caro/telemetry/events.db
```

### Test 1.3: Decline Telemetry
**Setup**: Reset config

```bash
rm ~/.config/caro/config.toml
```

**Action**: Run and decline consent

```bash
caro "list files"
# Decline when prompted
```

**Verify**:
- [ ] Command executes normally
- [ ] Config file shows telemetry.enabled = false
- [ ] No database file created OR database exists but empty

---

## Test Suite 2: Event Collection (45 min)

### Test 2.1: SessionStart Event
**Setup**: Ensure telemetry enabled

```bash
# Verify enabled
caro telemetry status

# If disabled, enable it
caro config set telemetry.enabled true
```

**Action**: Run a command

```bash
caro "list files"
```

**Verify**: Check events

```bash
caro telemetry show --limit 1
```

**Expected Output**:
- [ ] Event type: SessionStart
- [ ] Version matches CARGO_PKG_VERSION
- [ ] Platform shows correct OS (darwin/linux)
- [ ] Shell type detected
- [ ] Backend list includes "static" and "embedded"

**Manual Check**:
- Verify timestamp is recent
- Verify session ID format (16 hex chars)

### Test 2.2: CommandGeneration Event (Static Matcher)
**Action**: Trigger static matcher

```bash
caro "files modified today"
```

**Verify**:

```bash
caro telemetry show --limit 2
```

**Expected**:
- [ ] Event type: CommandGeneration
- [ ] Backend: "static"
- [ ] Success: true
- [ ] Duration < 100ms (very fast)
- [ ] No error_category

### Test 2.3: CommandGeneration Event (LLM Backend)
**Action**: Trigger LLM fallback (complex query)

```bash
caro "find all Python files larger than 1MB modified in the last week"
```

**Verify**:

```bash
caro telemetry show --limit 1
```

**Expected**:
- [ ] Event type: CommandGeneration
- [ ] Backend: "embedded"
- [ ] Success: true
- [ ] Duration > 500ms (LLM inference)
- [ ] No error_category

### Test 2.4: CommandGeneration Event (Error)
**Action**: Trigger an error (intentionally)

```bash
# This should trigger an error if backend not available
# or we can simulate by disabling backends temporarily

# For now, document that errors are tracked when they occur
```

**Note**: Error testing requires specific failure scenarios. Document that error telemetry is implemented and will be validated in production.

---

## Test Suite 3: CLI Commands (30 min)

### Test 3.1: Show Command
**Action**: View telemetry events

```bash
caro telemetry show
```

**Verify**:
- [ ] Events displayed with colors
- [ ] Event numbers shown [1], [2], etc.
- [ ] Timestamps formatted correctly
- [ ] Event details readable
- [ ] Different event types distinguished visually

### Test 3.2: Show with Limit
**Action**: Limit displayed events

```bash
caro telemetry show --limit 5
```

**Verify**:
- [ ] Only 5 events shown
- [ ] Total count matches expectation

### Test 3.3: Export Command
**Action**: Export telemetry data

```bash
caro telemetry export --output /tmp/telemetry-test.json
```

**Verify**:
- [ ] File created at /tmp/telemetry-test.json
- [ ] File contains valid JSON
- [ ] Events match what's shown in `show` command

```bash
# Validate JSON
cat /tmp/telemetry-test.json | jq '.'

# Count events
cat /tmp/telemetry-test.json | jq 'length'
```

### Test 3.4: Status Command
**Action**: Check telemetry status

```bash
caro telemetry status
```

**Verify**:
- [ ] Shows enabled/disabled status
- [ ] Shows telemetry level (normal/minimal/verbose)
- [ ] Shows air-gapped mode status
- [ ] Shows endpoint URL
- [ ] Shows storage location
- [ ] Shows queued event count
- [ ] Shows helpful commands

### Test 3.5: Clear Command
**Action**: Clear telemetry data

```bash
# First, check current count
caro telemetry show | grep "Total:"

# Clear with force (no prompt)
caro telemetry clear --force

# Verify empty
caro telemetry show
```

**Verify**:
- [ ] Success message shown
- [ ] Event count now 0
- [ ] `show` command displays "No events" message

---

## Test Suite 4: Privacy Validation (45 min)

**CRITICAL**: This is the most important test suite

### Test 4.1: No Command Content
**Action**: Generate multiple commands with sensitive content

```bash
caro "find files containing password"
caro "list files in /Users/myname/Documents/secrets"
caro "search for API_KEY in environment variables"
caro "show me my ssh keys"
```

**Verify**: Check telemetry doesn't contain sensitive data

```bash
caro telemetry show

# Export and search
caro telemetry export --output /tmp/privacy-test.json

# Search for sensitive patterns
cat /tmp/privacy-test.json | grep -i "password"
cat /tmp/privacy-test.json | grep -i "secret"
cat /tmp/privacy-test.json | grep -i "api_key"
cat /tmp/privacy-test.json | grep -i "ssh"
cat /tmp/privacy-test.json | grep "/Users/"
cat /tmp/privacy-test.json | grep "/home/"
```

**Expected**:
- [ ] No command text appears
- [ ] No file paths appear
- [ ] No environment variables appear
- [ ] No sensitive keywords appear

**Pass Criteria**: ALL searches return empty

### Test 4.2: Session ID Anonymity
**Action**: Generate session across multiple days

```bash
# Check today's session ID
caro telemetry show --limit 1 | grep "Session:"

# Note the session ID format: 16 hex characters
# Verify it's not identifiable (no username, hostname, etc.)
```

**Verify**:
- [ ] Session ID is 16 hex characters
- [ ] Session ID doesn't contain username
- [ ] Session ID doesn't contain hostname
- [ ] Session ID changes daily (test tomorrow)

### Test 4.3: Event Data Inspection
**Action**: Manually inspect all event types

```bash
caro telemetry export --output /tmp/full-audit.json

# Pretty print for manual review
cat /tmp/full-audit.json | jq '.'
```

**Manual Review Checklist**:
- [ ] No natural language prompts
- [ ] No generated commands
- [ ] No file paths
- [ ] No environment variables
- [ ] No IP addresses
- [ ] No email addresses
- [ ] No API keys or tokens
- [ ] No personally identifiable information

**Document**: Take notes on what IS collected:
- Event types
- Timestamps
- Session IDs
- Durations (ms)
- Backend names
- Success/failure booleans
- Error categories (generic)
- Platform/OS/shell (generic)
- Version numbers

---

## Test Suite 5: Configuration (30 min)

### Test 5.1: Disable Telemetry
**Action**:

```bash
caro config set telemetry.enabled false
```

**Verify**:

```bash
caro telemetry status
# Should show DISABLED

# Run a command
caro "list files"

# Check if events were collected
caro telemetry show
# Should show same events as before (no new events)
```

**Expected**:
- [ ] Status shows DISABLED
- [ ] No new events collected
- [ ] Commands still work normally

### Test 5.2: Re-enable Telemetry
**Action**:

```bash
caro config set telemetry.enabled true
caro "list files"
caro telemetry show --limit 1
```

**Verify**:
- [ ] Status shows ENABLED
- [ ] New events collected

### Test 5.3: Air-Gapped Mode
**Action**:

```bash
caro config set telemetry.air_gapped true
caro telemetry status
```

**Verify**:
- [ ] Status shows air-gapped mode ON
- [ ] Note: "(manual export only)"
- [ ] Events still collected locally
- [ ] Export command still works

```bash
caro "list files"
caro telemetry export --output /tmp/airgapped.json
```

### Test 5.4: Telemetry Levels
**Action**: Test different levels

```bash
# Minimal
caro config set telemetry.level minimal
caro telemetry status

# Normal (default)
caro config set telemetry.level normal
caro telemetry status

# Verbose
caro config set telemetry.level verbose
caro telemetry status
```

**Verify**:
- [ ] Config accepts all three levels
- [ ] Status displays current level
- [ ] Note: Event filtering by level not implemented yet (future work)

---

## Test Suite 6: Performance (30 min)

### Test 6.1: Startup Overhead
**Goal**: Verify <5ms overhead target

**Action**: Benchmark with telemetry on/off

```bash
# With telemetry
time caro --version

# Disable telemetry
caro config set telemetry.enabled false

# Without telemetry
time caro --version

# Re-enable
caro config set telemetry.enabled true
```

**Measure**: Run each 10 times, compare average

```bash
# More precise benchmark
for i in {1..10}; do
  /usr/bin/time -p caro --version 2>&1 | grep real
done
```

**Expected**:
- [ ] Overhead < 5ms
- [ ] No noticeable delay

### Test 6.2: Storage Size
**Action**: Check database size after many events

```bash
# Generate 100 events
for i in {1..100}; do
  caro "list files" > /dev/null 2>&1
done

# Check database size
ls -lh ~/Library/Application\ Support/caro/telemetry/events.db

# Check event count
caro telemetry show | tail -1
```

**Expected**:
- [ ] Database size reasonable (<1MB for 100 events)
- [ ] No performance degradation

---

## Test Suite 7: Edge Cases (30 min)

### Test 7.1: Concurrent Executions
**Action**: Run multiple caro instances

```bash
# Terminal 1
caro "find large files" &

# Terminal 2
caro "list directories" &

# Wait for completion
wait

# Check telemetry
caro telemetry show --limit 4
```

**Verify**:
- [ ] All events captured
- [ ] No database corruption
- [ ] No race conditions

### Test 7.2: Database Permissions
**Action**: Test with restricted permissions

```bash
# Make database read-only (simulate failure)
chmod 444 ~/Library/Application\ Support/caro/telemetry/events.db

# Run command
caro "list files"

# Should still work (telemetry fails gracefully)

# Restore permissions
chmod 644 ~/Library/Application\ Support/caro/telemetry/events.db
```

**Verify**:
- [ ] Command executes despite telemetry failure
- [ ] Warning logged (check with --verbose)
- [ ] User not impacted

### Test 7.3: Missing Database
**Action**: Delete database mid-session

```bash
rm ~/Library/Application\ Support/caro/telemetry/events.db

caro "list files"
```

**Verify**:
- [ ] Command executes successfully
- [ ] New database created automatically
- [ ] No crashes or errors shown to user

---

## Test Suite 8: Cross-Platform (Linux/macOS) (30 min)

**Note**: If possible, test on both macOS and Linux

### Platform-Specific Paths

**macOS**:
- Config: `~/.config/caro/config.toml`
- Data: `~/Library/Application Support/caro/telemetry/`

**Linux**:
- Config: `~/.config/caro/config.toml`
- Data: `~/.local/share/caro/telemetry/`

### Test on Both Platforms
- [ ] First-run consent works
- [ ] Events collected correctly
- [ ] Platform field shows correct OS
- [ ] File paths use correct separators
- [ ] CLI commands work identically

---

## Test Results Documentation

### Template for Each Test

```markdown
## Test: [Test Name]
**Date**: [Date]
**Tester**: [Name]
**Result**: PASS / FAIL / BLOCKED

**Details**:
- [What was tested]
- [Actual results]
- [Any issues found]

**Screenshots** (if applicable):
[Attach screenshots]

**Notes**:
[Additional observations]
```

---

## Pass/Fail Criteria

### Must Pass (Blocking Release)
- [ ] All Privacy Validation tests pass (Suite 4)
- [ ] First-run consent works (Suite 1)
- [ ] Events collect correctly (Suite 2)
- [ ] CLI commands work (Suite 3)
- [ ] Performance acceptable (Suite 6)

### Should Pass (Fix if Failed)
- [ ] Configuration tests (Suite 5)
- [ ] Edge cases handled gracefully (Suite 7)

### Nice to Have
- [ ] Cross-platform validation (Suite 8)

---

## Issues Found Template

```markdown
## Issue: [Short Description]
**Severity**: Critical / High / Medium / Low
**Test Suite**: [Which suite found it]
**Steps to Reproduce**:
1. [Step 1]
2. [Step 2]

**Expected**: [What should happen]
**Actual**: [What actually happened]

**Impact**: [How it affects users]
**Proposed Fix**: [Suggested solution]
```

---

## Final Checklist

After completing all tests:

- [ ] All critical tests passed
- [ ] Privacy audit completed with ZERO sensitive data found
- [ ] Performance within acceptable range (<5ms overhead)
- [ ] Documentation updated with test results
- [ ] Any issues found are logged and prioritized
- [ ] Test results reviewed by team
- [ ] Decision made: Ship / Fix Issues / More Testing

---

## Next Steps After Testing

1. **If All Tests Pass**:
   - Document results
   - Update README with telemetry section
   - Prepare beta release notes

2. **If Issues Found**:
   - Prioritize by severity
   - Fix critical issues immediately
   - Schedule medium/low issues for v1.1.1
   - Re-run affected test suites

3. **Documentation**:
   - Create telemetry FAQ
   - Write privacy policy statement
   - Update user guides

---

**Estimated Total Time**: 3-4 hours for complete test execution
**Recommended**: Run in one focused session to maintain context
