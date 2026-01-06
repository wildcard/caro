# Known Bugs

Tracking known bugs for future investigation and fixes.

---

## BUG-001: Search highlight double-counting with global regex

**Status:** Fixed (2026-01-02)
**Component:** `website/src/components/SearchHighlight.astro`
**Severity:** Medium

### Description
The search highlight feature was counting matches twice (e.g., showing "6 of 6" when there were only 3 actual matches on the page).

### Root Cause
Using `.test()` on a regex with the `g` (global) flag is **stateful**. JavaScript maintains a `lastIndex` property on global regexes that advances after each successful match. Subsequent `.test()` calls start searching from `lastIndex`, causing alternating true/false results for the same input.

```javascript
// Problematic code:
const pattern = new RegExp(`(${tokens.join('|')})`, 'gi');  // global flag

// Each call advances lastIndex, causing inconsistent results
if (pattern.test(part)) { ... }  // true
if (pattern.test(part)) { ... }  // false (starts from lastIndex)
if (pattern.test(part)) { ... }  // true (wrapped around)
```

### Fix Applied
Reset `lastIndex` to 0 before each `.test()` call:

```javascript
pattern.lastIndex = 0;  // Reset stateful regex
if (pattern.test(part)) { ... }
```

### Lessons Learned
- Never use `.test()` with global (`g`) regex in loops without resetting `lastIndex`
- Alternatives: use `.match()` instead, or create a non-global regex for testing
- This is a common JavaScript gotcha with stateful regex operations

### References
- MDN: [RegExp.prototype.test() - Using test() on a regex with the global flag](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/RegExp/test#using_test_on_a_regex_with_the_global_flag)
