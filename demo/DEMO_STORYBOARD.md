# cmdai - ASCII Cinema Demo Storyboard

## Overview
This demo showcases cmdai's core value proposition: converting natural language to safe, POSIX-compliant shell commands using local LLMs with comprehensive safety validation.

## Target Audience
- Developers looking for CLI productivity tools
- System administrators managing complex shell operations
- DevOps engineers automating workflows
- Anyone who struggles with complex shell syntax

## Demo Duration
Approximately 2-3 minutes (optimal for attention span and showcasing features)

## Color Scheme
- **Cyan/Bright Blue**: Generated commands (primary output)
- **Green**: Success states, confirmations, safe operations
- **Yellow**: Warnings, moderate risk operations
- **Red**: Blocked/dangerous operations, critical warnings
- **Gray/Dimmed**: Explanations, metadata, timing info

---

## Scene Breakdown

### Scene 1: Title Card (0:00-0:05)
**Purpose**: Establish brand identity and value proposition

```
╔══════════════════════════════════════════════════════════════╗
║                                                              ║
║                         cmdai                                ║
║                                                              ║
║        Natural Language → Safe Shell Commands                ║
║                                                              ║
║              Powered by Local LLMs                           ║
║                                                              ║
╚══════════════════════════════════════════════════════════════╝

Press Enter to begin demo...
```

**Timing**: 2-3 seconds pause

---

### Scene 2: Simple File Listing (0:05-0:15)
**Purpose**: Show basic functionality - simple natural language query

**Input**:
```bash
$ cmdai "list all files in the current directory"
```

**Output**:
```
Command:
  ls -la

Explanation:
  Lists all files including hidden files with detailed information

⚡ Generated in 342ms
```

**Notes**:
- Shows instant command generation
- Clean, readable output
- Timing info demonstrates speed

---

### Scene 3: Complex File Operation (0:15-0:30)
**Purpose**: Demonstrate handling of complex queries with multiple conditions

**Input**:
```bash
$ cmdai "find all PDF files in Downloads larger than 10MB and sort by date"
```

**Output**:
```
Command:
  find ~/Downloads -name "*.pdf" -size +10M -printf "%T@ %p\n" | sort -n | cut -d' ' -f2-

Explanation:
  Searches Downloads directory for PDF files over 10MB and sorts them
  by modification time in ascending order

Alternatives:
  • find ~/Downloads -name "*.pdf" -size +10M -ls | sort -k8,9
  • ls -lhS ~/Downloads/*.pdf | awk '$5 ~ /[0-9]+M/ && $5+0 > 10'

⚡ Generated in 581ms
```

**Notes**:
- Shows handling of complex multi-part queries
- Provides alternatives for different approaches
- Proper quoting and path handling

---

### Scene 4: Safety Feature - Dangerous Command Blocked (0:30-0:45)
**Purpose**: Highlight safety-first design with critical risk blocking

**Input**:
```bash
$ cmdai "delete everything in root directory recursively"
```

**Output**:
```
🚫 Command Blocked

Generated command:
  rm -rf /

Risk Level: CRITICAL

Blocked: Detected 2 dangerous pattern(s) at Critical risk level (deletion, recursive)
  • Critical: Recursive deletion of root directory
  • Critical: Command would delete entire filesystem

This command is blocked in strict mode for safety.
Use --safety permissive to override (NOT RECOMMENDED).

⚡ Generated in 198ms
```

**Notes**:
- Clear visual indication of blocking (🚫)
- Explains WHY it's dangerous
- Shows the command would have been generated (transparency)
- Provides override path (but discourages it)

---

### Scene 5: Interactive Confirmation - Moderate Risk (0:45-1:10)
**Purpose**: Show interactive confirmation workflow for risky operations

**Input**:
```bash
$ cmdai "remove all log files older than 30 days"
```

**Output**:
```
Command:
  find . -name "*.log" -type f -mtime +30 -delete

Explanation:
  Finds and removes log files not modified in the last 30 days

⚠️  Warning: High risk level (deletion, recursive)
  • High: Recursive file deletion operation

Execute this command? (y/N) █
```

