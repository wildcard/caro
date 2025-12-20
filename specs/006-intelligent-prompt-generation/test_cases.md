# Test Cases: Intelligent Prompt Generation & Validation

## Test Cases Derived from Session Analysis

### TC-001: Platform-Specific Command Syntax (macOS)

**Input:**
```
User Prompt: "which files I can delete to de clutter my mac"
Platform: macOS (BSD)
Shell: zsh
```

**Current Behavior (FAILING):**
```bash
# Generated commands use GNU-specific flags
find / -type f | xargs ls -lh --color=none
# Error: ls: unrecognized option `--color=none'

find / -type f | xargs ls -lh --sort=size
# Error: ls: unrecognized option `--sort=size'
```

**Expected Behavior (PASSING):**
```bash
# Should use BSD-compatible flags
du -sh ~/* ~/.[^.]* 2>/dev/null | sort -hr | head -20
# Or
find ~ -type f -size +100M -exec ls -lh {} \; 2>/dev/null | sort -k5 -hr
```

**Validation Criteria:**
- ✅ Command uses only BSD-compatible flags
- ✅ No GNU-specific long options (--sort, --color)
- ✅ Addresses user intent (finding large files for deletion)
- ✅ Includes human-readable output
- ✅ Handles permission errors gracefully (2>/dev/null)

---

### TC-002: Inconsistent Fallback Behavior

**Input:**
```
User Prompt: "which files I can delete to de clutter my macos sort by size from the biggest to the smallest"
Platform: macOS
Shell: zsh
```

**Current Behavior (FAILING):**
```bash
echo 'Unable to generate command'
```

**Expected Behavior (PASSING):**
```bash
# Should either:
# Option A: Generate working command
du -sh ~/* ~/.[^.]* 2>/dev/null | sort -rh | head -20

# Option B: Ask clarifying questions
# System: "Would you like to:
# 1. Find large files in your home directory?
# 2. Find large files system-wide (requires sudo)?
# 3. Analyze specific directory?"
```

**Validation Criteria:**
- ✅ Never return generic fallback for clear requests
- ✅ Trigger clarification flow for ambiguous requests
- ✅ Confidence score < 0.6 → clarification mode
- ✅ Confidence score >= 0.6 → generate command

---

### TC-003: Platform Detection & Configuration

**Setup Scenario:**
```
Initial Run: cmdai first execution
System: macOS 14.0 (Sonoma) on Apple Silicon
Installed Tools: BSD utils, homebrew, standard macOS tools
```

**Expected Behavior:**
1. **Auto-Detection Phase:**
   ```yaml
   detected_platform: macos
   detected_os_variant: darwin
   detected_shell: zsh
   detected_unix_flavor: bsd
   detected_arch: aarch64
   ```

2. **Configuration Generation:**
   ```toml
   # ~/.config/cmdai/config.toml (auto-generated)
   [platform]
   os = "macos"
   unix_flavor = "bsd"  # Auto-set based on OS
   shell = "zsh"
   arch = "aarch64"

   [generation]
   confidence_threshold = 0.6
   enable_multi_turn = true
   enable_clarification = true
   max_retries = 3

   [prompts]
   base_template = "default-bsd.toml"
   fallback_templates = ["detailed-bsd.toml", "interactive-bsd.toml"]
   ```

3. **User Override Support:**
   ```bash
   # User can override for cross-platform generation
   cmdai config set platform.unix_flavor gnu
   cmdai config set platform.os linux

   # Show current detection
   cmdai config show
   ```

**Validation Criteria:**
- ✅ Correctly detects macOS → sets unix_flavor=bsd
- ✅ Creates config file on first run
- ✅ Allows user to override any setting
- ✅ Warns user when generating for different platform
- ✅ Config survives between sessions

---

### TC-004: Tool Availability Validation

**Scenario:**
```
Platform: macOS
Available Tools: ls, find, du, sort, head, tail (BSD versions)
Unavailable Tools: --sort flag, --color flag, GNU coreutils
```

**Test Cases:**

#### TC-004a: Valid Command with Available Tools
```
Input: "list files sorted by size"
Expected: ls -lhS
Validation: ✅ PASS (uses BSD -S flag)
```

#### TC-004b: Invalid Command with Unavailable Flags
```
Input: (LLM generates) "ls -lh --sort=size"
Validation Agent Analysis:
  - Tool: ls (BSD version 8.3)
  - Flag: -l ✅ (available)
  - Flag: -h ✅ (available)
  - Flag: --sort ❌ (not available in BSD ls)

Action: REJECT → Regenerate with feedback
Feedback to LLM: "BSD ls doesn't support --sort. Use -S for size sorting."
Regenerated: ls -lhS
Final Validation: ✅ PASS
```

#### TC-004c: Command with Piped Tools
```
Input: "find large files and sort by size"
Generated: find . -type f -size +10M | xargs ls -lh | sort -k5 -hr
Validation Agent Analysis:
  - Tool: find ✅ (BSD find)
    - Flag: -type ✅
    - Flag: -size ✅
  - Tool: xargs ✅ (available)
  - Tool: ls ✅ (BSD ls)
    - Flag: -l ✅
    - Flag: -h ✅
  - Tool: sort ✅ (BSD sort)
    - Flag: -k ✅
    - Flag: -h ✅
    - Flag: -r ✅

Final Validation: ✅ PASS
```

**Validation Criteria:**
- ✅ Parses command into individual tools and flags
- ✅ Validates each tool against man pages
- ✅ Validates each flag against tool's available options
- ✅ Handles piped commands correctly
- ✅ Provides specific feedback for regeneration
- ✅ Caches man page data for performance

---

### TC-005: Ambiguity Detection & Clarification

**Test Cases:**

#### TC-005a: Clear Intent (No Clarification Needed)
```
Input: "list all .txt files in current directory"
Ambiguity Score: 0.1 (low)
Action: Generate directly
Generated: find . -maxdepth 1 -name "*.txt"
```

#### TC-005b: Moderate Ambiguity (Optional Clarification)
```
Input: "which files I can delete to de clutter my mac"
Ambiguity Score: 0.5 (moderate)
Clarification Questions Generated:
  1. "Which location? (home directory / entire system)"
  2. "Size threshold? (show all / > 100MB / > 1GB)"
  3. "Include hidden files and system files?"

User Response: "home directory, over 100MB, no system files"
Enhanced Prompt: "Find files in home directory larger than 100MB, excluding system files"
Generated: find ~ -type f -size +100M ! -path "*/Library/*" -exec ls -lh {} \; | sort -k5 -hr
```

#### TC-005c: High Ambiguity (Mandatory Clarification)
```
Input: "delete stuff"
Ambiguity Score: 0.9 (high)
Action: BLOCK generation → Request clarification
Clarification Questions:
  1. "What type of content? (files / directories / applications)"
  2. "What criteria? (by size / by date / by name pattern)"
  3. "Where? (specific directory or system-wide)"

System: "Please provide more specific details before I can safely generate a deletion command."
```

**Validation Criteria:**
- ✅ Ambiguity score < 0.3 → direct generation
- ✅ Ambiguity score 0.3-0.7 → optional clarification (configurable)
- ✅ Ambiguity score > 0.7 → mandatory clarification
- ✅ Questions are specific and actionable
- ✅ Regenerated command reflects user answers

---

### TC-006: Multi-Turn Agent Flow

**Scenario: Single-Shot Success**
```
Input: "list current directory files"
Flow:
  1. Generate: ls -la
  2. Validate: ✅ PASS
  3. Confidence: 0.95
  4. Present to user

Total Turns: 1
```

**Scenario: Validation Failure → Retry**
```
Input: "show files sorted by size"
Flow:
  1. Generate: ls -lh --sort=size
  2. Validate: ❌ FAIL (--sort not available on BSD)
  3. Feedback: "Use -S flag for size sorting on BSD ls"
  4. Regenerate: ls -lhS
  5. Validate: ✅ PASS
  6. Confidence: 0.85
  7. Present to user

Total Turns: 2
Max Retries: 3
```

**Scenario: Low Confidence → Clarification**
```
Input: "clean up disk space"
Flow:
  1. Analyze: Ambiguity Score 0.75
  2. Generate Clarifications: [location, criteria, safety]
  3. User Response: "home dir, old downloads, safe delete"
  4. Enhanced Prompt: "Find and suggest deletion of files in ~/Downloads older than 30 days"
  5. Generate: find ~/Downloads -type f -mtime +30 -ls
  6. Validate: ✅ PASS
  7. Confidence: 0.88
  8. Present to user

Total Turns: 2 (clarification + generation)
```

**Scenario: Multiple Failures → Escalate Prompt**
```
Input: "complex query about system files"
Flow:
  1. Generate (base prompt): <command1>
  2. Validate: ❌ FAIL
  3. Regenerate with feedback: <command2>
  4. Validate: ❌ FAIL
  5. Regenerate (attempt 3): <command3>
  6. Validate: ❌ FAIL
  7. Escalate to detailed prompt with man pages
  8. Generate (detailed prompt): <command4>
  9. Validate: ✅ PASS
  10. Confidence: 0.72
  11. Present to user

Total Turns: 4
Prompt Escalation: base → detailed
```

**Validation Criteria:**
- ✅ Single-shot is default path
- ✅ Max 3 validation retries before escalation
- ✅ Clarification flow separate from validation retries
- ✅ Each turn has timeout (30s)
- ✅ User can abort multi-turn flow
- ✅ System logs all turns for debugging

---

### TC-007: Prompt Template System

**Test Setup:**
```
Prompt Templates Location: ~/.config/cmdai/prompts/
Available Templates:
  - base-bsd.toml
  - base-gnu.toml
  - detailed-bsd.toml
  - detailed-gnu.toml
  - interactive-clarification.toml
```

**Template Structure:**
```toml
# base-bsd.toml
[meta]
name = "BSD Default Prompt"
version = "1.0.0"
platform = "bsd"
confidence_threshold = 0.6

[prompt]
system = """
You are a command-line expert for BSD/macOS systems.

CRITICAL REQUIREMENTS:
1. Generate commands compatible with BSD utilities (macOS default)
2. NEVER use GNU-specific long options (--sort, --color, --human-readable)
3. Use BSD flags: -S (size sort), -h (human readable), -r (reverse)

Platform Context:
- OS: {{os}}
- Unix Flavor: {{unix_flavor}}
- Shell: {{shell}}
- Available Tools: {{tools}}

Response Format:
{"cmd": "your_command_here"}

Request: {{user_input}}
"""

[examples]
list_sorted = "ls -lhS"
find_large = "du -sh * | sort -rh"
disk_usage = "df -h"

[validation]
required_patterns = ["^[a-z]"]  # Must start with command
forbidden_patterns = ["--sort", "--color"]  # GNU-specific flags
```

**Test Cases:**

#### TC-007a: Template Selection
```
Platform: macos, unix_flavor: bsd
Selected Template: base-bsd.toml ✅
```

#### TC-007b: Template Variable Substitution
```
Variables:
  os: macos
  unix_flavor: bsd
  shell: zsh
  tools: ls, find, du, sort
  user_input: "list files by size"

Rendered Prompt: (contains all substituted values) ✅
```

#### TC-007c: Community Template Override
```bash
# User creates custom template
cat > ~/.config/cmdai/prompts/custom-bsd.toml << EOF
[meta]
name = "My Custom BSD Prompt"
parent = "base-bsd.toml"

[prompt.additions]
style = "Prefer modern macOS commands like 'fd' and 'ripgrep' when available"
EOF

# Use custom template
cmdai config set prompts.base_template custom-bsd.toml
```

**Validation Criteria:**
- ✅ Templates loaded from config directory
- ✅ Variables properly substituted
- ✅ Template inheritance supported (parent templates)
- ✅ Community can add new templates without code changes
- ✅ Version compatibility checking
- ✅ Fallback to default if custom template fails

---

### TC-008: Man Page Analysis Agent

**Initial Setup Test:**

```
First Run Detection: No cache exists
Action: Launch Man Page Analysis Agent

Agent Tasks:
1. Detect platform: macos (bsd)
2. Scan available commands:
   - /bin/ls
   - /usr/bin/find
   - /usr/bin/du
   - /usr/bin/sort
   - ... (50+ standard tools)

3. Parse man pages:
   For each tool:
   - Read: man <tool>
   - Extract: available flags
   - Parse: flag descriptions
   - Store: structured format

4. Validate with --help:
   - Run: <tool> --help 2>&1
   - Cross-reference with man page
   - Note discrepancies

5. Generate cache:
   ~/.cache/cmdai/man-pages.json

Cache Structure:
{
  "platform": "macos",
  "unix_flavor": "bsd",
  "generated_at": "2025-11-27T12:00:00Z",
  "tools": {
    "ls": {
      "path": "/bin/ls",
      "version": "BSD ls 8.3",
      "flags": {
        "-l": "Long format listing",
        "-h": "Human-readable sizes",
        "-S": "Sort by size",
        "-r": "Reverse order"
      },
      "forbidden_flags": ["--sort", "--color"]
    },
    "find": { ... },
    "du": { ... }
  }
}

Execution Time: ~30-60 seconds
Cache Validity: 30 days
```

**Validation Test Cases:**

#### TC-008a: Flag Validation
```
Command: ls -lhS
Validation:
  - Tool: ls ✅ (exists in cache)
  - Flag: -l ✅ (valid)
  - Flag: -h ✅ (valid)
  - Flag: -S ✅ (valid)
Result: PASS
```

#### TC-008b: Invalid Flag Detection
```
Command: ls --sort=size
Validation:
  - Tool: ls ✅ (exists in cache)
  - Flag: --sort ❌ (in forbidden_flags)
Suggestion: "Replace '--sort=size' with '-S' for BSD ls"
Result: FAIL with correction
```

#### TC-008c: Unknown Tool
```
Command: rg -i pattern
Validation:
  - Tool: rg ❌ (not in cache)
Action:
  1. Check if tool exists: which rg
  2. If exists: Dynamic man page lookup + cache update
  3. If not exists: FAIL with "Tool 'rg' not available"
```

**Validation Criteria:**
- ✅ Man pages parsed correctly
- ✅ Flags extracted accurately
- ✅ Cross-validation with --help
- ✅ Cache persists between sessions
- ✅ Cache auto-refreshes after 30 days
- ✅ Dynamic lookup for unknown tools
- ✅ Performance: < 100ms validation time (with cache)

---

### TC-009: Confidence Scoring System

**Scoring Factors:**

```rust
struct ConfidenceScore {
    validation_passed: f32,      // 0.0-0.4 weight
    ambiguity_resolved: f32,     // 0.0-0.3 weight
    platform_compatibility: f32, // 0.0-0.2 weight
    safety_level: f32,          // 0.0-0.1 weight
}

fn calculate_confidence(factors: ConfidenceScore) -> f32 {
    factors.validation_passed * 0.4 +
    factors.ambiguity_resolved * 0.3 +
    factors.platform_compatibility * 0.2 +
    factors.safety_level * 0.1
}
```

**Test Cases:**

#### TC-009a: High Confidence Command
```
Command: ls -la
Validation: ✅ PASS (1.0)
Ambiguity: Clear request (1.0)
Platform: Perfect match (1.0)
Safety: Safe operation (1.0)

Confidence: 0.4 + 0.3 + 0.2 + 0.1 = 1.0
Action: Present immediately
```

#### TC-009b: Medium Confidence Command
```
Command: find ~ -name "*.log" -delete
Validation: ✅ PASS (1.0)
Ambiguity: Moderate - no size/date filter (0.6)
Platform: Compatible (1.0)
Safety: Destructive operation (0.3)

Confidence: 0.4 + 0.18 + 0.2 + 0.03 = 0.81
Action: Present with safety warning
```

#### TC-009c: Low Confidence Command
```
Command: rm -rf /tmp/*
Validation: ❌ FAIL (0.0) - flagged as dangerous
Ambiguity: Clear but risky (0.8)
Platform: Compatible (1.0)
Safety: Critical risk (0.0)

Confidence: 0.0 + 0.24 + 0.2 + 0.0 = 0.44
Action: BLOCK or request explicit confirmation
```

**Validation Criteria:**
- ✅ Score range: 0.0-1.0
- ✅ Threshold configurable (default: 0.6)
- ✅ Below threshold → clarification/retry
- ✅ Score visible to user
- ✅ Factors logged for debugging

---

### TC-010: Cross-Platform Generation

**Scenario: User on macOS generating for Linux**

```bash
# User configuration
cmdai config set platform.target_os linux
cmdai config set platform.target_unix_flavor gnu

# Generation
cmdai "list files sorted by size"

System Warning:
┌────────────────────────────────────────────┐
│ Cross-Platform Mode Active                 │
│ Current: macOS (BSD)                       │
│ Target: Linux (GNU)                        │
│                                            │
│ Generated command may not work locally!    │
└────────────────────────────────────────────┘

Generated Command:
ls -lh --sort=size --color=auto

Validation:
  - Platform: Linux (GNU) ✅
  - Tool: ls (GNU coreutils) ✅
  - Flags: --sort ✅, --color ✅

Confidence: 0.85
Note: "This command targets Linux/GNU. To test locally, install 'coreutils' via Homebrew."
```

**Validation Criteria:**
- ✅ Detects current vs target platform mismatch
- ✅ Warns user clearly
- ✅ Generates for target platform
- ✅ Provides local testing instructions
- ✅ Validates against target platform specs

---

## Performance Benchmarks

### Benchmark Requirements:

```yaml
cold_start:
  first_run_with_man_scan: < 60s
  first_run_without_scan: < 2s

warm_start:
  generation_time: < 2s
  validation_time: < 100ms
  total_single_shot: < 3s

multi_turn:
  retry_with_feedback: < 5s
  clarification_round: < 3s
  max_total_time: < 15s

cache:
  man_page_cache_hit: < 10ms
  cache_miss_lookup: < 500ms
  cache_rebuild: < 60s
```

---

## Test Execution Strategy

### Unit Tests:
- Platform detection logic
- Configuration parsing
- Template variable substitution
- Man page parsing
- Confidence scoring algorithm
- Flag validation logic

### Integration Tests:
- End-to-end generation flow
- Multi-turn retry logic
- Clarification workflow
- Cache management
- Cross-platform scenarios

### Contract Tests:
- Validation agent contract
- Clarification agent contract
- Man page analyzer contract
- Prompt template interface

### Regression Tests:
- All session failure cases (TC-001, TC-002)
- Known edge cases
- Platform-specific quirks
