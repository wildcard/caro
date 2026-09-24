# Context Inference Patterns

Strategies for extracting maximum signal from minimal user input, reducing the need for clarification while maintaining accuracy.

## Core Philosophy

> **Users shouldn't need to over-provide information. The tool should be aware of what they're trying to do.**

Every piece of context we can infer is one less question we ask and one more step toward seamless interaction. The goal is to make Caro feel like a knowledgeable colleague who "just gets it."

---

## The Inference Hierarchy

Context should be gathered in priority order. Earlier sources override later ones when they conflict.

```
┌─────────────────────────────────────────┐
│ 1. Explicit User Statement              │ ← Highest priority
│    "use tar", "the zip file"            │
├─────────────────────────────────────────┤
│ 2. File Type Signals                    │
│    .tar.gz → tar, .zip → unzip          │
├─────────────────────────────────────────┤
│ 3. Platform Context                     │
│    macOS → BSD tools, Linux → GNU       │
├─────────────────────────────────────────┤
│ 4. Working Directory Signals            │
│    Cargo.toml → Rust, package.json → Node│
├─────────────────────────────────────────┤
│ 5. Session History                      │
│    Recent commands suggest workflow     │
├─────────────────────────────────────────┤
│ 6. Default Conventions                  │ ← Lowest priority
│    Platform/community standards         │
└─────────────────────────────────────────┘
```

---

## Pattern 1: File Type Inference

### Trigger
User mentions a file by name or references "the file," "this archive," etc.

### Strategy
1. Extract filename from query
2. Parse extension
3. Map to tool via file-tool-map.md
4. Constrain command space accordingly

### Examples

**Strong inference (explicit filename):**
```
Query: "extract data.tar.gz"
Inference:
  - File: data.tar.gz
  - Extension: .tar.gz
  - Tool: tar -xzf
  - Command: tar -xzf data.tar.gz
```

**Medium inference (partial reference):**
```
Query: "unzip the backup"
Context: ls shows backup.zip
Inference:
  - Intent: extract
  - Pattern: *backup*
  - Match: backup.zip
  - Extension: .zip
  - Tool: unzip
  - Command: unzip backup.zip
```

**Weak inference (no file mentioned):**
```
Query: "extract the archive"
Context: ls shows multiple archives (data.tar.gz, docs.zip)
Inference:
  - Intent: extract
  - Multiple matches found
  - Action: Ask "Which archive? data.tar.gz or docs.zip?"
```

### Anti-Pattern: Ignoring Extension

```
WRONG:
  Query: "extract config.zip"
  Command: tar -xzf config.zip  ← Wrong! .zip ≠ tar

RIGHT:
  Query: "extract config.zip"
  Command: unzip config.zip     ← Matches extension
```

---

## Pattern 2: Platform Inference

### Trigger
Always active—platform context should inform every command.

### Strategy
1. Detect OS (macOS, Linux, Windows)
2. Detect distribution (if Linux)
3. Detect shell (bash, zsh, fish, PowerShell)
4. Adjust command syntax and tools accordingly

### Platform-Specific Adaptations

**macOS Adaptations:**
```yaml
sed_in_place: sed -i ''      # BSD sed requires empty string
stat_format: stat -f '%z'    # Different format specifier
date_modification: date -v   # -v for modification
tar_exclude: --exclude       # Works same as GNU
clipboard_copy: pbcopy       # macOS-specific
clipboard_paste: pbpaste     # macOS-specific
open_file: open              # Opens in default app
package_manager: brew        # Homebrew
```

**Linux Adaptations:**
```yaml
sed_in_place: sed -i         # GNU sed, no empty string
stat_format: stat -c '%s'    # Different format specifier
date_modification: date -d   # -d for date strings
clipboard_copy: xclip -sel clip  # X11
clipboard_paste: xclip -sel clip -o
open_file: xdg-open          # Opens in default app
package_manager: apt/yum/dnf # Distribution-dependent
```

