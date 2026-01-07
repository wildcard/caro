---
title: Performance Analysis
description: "Documentation: Performance Analysis"
editUrl: false
---
**Date:** 2025-12-28
**Analyzed by:** Claude Code Performance Audit

## Executive Summary

Analysis of the caro codebase identified **14 performance anti-patterns** ranging from critical startup blockers to minor allocation inefficiencies. The most impactful issues relate to process spawning, regex compilation in hot paths, and unnecessary I/O operations.

## Critical Issues

### 1. Process Spawning at Startup (CRITICAL)

**Location:** `src/context/mod.rs:167-182`

**Problem:** The `scan_available_commands()` function spawns 40+ `which` subprocesses to detect available commands at startup.

```rust
fn command_exists(command: &str) -> bool {
    Command::new("which")
        .arg(command)
        .output()  // Spawns process for EACH command
        .map(|output| output.status.success())
        .unwrap_or(false)
}
```

**Impact:** Could add 500ms+ to startup, blocking the <100ms startup goal.

**Fix:**
- Use `which-rs` crate for in-process PATH resolution
- Cache results in a static
- Only check critical commands (ps, find, grep, sed, awk)

---

### 2. Regex Compiled in Validation Loop (HIGH)

**Location:** `src/safety/mod.rs:223`

**Problem:** Allowlist patterns are compiled from string on every validation call.

```rust
for allow_pattern in &self.config.allowlist_patterns {
    if let Ok(regex) = regex::Regex::new(allow_pattern) {  // Compiles EVERY validation
        if regex.is_match(command) {
```

**Impact:** O(n) regex compilations per command validation where n = number of allowlist patterns.

**Fix:** Pre-compile patterns in `SafetyConfig` constructor:
```rust
pub struct SafetyConfig {
    pub allowlist_patterns_compiled: Vec<Regex>,  // Pre-compiled
}
```

---

### 3. Pattern Filtering Repeated (HIGH)

**Location:** `src/safety/patterns.rs:366-410`

**Problem:** `get_compiled_patterns_for_shell()` filters all 50+ patterns for every validation.

```rust
pub fn get_compiled_patterns_for_shell(shell: ShellType) -> Vec<&'static CompiledPattern> {
    COMPILED_PATTERNS
        .iter()
        .filter(|(_, _, _, shell_specific)| {
            shell_specific.is_none() || *shell_specific == Some(shell)
        })
        .collect()  // Allocates Vec every time
}
```

**Impact:** Unnecessary filtering and allocation for every command validated.

**Fix:** Pre-split at startup:
```rust
pub static PATTERNS_BY_SHELL: Lazy<HashMap<ShellType, Vec<&CompiledPattern>>> = Lazy::new(|| {
    // Build once at startup
});
```

---

### 4. Cache Manifest Written on Every Access (HIGH)

**Location:** `src/cache/manifest.rs:127`

**Problem:** The entire manifest is serialized and written to disk whenever `update_last_accessed()` is called.

**Impact:** Severe I/O overhead for heavy cache usage.

**Fix:** Implement write-through caching with dirty flag:
```rust
pub struct ManifestManager {
    manifest: CacheManifest,
    dirty: bool,
}

impl Drop for ManifestManager {
    fn drop(&mut self) {
        if self.dirty { self.save().ok(); }
    }
}
```

---

## Medium Priority Issues

### 5. Double Regex Compilation

**Location:** `src/safety/mod.rs:104, 118`

Custom patterns are compiled twice - once for validation, once for use.

### 6. Unnecessary Clone on CLI Args

**Location:** `src/main.rs:230-254`

```rust
fn prompt(&self) -> Option<String> {
    self.prompt.clone()  // Could return Option<&str>
}
```

### 7. Multiple Config File Reads

**Location:** `src/config/mod.rs:90, 171`

Config file read twice if `load()` and `validate_schema()` called in sequence.

### 8. JSON Context Serialized Twice

**Location:** `src/agent/mod.rs:91, 117`

`ExecutionContext` serialized in both `generate_initial()` and `refine_command()`.

### 9. String Contains() Multiple Times

**Location:** `src/safety/mod.rs:285-305`

Multiple `.contains()` calls on same lowercase string for keyword detection.

**Fix:** Use single regex pattern:
```rust
static KEYWORD_PATTERNS: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"delet|remov|recursive|privilege|root|sudo").unwrap()
});
```

### 10. Blocking SHA256 in Async

**Location:** `src/cache/mod.rs:300`

SHA256 digest blocks executor for large model files.

**Fix:**
```rust
let hash = tokio::task::spawn_blocking(move || Sha256::digest(&contents)).await;
```

---

## Low Priority Issues

### 11. Redundant collect()

**Location:** `src/agent/mod.rs:289`

```rust
commands.into_iter().collect()  // Redundant - commands is already Vec
```

### 12. HashSet for Small Deduplication

**Location:** `src/safety/mod.rs:313-314`

Using HashSet to deduplicate 3-5 keywords is overkill.

### 13. trim().to_string() Allocation

**Location:** `src/main.rs:77`

```rust
Ok(buffer.trim().to_string())  // Could trim in-place
```

### 14. Excessive Cloning in resolve_prompt

**Location:** `src/main.rs:287`

Could use `take()` instead of `clone()`.

---

## Implementation Priority

| Priority | Issue | Estimated Impact |
|----------|-------|------------------|
| P0 | Process spawning (which) | -500ms startup |
| P0 | Regex in validation loop | -50ms per validation |
| P1 | Pattern pre-splitting | -10ms per validation |
| P1 | Manifest dirty flag | -50ms per cache access |
| P2 | Double regex compilation | -5ms startup |
| P2 | Config caching | -2ms startup |
| P2 | Context JSON caching | -5ms per generation |
| P3 | Minor allocations | Negligible |

---

## Metrics to Track

After implementing fixes, measure:

1. **Startup time** (target: <100ms)
2. **First inference latency** (target: <2s on M1)
3. **Validation throughput** (commands/second)
4. **Memory usage** (peak RSS)

Use `hyperfine` for benchmarking:
```bash
hyperfine --warmup 3 'caro --help'
hyperfine --warmup 3 'caro "list files"'
```
