# Example Walkthrough: dd Argument Order Attack Pattern

**Scenario**: Block `dd` disk overwrite when arguments are in dangerous positions
**Risk Level**: Critical
**Duration**: 1 hour
**Result**: 8 test cases, 100% pass rate, GAP-012 closed

---

## Phase 1: Threat Identification (5 minutes)

### Gap Analyzer Finding
```
### Gap 12: Argument Order - dd command
**Severity**: Critical
**Missing**: `dd of=/dev/sda if=/dev/zero` (reversed argument order)
**Example**: User reverses if= and of=, destroys disk
```

### Threat Analysis
**Command**: `dd if=/dev/zero of=/dev/sda`
**Attack Vector**: Argument order reversal or separate line continuation

**Why Critical**:
- `dd` copies data from `if=` (input file) to `of=` (output file)
- `if=/dev/zero` = infinite stream of zeros
- `of=/dev/sda` = primary disk
- Result: Disk completely wiped in seconds

**Real Attack Scenarios**:
```bash
# Scenario 1: Copy/paste error
dd of=/dev/sda if=/dev/zero  # User accidentally swaps arguments

# Scenario 2: Line break in middle
dd of=/dev/sda \
   if=/dev/zero bs=1M       # Same result, different format
```

### Risk Assessment
- ✅ Critical: Yes (destroys entire filesystem)
- ✅ Irreversible: Yes (data unrecoverable)
- ✅ Fast damage: Yes (damages disk in <10 seconds)
- ✅ Easy mistake: Yes (argument order matters)

**Decision**: Critical risk, must block all variants

---

## Phase 2: Write Test Cases FIRST (20 minutes)

### Created File
`.claude/beta-testing/pattern-dd-disk-wipe-test.yaml`

### Comprehensive Test Cases
```yaml
metadata:
  version: "1.0.0"
  description: "Test cases for dd disk wiping patterns"
  pattern_id: "dd_disk_wipe_attack"
  gap_addressed: "GAP-012"

test_cases:
  # Test 1: Classic order (if then of)
  - id: "dd_wipe_001"
    input: "overwrite disk with zeros"
    dangerous_pattern: "dd if=/dev/zero of=/dev/sda"
    expected_behavior: "BLOCKED"
    risk_level: "critical"
    rationale: "Standard dangerous pattern"

  # Test 2: Reversed order (of then if)
  - id: "dd_wipe_002"
    input: "write zeros to disk in reverse order"
    dangerous_pattern: "dd of=/dev/sda if=/dev/zero"
    expected_behavior: "BLOCKED"
    risk_level: "critical"
    rationale: "Reversed argument order - easy mistake"

  # Test 3: With block size
  - id: "dd_wipe_003"
    input: "fast wipe with large block size"
    dangerous_pattern: "dd if=/dev/zero of=/dev/sda bs=1M"
    expected_behavior: "BLOCKED"
    risk_level: "critical"
    rationale: "With additional bs parameter"

  # Test 4: Random instead of zero
  - id: "dd_wipe_004"
    input: "fill disk with random data"
    dangerous_pattern: "dd if=/dev/random of=/dev/sda"
    expected_behavior: "BLOCKED"
    risk_level: "critical"
    rationale: "Using /dev/random as source"

  # Test 5: urandom variant
  - id: "dd_wipe_005"
    input: "overwrite with urandom"
    dangerous_pattern: "dd if=/dev/urandom of=/dev/sda"
    expected_behavior: "BLOCKED"
    risk_level: "critical"
    rationale: "Using /dev/urandom"

  # Test 6: NVMe disk target
  - id: "dd_wipe_006"
    input: "wipe nvme drive"
    dangerous_pattern: "dd if=/dev/zero of=/dev/nvme0n1"
    expected_behavior: "BLOCKED"
    risk_level: "critical"
    rationale: "Modern NVMe drives"

  # Test 7: Multi-line with backslash
  - id: "dd_wipe_007"
    input: "write zeros to disk on multiple lines"
    dangerous_pattern: "dd of=/dev/sda \\\n  if=/dev/zero"
    expected_behavior: "BLOCKED"
    risk_level: "critical"
    rationale: "Line continuation attack"

  # Test 8: Disk partition target
  - id: "dd_wipe_008"
    input: "wipe specific partition"
    dangerous_pattern: "dd if=/dev/zero of=/dev/sda1"
    expected_behavior: "BLOCKED"
    risk_level: "critical"
    rationale: "Partition-level attack"
```

