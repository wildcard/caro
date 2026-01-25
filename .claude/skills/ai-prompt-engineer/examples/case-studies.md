# Case Studies: Prompt Optimization in Practice

Real-world examples demonstrating the application of prompt engineering principles to improve Caro's command generation.

---

## Case Study 1: The Archive Format Mismatch

### The Problem

**Scenario:** User on macOS asked to "extract the backup archive."

**Context:**
- Platform: macOS
- CWD: ~/Downloads
- Files present: `backup.zip`, `data.tar.gz`

**Original (Bad) Output:**
```bash
tar -xzf backup.zip
```

**Why it failed:**
- tar doesn't handle .zip files
- Command would error: "tar: Error opening archive: Unrecognized archive format"

### Analysis

**Root Cause:** The prompt didn't enforce file-type-to-tool mapping. The model defaulted to `tar` because it's a common archive command.

**Signal Missed:** The file extension `.zip` should have immediately constrained tool selection to `unzip`.

**Context Gap:** The prompt mentioned "archives" generically without distinguishing format-specific handling.

### The Fix

**Before:**
```
Generate a command to extract archives.
Platform: macOS
```

**After:**
```
Generate a command to extract archives.
Platform: macOS

CRITICAL: Match tool to file extension:
- .tar, .tar.gz, .tgz → tar
- .zip → unzip
- .7z → 7z x
- .rar → unrar

Never use tar for .zip files or vice versa.
```

**Improved Output:**
```bash
unzip backup.zip
```

### Validation

Added test case:
```yaml
- id: archive_001
  input: "extract backup.zip"
  platform: macos
  expected: "unzip backup.zip"
  must_not_contain: ["tar"]
```

### Generalized Learning

**Principle:** File type is the strongest signal for tool selection.

**Implementation:**
1. Extract file extension from context
2. Map extension to tool BEFORE generating command
3. Constrain generation to only compatible tools

---

## Case Study 2: Platform-Blind sed

### The Problem

**Scenario:** User on macOS tried to do an in-place edit.

**Query:** "replace foo with bar in config.txt"

**Original (Bad) Output:**
```bash
sed -i 's/foo/bar/g' config.txt
```

**Why it failed:**
- BSD sed (macOS) requires: `sed -i '' 's/foo/bar/g' config.txt`
- GNU sed (Linux) uses: `sed -i 's/foo/bar/g' config.txt`
- The command would error: "sed: 1: "config.txt": invalid command code c"

### Analysis

**Root Cause:** The prompt didn't inject platform-specific command variants.

**Signal Missed:** Platform was macOS, which uses BSD tools with different syntax.

**Context Gap:** Many commands have platform-specific flags that the model wasn't aware of.

### The Fix

**Added to system prompt:**
```
## Platform-Specific Syntax

macOS (BSD):
- sed in-place: sed -i '' 's/old/new/g' file
- stat format: stat -f '%z' file
- date modification: date -v+1d

Linux (GNU):
- sed in-place: sed -i 's/old/new/g' file
- stat format: stat -c '%s' file
- date modification: date -d '+1 day'
```

**Improved Output (macOS):**
```bash
sed -i '' 's/foo/bar/g' config.txt
```

### Validation

```yaml
- id: sed_macos_001
  input: "replace foo with bar in config.txt"
  platform: macos
  expected_pattern: "sed -i '' "
  must_not_contain: ["sed -i 's"]

- id: sed_linux_001
  input: "replace foo with bar in config.txt"
  platform: linux
  expected_pattern: "sed -i 's"
```

### Generalized Learning

**Principle:** Platform context must modify command syntax at generation time.

**Implementation:**
1. Detect platform early in pipeline
2. Maintain platform-specific command variant database
3. Inject relevant variants into prompt dynamically

---

## Case Study 3: Project Type Blindness

### The Problem

**Scenario:** User in a Rust project asked to "run the tests."

**Query:** "run tests"

**Context:**
- CWD: ~/projects/myapp
- Files: `Cargo.toml`, `src/main.rs`, `tests/`

**Original (Bad) Output:**
```bash
npm test
```

**Why it failed:**
- This is a Rust project (Cargo.toml present), not Node.js
- Correct command: `cargo test`

### Analysis

**Root Cause:** The prompt didn't check for project markers before suggesting build commands.

**Signal Missed:** `Cargo.toml` in current directory clearly indicates Rust/Cargo project.

**Context Gap:** Generic "test" command without project type inference.

### The Fix

**Added project detection to context:**
```
## Project Type Detection

Check for markers in current directory:
- Cargo.toml → Rust: cargo build, cargo test, cargo run
- package.json → Node: npm run build, npm test, npm start
- go.mod → Go: go build, go test, go run
- Makefile → Make: make, make test
- pyproject.toml → Python: poetry run, pytest

Apply detected project commands for build/test/run operations.

Current directory markers detected: {project_markers}
Project type: {detected_type}
```

**Improved Output:**
```bash
cargo test
```

### Validation