**User Action**: Type `n` and press Enter

**Continued Output**:
```
Execute this command? (y/N) n

⚠️  Operation cancelled by user.

⚡ Generated in 445ms
```

**Notes**:
- Shows interactive prompt with default=No for safety
- Clear warning indicators
- Respects user decision
- Timing shown after cancellation

---

### Scene 6: Verbose Mode with Debug Info (1:10-1:30)
**Purpose**: Demonstrate verbose mode for power users and debugging

**Input**:
```bash
$ cmdai --verbose "show disk usage sorted by size"
```

**Output**:
```
[DEBUG] Loading configuration from ~/.config/cmdai/config.toml
[DEBUG] Using backend: Embedded (MLX - Apple Silicon)
[DEBUG] Model: Qwen2.5-Coder-1.5B-Instruct-Q8
[DEBUG] Safety level: Moderate

Command:
  du -sh * | sort -h

Explanation:
  Displays disk usage for all items in current directory,
  sorted from smallest to largest in human-readable format

Debug Info:
  • Backend: Embedded/MLX
  • Model tokens: 127 (prompt), 43 (completion)
  • Inference time: 1,243ms
  • Safety checks: 0 patterns matched
  • Risk level: Safe

⚡ Total: 1,334ms (generation: 1,243ms, safety: 91ms)
```

**Notes**:
- Shows internal workings for transparency
- Token usage and performance metrics
- Backend information
- Detailed timing breakdown

---

### Scene 7: JSON Output Format (1:30-1:45)
**Purpose**: Show machine-readable output for scripting/integration

**Input**:
```bash
$ cmdai --output json "count files in each subdirectory"
```

**Output**:
```json
{
  "generated_command": "find . -maxdepth 1 -type d -exec sh -c 'echo -n \"{}: \"; find \"{}\" -type f | wc -l' \\;",
  "explanation": "Counts files in each immediate subdirectory",
  "executed": false,
  "blocked_reason": null,
  "requires_confirmation": false,
  "alternatives": [
    "for dir in */; do echo \"$dir: $(find \"$dir\" -type f | wc -l)\"; done"
  ],
  "shell_used": "bash",
  "risk_level": "Safe",
  "timing_info": {
    "generation_time_ms": 512,
    "safety_validation_ms": 34,
    "total_time_ms": 546
  }
}
```

**Notes**:
- Clean JSON formatting
- All relevant fields included
- Perfect for CI/CD integration

---

### Scene 8: Different Shell Targets (1:45-2:00)
**Purpose**: Show cross-shell support

**Input**:
```bash
$ cmdai --shell fish "list processes using more than 100MB memory"
```

**Output**:
```
Command:
  ps aux | awk '$6 > 102400 {printf "%s\t%sMB\t%s\n", $2, $6/1024, $11}'

Explanation:
  Lists processes using more than 100MB of memory
  (Fish-compatible POSIX command)

Target Shell: fish
Safety: ✓ Safe (no dangerous patterns detected)

⚡ Generated in 423ms
```

**Notes**:
- Shows --shell flag usage
- POSIX-compliant output works across shells
- Clear indication of target shell

---

### Scene 9: Real-World Workflow (2:00-2:30)
**Purpose**: Demonstrate practical use case with multiple steps

**Input**:
```bash
$ cmdai "compress all images in downloads to 80% quality and move to archive folder"
```

**Output**:
```
Command:
  mkdir -p archive && find ~/Downloads -type f \( -iname "*.jpg" -o -iname "*.jpeg" -o -iname "*.png" \) -exec sh -c 'mogrify -quality 80 "$1" && mv "$1" archive/' _ {} \;

Explanation:
  Creates archive directory if needed, finds all image files (JPG, JPEG, PNG)
  in Downloads, compresses each to 80% quality, and moves them to archive/

⚠️  Note: Requires ImageMagick (mogrify) to be installed
⚠️  Note: This operation modifies original files before moving

Alternatives:
  • Use convert instead: for img in ~/Downloads/*.{jpg,png}; do convert "$img" -quality 80 "archive/$(basename "$img")"; done

Execute this command? (y/N) y

✓ Confirmed. Proceeding with command execution.

Command:
  mkdir -p archive && find ~/Downloads -type f \( -iname "*.jpg" -o -iname "*.jpeg" -o -iname "*.png" \) -exec sh -c 'mogrify -quality 80 "$1" && mv "$1" archive/' _ {} \;

⚡ Generated in 734ms
```

