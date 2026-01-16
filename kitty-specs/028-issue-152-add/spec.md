# Issue #152: Add Local Directory Context Awareness

## Summary

Add context awareness for the current directory to improve command generation quality by detecting project type, available tools, and relevant files.

## Problem

Currently, Caro generates commands without awareness of the project context:
- User in a Node.js project might want `npm` commands
- User in a Rust project might want `cargo` commands
- User in a Python project might want `pip`/`uv`/`poetry` commands

Without directory context, the LLM lacks information to suggest project-specific commands.

## Solution

Create a `DirectoryContext` module that:

1. **Scans for project markers** - Detects files that indicate project type:
   - `package.json` → Node.js project
   - `Cargo.toml` → Rust project
   - `pyproject.toml`, `requirements.txt` → Python project
   - `go.mod` → Go project
   - `Makefile` → Has make targets
   - `.git` → Git repository
   - `docker-compose.yml` → Docker project
   - `Dockerfile` → Has Docker support

2. **Extracts relevant info** from project files:
   - NPM scripts from `package.json`
   - Make targets from `Makefile`
   - Cargo aliases/scripts

3. **Integrates with prompt builder** via `with_context()` method

## Implementation

### New Module: `src/context/directory.rs`

```rust
pub struct DirectoryContext {
    pub project_types: Vec<ProjectType>,
    pub has_git: bool,
    pub has_makefile: bool,
    pub has_docker: bool,
    pub npm_scripts: Vec<String>,
    pub make_targets: Vec<String>,
}

pub enum ProjectType {
    NodeJs,
    Rust,
    Python,
    Go,
    Generic,
}

impl DirectoryContext {
    pub fn scan(path: &Path) -> Self;
    pub fn to_context_string(&self) -> String;
}
```

### Integration Point

In `SmolLMPromptBuilder.with_context()`, include directory context:

```rust
let dir_context = DirectoryContext::scan(&current_dir);
builder.with_context(dir_context.to_context_string())
```

## Success Criteria

1. `DirectoryContext::scan()` correctly detects project types
2. Context string includes relevant project info
3. Unit tests pass for all project type detection
4. Integration with prompt builder works

## Files to Modify

- `src/context/mod.rs` - Add directory module export
- `src/context/directory.rs` - New file with DirectoryContext
- `src/lib.rs` - Export DirectoryContext

## Test Cases

1. Detect Node.js project from `package.json`
2. Detect Rust project from `Cargo.toml`
3. Detect Python project from `pyproject.toml`
4. Detect Git repository from `.git`
5. Detect Make targets from `Makefile`
6. Multiple project types in same directory