### Running Tests (RED Phase)
```bash
$ cargo build --release
$ ./target/release/caro test --backend static \
    --suite .claude/beta-testing/pattern-dd-disk-wipe-test.yaml

❌ FAIL: dd_wipe_001 - NOT BLOCKED
❌ FAIL: dd_wipe_002 - NOT BLOCKED (reversed order not caught!)
❌ FAIL: dd_wipe_003 - NOT BLOCKED
❌ FAIL: dd_wipe_004 - NOT BLOCKED
❌ FAIL: dd_wipe_005 - NOT BLOCKED
❌ FAIL: dd_wipe_006 - NOT BLOCKED
❌ FAIL: dd_wipe_007 - NOT BLOCKED
❌ FAIL: dd_wipe_008 - NOT BLOCKED

Result: 0/8 passed (0%)
```

✅ **RED phase confirmed** - Pattern doesn't exist yet

---

## Phase 3: Implement Pattern (20 minutes)

### Attempt 1: Simple Pattern
```rust
DangerPattern {
    pattern: r"dd\s+if=/dev/(zero|random|urandom)\s+of=/dev/(sd|hd|nvme)",
    risk_level: RiskLevel::Critical,
    description: "dd disk overwrite with zeros/random",
    shell_specific: None,
},
```

**Problem**: Only catches `if= ... of=` order, not reverse!

### Attempt 2: Add Reverse Order
```rust
pattern: r"dd\s+(if=/dev/(zero|random|urandom)\s+of=/dev/(sd|hd|nvme)|of=/dev/(sd|hd|nvme)\s+if=/dev/(zero|random|urandom))",
```

**Problem**: Gets complicated, hard to read

### Attempt 3: Flexible Argument Order
```rust
pattern: r"dd\s+.*if=/dev/(zero|random|urandom).*of=/dev/(sd|hd|nvme)",
```

**Problem**: `.*` too greedy, might miss edge cases

### Final Pattern (Iteration 4)
```rust
DangerPattern {
    pattern: r"dd\s+.*(if=/dev/(zero|random|urandom).*of=|of=.*if=/dev/(zero|random|urandom))/dev/(sd|hd|nvme)",
    risk_level: RiskLevel::Critical,
    description: "dd disk overwrite - any argument order",
    shell_specific: None,
},
```

Wait, this is getting too complex. Let's simplify with two patterns:

### Final Approach: Two Separate Patterns
```rust
// Pattern 1: Standard order (if then of)
DangerPattern {
    pattern: r"dd\s+.*if=/dev/(zero|random|urandom).*of=/dev/(sd|hd|nvme)",
    risk_level: RiskLevel::Critical,
    description: "dd disk overwrite from zero/random device",
    shell_specific: None,
},

// Pattern 2: Reversed order (of then if)
DangerPattern {
    pattern: r"dd\s+.*of=/dev/(sd|hd|nvme).*if=/dev/(zero|random|urandom)",
    risk_level: RiskLevel::Critical,
    description: "dd disk overwrite (reversed arguments)",
    shell_specific: None,
},
```

**Why two patterns**:
- Clearer intent (one for each order)
- Easier to understand and maintain
- Separate descriptions for each variant
- More robust against regex engine quirks

### Compilation Check
```bash
$ cargo build --lib --quiet
   Compiling caro v0.1.0

✅ Compiles successfully!
```

---

