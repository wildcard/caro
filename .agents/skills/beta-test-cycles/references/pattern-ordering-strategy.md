# Pattern Ordering Strategy

This reference provides comprehensive guidance on pattern ordering, specificity calculation, and reordering workflows for static pattern matchers.

## Core Principle

**Specific patterns must be checked before general patterns.** This is the single most important rule for pattern ordering.

### Why Ordering Matters

The pattern matcher iterates through patterns sequentially and returns the FIRST match found. If a general pattern comes before a specific pattern, the general pattern will match queries intended for the specific pattern.

**Example Problem:**
```rust
// BAD ORDERING
Pattern 1: "files modified today" (general, 3 keywords)
Pattern 46: "Python files modified today" (specific, 4 keywords)

// Query: "find Python files modified today"
// Matches: Pattern 1 ❌ (wrong - too general)
// Expected: Pattern 46 ✅ (correct - has Python specificity)
```

**Example Solution:**
```rust
// GOOD ORDERING
Pattern 1: "Python files modified today" (specific, 4 keywords)
Pattern 2: "files modified today" (general, 3 keywords)

// Query: "find Python files modified today"
// Matches: Pattern 1 ✅ (correct - most specific match)
```

## Specificity Calculation

### Formula

```
Specificity Score = (Required Keywords × 10) + Constraint Score + Regex Score
```

Where:
- **Required Keywords**: Count of keywords in `required_keywords` vector
- **Constraint Score**: Count of constraints (file type, size, location, time, sorting)
- **Regex Score**: 5 if regex pattern exists and is restrictive, 0 otherwise

### Constraint Types

Each constraint adds +1 to specificity:

1. **File Type Constraint**: Pattern specifies file extension (.py, .pdf, .log)
2. **Size Constraint**: Pattern specifies file size (>100MB, <1KB, =50MB)
3. **Location Constraint**: Pattern specifies directory (~/Downloads, /var/log)
4. **Time Constraint**: Pattern specifies time range (today, last 7 days, -24h)
5. **Sorting Constraint**: Pattern specifies sort order (sorted, sorted by size)
6. **Action Constraint**: Pattern specifies action (delete, count, list)

### Examples

**Example 1: Python Files Modified Today**
```rust
PatternEntry {
    required_keywords: vec!["python", "modified", "today"],  // 3 keywords
    regex_pattern: Some(Regex::new(r"...").unwrap()),      // +5
    // Constraints: file_type (python), time (today) = +2
}
// Specificity = (3 × 10) + 2 + 5 = 37
```

**Example 2: Files Modified Today (General)**
```rust
PatternEntry {
    required_keywords: vec!["file", "modified", "today"],  // 3 keywords
    regex_pattern: Some(Regex::new(r"...").unwrap()),     // +5
    // Constraints: time (today) = +1
}
// Specificity = (3 × 10) + 1 + 5 = 36
```

**Result**: Pattern 1 (score 37) should come before Pattern 2 (score 36)

**Example 3: PDF Files >10MB in Downloads**
```rust
PatternEntry {
    required_keywords: vec!["pdf", "downloads"],  // 2 keywords
    // Constraints: file_type (pdf), size (10MB), location (Downloads) = +3
}
// Specificity = (2 × 10) + 3 + 0 = 23
```

**Example 4: Files >10MB (General)**
```rust
PatternEntry {
    required_keywords: vec!["file"],  // 1 keyword
    // Constraints: size (10MB) = +1
}
// Specificity = (1 × 10) + 1 + 0 = 11
```

**Result**: Pattern 3 (score 23) should come before Pattern 4 (score 11)

## Specificity Hierarchy

### Visual Hierarchy

```
┌─────────────────────────────────────────────┐
│ MOST SPECIFIC (Check First)                │
├─────────────────────────────────────────────┤
│ File Type + Time + Location + Size         │
│ (e.g., "PDF files >10MB in Downloads       │
│  modified today")                           │
│ Score: 40-50+                               │
├─────────────────────────────────────────────┤
│ File Type + Time + Size                    │
│ (e.g., "Python files >100MB modified       │
│  last week")                                │
│ Score: 35-45                                │
├─────────────────────────────────────────────┤
│ File Type + Time                            │
│ (e.g., "Python files modified today")      │
│ Score: 30-40                                │
├─────────────────────────────────────────────┤
│ File Type + Location                        │
│ (e.g., "PDF files in Downloads")           │
│ Score: 25-35                                │
├─────────────────────────────────────────────┤
│ File Type Only                              │
│ (e.g., "Python files")                      │
│ Score: 20-30                                │
├─────────────────────────────────────────────┤
│ General with Single Constraint             │
│ (e.g., "files modified today")             │
│ Score: 15-25                                │
├─────────────────────────────────────────────┤
│ General Query                               │
│ (e.g., "list files")                        │
│ Score: 10-20                                │
└─────────────────────────────────────────────┘
  LEAST SPECIFIC (Check Last)
```