**Windows Adaptations:**
```yaml
shell: PowerShell / cmd
archive_extract: Expand-Archive
archive_create: Compress-Archive
clipboard: Set-Clipboard / Get-Clipboard
open_file: Start-Process / start
package_manager: winget / choco / scoop
path_separator: \
```

### Detection Methods

```rust
// In Caro context detection
match std::env::consts::OS {
    "macos" => Platform::MacOS,
    "linux" => Platform::Linux,
    "windows" => Platform::Windows,
    _ => Platform::Unknown,
}

// Distribution detection (Linux)
if Path::new("/etc/os-release").exists() {
    // Parse ID= and VERSION_ID=
}
```

---

## Pattern 3: Project Type Inference

### Trigger
Commands that might be project-specific: build, test, run, deploy.

### Strategy
1. Scan current directory for project markers
2. Identify project type from markers
3. Map to project-native commands
4. Check for custom scripts/targets

### Project Marker Detection

```
Current directory: /home/user/myproject
Contents: Cargo.toml, src/, tests/

Detection:
  - Cargo.toml present → Rust project
  - Build command: cargo build
  - Test command: cargo test
  - Run command: cargo run
```

### Marker Priority (if multiple present)

1. **Primary marker** - Explicit project config
2. **Secondary marker** - Supporting evidence
3. **Tertiary marker** - Language files only

```
Priority Examples:
- Cargo.toml > *.rs files (explicit > implicit)
- package.json > *.js files
- go.mod > *.go files
- pyproject.toml > requirements.txt > *.py files
```

### Multi-Project Detection

```
Directory contains: Cargo.toml, package.json, Dockerfile

Interpretation:
- Primary: Rust (Cargo.toml)
- Secondary: Node (package.json) - likely for frontend/tooling
- Supporting: Docker (Dockerfile) - containerized deployment

For "build" command:
- Default to: cargo build (primary project)
- But mention: "Also found package.json, need npm build too?"
```

---

## Pattern 4: Intent Inference from Verbs

### Trigger
User query contains action verbs.

### Strategy
1. Extract primary verb
2. Map to command category
3. Combine with object (file, target) for full intent

### Verb-to-Intent Mapping

| Verb | Primary Intent | Secondary Intents |
|------|----------------|-------------------|
| "find" | search filesystem | locate, search content |
| "search" | search content | find files, grep |
| "show" | display | list, cat, view |
| "list" | enumerate | ls, find |
| "open" | launch | view, edit |
| "edit" | modify | open in editor |
| "run" | execute | start, launch |
| "build" | compile | make, construct |
| "test" | validate | check, verify |
| "delete" | remove | rm, unlink |
| "copy" | duplicate | cp, replicate |
| "move" | relocate | mv, rename |
| "extract" | decompress | unzip, untar |
| "compress" | archive | zip, tar |
| "download" | fetch | curl, wget |
| "install" | add | package manager |
| "update" | upgrade | refresh, pull |

### Compound Intent Resolution

```
Query: "find and delete all .tmp files"
Verbs: find, delete
Object: .tmp files
Resolution:
  - Primary: find (search)
  - Secondary: delete (action on results)
  - Command: find . -name "*.tmp" -delete
  - Or safer: find . -name "*.tmp" -exec rm {} \;
```

---

## Pattern 5: History-Based Inference

### Trigger
Ambiguous query where history provides clarity.

### Strategy
1. Review recent commands in session
2. Identify workflow patterns
3. Use patterns to disambiguate current query

### Examples

**Workflow continuation:**
```
History:
  1. cd /var/log
  2. ls
  3. grep "error" syslog

Query: "show me the last 100 lines"

Inference:
  - Context: User was exploring /var/log
  - Recent file: syslog
  - Intent: Continue exploring same file
  - Command: tail -n 100 syslog
```