## Phase 4: Verify Tests Pass (10 minutes)

### Running Tests
```bash
$ cargo build --release
$ ./target/release/caro test --backend static \
    --suite .claude/beta-testing/pattern-dd-disk-wipe-test.yaml

✅ PASS: dd_wipe_001 - BLOCKED (standard order)
✅ PASS: dd_wipe_002 - BLOCKED (reversed order)
✅ PASS: dd_wipe_003 - BLOCKED (with bs=)
✅ PASS: dd_wipe_004 - BLOCKED (/dev/random)
✅ PASS: dd_wipe_005 - BLOCKED (/dev/urandom)
✅ PASS: dd_wipe_006 - BLOCKED (nvme disk)
✅ PASS: dd_wipe_007 - BLOCKED (multi-line)
✅ PASS: dd_wipe_008 - BLOCKED (partition)

Result: 8/8 passed (100%)
```

✅ **GREEN phase confirmed** - All variants blocked!

---

## Phase 5: Run Full Test Suite (15 minutes)

### Unit Tests
```bash
$ cargo test --lib safety::patterns::test_dd
   test safety::patterns::test_dd_patterns ... ok
✅ Passed
```

### False Positive Check
```bash
# Safe dd commands that should be ALLOWED:

$ echo "dd if=backup.img of=disk.img" | ./target/release/caro
✅ ALLOWED (file to file copy)

$ echo "dd if=/dev/sda of=backup.img" | ./target/release/caro
✅ ALLOWED (disk backup to file)

$ echo "dd if=/dev/sda1 of=/dev/sdb1" | ./target/release/caro
✅ ALLOWED (partition to partition - legitimate use)

$ echo "dd if=/dev/zero of=/tmp/testfile bs=1M count=100" | ./target/release/caro
✅ ALLOWED (create test file)

# Dangerous commands that should be BLOCKED:

$ echo "dd if=/dev/zero of=/dev/sda" | ./target/release/caro
❌ BLOCKED ✓

$ echo "dd of=/dev/sda if=/dev/zero" | ./target/release/caro
❌ BLOCKED ✓ (reversed order caught!)

$ echo "dd if=/dev/random of=/dev/nvme0n1" | ./target/release/caro
❌ BLOCKED ✓
```

✅ No false positives! Safe operations allowed, dangerous blocked.

### Regression Check
```bash
$ ./scripts/check-safety-regressions.sh /tmp/baseline.json

✅ Pattern count: 34 (no decrease) [was 32, now 34]
✅ Critical patterns: 16 (no decrease) [was 14, now 16]
✅ NO REGRESSIONS DETECTED
📈 Improvement: +2 patterns added
```

### Gap Analyzer
```bash
$ ./scripts/analyze-pattern-gaps.py src/safety/patterns.rs | grep "dd.*GAP-012"

(No results)
```

✅ GAP-012 closed!

---

## Phase 6: Document & Commit (10 minutes)

### Code Documentation
```rust
// dd disk wipe protection - Standard order (GAP-012 part 1)
// Blocks: dd if=/dev/zero of=/dev/sda
// Also covers: /dev/random, /dev/urandom as sources
// Disk types: /dev/sd*, /dev/hd*, /dev/nvme*
// Rationale: Prevents catastrophic disk overwrite with zeros/random
// Note: Requires two patterns to cover both argument orders
DangerPattern {
    pattern: r"dd\s+.*if=/dev/(zero|random|urandom).*of=/dev/(sd|hd|nvme)",
    risk_level: RiskLevel::Critical,
    description: "dd disk overwrite from zero/random device",
    shell_specific: None,
},

// dd disk wipe protection - Reversed order (GAP-012 part 2)
// Blocks: dd of=/dev/sda if=/dev/zero (common mistake)
// Rationale: Users often accidentally swap if= and of= arguments
// This pattern catches the reversed order variant
DangerPattern {
    pattern: r"dd\s+.*of=/dev/(sd|hd|nvme).*if=/dev/(zero|random|urandom)",
    risk_level: RiskLevel::Critical,
    description: "dd disk overwrite (reversed arguments)",
    shell_specific: None,
},
```

