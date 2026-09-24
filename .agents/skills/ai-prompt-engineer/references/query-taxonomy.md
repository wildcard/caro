# Query Taxonomy

A systematic classification of user queries to enable intelligent context collection and appropriate tool routing.

## Overview

User queries to a terminal assistant fall into distinct categories. Each category has:
- Characteristic language patterns
- Expected context requirements
- Typical tool sets
- Optimization strategies

Understanding the category early enables targeted context gathering and reduces unnecessary clarification.

---

## Category 1: Terminal Exploration

**Definition:** Queries about navigating, understanding, or searching the filesystem.

### Subcategories

| Subcategory | Description | Examples |
|-------------|-------------|----------|
| **Navigation** | Moving around the filesystem | "go to home", "cd to project" |
| **Listing** | Viewing directory contents | "what's here", "show files", "list folder" |
| **Search** | Finding files or content | "find the config", "where is main.rs" |
| **Inspection** | Understanding file details | "how big is this", "when was it modified" |
| **Structure** | Understanding project layout | "show me the structure", "what folders exist" |

### Detection Signals

**Strong Signals (high confidence):**
- Navigation verbs: "go", "cd", "navigate", "move to"
- Listing verbs: "list", "show", "what's in", "ls"
- Search verbs: "find", "search", "where is", "locate"
- Inspection verbs: "size of", "how big", "when", "details"

**Weak Signals (supporting evidence):**
- Location references: "here", "this folder", "that directory"
- Question words: "what", "where", "how many"
- No specific tool mentioned

### Context Requirements

| Priority | Context | Purpose |
|----------|---------|---------|
| 1 | Current working directory | Base for relative operations |
| 2 | Directory existence | Validate target paths |
| 3 | Permission model | Determine accessible areas |
| 4 | Available tools | find, fd, tree, ls variants |

### Prompt Optimization

```
For exploration queries:
- Prioritize readability over efficiency
- Include common flags for human-friendly output
- Consider pagination for large outputs
- Default to non-recursive for safety
```

---

## Category 2: Runbook Execution

**Definition:** Project-specific workflows that follow established patterns or user history.

### Subcategories

| Subcategory | Description | Examples |
|-------------|-------------|----------|
| **Build** | Compile or bundle project | "build it", "run the build" |
| **Test** | Execute test suites | "run tests", "check if it passes" |
| **Run** | Start application | "run the server", "start it" |
| **Clean** | Remove build artifacts | "clean up", "fresh start" |
| **Deploy** | Ship to environment | "deploy", "push to staging" |

### Detection Signals

**Strong Signals:**
- Build verbs: "build", "compile", "make", "bundle"
- Test verbs: "test", "check", "verify", "validate"
- Run verbs: "run", "start", "launch", "serve"
- Deploy verbs: "deploy", "ship", "publish", "release"
- Pronouns suggesting known context: "the build", "our tests"

**Project Markers (for context):**
- `Cargo.toml` → Rust (`cargo build`, `cargo test`)
- `package.json` → Node (`npm run`, `yarn`)
- `Makefile` → Make-based (`make`, `make test`)
- `pom.xml` → Maven (`mvn compile`, `mvn test`)
- `build.gradle` → Gradle (`gradle build`)
- `CMakeLists.txt` → CMake (`cmake --build`)
- `pyproject.toml` → Python (`poetry run`, `pytest`)
- `go.mod` → Go (`go build`, `go test`)

### Context Requirements

| Priority | Context | Purpose |
|----------|---------|---------|
| 1 | Project markers | Identify build system |
| 2 | Available scripts | Check package.json scripts, Makefile targets |
| 3 | Recent commands | Infer typical workflow |
| 4 | Environment variables | Build configuration |

### Prompt Optimization

```
For runbook queries:
- Check project type FIRST before generating command
- Prefer project's native tooling
- Look for custom scripts before generic commands
- Include common flags for verbose output during builds
```

---

## Category 3: Language Development

**Definition:** Tasks specific to programming language workflows—editing, compiling, running, debugging.

### Subcategories

| Subcategory | Description | Examples |
|-------------|-------------|----------|
| **Edit** | Modify source code | "open main.rs", "edit the config" |
| **Compile** | Build from source | "compile this", "make the binary" |
| **Execute** | Run code/scripts | "run this script", "execute it" |
| **Debug** | Troubleshoot issues | "why is it failing", "debug mode" |
| **Format** | Code formatting | "format the code", "lint this" |
| **Dependency** | Package management | "add this library", "update deps" |