**Pattern recognition:**
```
History:
  1. git add .
  2. git commit -m "..."
  3. (user is now typing)

Query: "push"

Inference:
  - Pattern: git workflow
  - Previous: commit completed
  - Next logical step: push
  - Command: git push
```

**Tool preference:**
```
History:
  1. fd "*.rs"
  2. fd "config"
  3. fd -t f "main"

Query: "find all json files"

Inference:
  - User preference: fd over find (used 3x)
  - Command: fd -e json
  - Not: find . -name "*.json"
```

---

## Pattern 6: Implicit Object Resolution

### Trigger
User uses pronouns or vague references: "it," "this," "the file."

### Strategy
1. Identify referent from context
2. Check recent files, commands, or outputs
3. Resolve to specific target

### Resolution Rules

**"it" / "this" resolution:**
```
Priority order:
1. File mentioned in previous turn
2. Last file argument in recent command
3. Last modified file in cwd
4. Ask for clarification
```

**"the file" resolution:**
```
Priority order:
1. Single file matching recent context
2. File referenced in previous query
3. Most recently accessed file
4. Ask which file
```

**"the output" resolution:**
```
Priority order:
1. stdout from last command (if captured)
2. File created by last command
3. Explain how to capture output
```

### Examples

```
Previous: "compile main.rs"
Query: "run it"
Resolution: "it" → main.rs (binary output)
Command: ./main or cargo run

Previous: "download https://example.com/data.zip"
Query: "extract it"
Resolution: "it" → data.zip
Command: unzip data.zip

Previous: "cat config.json"
Query: "edit this"
Resolution: "this" → config.json
Command: ${EDITOR:-vi} config.json
```

---

## Pattern 7: Constraint Propagation

### Trigger
Multiple context signals that can constrain each other.

### Strategy
1. Gather all available signals
2. Apply constraints to narrow possibilities
3. Verify consistency
4. Generate most specific valid command

### Example: Constraint Narrowing

```
Signals:
  - Query: "compress the folder"
  - Platform: macOS
  - Target: ./myproject/
  - User preference (history): Used tar for .tar.gz before

Constraints applied:
  1. "compress" → archive creation intent
  2. "folder" → directory, needs recursive
  3. macOS → tar and zip both available
  4. History shows tar preference

Narrowed options:
  - tar -czf myproject.tar.gz myproject/  ← Matches history
  - zip -r myproject.zip myproject/        ← Platform native

Decision: Offer tar first (matches user history), mention zip as alternative
```

---

## Pattern 8: Clarification Minimization

### Trigger
Query could go multiple ways, but one path is significantly more likely.

### Strategy
1. Calculate confidence for each interpretation
2. If one significantly higher, proceed with explanation
3. If close, ask targeted question
4. Never ask open-ended questions

### Confidence Calculation

```python
def calculate_confidence(interpretation, context):
    score = 0

    # Explicit signals (high weight)
    if interpretation matches explicit_mention:
        score += 3.0

    # File type match (high weight)
    if file_extension matches tool:
        score += 2.5

    # Platform alignment (medium weight)
    if tool available_on platform:
        score += 1.5

    # History support (medium weight)
    if similar_command in recent_history:
        score += 1.0

    # Convention match (low weight)
    if matches platform_convention:
        score += 0.5

    return score
```

### Action Thresholds

| Confidence | Action |
|------------|--------|
| ≥ 4.0 | Execute directly |
| 3.0 - 4.0 | Execute with brief explanation |
| 2.0 - 3.0 | Suggest with confirmation |
| < 2.0 | Ask targeted clarifying question |

### Targeted Question Format

**Good (specific):**
```
"Which archive? data.tar.gz or backup.zip?"
"Build for debug or release?"
"Delete files matching *.tmp, correct?"
```

**Bad (open-ended):**
```
"What would you like to do?"
"Can you provide more details?"
"Which files are you referring to?"
```

---

