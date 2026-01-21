#!/bin/bash
# Create GitHub issues from macOS CLI Tools Research
#
# Usage:
#   ./bin/create-research-issues.sh           # Create all issues
#   ./bin/create-research-issues.sh --dry-run # Preview without creating
#   ./bin/create-research-issues.sh --quick   # Create only quick wins
#   ./bin/create-research-issues.sh --complex # Create only complex tasks
#
# Prerequisites:
#   - gh (GitHub CLI) installed and authenticated
#   - Repository has milestones: v1.2.0, v1.3.0, v2.0.0
#
# Source: docs/research/MACOS_CLI_TOOLS_INSIGHTS.md

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
DRY_RUN=false
FILTER="all"

# Parse arguments
while [[ $# -gt 0 ]]; do
  case $1 in
    --dry-run)
      DRY_RUN=true
      shift
      ;;
    --quick)
      FILTER="quick"
      shift
      ;;
    --complex)
      FILTER="complex"
      shift
      ;;
    *)
      echo "Unknown option: $1"
      echo "Usage: $0 [--dry-run] [--quick|--complex]"
      exit 1
      ;;
  esac
done

# Check prerequisites
if ! command -v gh &> /dev/null; then
  echo -e "${RED}Error: gh (GitHub CLI) is not installed${NC}"
  echo "Install: brew install gh"
  exit 1
fi

if ! gh auth status &> /dev/null; then
  echo -e "${RED}Error: gh is not authenticated${NC}"
  echo "Run: gh auth login"
  exit 1
fi

echo -e "${BLUE}Creating GitHub issues from macOS CLI Tools Research${NC}"
echo "=============================================="
echo ""

if $DRY_RUN; then
  echo -e "${YELLOW}DRY RUN MODE - No issues will be created${NC}"
  echo ""
fi

# Function to create an issue
create_issue() {
  local id="$1"
  local title="$2"
  local labels="$3"
  local milestone="$4"
  local body="$5"

  if $DRY_RUN; then
    echo -e "${GREEN}Would create:${NC} $title"
    echo "  ID: $id"
    echo "  Labels: $labels"
    echo "  Milestone: $milestone"
    echo ""
  else
    echo -e "${BLUE}Creating:${NC} $title"
    gh issue create \
      --title "$title" \
      --label "$labels" \
      --milestone "$milestone" \
      --body "$body"
    echo -e "${GREEN}Created!${NC}"
    echo ""
    # Rate limit protection
    sleep 1
  fi
}

# ============================================================================
# QUICK WINS
# ============================================================================

if [[ "$FILTER" == "all" || "$FILTER" == "quick" ]]; then
  echo -e "${YELLOW}=== Quick Wins (v1.2.0 - v1.3.0) ===${NC}"
  echo ""

  # QW-1: Colorized Output
  create_issue "QW-1" \
    "feat(ux): Add colorized command output with risk level indicators" \
    "enhancement,ux,quick-win,good-first-issue" \
    "v1.2.0" \
    "## Summary

Enhance the command output display with syntax highlighting and color-coded risk levels, inspired by modern CLI tools like bat, lsd, and exa.

## Background