### Language Detection Matrix

| File Extension | Language | Compile | Run | Package Manager |
|----------------|----------|---------|-----|-----------------|
| `.rs` | Rust | `cargo build` | `cargo run` | `cargo` |
| `.go` | Go | `go build` | `go run` | `go mod` |
| `.py` | Python | N/A | `python` | `pip`, `poetry` |
| `.js`, `.ts` | JavaScript/TS | `tsc` (TS) | `node` | `npm`, `yarn`, `pnpm` |
| `.c`, `.h` | C | `gcc`, `clang` | direct | N/A |
| `.cpp`, `.hpp` | C++ | `g++`, `clang++` | direct | N/A |
| `.java` | Java | `javac` | `java` | `maven`, `gradle` |
| `.rb` | Ruby | N/A | `ruby` | `bundler`, `gem` |
| `.sh` | Shell | N/A | `bash`, `sh` | N/A |

### Detection Signals

**Strong Signals:**
- Language-specific verbs: "cargo", "npm", "pip", "go"
- Compile intent: "compile", "build", "make"
- File extensions in query: "run main.py", "compile lib.rs"

**Context Signals:**
- Project root markers (see Runbook category)
- Active file in editor context
- Recent file modifications

### Prompt Optimization

```
For language development:
- Detect language from file extension OR project markers
- Use language-idiomatic commands
- Include verbose/debug flags for compile errors
- Consider toolchain version (rustc version, node version)
```

---

## Category 4: DevOps Flow

**Definition:** Infrastructure, deployment, container, and operational tasks.

### Subcategories

| Subcategory | Description | Examples |
|-------------|-------------|----------|
| **Container** | Docker operations | "build the image", "run container" |
| **Orchestration** | Kubernetes/Swarm | "get pods", "scale deployment" |
| **CI/CD** | Pipeline operations | "trigger build", "check actions" |
| **Monitoring** | Logs and metrics | "show logs", "check status" |
| **Infrastructure** | Cloud/IaC | "terraform plan", "apply changes" |
| **Version Control** | Git operations | "commit this", "push to main" |

### Tool Detection Matrix

| Tool | Commands | Config Files |
|------|----------|--------------|
| Docker | `docker`, `docker-compose` | `Dockerfile`, `docker-compose.yml` |
| Kubernetes | `kubectl`, `k9s` | `*.yaml` in k8s context |
| Terraform | `terraform` | `*.tf`, `.terraform/` |
| Ansible | `ansible`, `ansible-playbook` | `playbook.yml`, `inventory` |
| GitHub Actions | `gh` | `.github/workflows/` |
| GitLab CI | `gitlab-runner` | `.gitlab-ci.yml` |
| AWS | `aws` | `~/.aws/` |
| Git | `git` | `.git/` |

### Detection Signals

**Strong Signals:**
- Tool names: "docker", "kubectl", "terraform", "git"
- DevOps vocabulary: "deploy", "container", "pod", "pipeline"
- Environment references: "staging", "production", "dev"

**Context Signals:**
- Config file presence (see matrix above)
- Environment variables (AWS_*, KUBECONFIG, etc.)
- Current directory patterns

### Prompt Optimization

```
For DevOps:
- Check for tool availability first
- Use --dry-run or plan modes when available
- Include resource names from context
- Add namespace/environment qualifiers
- Warn on production-affecting commands
```

---

## Category 5: Casual Scripting

**Definition:** Quick, one-off tasks that don't require deep project context.

### Subcategories

| Subcategory | Description | Examples |
|-------------|-------------|----------|
| **Archive** | Compress/extract | "unzip this", "create tar" |
| **Transform** | File conversion | "convert to PDF", "resize image" |
| **Rename** | Batch file operations | "rename all to lowercase" |
| **Download** | Fetch from web | "download this URL" |
| **Clipboard** | Copy/paste operations | "copy output", "paste here" |
| **Quick Edit** | Simple text changes | "replace foo with bar" |

### File-Type-Driven Tool Selection

This is the **primary inference mechanism** for casual scripting.