**Notes**:
- Complex real-world scenario
- Helpful warnings about dependencies
- Alternative approaches offered
- Confirmation flow for file operations

---

### Scene 10: Feature Summary (2:30-2:45)
**Purpose**: Recap key features and call-to-action

```
╔══════════════════════════════════════════════════════════════╗
║                                                              ║
║                    cmdai Features                            ║
║                                                              ║
║  ✓ Natural language → Shell commands                         ║
║  ✓ Comprehensive safety validation                           ║
║  ✓ Interactive confirmation for risky operations             ║
║  ✓ Multiple output formats (JSON, YAML, Plain)               ║
║  ✓ Cross-shell support (bash, zsh, fish, sh)                 ║
║  ✓ Local LLM inference (Apple Silicon optimized)             ║
║  ✓ Fast generation (<2s on M1 Mac)                           ║
║  ✓ POSIX-compliant commands                                  ║
║                                                              ║
║  Get started: cargo install cmdai                            ║
║  Learn more: github.com/wildcard/cmdai                       ║
║                                                              ║
╚══════════════════════════════════════════════════════════════╝
```

**Timing**: Hold for 3-4 seconds

---

## Technical Requirements

### Dependencies for Recording
- `asciinema` for recording terminal sessions
- `svg-term-cli` for converting to animated SVG (optional)
- Mock responses prepared for consistent timing
- Color configuration in terminal (ANSI colors)

### Terminal Configuration
- Size: 100x30 (width x height)
- Font: Monospace, 14px
- Theme: Dark background with high contrast
- Shell: bash (most universal)

### Recording Commands
```bash
# Record the demo
asciinema rec demo.cast --overwrite

# Convert to GIF (optional)
agg demo.cast demo.gif

# Convert to SVG (optional)
svg-term --in demo.cast --out demo.svg --window
```

### Mock Data Preparation
Since cmdai is in development, prepare mock responses that match expected output:
- Pre-generated commands
- Realistic timing (200-800ms)
- Consistent safety warnings
- Proper ANSI color codes

---

## Narrative Flow Summary

1. **Hook** (0-15s): Quick win with simple command generation
2. **Complexity** (15-30s): Show it handles complex queries
3. **Safety** (30-1:10): Core differentiator - safety features
4. **Power User** (1:10-2:00): Advanced features (verbose, JSON, shells)
5. **Real World** (2:00-2:30): Practical workflow example
6. **CTA** (2:30-2:45): Summary and next steps

---

## Key Messages

1. **Speed**: "⚡ Generated in XXXms" appears consistently
2. **Safety**: Multiple demonstrations of safety validation
3. **Flexibility**: Different modes, formats, and shells
4. **Transparency**: Explanations, alternatives, debug info
5. **Production Ready**: Professional output and error handling

---

## Recording Tips

1. **Pacing**: Pause 1-2 seconds between commands to let viewers read
2. **Typing Speed**: Use realistic typing speed (50-80 WPM)
3. **Errors**: Don't include typos/corrections in final demo
4. **Consistency**: Keep terminal size and colors consistent
5. **Audio**: Consider adding background music (optional)
6. **Captions**: Add text overlays for key features (if using video)

---

## Alternative Format: GIF Loop

For GitHub README, create a shorter 30-second loop showing:
1. Simple command (5s)
2. Complex command (8s)
3. Safety blocking (7s)
4. Interactive confirmation (7s)
5. Feature banner (3s)

This loops seamlessly and fits GitHub's 10MB GIF limit.
