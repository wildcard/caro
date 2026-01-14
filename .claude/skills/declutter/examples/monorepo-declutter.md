# Example: Decluttering a Mono-Repo

This example demonstrates the level-by-level approach to decluttering a mono-repo with multiple sub-projects.

---

## Initial State

```
monorepo/
├── packages/
│   ├── core/
│   │   ├── src/
│   │   ├── NOTES.md              # Should be in docs
│   │   ├── test-data.json        # Should be in tests
│   │   └── package.json
│   └── cli/
│       ├── src/
│       ├── build.sh              # Should be in scripts
│       ├── old-cli.js            # Delete
│       └── package.json
├── scripts/
│   ├── release.sh
│   └── old-deploy.sh             # Delete
├── docs/                         # Mostly empty
│   └── placeholder.md
├── README.md
├── ARCHITECTURE.md               # Move to docs
├── docker-compose.yml
├── docker-compose.dev.yml        # Consolidate or move
├── docker-compose.prod.yml       # Consolidate or move
├── .env.example
├── .env.local                    # Should be gitignored!
├── backup.sql                    # Should be gitignored!
├── temp/                         # Should be gitignored!
│   └── cache.json
├── setup-notes.txt               # Move to docs
├── meeting-notes-2025-01.md      # Move to docs/notes or delete
├── package.json                  # Root workspace
├── lerna.json
└── .github/
    └── workflows/
        └── ci.yml
```

**Root items**: 16+
**Status**: Critical - needs major reorganization

---

## Level 1: Root Cleanup

### Assessment

```markdown
## Root Analysis

**Essential (keep):**
- README.md
- package.json
- lerna.json
- packages/
- scripts/
- docs/
- .github/

**Docker configs (consolidate):**
- docker-compose.yml
- docker-compose.dev.yml
- docker-compose.prod.yml
→ Move to /deploy/ or keep 1 at root

**Move to docs/:**
- ARCHITECTURE.md
- setup-notes.txt
- meeting-notes-2025-01.md (or delete)

**Security issues (gitignore + delete):**
- .env.local (committed secrets!)
- backup.sql (database dump!)

**Gitignore:**
- temp/

**Delete:**
- scripts/old-deploy.sh
```

### Root Moves

```bash
# Create deploy directory for Docker configs
mkdir -p deploy

# Move architecture docs
git mv ARCHITECTURE.md docs/
git mv setup-notes.txt docs/setup-guide.md

# Move Docker configs
git mv docker-compose.dev.yml deploy/
git mv docker-compose.prod.yml deploy/
# Keep main docker-compose.yml at root as entry point

# Security cleanup (URGENT!)
echo ".env.local" >> .gitignore
echo "backup.sql" >> .gitignore
echo "temp/" >> .gitignore
rm .env.local
rm backup.sql
rm -rf temp/

# Delete obsolete
rm scripts/old-deploy.sh

# Delete or archive meeting notes
rm meeting-notes-2025-01.md  # User confirmed: outdated
```

---

## Level 2: Sub-Project Cleanup (packages/core)

### Assessment

```markdown
## packages/core Analysis

**Essential:**
- src/
- package.json

**Move:**
- NOTES.md → Move to /docs/packages/core/ or inline in README
- test-data.json → Move to tests/ or fixtures/
```

### Moves

```bash
# Create test fixtures location
mkdir -p packages/core/tests/fixtures

# Move files
git mv packages/core/NOTES.md docs/packages-core-notes.md
git mv packages/core/test-data.json packages/core/tests/fixtures/
```

---

## Level 2: Sub-Project Cleanup (packages/cli)

### Assessment

```markdown
## packages/cli Analysis

**Essential:**
- src/
- package.json

**Move:**
- build.sh → Move to /scripts/cli/ or package.json scripts

**Delete:**
- old-cli.js (obsolete)
```

### Moves

```bash
# Create CLI scripts location
mkdir -p scripts/cli

# Move build script
git mv packages/cli/build.sh scripts/cli/build.sh

# Delete obsolete
rm packages/cli/old-cli.js
```

---

## Level 3: Scripts Reorganization

### Assessment

```markdown
## scripts/ Analysis

Current:
- release.sh
- cli/build.sh (just added)

Proposed structure:
scripts/
├── cli/
│   └── build.sh
├── release/
│   └── release.sh
└── dev/
    └── (future dev scripts)
```

### Moves

```bash
mkdir -p scripts/release
mkdir -p scripts/dev
git mv scripts/release.sh scripts/release/
```

---

## Proposed Final Structure

```
monorepo/
├── .github/
│   └── workflows/
│       └── ci.yml
├── deploy/
│   ├── docker-compose.dev.yml
│   └── docker-compose.prod.yml
├── docs/
│   ├── ARCHITECTURE.md
│   ├── setup-guide.md
│   └── packages-core-notes.md
├── packages/
│   ├── core/
│   │   ├── src/
│   │   ├── tests/
│   │   │   └── fixtures/
│   │   │       └── test-data.json
│   │   └── package.json
│   └── cli/
│       ├── src/
│       └── package.json
├── scripts/
│   ├── cli/
│   │   └── build.sh
│   ├── dev/
│   └── release/
│       └── release.sh
├── .env.example
├── .gitignore
├── docker-compose.yml
├── lerna.json
├── package.json
└── README.md
```

**Root items**: 10
**Status**: Clean and organized

---

## Reference Updates Needed

### CI Updates

Check if `.github/workflows/ci.yml` references any moved files:

```yaml
# Before
- run: ./scripts/release.sh

# After
- run: ./scripts/release/release.sh
```

**Delegation**: If changes needed, delegate to `devops` skill.

### Package.json Updates

Check if root `package.json` has script references:

```json
// Before
"scripts": {
  "build:cli": "./packages/cli/build.sh"
}

// After
"scripts": {
  "build:cli": "./scripts/cli/build.sh"
}
```

---

## Commit Strategy

For mono-repos, consider multiple commits:

```bash
# Commit 1: Security fixes (urgent)
git add .gitignore
git commit -m "security: Remove and gitignore sensitive files

- Add .env.local to .gitignore
- Add backup.sql to .gitignore
- Add temp/ to .gitignore

IMPORTANT: These files were committed and should be rotated!"

# Commit 2: Root reorganization
git add -A
git commit -m "refactor: Declutter monorepo root

Moved:
- ARCHITECTURE.md → docs/
- setup-notes.txt → docs/setup-guide.md
- docker-compose.*.yml → deploy/

Deleted:
- scripts/old-deploy.sh (obsolete)
- meeting-notes-2025-01.md (outdated)

Root reduced from 16 to 10 items."

# Commit 3: Sub-project cleanup
git add -A
git commit -m "refactor: Organize sub-project structures

packages/core:
- Moved test-data.json to tests/fixtures/
- Moved NOTES.md to docs/

packages/cli:
- Moved build.sh to scripts/cli/
- Deleted old-cli.js (obsolete)

scripts:
- Reorganized into cli/, release/, dev/ structure"
```

---

## Verification Checklist

```
☐ All packages build successfully
☐ CI pipeline passes
☐ No broken references in scripts
☐ Sensitive files removed from git history (if needed)
☐ Documentation links updated
☐ README references updated
```

---

*Mono-repo declutter requires careful level-by-level approach to maintain project boundaries while achieving overall organization.*
