---
description: Show availability and configuration of external research models (Codex, Gemini)
---

## User Input

```text
$ARGUMENTS
```

---

## What This Command Does

`/mw.status` checks which external CLI models are installed and displays the current multi-workflow configuration. No external model calls are made.

---

## Execution Steps

### 1. Check Model Availability

Run these checks via Bash:

```bash
echo "=== Multi-Workflow Status ==="
echo ""

# Check Codex
if command -v codex >/dev/null 2>&1; then
    CODEX_VERSION=$(codex --version 2>/dev/null || echo "installed (version unknown)")
    echo "codex: AVAILABLE ($CODEX_VERSION)"
else
    echo "codex: NOT INSTALLED"
    echo "  Install: npm install -g @openai/codex"
fi

echo ""

# Check Gemini
if command -v gemini >/dev/null 2>&1; then
    GEMINI_VERSION=$(gemini --version 2>/dev/null || echo "installed (version unknown)")
    echo "gemini: AVAILABLE ($GEMINI_VERSION)"
else
    echo "gemini: NOT INSTALLED"
    echo "  Install: npm install -g @google/gemini-cli"
fi
```

### 2. Read Configuration

Read the config file at `.claude/config/mw-config.toml` using the Read tool.

### 3. Display Status

Present a formatted summary:

```
=== Multi-Workflow Status ===

Models:
  codex:  [AVAILABLE|NOT INSTALLED]
  gemini: [AVAILABLE|NOT INSTALLED]

Configuration:
  Enabled:      true/false
  Auto-suggest: true/false
  Timeout:      60s default

Routing:
  research:       gemini → codex
  code_review:    gemini → codex
  implementation: codex
  architecture:   codex → gemini

Security:
  Claude review required: true
  Max output lines: 500

Degradation Mode:
  [FULL|SINGLE-MODEL|CLAUDE-ONLY]
```

### 4. Degradation Mode

Determine and display the current operating mode:
- **FULL**: Both Codex and Gemini available — cross-validation research enabled
- **SINGLE-MODEL**: Only one model available — single-source research with Claude self-validation
- **CLAUDE-ONLY**: No external models — standard Claude Code behavior (all `mw.*` commands gracefully skip external calls)
