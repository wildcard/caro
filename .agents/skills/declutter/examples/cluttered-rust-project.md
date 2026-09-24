# Example: Decluttering a Rust CLI Project

This example walks through decluttering a typical Rust CLI project that has accumulated clutter over time.

---

## Initial State

### Current Tree

```
myproject/
├── src/
│   ├── main.rs
│   └── lib.rs
├── tests/
│   └── integration.rs
├── Cargo.toml
├── Cargo.lock
├── README.md
├── LICENSE
├── ARCHITECTURE.md           # Should be in docs/
├── DESIGN_DECISIONS.md       # Should be in docs/
├── TODO.md                   # Should be in docs/ or .github/
├── setup.sh                  # Should be in scripts/
├── build-release.sh          # Should be in scripts/
├── run-tests.sh              # Should be in scripts/
├── dev-notes.txt             # Should be in docs/
├── ideas.md                  # Should be in docs/ or sandbox/
├── benchmark_results.csv     # Should be gitignored
├── output.log                # Should be gitignored
├── test.rs.bak               # Delete
├── old_main.rs               # Delete
├── config.example.toml       # Keep or move to examples/
├── .github/
│   └── workflows/
│       └── ci.yml
└── .gitignore
```

**Root item count**: 18 (excluding hidden)
**Status**: Needs attention

---

## Assessment Output

```markdown
## Declutter Assessment - myproject

### Tech Stack
- Language: Rust
- Build: Cargo
- CI: GitHub Actions

### Root Inventory (18 items)

**Essential (keep):**
- README.md
- LICENSE
- Cargo.toml
- Cargo.lock
- src/
- tests/
- .github/
- .gitignore

**Documentation (move to docs/):**
- ARCHITECTURE.md
- DESIGN_DECISIONS.md
- TODO.md
- dev-notes.txt
- ideas.md

**Scripts (move to scripts/):**
- setup.sh
- build-release.sh
- run-tests.sh

**Artifacts (gitignore):**
- benchmark_results.csv
- output.log

**Obsolete (delete):**
- test.rs.bak
- old_main.rs

**Decide:**
- config.example.toml (→ examples/ or root?)
```

---

## Proposed Structure

```
myproject/
├── .github/
│   └── workflows/
│       └── ci.yml
├── docs/
│   ├── ARCHITECTURE.md
│   ├── DESIGN_DECISIONS.md
│   └── notes/
│       ├── todo.md
│       ├── dev-notes.md
│       └── ideas.md
├── examples/
│   └── config.example.toml
├── scripts/
│   ├── setup.sh
│   ├── build-release.sh
│   └── run-tests.sh
├── src/
│   ├── main.rs
│   └── lib.rs
├── tests/
│   └── integration.rs
├── .gitignore
├── Cargo.toml
├── Cargo.lock
├── LICENSE
└── README.md
```

**Root item count**: 9 (target achieved!)

---

## Execution Plan

### Phase 1: Create Structure

```bash
mkdir -p docs/notes
mkdir -p examples
mkdir -p scripts
```

### Phase 2: Move Documentation

```bash
git mv ARCHITECTURE.md docs/
git mv DESIGN_DECISIONS.md docs/
git mv TODO.md docs/notes/todo.md
git mv dev-notes.txt docs/notes/dev-notes.md
git mv ideas.md docs/notes/
```

### Phase 3: Move Scripts

```bash
git mv setup.sh scripts/
git mv build-release.sh scripts/
git mv run-tests.sh scripts/
```

### Phase 4: Move Examples

```bash
git mv config.example.toml examples/
```

### Phase 5: Clean Up

```bash
# Add to .gitignore
echo "benchmark_results.csv" >> .gitignore
echo "output.log" >> .gitignore

# Remove artifacts
rm benchmark_results.csv
rm output.log

# Remove obsolete files
rm test.rs.bak
rm old_main.rs
```

### Phase 6: Update References

Check for broken references in scripts:

```bash
# Old reference in ci.yml might be:
#   ./build-release.sh
# New reference should be:
#   ./scripts/build-release.sh
```

**Delegation needed**: If CI references moved scripts, delegate to `devops` skill or manually update `.github/workflows/ci.yml`.

---

## Verification

```bash
# Check build still works
cargo build
cargo test

# Verify tree
tree -L 2 -a --dirsfirst -I '.git|target'

# Check CI (if applicable)
# Push and verify GitHub Actions
```

---

## Commit

```bash
git add -A
git commit -m "$(cat <<'EOF'
refactor: Declutter project structure

Created:
- docs/ for documentation
- docs/notes/ for working notes
- scripts/ for automation
- examples/ for config examples

Moved:
- ARCHITECTURE.md → docs/
- DESIGN_DECISIONS.md → docs/
- TODO.md → docs/notes/todo.md
- dev-notes.txt → docs/notes/dev-notes.md
- ideas.md → docs/notes/
- setup.sh → scripts/
- build-release.sh → scripts/
- run-tests.sh → scripts/
- config.example.toml → examples/

Deleted:
- test.rs.bak (obsolete backup)
- old_main.rs (obsolete backup)

Added to .gitignore:
- benchmark_results.csv
- output.log

Root reduced from 18 to 9 items.
EOF
)"
```

---

## Result

### Before

```
18 root items, cluttered, no clear organization
```

### After

```
myproject/
├── .github/           # CI/CD
├── docs/              # All documentation
├── examples/          # Usage examples
├── scripts/           # Automation
├── src/               # Source code
├── tests/             # Tests
├── .gitignore
├── Cargo.toml
├── Cargo.lock
├── LICENSE
└── README.md

9 root items, clean, predictable
```

---

*This project now presents a welcoming, professional structure that follows Rust and OSS conventions.*