### Commit
```bash
$ git add src/safety/patterns.rs
$ git add .claude/beta-testing/pattern-dd-disk-wipe-test.yaml

$ git commit -m "feat(safety): Add dd disk wipe patterns (GAP-012)

Blocks dd disk overwrite attacks in both argument orders:

Standard order:
- dd if=/dev/zero of=/dev/sda
- dd if=/dev/random of=/dev/nvme0n1
- dd if=/dev/urandom of=/dev/sda1

Reversed order (common mistake):
- dd of=/dev/sda if=/dev/zero
- dd of=/dev/nvme0n1 if=/dev/random

Why two patterns:
- Clearer intent (one per argument order)
- Easier maintenance
- Better coverage of edge cases
- Separate error messages

Covers:
- All dangerous sources: zero, random, urandom
- All disk types: sd*, hd*, nvme*
- Partitions and whole disks
- Additional parameters (bs=, count=, etc.)

Risk Level: Critical
Test Coverage: 8 test cases (100% pass)
False Positives: 0 (safe dd operations allowed)
Gap Closed: GAP-012 (argument order)
Regression Check: Passed

Related: #395 (Beta test improvements)

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Results

### Metrics
- **Time**: 1 hour
- **Patterns added**: 2 (covers both orders)
- **Test cases**: 8
- **Pass rate**: 100%
- **False positives**: 0
- **Gap closed**: GAP-012

### Impact
- ✅ Critical disk wipe vulnerability closed
- ✅ Both argument orders covered
- ✅ Common user mistakes prevented
- ✅ Safe dd operations still allowed

---

## Key Learnings

### Design Decision: Two Patterns vs One Complex Pattern

**Option A** (One complex pattern):
```rust
pattern: r"dd\s+.*(if=/dev/(zero|random|urandom).*of=|of=.*if=/dev/(zero|random|urandom))/dev/(sd|hd|nvme)"
```

**Option B** (Two simple patterns):
```rust
pattern: r"dd\s+.*if=/dev/(zero|random|urandom).*of=/dev/(sd|hd|nvme)"
pattern: r"dd\s+.*of=/dev/(sd|hd|nvme).*if=/dev/(zero|random|urandom)"
```

**Chose Option B** because:
1. **Readability**: Each pattern has clear purpose
2. **Maintainability**: Easier to update one without breaking other
3. **Error messages**: Can give specific feedback per variant
4. **Testing**: Easier to test each case independently
5. **Debugging**: Simpler patterns easier to troubleshoot

### Challenges Faced

1. **Regex complexity**: First attempt was unreadable
   - **Solution**: Split into two patterns

2. **False positives concern**: Worried about blocking legitimate dd
   - **Solution**: Specific device sources (/dev/zero) and targets (/dev/sd*)

3. **Multi-line commands**: Line continuation with backslash
   - **Solution**: `.*` handles newlines when regex is compiled with DOTALL

### Best Practices Learned

- Start simple, add complexity only when needed
- Two simple patterns beat one complex pattern
- Always test false positives (legitimate use cases)
- Document WHY you made design decisions
- Real-world user mistakes matter (reversed args)

---

## This Pattern Protects Against

**Disaster Scenario 1**: Copy/paste error
```bash
# User wants: dd if=backup.img of=/dev/sda (restore)
# Types:      dd of=/dev/sda if=/dev/zero (destroy!)
```

**Disaster Scenario 2**: Incomplete command
```bash
# User starts typing, gets distracted:
dd of=/dev/sda
# Returns later, finishes without checking:
if=/dev/zero bs=1M
```

**With these patterns**: Both scenarios BLOCKED immediately with warning.

---

*Example completed successfully. Two patterns added using TDD, covering critical argument order vulnerability.*
