# Open Source Project Conventions

This reference documents the file organization patterns used by well-maintained open source projects across various technology stacks.

---

## Universal Root Files

These files are universally expected at the project root:

| File | Purpose | Notes |
|------|---------|-------|
| `README.md` | Project entry point | Required |
| `LICENSE` | Legal terms | Required for OSS |
| `CHANGELOG.md` | Version history | Highly recommended |
| `CONTRIBUTING.md` | Contribution guide | Recommended |
| `CODE_OF_CONDUCT.md` | Community standards | GitHub special file |
| `SECURITY.md` | Security policy | GitHub special file |
| `.gitignore` | Git exclusions | Required |

---

## GitHub Special Files

GitHub recognizes these files in root or `.github/`:

```
.github/
├── ISSUE_TEMPLATE/
│   ├── bug_report.md
│   ├── feature_request.md
│   └── config.yml
├── PULL_REQUEST_TEMPLATE.md
├── CODEOWNERS
├── FUNDING.yml
├── SECURITY.md
├── SUPPORT.md
└── workflows/
    └── *.yml
```

**Rule**: If you have 3+ GitHub-specific files, move them to `.github/`.

---

## Standard Directories

### Source Code

| Language | Convention | Alternative |
|----------|------------|-------------|
| Rust | `src/` | - |
| JavaScript | `src/` | `lib/` |
| TypeScript | `src/` | - |
| Python | `src/package_name/` | `package_name/` |
| Go | Root or `cmd/`, `pkg/` | `internal/` |

### Tests

| Language | Convention | Notes |
|----------|------------|-------|
| Rust | `tests/` (integration) | Unit tests in `src/` |
| JavaScript | `__tests__/` or `tests/` | Often colocated |
| TypeScript | `__tests__/` or `tests/` | Often colocated |
| Python | `tests/` | Mirror src structure |
| Go | `*_test.go` (colocated) | Standard practice |

### Documentation

```
docs/
├── architecture/          # ADRs, design docs
│   └── adr-001-*.md
├── api/                   # API documentation
├── guides/                # User guides, tutorials
└── development/           # Developer docs
```

### Scripts

```
scripts/
├── build/                 # Build scripts
├── dev/                   # Development helpers
├── release/               # Release automation
└── ci/                    # CI helpers
```

---

## Language-Specific Patterns

### Rust (Cargo)

```
project/
├── .cargo/                # Cargo config
│   └── config.toml
├── benches/               # Benchmarks
├── examples/              # Example binaries
├── src/
│   ├── bin/               # Additional binaries
│   ├── lib.rs             # Library root
│   └── main.rs            # Default binary
├── tests/                 # Integration tests
├── Cargo.toml             # Manifest
├── Cargo.lock             # Dependencies (commit for binaries)
└── README.md
```

**Rust Conventions**:
- `lib.rs` for library crates
- `main.rs` for binary crates
- Tests in `tests/` for integration, inline for unit
- `examples/` for usage examples
- `benches/` for criterion benchmarks

### JavaScript/Node

```
project/
├── .husky/                # Git hooks
├── dist/                  # Build output (gitignored)
├── node_modules/          # Dependencies (gitignored)
├── public/                # Static assets
├── scripts/               # Build/dev scripts
├── src/
│   ├── components/        # UI components
│   ├── lib/               # Utilities
│   └── index.js           # Entry point
├── tests/ or __tests__/
├── .eslintrc.js           # Linter config
├── .prettierrc            # Formatter config
├── package.json
├── package-lock.json      # or yarn.lock/pnpm-lock.yaml
├── tsconfig.json          # TypeScript config
└── README.md
```

**JS Conventions**:
- Framework configs at root (Next.js, Vite, etc.)
- Build output in `dist/` or `build/`
- Use `scripts/` for non-npm scripts

### Python

```
project/
├── .venv/                 # Virtual env (gitignored)
├── docs/                  # Sphinx/mkdocs
├── scripts/               # Automation
├── src/
│   └── package_name/
│       ├── __init__.py
│       └── *.py
├── tests/
│   ├── conftest.py
│   └── test_*.py
├── .python-version        # pyenv
├── pyproject.toml         # Modern config
├── requirements.txt       # or use pyproject.toml
└── README.md
```

**Python Conventions**:
- `src/` layout preferred (PEP 517)
- `pyproject.toml` for all config (PEP 518)
- `tests/` mirror `src/` structure

### Go

```
project/
├── cmd/
│   ├── app1/
│   │   └── main.go
│   └── app2/
│       └── main.go
├── internal/              # Private packages
│   └── *.go
├── pkg/                   # Public packages
│   └── *.go
├── api/                   # API definitions
├── configs/               # Config templates
├── scripts/
├── go.mod
├── go.sum
└── README.md
```

**Go Conventions**:
- `cmd/` for entry points
- `internal/` for private code
- `pkg/` for public libraries
- Tests colocated with source

---

## Mono-Repo Patterns

### JavaScript (Workspaces)

```
monorepo/
├── .github/
├── packages/
│   ├── core/
│   │   ├── src/
│   │   └── package.json
│   ├── cli/
│   │   ├── src/
│   │   └── package.json
│   └── web/
│       ├── src/
│       └── package.json
├── scripts/
├── lerna.json             # or turborepo.json
├── package.json           # Root workspace
└── README.md
```

### Rust (Workspace)

```
monorepo/
├── crates/
│   ├── core/
│   │   ├── src/
│   │   └── Cargo.toml
│   └── cli/
│       ├── src/
│       └── Cargo.toml
├── Cargo.toml             # Workspace root
└── README.md
```

---

## Configuration File Placement

### Keep at Root
- Main package manifest (package.json, Cargo.toml, etc.)
- Editor configs (.editorconfig, .prettierrc)
- Linter configs (.eslintrc, rustfmt.toml)
- Git configs (.gitignore, .gitattributes)

### Consider Moving to `/config`
- Multiple environment configs
- Deployment configurations
- Tool-specific complex configs

### Move to `.github/`
- GitHub Actions workflows
- Issue/PR templates
- CODEOWNERS

---

## Anti-Patterns to Avoid

### Root Clutter
- Temporary files at root
- Multiple README variants
- Orphaned scripts
- Backup files (*.bak, *.old)
- Versioned copies (file_v2.txt)

### Wrong Locations
- Build output in source control
- node_modules committed
- .env files committed
- IDE settings committed (.idea/, .vscode/ with user prefs)

### Missing Structure
- All code in root (no src/)
- Tests mixed with source
- No docs/ for documentation
- Scripts scattered everywhere

---

## Decision Framework

When unsure where a file belongs:

1. **Is it a universal OSS file?** → Root
2. **Is it GitHub-specific?** → `.github/`
3. **Is it source code?** → `src/`
4. **Is it documentation?** → `docs/`
5. **Is it a script?** → `scripts/`
6. **Is it a test?** → `tests/` or colocate
7. **Is it build output?** → Gitignore it
8. **Is it unique to this project?** → Follow existing patterns
9. **Still unsure?** → Ask the user

---

*These conventions are derived from observing thousands of successful open source projects and official language/framework recommendations.*