### Detailed Breakdown

**Level 1: Highly Specific (4+ Required Keywords + 3+ Constraints)**
- Multiple constraints stack
- Very narrow match scope
- Should be checked first
- Example: "PDF files larger than 10MB in Downloads modified today"

**Level 2: Moderately Specific (3-4 Required Keywords + 2 Constraints)**
- Two constraints (typically file type + one other)
- Moderate match scope
- Check after Level 1, before Level 3
- Example: "Python files modified in last 7 days"

**Level 3: Somewhat Specific (2-3 Required Keywords + 1 Constraint)**
- Single constraint (file type OR time/size/location)
- Broader match scope
- Check after Level 2, before Level 4
- Example: "PDF files in Downloads" or "files modified today"

**Level 4: General (1-2 Required Keywords + 0-1 Constraints)**
- No file type specificity OR minimal constraints
- Broad match scope
- Check last (fallback patterns)
- Example: "files larger than 100MB" or "list files"

## Reordering Workflow

### Step 1: Identify Ordering Issue

**Symptoms:**
- Test expects specific command but gets generic command
- More specific pattern exists but isn't matching
- Test case explicitly mentions specific attribute (Python, PDF, etc.)

**Verification:**
```bash
# Check which pattern is matching
./target/release/caro test --backend static --suite test-cases.yaml | grep -A 2 "✗ [Category] test name"
```

Look for "Expected: <specific command>" but "Got: <generic command>"

### Step 2: Calculate Specificity Scores

For each pattern involved:

1. Count required keywords
2. Identify constraints (file type, size, location, time, sorting, action)
3. Check if regex is restrictive
4. Calculate: `(keywords × 10) + constraints + (regex ? 5 : 0)`

**Example:**
```
Pattern A: "Python files modified today"
- Keywords: 3 (python, modified, today)
- Constraints: 2 (file_type=python, time=today)
- Regex: restrictive (+5)
- Score: (3 × 10) + 2 + 5 = 37

Pattern B: "files modified today"
- Keywords: 3 (file, modified, today)
- Constraints: 1 (time=today)
- Regex: loose (+0)
- Score: (3 × 10) + 1 + 0 = 31

Result: Pattern A (37) should come before Pattern B (31)
```

### Step 3: Determine Target Position

**Rule**: Insert the more specific pattern immediately before the less specific pattern it's conflicting with.

**Example:**
```
Current Order:
Pattern 5: "files larger than 10MB" (general)
...
Pattern 41: "PDF files larger than 10MB in Downloads" (specific)

Target Order:
Pattern 5: "PDF files larger than 10MB in Downloads" (specific) ← moved here
Pattern 6: "files larger than 10MB" (general) ← renumbered
```

### Step 4: Reorder in Code

**Process:**
1. Find the specific pattern in `src/backends/static_matcher.rs`
2. Copy the entire `PatternEntry { ... }` block
3. Paste it at the target position (before the general pattern)
4. Update pattern number in comment
5. Renumber patterns after the insertion point
6. Remove the old pattern entry
7. Update comment describing the ordering

**Example:**
```rust
// BEFORE
// Pattern 5: "find files larger than 10MB" (GENERAL)
PatternEntry {
    required_keywords: vec!["file", "10mb"],
    // ...
},
// ... many patterns later ...
// Pattern 41: "find PDF files larger than 10MB in Downloads" (SPECIFIC)
PatternEntry {
    required_keywords: vec!["pdf", "downloads"],
    // ...
},

// AFTER
// Pattern 5: "find PDF files larger than 10MB in Downloads" (SPECIFIC - moved from Pattern 41)
PatternEntry {
    required_keywords: vec!["pdf", "downloads"],
    // ...
},
// Pattern 6: "find files larger than 10MB" (GENERAL - was Pattern 5)
PatternEntry {
    required_keywords: vec!["file", "10mb"],
    // ...
},
// ... Pattern 41 removed (now at Position 5) ...
```