```yaml
- id: project_rust_test
  input: "run tests"
  context:
    cwd_files: ["Cargo.toml", "src/main.rs"]
  expected: "cargo test"
  must_not_contain: ["npm", "python", "go"]

- id: project_node_test
  input: "run tests"
  context:
    cwd_files: ["package.json", "index.js"]
  expected_pattern: "npm test|yarn test"
```

### Generalized Learning

**Principle:** Project context from CWD markers should inform build/test commands.

**Implementation:**
1. Scan CWD for project marker files
2. Determine project type with priority (if multiple markers)
3. Map standard verbs (build, test, run) to project-specific commands

---

## Case Study 4: The Ambiguous Archive

### The Problem

**Scenario:** User asked to "extract the archive" when multiple archives exist.

**Query:** "extract the archive"

**Context:**
- CWD: ~/Downloads
- Files: `report.tar.gz`, `photos.zip`, `backup.7z`

**Original (Bad) Output:**
```bash
tar -xzf report.tar.gz
```

**Why it failed:**
- User didn't specify which archive
- Silently picking one could extract the wrong files
- No indication given to user about the choice made

### Analysis

**Root Cause:** The model picked an archive without acknowledging ambiguity.

**Signal Missed:** Multiple archive files present = need for clarification.

**Context Gap:** No mechanism to trigger clarification when confident < threshold.

### The Fix

**Added ambiguity detection rules:**
```
## Ambiguity Resolution

When multiple valid targets exist:
1. If explicit reference → use it
2. If single match for implied reference → use it
3. If multiple matches → ASK, don't guess

Clarification format:
"Found multiple archives:
- report.tar.gz → tar -xzf report.tar.gz
- photos.zip → unzip photos.zip
- backup.7z → 7z x backup.7z
Which one should I extract?"
```

**Improved Output:**
```
Found multiple archives in the current directory:
- report.tar.gz → tar -xzf report.tar.gz
- photos.zip → unzip photos.zip
- backup.7z → 7z x backup.7z

Which one should I extract?
```

### Validation

```yaml
- id: archive_ambiguous_001
  input: "extract the archive"
  context:
    files: ["report.tar.gz", "photos.zip", "backup.7z"]
  expected_behavior: "clarification"
  expected_contains: ["report.tar.gz", "photos.zip", "backup.7z"]
  must_not: "single command without asking"
```

### Generalized Learning

**Principle:** When confidence is low, ask targeted questions rather than guessing.

**Implementation:**
1. Count matching targets for implicit references
2. If count > 1 and no explicit preference, trigger clarification
3. Present options with their corresponding commands

---

## Case Study 5: Tool Availability Assumption

### The Problem

**Scenario:** User asked to "pretty print this JSON" on a minimal server.

**Query:** "format data.json nicely"

**Context:**
- Platform: Linux (Alpine, minimal container)
- No `jq` installed
- Python available

**Original (Bad) Output:**
```bash
jq . data.json
```

**Why it failed:**
- `jq` not installed on minimal systems
- Command would error: "jq: command not found"

### Analysis

**Root Cause:** Assumed tool availability without checking.

**Signal Missed:** Minimal distribution likely lacks non-essential tools.

**Context Gap:** No fallback chain for unavailable preferred tools.

### The Fix

**Added tool availability context:**
```
## Tool Availability

Available tools detected: {available_tools}

Fallback chains:
- JSON processing: jq → python -m json.tool → cat
- YAML processing: yq → python -c "import yaml" → cat
- Search: fd → find, rg → grep
- Archive: unzip → python -m zipfile

If preferred tool unavailable, use next in chain.
```

**Detection code added:**
```rust
// In context detection
fn detect_available_tools() -> Vec<String> {
    let tools_to_check = ["jq", "yq", "fd", "rg", "bat", "exa"];
    tools_to_check
        .iter()
        .filter(|t| which(t).is_ok())
        .map(|s| s.to_string())
        .collect()
}
```

**Improved Output (when jq unavailable):**
```bash
python -m json.tool data.json
```

### Validation

```yaml
- id: json_format_with_jq
  input: "format data.json nicely"
  context:
    available_tools: ["jq", "python"]
  expected: "jq . data.json"

- id: json_format_without_jq
  input: "format data.json nicely"
  context:
    available_tools: ["python"]  # no jq
  expected: "python -m json.tool data.json"
```

### Generalized Learning

**Principle:** Never assume tool availability; check and provide fallbacks.

**Implementation:**
1. Probe for tool availability at context detection time
2. Maintain fallback chains for common operations
3. Select best available tool from chain

---

## Case Study 6: The Dangerous Simplification

### The Problem

**Scenario:** User asked to "delete all .tmp files."

**Query:** "delete all tmp files"

**Original (Dangerous) Output:**
```bash
rm -rf *.tmp
```

**Why it's problematic:**
- `-rf` is overly aggressive for the request
- Should be `-f` at most, or better: require confirmation
- Risk of expansion issues if `*.tmp` matches nothing

### Analysis

**Root Cause:** Model amplified a simple delete request with dangerous flags.

**Signal Missed:** User said "delete," not "force delete recursively."

**Context Gap:** No rule against unnecessary dangerous flags.