Research on successful macOS CLI tools reveals that beautiful, readable output significantly improves user experience. Tools like \`bat\` (syntax-highlighted cat) and \`lsd\` (colorized ls) have gained massive adoption partly due to their visual appeal.

## Proposed Changes

1. **Syntax highlighting for generated commands**
   - Highlight command name, flags, arguments differently
   - Use consistent color scheme

2. **Risk level color coding**
   - Safe: Green text
   - Moderate: Yellow/Orange text
   - High: Red text
   - Critical: Red background/bold

3. **Visual risk indicator**
   - Add icons or symbols: \`[SAFE]\`, \`[WARN]\`, \`[DANGER]\`

## Acceptance Criteria

- [ ] Commands are syntax-highlighted with consistent colors
- [ ] Risk levels are color-coded (green/yellow/red)
- [ ] Visual indicators show risk level clearly
- [ ] Colors can be disabled via \`--no-color\` flag
- [ ] Respects \`NO_COLOR\` environment variable

## Research Reference

See: \`docs/research/MACOS_CLI_TOOLS_INSIGHTS.md\` - Inspired by bat, lsd, exa

**RICE Score: 45.9**"

  # QW-2: Explain Command
  create_issue "QW-2" \
    "feat(cli): Add \`caro explain\` command for reverse translation" \
    "enhancement,feature,quick-win" \
    "v1.2.0" \
    "## Summary

Add a \`caro explain\` subcommand that takes a shell command and explains what it does in plain English, providing a learning tool for users.

## Background

The success of \`tldr\` demonstrates that users want simplified explanations of commands. Caro can do the reverse of command generation: explain existing commands.

## Proposed Usage

\`\`\`bash
\$ caro explain \"ps aux | grep nginx | awk '{print \$2}' | xargs kill\"

This command kills all nginx processes:
  1. ps aux        - List all running processes
  2. grep nginx    - Filter for lines containing 'nginx'
  3. awk '{print \$2}' - Extract the 2nd column (process IDs)
  4. xargs kill    - Kill each process by ID

Risk: HIGH - Terminates processes without confirmation
\`\`\`

## Acceptance Criteria

- [ ] \`caro explain <command>\` parses and explains commands
- [ ] Breaks down pipes and chains into steps
- [ ] Explains common flags and options
- [ ] Shows risk assessment for the command
- [ ] Works with complex commands (pipes, subshells, redirects)

## Research Reference

See: \`docs/research/MACOS_CLI_TOOLS_INSIGHTS.md\` - Inspired by tldr

**RICE Score: 25.6**"

  # QW-3: Sensible Defaults
  create_issue "QW-3" \
    "feat(defaults): Apply sensible defaults inspired by fd/ripgrep" \
    "enhancement,ux,quick-win" \
    "v1.2.0" \
    "## Summary

Apply sensible defaults to generated commands, following the philosophy of \`fd\` and \`ripgrep\` which optimize for the 90% use case.

## Background

\`fd\` succeeds because it ignores \`.git\`, hidden files, and \`.gitignore\` patterns by default. Users can override, but the defaults are right for most cases.

## Proposed Defaults

1. **File operations**
   - Exclude \`.git\`, \`node_modules\`, \`.venv\` by default
   - Respect \`.gitignore\` when in a git repo
   - Use human-readable sizes (\`-h\` flag where applicable)

2. **Process operations**
   - Sort by CPU/memory by default (most useful)

3. **Network operations**
   - Use HTTPS over HTTP when generating URLs
   - Include timeout flags by default

## Acceptance Criteria

- [ ] Generated \`find\` commands exclude common noise directories
- [ ] File listings use human-readable sizes
- [ ] Defaults can be overridden via config or prompt
- [ ] Document default behaviors in help/docs

## Research Reference

See: \`docs/research/MACOS_CLI_TOOLS_INSIGHTS.md\` - Inspired by fd, ripgrep

**RICE Score: 22.5**"

  # QW-4: Undo Suggestions
  create_issue "QW-4" \
    "feat(safety): Suggest undo commands for destructive operations" \
    "enhancement,safety,quick-win" \
    "v1.2.0" \
    "## Summary

For destructive commands, suggest an undo approach or safer alternative, inspired by \`trash-cli\` which moves files to trash instead of deleting.

## Background

\`trash-cli\` replaces \`rm\` with a recoverable alternative. Caro can apply this philosophy by suggesting safer alternatives or undo approaches.

## Proposed Behavior

When a destructive command is generated:

\`\`\`bash
\$ caro \"delete all .log files\"

Generated command:
  find . -name \"*.log\" -delete

Risk Level: [MODERATE]

Safer alternative:
  find . -name \"*.log\" -exec mv {} ~/.Trash/ \\;

To undo (if you proceed):
  # Check ~/.Trash for .log files
\`\`\`

## Acceptance Criteria

- [ ] Destructive commands show safer alternatives
- [ ] Undo suggestions provided where applicable
- [ ] Integrate with existing safety module
- [ ] Don't block execution, just inform
- [ ] Configurable via safety level setting

## Research Reference

See: \`docs/research/MACOS_CLI_TOOLS_INSIGHTS.md\` - Inspired by trash-cli

**RICE Score: 20.0**"

  # QW-5: Fuzzy Matching
  create_issue "QW-5" \
    "feat(ux): Add fuzzy matching for typos in natural language prompts" \
    "enhancement,ux,quick-win" \
    "v1.2.0" \
    "## Summary

Implement fuzzy matching for user prompts to handle typos and near-matches, inspired by fzf's forgiving input matching.

## Proposed Behavior

\`\`\`bash
\$ caro \"lst files in currnt directroy\"
            ^^^      ^^^^^^  ^^^^^^^^^
            (typos detected, auto-corrected)

Interpreted as: \"list files in current directory\"

Generated command:
  ls -la
\`\`\`

## Implementation Approach

1. **Typo detection** - Levenshtein distance for common words
2. **Fuzzy matching** - \"lst\" -> \"list\", \"currnt\" -> \"current\"
3. **Confirmation** - Show interpreted query if different from input

## Acceptance Criteria

- [ ] Common typos are auto-corrected
- [ ] User is shown interpreted query
- [ ] Original query preserved if user wants it
- [ ] Performance: < 50ms for typo detection

## Research Reference

See: \`docs/research/MACOS_CLI_TOOLS_INSIGHTS.md\` - Inspired by fzf

**RICE Score: 18.0**"

  # QW-6: Examples Subcommand
  create_issue "QW-6" \
    "feat(cli): Add \`caro examples\` subcommand for common patterns" \
    "enhancement,feature,quick-win" \
    "v1.3.0" \
    "## Summary

Add a \`caro examples\` subcommand that shows common command patterns for a given topic, inspired by tldr and navi cheatsheets.

## Proposed Usage

\`\`\`bash
\$ caro examples find

Common 'find' patterns:

  Find files by name:
    find . -name \"*.txt\"

  Find files modified in last 7 days:
    find . -type f -mtime -7

  Find large files (>100MB):
    find . -type f -size +100M

Run \\\`caro \"find ...\"\\\` to generate custom commands.
\`\`\`

## Acceptance Criteria

- [ ] \`caro examples <topic>\` shows common patterns
- [ ] Examples are platform-aware (BSD vs GNU)
- [ ] Can list available topics: \`caro examples --list\`
- [ ] Examples include brief explanations

## Research Reference

See: \`docs/research/MACOS_CLI_TOOLS_INSIGHTS.md\` - Inspired by tldr, navi

**RICE Score: 15.0**"

fi

# ============================================================================
# LONG-RUNNING COMPLEX TASKS
# ============================================================================

if [[ "$FILTER" == "all" || "$FILTER" == "complex" ]]; then
  echo -e "${YELLOW}=== Long-Running Complex Tasks (v1.3.0 - v2.0.0) ===${NC}"
  echo ""

  # LR-1: Context-Aware Suggestions
  create_issue "LR-1" \
    "feat(context): Context-aware command suggestions based on project type" \
    "enhancement,feature,complex" \
    "v1.3.0" \
    "## Summary

Implement context-aware suggestions that detect project type (git repo, npm project, Docker, etc.) and adapt command suggestions accordingly.

## Proposed Behavior

\`\`\`bash
# In a git repository
\$ caro \"show recent changes\"
-> git log --oneline -10

# In an npm project
\$ caro \"install dependencies\"
-> npm install

# In a directory with Dockerfile
\$ caro \"build the container\"
-> docker build -t \$(basename \$PWD) .
\`\`\`

## Context Detection

| Indicator | Context | Command Adaptations |
|-----------|---------|-------------------|
| \`.git/\` | Git repo | Use git commands |
| \`package.json\` | Node.js | Use npm/yarn |
| \`Cargo.toml\` | Rust | Use cargo |
| \`Dockerfile\` | Docker | Use docker commands |

## Acceptance Criteria

- [ ] Detect 10+ common project types
- [ ] Adapt suggestions based on context
- [ ] Context detection < 100ms
- [ ] Allow manual context override

## Research Reference

See: \`docs/research/MACOS_CLI_TOOLS_INSIGHTS.md\` - Inspired by navi, zoxide

**Complexity: Medium**"

  # LR-2: Interactive Cheatsheet
  create_issue "LR-2" \
    "feat(interactive): Add interactive cheatsheet/wizard mode" \
    "enhancement,feature,complex" \
    "v1.3.0" \
    "## Summary

Add an interactive cheatsheet mode where users can browse and select commands from categorized lists, inspired by navi.

## Proposed Usage

\`\`\`bash
\$ caro browse

Select category:
  > File Operations
    Git Commands
    Process Management
    Network Tools

File Operations:
  > Find files
    List files
    Move/Copy files
    Delete files
\`\`\`

## Acceptance Criteria

- [ ] \`caro browse\` launches interactive mode
- [ ] Categories are navigable with arrow keys
- [ ] Fuzzy search within categories
- [ ] Parameters can be filled interactively
- [ ] Generated command shown before execution

## Research Reference

See: \`docs/research/MACOS_CLI_TOOLS_INSIGHTS.md\` - Inspired by navi

**Complexity: Medium**"

  # LR-3: Shell Integration
  create_issue "LR-3" \
    "feat(shell): Inline shell integration for zsh/bash" \
    "enhancement,feature,complex" \
    "v1.3.0" \
    "## Summary

Create shell plugins that enable inline command suggestions without leaving the shell prompt.

## Proposed Usage

\`\`\`bash
# Type trigger sequence
\$ ?? find large files

# Caro suggests inline
\$ find . -type f -size +100M  # [TAB to accept, ESC to cancel]
\`\`\`

## Shell Support

- **Zsh**: Widget-based plugin
- **Bash**: readline integration
- **Fish**: abbreviation or function

## Acceptance Criteria

- [ ] Zsh plugin works with \`??\` trigger
- [ ] Bash plugin works with readline
- [ ] Fish plugin via \`caro.fish\`
- [ ] Installation via \`caro init <shell>\`
- [ ] Async suggestion (non-blocking)

## Research Reference

See: \`docs/research/MACOS_CLI_TOOLS_INSIGHTS.md\` - Inspired by navi, fzf

**Complexity: High**"

  # LR-4: Learning System
  create_issue "LR-4" \
    "feat(learning): Learn from user edits to improve suggestions" \
    "enhancement,feature,complex,privacy" \
    "v2.0.0" \
    "## Summary

Track when users edit generated commands and learn from those corrections to improve future suggestions, inspired by zoxide's frecency algorithm.

## Learning Signals

1. **User accepts command unchanged** -> High confidence pattern
2. **User edits command slightly** -> Learn correction
3. **User rejects command** -> Pattern mismatch

## Privacy Considerations

- All learning happens locally
- No data uploaded
- User can clear learning data
- Learning is opt-in

## Acceptance Criteria

- [ ] Track accept/edit/reject interactions
- [ ] Learn from user corrections
- [ ] Improve suggestions over time
- [ ] All data stored locally
- [ ] User can view/clear learning data
- [ ] Opt-in with clear privacy explanation

## Research Reference

See: \`docs/research/MACOS_CLI_TOOLS_INSIGHTS.md\` - Inspired by zoxide

**Complexity: High**"

  # LR-5: TUI Mode
  create_issue "LR-5" \
    "feat(tui): Add visual TUI mode for complex operations" \
    "enhancement,feature,complex" \
    "v2.0.0" \
    "## Summary

Add a TUI (Terminal User Interface) mode for complex multi-step operations, inspired by LazyGit and bottom.

## Proposed Usage

\`\`\`
\$ caro tui

+----------------------------------------------------------+
| Caro - Command Assistant                                 |
+----------------------------------------------------------+
| Prompt: find large files in Downloads                    |
+----------------------------------------------------------+
| Generated Command:                                       |
|   find ~/Downloads -type f -size +100M                   |
|                                                          |
| Risk: [SAFE]  Platform: macOS  Shell: zsh                |
+----------------------------------------------------------+
| [E]xecute  [C]opy  [M]odify  [H]istory  [Q]uit           |
+----------------------------------------------------------+
\`\`\`

## Acceptance Criteria

- [ ] \`caro tui\` launches TUI mode
- [ ] Prompt input with live generation
- [ ] Command preview with explanation
- [ ] History browsing
- [ ] Keyboard navigation

## Research Reference

See: \`docs/research/MACOS_CLI_TOOLS_INSIGHTS.md\` - Inspired by LazyGit, bottom

**Complexity: High**"

  # LR-6: Conversational Mode
  create_issue "LR-6" \
    "feat(chat): Add conversational mode for iterative refinement" \
    "enhancement,feature,complex" \
    "v2.0.0" \
    "## Summary

Add a conversational mode where users can iteratively refine commands through dialogue.

## Proposed Usage

\`\`\`bash
\$ caro chat

You: find large files
Caro: How large? (e.g., >10MB, >100MB, >1GB)

You: bigger than 500MB
Caro: Where should I look? (current directory, home, specific path)

You: in my downloads folder
Caro: Here's your command:
  find ~/Downloads -type f -size +500M

[Execute? Y/n]
\`\`\`

## Acceptance Criteria

- [ ] \`caro chat\` enters conversational mode
- [ ] Caro asks clarifying questions
- [ ] Context retained within session
- [ ] Commands can be refined iteratively
- [ ] Clear exit mechanism

## Research Reference

See: \`docs/research/MACOS_CLI_TOOLS_INSIGHTS.md\` - Inspired by Gitless

**Complexity: High**"

fi

# Summary
echo ""
echo "=============================================="
if $DRY_RUN; then
  echo -e "${YELLOW}DRY RUN COMPLETE - No issues created${NC}"
  echo "Run without --dry-run to create issues"
else
  echo -e "${GREEN}Issue creation complete!${NC}"
  echo "View issues: gh issue list"
fi