## Pattern 9: Negative Inference

### Trigger
Absence of expected signals can also inform interpretation.

### Strategy
1. Note what signals are NOT present
2. Use absence to rule out interpretations
3. Narrow remaining possibilities

### Examples

```
Query: "list files"
Missing: No path mentioned, no flags specified

Negative inference:
  - No path → use current directory
  - No flags → use simple default (ls, not ls -la)
  - No filter → show all (not hidden)

Command: ls
```

```
Query: "build"
Missing: No target, no configuration specified

Negative inference:
  - No target → build all/default
  - No config → development/debug mode
  - No clean flag → incremental build

Command: cargo build (if Rust) / npm run build (if Node)
```

---

## Pattern 10: Graceful Degradation

### Trigger
Unable to infer required context with sufficient confidence.

### Strategy
1. Acknowledge what IS known
2. Clearly state what's missing
3. Offer most reasonable default
4. Provide escape hatch

### Template

```
I can see you want to [inferred intent].

Based on [available context], the most likely command is:
[suggested command]

However, I noticed [ambiguity]. If you meant [alternative], use:
[alternative command]
```

### Example

```
Query: "extract the archive"
Context: Multiple archives in directory

Response:
I can see you want to extract an archive.

Found multiple archives in the current directory:
- data.tar.gz → tar -xzf data.tar.gz
- backup.zip → unzip backup.zip

Which one should I extract?
```

---

## Integration with Prompt System

### Context Injection Points

```
System prompt structure:
┌─────────────────────────────────────────────┐
│ Role and capabilities                        │
├─────────────────────────────────────────────┤
│ Platform context (from Pattern 2)            │ ← Injected
│ - OS: macOS                                  │
│ - Shell: zsh                                 │
│ - Available tools: [list]                    │
├─────────────────────────────────────────────┤
│ Project context (from Pattern 3)             │ ← Injected
│ - Type: Rust                                 │
│ - Build: cargo                               │
├─────────────────────────────────────────────┤
│ Session context (from Pattern 5)             │ ← Injected
│ - Recent commands                            │
│ - Working directory                          │
├─────────────────────────────────────────────┤
│ File context (from Pattern 1)                │ ← Injected
│ - Relevant files in directory                │
│ - File types present                         │
├─────────────────────────────────────────────┤
│ Rules and constraints                        │
│ - Platform-specific syntax                   │
│ - Tool preferences                           │
│ - Safety requirements                        │
└─────────────────────────────────────────────┘
```

### Dynamic Context Template

```markdown
## Current Environment
OS: {platform.os}
Shell: {platform.shell}
CWD: {context.cwd}

## Project Context
Type: {project.type or "Unknown"}
Build System: {project.build_system or "None detected"}

## Available Files
{context.relevant_files}

## Recent Commands
{context.recent_commands[-5:]}

## Tool Availability
Archives: {available.archive_tools}
Search: {available.search_tools}
```

---

## Measuring Inference Quality

### Metrics

1. **Inference Accuracy**
   - Correct tool/command without explicit user specification
   - Target: > 90% on unambiguous queries

2. **Clarification Rate**
   - Questions asked per 100 queries
   - Target: < 15% clarification rate

3. **Context Hit Rate**
   - Queries where useful context was inferred
   - Target: > 70% context utilization

4. **False Positive Rate**
   - Incorrect inferences that led to wrong commands
   - Target: < 5% false positives

### Test Cases for Validation

```yaml
inference_tests:
  - query: "extract data.tar.gz"
    expected_tool: tar
    expected_confidence: high

  - query: "unzip the backup"
    context:
      files: ["backup.zip", "backup.tar.gz"]
    expected_tool: unzip  # .zip from filename pattern
    expected_confidence: medium

  - query: "extract the archive"
    context:
      files: ["data.tar.gz", "backup.zip"]
    expected_action: clarify
    expected_question: "Which archive?"
```
