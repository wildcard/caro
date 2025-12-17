# Agent Rules for Spec Kitty Projects

**⚠️ CRITICAL**: All AI agents working in this project must follow these rules.

These rules apply to **all commands** (specify, plan, research, tasks, implement, review, merge, etc.).

---

## 1. Path Reference Rule

**When you mention directories or files, provide either the absolute path or a path relative to the project root.**

✅ **CORRECT**:
- `kitty-specs/001-feature/tasks/planned/WP01.md`
- `/Users/robert/Code/myproject/kitty-specs/001-feature/spec.md`
- `tasks/planned/WP01.md` (relative to feature directory)

❌ **WRONG**:
- "the tasks folder" (which one? where?)
- "WP01.md" (in which lane? which feature?)
- "the spec" (which feature's spec?)

**Why**: Clarity and precision prevent errors. Never refer to a folder by name alone.

---

## 2. UTF-8 Encoding Rule

**When writing ANY markdown, JSON, YAML, CSV, or code files, use ONLY UTF-8 compatible characters.**

### What to Avoid (Will Break the Dashboard)

❌ **Windows-1252 smart quotes**: " " ' ' (from Word/Outlook/Office)
❌ **Em/en dashes and special punctuation**: — –
❌ **Copy-pasted arrows**: → (becomes illegal bytes)
❌ **Multiplication sign**: × (0xD7 in Windows-1252)
❌ **Plus-minus sign**: ± (0xB1 in Windows-1252)
❌ **Degree symbol**: ° (0xB0 in Windows-1252)
❌ **Copy/paste from Microsoft Office** without cleaning

**Real examples that crashed the dashboard:**
- "User's favorite feature" → "User's favorite feature" (smart quote)
- "Price: $100 ± $10" → "Price: $100 +/- $10"
- "Temperature: 72°F" → "Temperature: 72 degrees F"
- "3 × 4 matrix" → "3 x 4 matrix"

### What to Use Instead

✅ Standard ASCII quotes: `"`, `'`
✅ Hyphen-minus: `-` instead of en/em dash
✅ ASCII arrow: `->` instead of →
✅ Lowercase `x` for multiplication
✅ `+/-` for plus-minus
✅ ` degrees` for temperature
✅ Plain punctuation

### Safe Characters

✅ Emoji (proper UTF-8)  
✅ Accented characters typed directly: café, naïve, Zürich  
✅ Unicode math typed directly (√ ≈ ≠ ≤ ≥)  

### Copy/Paste Guidance

1. Paste into a plain-text buffer first (VS Code, TextEdit in plain mode)
2. Replace smart quotes and dashes
3. Verify no � replacement characters appear
4. Run `spec-kitty validate-encoding --feature <feature-id>` to check
5. Run `spec-kitty validate-encoding --feature <feature-id> --fix` to auto-repair

**Failure to follow this rule causes the dashboard to render blank pages.**

### Auto-Fix Available

If you accidentally introduce problematic characters:
```bash
# Check for encoding issues
spec-kitty validate-encoding --feature 001-my-feature

# Automatically fix all issues (creates .bak backups)
spec-kitty validate-encoding --feature 001-my-feature --fix

# Check all features at once
spec-kitty validate-encoding --all --fix
```

---

## 3. Context Management Rule

**Build the context you need, then maintain it intelligently.**

- Session start (0 tokens): You have zero context. Read plan.md, tasks.md, relevant artifacts.  
- Mid-session (you already read them): Use your judgment—don’t re-read everything unless necessary.  
- Never skip relevant information; do skip redundant re-reads to save tokens.  
- Rely on the steps in the command you are executing.

---

## 4. Work Quality Rule

**Produce secure, tested, documented work.**

- Follow the plan and constitution requirements.  
- Prefer existing patterns over invention.  
- Treat security warnings as fatal—fix or escalate.  
- Run all required tests before claiming work is complete.  
- Be transparent: state what you did, what you didn’t, and why.

---

## 5. Git Discipline Rule

**Keep commits clean and auditable.**

- Commit only meaningful units of work.  
- Write descriptive commit messages (imperative mood).  
- Do not rewrite history of shared branches.  
- Keep feature branches up to date with main via merge or rebase as appropriate.  
- Never commit secrets, tokens, or credentials.

---

---

## 6. Workflow Selection Rule (cmdai-specific)

**This project uses dual spec-driven workflows. Choose the right one for the task.**

### Spec-Kitty Workflow (You Are Here)

**Use when:**
- Feature size: < 2 weeks
- Complexity: Low to Medium
- Scope: Well-defined with clear requirements
- Need: Parallel development capability
- Want: Visual dashboard tracking

**Location**: `kitty-specs/` (git worktrees)

**Commands**: `/spec-kitty.*` slash commands

**Quick test**: If the user says "add feature X" or "fix bug Y" and you can estimate it's under 2 weeks, use spec-kitty.

**Examples**:
- ✅ "Add Redis caching with TTL"
- ✅ "Fix memory leak in MLX backend"
- ✅ "Add Prometheus metrics endpoint"
- ✅ "Implement command history"
- ✅ "Add --json output flag"

### Spec-Kit Workflow (Alternative)

**Use when:**
- Feature size: > 2 weeks
- Complexity: High
- Scope: Requires extensive research
- Need: Deep architectural investigation
- Want: Constitution-driven governance

**Location**: `specs/` (traditional directories)

**Commands**: Custom commands in `.codex/prompts/`

**Quick test**: If the feature requires major architecture changes, research phase, or affects core systems, suggest spec-kit.

**Examples**:
- ✅ "Implement complete MLX backend with C++ FFI"
- ✅ "Build multi-backend inference system"
- ✅ "Design comprehensive safety framework"
- ✅ "Research model quantization pipeline"

### Decision Logic

When the user requests a feature/bug fix:

1. **Estimate complexity and time**
   - Can this be done in < 2 weeks? → spec-kitty
   - Needs > 2 weeks or extensive research? → spec-kit

2. **Check scope clarity**
   - Clear, well-defined scope? → spec-kitty
   - Requires investigation/research? → spec-kit

3. **Assess architecture impact**
   - Incremental changes to existing systems? → spec-kitty
   - Major refactoring or new core systems? → spec-kit

4. **Consider parallel work**
   - Working on multiple features? → spec-kitty (worktrees)
   - Single large feature? → either (prefer spec-kit for complexity)

### Both Workflows Coexist

The project supports **both simultaneously**:
- `kitty-specs/001-feature/` for rapid development (spec-kitty)
- `specs/004-implement-ollama-and/` for large features (spec-kit)

You can work on a quick bug fix in `kitty-specs/` while a large feature progresses in `specs/`.

### When in Doubt

If you're unsure which workflow to use:
1. **Ask the user**: "This looks like a [small/large] feature. Should we use spec-kitty for rapid development or spec-kit for comprehensive planning?"
2. **Default to spec-kitty**: For most features/bugs, spec-kitty is the faster, more practical choice
3. **Check existing work**: Look at `specs/` to see examples of large features that used spec-kit

### Workflow Switching

You **cannot switch** mid-feature. Once started in one workflow, complete it there.

If a spec-kitty feature grows beyond 2 weeks, continue in spec-kitty but note it for future planning.

---

### Quick Reference

- 📁 **Paths**: Always specify exact locations.
- 🔤 **Encoding**: UTF-8 only. Run the validator when unsure.
- 🧠 **Context**: Read what you need; don't forget what you already learned.
- ✅ **Quality**: Follow secure, tested, documented practices.
- 📝 **Git**: Commit cleanly with clear messages.
- 🔀 **Workflow**: < 2 weeks & clear scope → spec-kitty; > 2 weeks & complex → spec-kit