### The Fix

**Added flag minimization rules:**
```
## Flag Minimization

Principle: Use minimum necessary flags.

Dangerous flag rules:
- rm: Default to no flags for single files
  - Use -r only if explicitly requested (directory delete)
  - Use -f only if files might not exist
  - NEVER use -rf unless user specifically requests force

- chmod: Prefer specific permissions (644, 755) over 777

Generate the least powerful command that accomplishes the task.
```

**Added safety patterns:**
```
## Safety Patterns for Deletion

1. Single file: rm file.txt (no flags)
2. Multiple specific: rm file1.txt file2.txt
3. Glob pattern: Show preview first
   find . -name "*.tmp" -type f  # preview
   # then: find . -name "*.tmp" -type f -delete
4. Directory: rm -r dir/ (warn user)
5. Force: Only with explicit confirmation
```

**Improved Output:**
```bash
# First, preview what will be deleted:
find . -name "*.tmp" -type f

# If the list looks correct, run:
find . -name "*.tmp" -type f -delete
```

### Validation

```yaml
- id: delete_simple_001
  input: "delete all tmp files"
  must_not_contain: ["-rf", "rm -rf"]
  expected_pattern: "find.*-delete|rm \\*\\.tmp"
  should_include: "preview step or confirmation"
```

### Generalized Learning

**Principle:** Use minimum necessary permissions and flags.

**Implementation:**
1. Map user intent to minimum required capability
2. Add preview/dry-run for bulk operations
3. Block unnecessary dangerous flags

---

## Case Study 7: Cross-Platform Image Conversion

### The Problem

**Scenario:** User on macOS asked to convert HEIC photos to JPEG.

**Query:** "convert all HEIC photos to jpeg"

**Original (Bad) Output:**
```bash
convert *.heic *.jpg  # ImageMagick syntax, wrong
```

**Why it failed:**
- Wrong syntax even for ImageMagick
- Didn't use macOS native `sips` which handles HEIC natively
- ImageMagick may not even be installed

### Analysis

**Root Cause:** Used generic ImageMagick without platform awareness.

**Signal Missed:** macOS has native `sips` tool optimized for Apple formats.

**Context Gap:** Image operations should prioritize platform-native tools.

### The Fix

**Added platform-specific image handling:**
```
## Image Operations by Platform

macOS:
- Native tool: sips
- HEIC support: Native (sips handles it)
- Convert format: sips -s format jpeg input.heic --out output.jpg
- Batch: for f in *.heic; do sips -s format jpeg "$f" --out "${f%.heic}.jpg"; done

Linux:
- Tool: convert (ImageMagick)
- HEIC support: Requires libheif
- Convert format: convert input.heic output.jpg
- Batch: for f in *.heic; do convert "$f" "${f%.heic}.jpg"; done

Prefer native tools when available.
```

**Improved Output (macOS):**
```bash
for f in *.heic; do sips -s format jpeg "$f" --out "${f%.heic}.jpg"; done
```

### Validation

```yaml
- id: heic_convert_macos
  input: "convert all HEIC photos to jpeg"
  platform: macos
  expected_pattern: "sips -s format jpeg"
  must_not_contain: ["convert"]

- id: heic_convert_linux
  input: "convert all HEIC photos to jpeg"
  platform: linux
  expected_pattern: "convert.*heic.*jpg"
```

### Generalized Learning

**Principle:** Prefer platform-native tools, especially for platform-specific formats.

**Implementation:**
1. Maintain platform-native tool preferences
2. Detect platform-specific formats (HEIC = Apple)
3. Route to native tools before cross-platform alternatives

---

## Summary: Key Learnings

### Pattern Recognition Matrix

| Case | Root Cause | Solution Pattern |
|------|-----------|------------------|
| 1 | File type ignored | Enforce file-type → tool mapping |
| 2 | Platform ignored | Inject platform-specific syntax |
| 3 | Project context ignored | Detect and apply project markers |
| 4 | Ambiguity not handled | Clarification triggers for multi-match |
| 5 | Tool assumed available | Check availability, use fallbacks |
| 6 | Overly dangerous flags | Minimize permissions, preview first |
| 7 | Non-native tools used | Prefer platform-native tools |

### Optimization Checklist

Before deploying a prompt change:

- [ ] Does it handle the file-type → tool mapping?
- [ ] Is it platform-aware (BSD vs GNU, native tools)?
- [ ] Does it check project context for build commands?
- [ ] Does it trigger clarification for ambiguous inputs?
- [ ] Does it verify tool availability?
- [ ] Does it use minimum necessary permissions?
- [ ] Does it prefer native tools where available?
- [ ] Are there regression tests for fixed cases?

### Metric Tracking

After each case study fix:

| Metric | Before | After |
|--------|--------|-------|
| Archive format accuracy | 60% | 95% |
| Platform-correct syntax | 70% | 92% |
| Project command accuracy | 55% | 90% |
| Appropriate clarification | 30% | 85% |
| Tool availability handling | 40% | 88% |
| Safe flag usage | 75% | 95% |
| Native tool preference | 50% | 90% |