| Intent | File Type Signal | Platform-Aware Tool |
|--------|-----------------|---------------------|
| Extract | `.tar.gz`, `.tgz` | `tar -xzf` (all) |
| Extract | `.tar.bz2` | `tar -xjf` (all) |
| Extract | `.tar.xz` | `tar -xJf` (all) |
| Extract | `.zip` | `unzip` (macOS/Linux), `Expand-Archive` (Windows) |
| Extract | `.7z` | `7z x` (requires install) |
| Extract | `.rar` | `unrar x` (requires install) |
| Compress | → `.tar.gz` | `tar -czf` (all) |
| Compress | → `.zip` | `zip -r` (macOS/Linux), `Compress-Archive` (Windows) |
| Convert | `.heic` → `.jpg` | `sips` (macOS), `convert` (Linux/ImageMagick) |
| Resize | images | `sips` (macOS), `convert` (Linux) |
| Download | URL | `curl -O` or `wget` (availability) |

### Detection Signals

**Strong Signals:**
- Simple action verbs: "unzip", "extract", "compress", "rename"
- Single file/pattern reference
- No project context needed
- Standalone task

**File Type Signals:**
- Extension explicitly mentioned
- File path in query
- Glob pattern in query

### Prompt Optimization

```
For casual scripting:
- Infer tool from file extension FIRST
- Match tool to platform (unzip on Mac, tar -xzf for .tar.gz)
- Keep commands simple and single-purpose
- Don't over-engineer for one-off tasks
```

---

## Category 6: CLI Tool Interaction

**Definition:** Using specific command-line tools with their native interfaces.

### Subcategories

| Subcategory | Description | Examples |
|-------------|-------------|----------|
| **Help** | Understanding a tool | "how do I use grep", "git help" |
| **Status** | Check tool state | "git status", "docker ps" |
| **Execute** | Run tool commands | "git commit", "npm install" |
| **Configure** | Set tool options | "git config", "aws configure" |

### Detection Signals

**Strong Signals:**
- Tool name mentioned explicitly: "git", "docker", "kubectl"
- Tool-specific subcommands in query
- Tool flags in query

### Prompt Optimization

```
For CLI tools:
- Preserve user's intended tool
- Don't substitute with alternatives
- Add helpful flags user may have forgotten
- Include common gotchas for the tool
```

---

## Cross-Category Inference Rules

### Priority Order for Ambiguous Queries

1. **Explicit tool mention** → CLI Tool Interaction
2. **File extension present** → Casual Scripting (file-type-driven)
3. **Project markers in cwd** → Runbook Execution
4. **DevOps vocabulary** → DevOps Flow
5. **Language-specific terms** → Language Development
6. **Default** → Terminal Exploration

### Confidence Scoring

| Evidence Type | Score |
|---------------|-------|
| Explicit tool name | +3 |
| Action verb match | +2 |
| File extension match | +2 |
| Project marker match | +2 |
| Platform signal | +1 |
| Session history pattern | +1 |
| Weak vocabulary match | +0.5 |

**Thresholds:**
- Score ≥ 3: High confidence → Generate command directly
- Score 2-3: Medium confidence → Generate with brief explanation
- Score < 2: Low confidence → Ask clarifying question

---

## Appendix: Signal Word Lists

### Action Verbs by Category

```yaml
exploration:
  - list, show, display, view
  - find, search, locate, where
  - navigate, go, cd, move
  - inspect, check, examine

runbook:
  - build, compile, make
  - test, check, verify, validate
  - run, start, launch, serve
  - deploy, ship, publish, release
  - clean, reset, fresh

language:
  - compile, build, make
  - run, execute, interpret
  - debug, trace, profile
  - format, lint, fix
  - install, add, update (dependencies)

devops:
  - deploy, rollout, scale
  - push, pull, tag
  - apply, plan, destroy
  - logs, status, describe

casual:
  - unzip, extract, decompress
  - zip, compress, archive
  - rename, move, copy
  - download, fetch, get
  - convert, transform, resize

cli:
  - (tool name) + subcommand
  - help, man, info
  - config, configure, set
```

### Platform Keywords

```yaml
linux:
  - apt, yum, dnf, pacman
  - systemctl, journalctl
  - /etc, /var, /usr

macos:
  - brew, port
  - launchctl
  - /Applications, ~/Library

windows:
  - choco, winget, scoop
  - powershell, cmd
  - C:\, Program Files
```