### Step 5: Verify and Test

After reordering:

```bash
# Build
cargo build --release

# Run tests
./target/release/caro test --backend static --suite .claude/beta-testing/test-cases.yaml

# Verify:
# 1. Target test now passes
# 2. No new test failures (regressions)
# 3. Pass rate improved
```

**Regression check**: Ensure reordering didn't break tests that were previously passing.

## Batch Reordering

When multiple patterns have the same ordering issue, batch them together for efficiency.

### Identifying Batch Candidates

**Signs of batch opportunity:**
- Multiple tests failing with same root cause (general pattern matching first)
- Pattern group shares common attribute (all file-type-specific, all with sorting, etc.)
- Same general pattern being matched instead of multiple specific patterns

**Example from Cycle 7:**
```
Failing Tests:
- fm_017: "Python files modified today" → matches Pattern 1 (general)
- fm_011: "PDF files >10MB in Downloads" → matches Pattern 5 (general)
- fm_012: "Python files modified last 7 days" → matches Pattern 10 (general)

Root Cause: Three file-type-specific patterns all coming after their general counterparts

Batch Fix: Move all three specific patterns before their general counterparts in one cycle
```

### Batch Reordering Process

1. **Identify group**: List all patterns that need reordering for same reason
2. **Calculate specificity**: Rank within the group by specificity score
3. **Plan target positions**: Determine where each pattern should move
4. **Reorder sequentially**: Move patterns one at a time, testing between moves (optional)
5. **Remove duplicates**: Clean up old pattern entries
6. **Test batch**: Verify all target tests now pass

**Batch commit message:**
```
fix(static): [Cycle N] Batch file-type-specific pattern reordering

- Pattern 46 → Pattern 1 (Python modified today)
- Pattern 41 → Pattern 5 (PDF in Downloads)
- Pattern 42 → Pattern 10 (Python last 7 days)
- File Management: X% → Y% (+N tests)
```

## Special Cases

### Regex-Only Patterns (i18n, Specialized)

For patterns that should ONLY match via regex (not keywords):

**Strategy**: Empty both required_keywords and optional_keywords

```rust
// Japanese filename pattern
PatternEntry {
    required_keywords: vec![],           // Empty forces regex check
    optional_keywords: vec![],           // Empty prevents keyword fallback
    regex_pattern: Some(Regex::new(r"[ぁ-んァ-ヶー一-龯]").unwrap()),
    // ...
}
```

**Why**: Matching logic checks regex first, but falls back to keywords if regex doesn't match. Empty keywords eliminate the fallback, forcing regex-only matching.

**Use cases:**
- i18n patterns (Japanese, Chinese, Arabic, etc.)
- Binary content detection
- Highly specialized syntax patterns

### Conflicting Specific Forms

When two patterns are both specific but have different expected outputs:

**Example from Cycle 7:**
- fm_004: "Find all files larger than 1GB" → expects WITH exec
- fm_010: "list all files larger than 1GB" → expects WITHOUT exec

**Strategy**: Use regex to distinguish between phrasings
- Pattern A: Strict regex matching exact phrasing "Find all files..." → with exec
- Pattern B: Flexible regex matching variations → without exec

```rust
// Pattern 6: Specific form with strict regex
PatternEntry {
    regex_pattern: Some(Regex::new(r"^find\s+all\s+files?\s+(larger|bigger|over|above|greater).*1\s*(gb?|g)").unwrap()),
    gnu_command: "find . -type f -size +1G -exec ls -lh {} \\;".to_string(),
    // ...
}

// Pattern 7: General form with flexible regex
PatternEntry {
    regex_pattern: Some(Regex::new(r"(?i)(find|locate|list|show).*(files?).*(larger|bigger|over|above|greater).*(1|1gb|1g)").unwrap()),
    gnu_command: "find . -type f -size +1G".to_string(),
    // ...
}
```

**Order**: Strict/specific form before flexible/general form

## Automation Opportunities

### Auto-Sorting on Initialization

Implement automatic pattern sorting by specificity score:

```rust
impl StaticMatcher {
    pub fn new(profile: CapabilityProfile) -> Self {
        let mut patterns = Self::build_patterns();

        // Calculate specificity scores
        patterns.sort_by_key(|p| std::cmp::Reverse(Self::calculate_specificity(p)));

        Self {
            patterns: Arc::new(patterns),
            profile,
        }
    }

    fn calculate_specificity(pattern: &PatternEntry) -> u32 {
        let keyword_score = pattern.required_keywords.len() as u32 * 10;
        let constraint_score = Self::count_constraints(pattern);
        let regex_score = if pattern.regex_pattern.is_some() { 5 } else { 0 };
        keyword_score + constraint_score + regex_score
    }

    fn count_constraints(pattern: &PatternEntry) -> u32 {
        let mut count = 0;
        // Check command string for constraint patterns
        if pattern.gnu_command.contains("-name") { count += 1; } // file type
        if pattern.gnu_command.contains("-size") { count += 1; } // size
        if pattern.gnu_command.contains("-mtime") { count += 1; } // time
        if pattern.gnu_command.contains("~/") { count += 1; } // location
        if pattern.gnu_command.contains("sort") { count += 1; } // sorting
        count
    }
}
```

**Benefits:**
- Prevents future ordering bugs
- Maintains correctness as patterns grow
- Makes specificity explicit in code

### False Positive Detection

Test each pattern against all test queries to detect over-matching:

```bash
# Pseudo-code for detection script
for pattern in patterns:
    for test_query in all_test_queries:
        if pattern.matches(test_query):
            if test_query.expected_pattern != pattern.id:
                print(f"FALSE POSITIVE: Pattern {pattern.id} matches {test_query.id}")
```

## Common Mistakes

### Mistake 1: Not Calculating Specificity

❌ **Bad**: "Pattern A feels more specific, so I'll move it first"

✅ **Good**: Calculate scores: Pattern A (37) vs Pattern B (31) → A before B

### Mistake 2: Ignoring Constraints

❌ **Bad**: Only counting keywords (both patterns have 3 keywords → same specificity)

✅ **Good**: Count constraints too (Pattern A has file_type + time = +2 vs Pattern B has only time = +1)

### Mistake 3: Optional Keywords on Regex-Only Patterns

❌ **Bad**:
```rust
required_keywords: vec![],
optional_keywords: vec!["find", "files"],  // Creates keyword fallback!
regex_pattern: Some(Regex::new(r"[ぁ-んァ-ヶー一-龯]").unwrap()),
```

✅ **Good**:
```rust
required_keywords: vec![],
optional_keywords: vec![],  // No keyword fallback
regex_pattern: Some(Regex::new(r"[ぁ-んァ-ヶー一-龯]").unwrap()),
```

### Mistake 4: Not Testing After Reordering

❌ **Bad**: Make multiple reorderings, test once at the end

✅ **Good**: Test after each reordering to catch regressions immediately

## Success Metrics

Track these metrics to measure reordering success:

**Reordering Success Rate**: Should be 100%
- Every strategic reordering should fix its target test
- If not, specificity calculation was wrong

**Regression Rate**: Should be 0%
- No previously passing tests should fail after reordering
- If regressions occur, reordering disrupted different pattern matches

**ROI per Reordering**: Infinite (0 new patterns → +N tests)
- Reordering adds no new patterns
- Each successful reordering fixes 1+ tests
- Net pattern count may decrease (remove duplicates)

**Example from Cycles 5-8:**
- Reorderings attempted: 10
- Reorderings successful: 10 (100% success rate)
- Tests fixed: 12
- New patterns added: 0
- Regressions: 0
- ROI: Infinite

## Summary

**Golden Rules:**
1. Specific before general (always)
2. Calculate specificity objectively (keywords × 10 + constraints + regex)
3. Test after every change (catch regressions)
4. Batch related reorderings (higher efficiency)
5. Empty keywords for regex-only patterns (prevent fallback)

**Priority:**
1. File-type + multiple constraints (score 40+)
2. File-type + single constraint (score 30-40)
3. File-type only (score 20-30)
4. General + constraint (score 15-25)
5. General queries (score 10-20)

**Expected Outcome:**
- 100% reordering success rate
- 0% regression rate
- Infinite ROI (0 patterns → +N tests)
- Sustainable pattern growth (quality over quantity)
