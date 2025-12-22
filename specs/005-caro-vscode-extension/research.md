# Caro VS Code Extension - Product Research & Technical Plan

**Feature Branch**: `005-caro-vscode-extension`
**Created**: 2025-12-22
**Status**: Research & Planning
**Author**: Caro Development Team

---

## Executive Summary

This document presents a comprehensive product research and technical plan for implementing Caro's VS Code extension. The extension will serve as a new interface layer that connects to Caro's Rust core, providing developers with an accessible, IDE-integrated experience for natural language to shell command generation.

### Strategic Vision

Caro positions itself among elite AI coding assistants (Claude Code, GitHub Copilot, Cline) with a unique value proposition: **infrastructure-first, POSIX-native command intelligence**. While competitors focus on general code generation, Caro specializes in helping developers master their terminal, automate workflows, and write safer shell scripts.

### Key Differentiators

1. **Local-First Architecture**: Lean models run on-device, ensuring privacy and offline capability
2. **Infrastructure Focus**: Deep expertise in DevOps, shell scripting, and UNIX philosophy
3. **Safety-First Design**: 52+ compiled safety patterns with risk assessment
4. **Proactive Intelligence**: Background analysis of Dockerfiles, scripts, and config files
5. **Non-Duplicative**: VS Code is a thin UI layer; logic lives in Rust core

---

## Part 1: Competitive Landscape Analysis

### 1.1 Claude Code (Anthropic)

**Architecture Overview:**
- WebView-based chat panel accessible via spark icon in activity bar
- CLI and extension share conversation history (`claude --resume`)
- Deep integration with VS Code editor through inline diffs
- Terminal accessible for direct CLI usage

**Key Features:**
- Graphical chat panel with code block highlighting
- Inline diff visualization for code changes
- Real-time streaming responses
- File context awareness through editor selection
- Dual-mode: graphical panel or CLI-style interface

**VS Code Integration Points:**
- Activity Bar icon (spark icon)
- Editor toolbar button (top-right)
- Keyboard shortcut: `Cmd+ESC` (Mac) / `Ctrl+ESC` (Windows/Linux)
- Secondary sidebar positioning
- Shared history with CLI

**Strengths:**
- Seamless transition between IDE and terminal workflows
- Rich conversational interface
- Strong code understanding

**Limitations:**
- No proactive suggestions
- General-purpose (not infrastructure-specialized)
- Requires network connection

**Source:** [Claude Code VS Code Docs](https://code.claude.com/docs/en/vs-code)

---

### 1.2 GitHub Copilot

**Architecture Overview:**
- Two historically separate extensions now unified
- Ghost text inline suggestions (autocomplete)
- Chat panel in secondary sidebar
- Inline chat for context-specific questions

**Key Features:**
- **Ghost Text**: Real-time code suggestions as you type
- **Next Edit Suggestions (NES)**: Predicts next logical edit location
- **Chat Panel**: Conversational interface with context variables (`#file`, `#editor`)
- **Inline Chat**: `Ctrl+I` for quick questions about current code
- **Terminal Inline Chat**: Command-line assistance in integrated terminal
- **Custom Agents**: `.agent.md` files for specialized behavior (since VS Code 1.106)

**VS Code Integration Points:**
- Activity Bar chat icon
- Ghost text in editor
- Inline chat popup
- Terminal integration
- Context variables (`#file`, `#editor`, `@workspace`)
- Drag-and-drop file context

**Strengths:**
- Deep editor integration
- Predictive editing (NES)
- Terminal command assistance
- Participant API for extensions

**Limitations:**
- Subscription-based (limited free tier)
- Cloud-dependent
- General-purpose, not shell-specialized

**Source:** [VS Code Copilot Docs](https://code.visualstudio.com/docs/copilot/overview)

---

### 1.3 Cline (Formerly Claude Dev)

**Architecture Overview:**
- Three-tier architecture: Extension Host (backend), WebView UI (frontend), Core Logic
- gRPC-based communication layer
- Human-in-the-loop approval for all operations
- MCP (Model Context Protocol) integration

**Key Features:**
- **File Operations**: Create, edit, delete with real-time linter feedback
- **Terminal Integration**: Execute commands and monitor output
- **Browser Automation**: Headless browser for web debugging
- **MCP Integration**: Extensible capabilities through external servers
- **Checkpoint System**: ShadowGit for recovery
- **Multi-Mode Deployment**: VS Code extension + standalone mode

**Deep Integration:**
- Full access to VS Code APIs
- Terminal control and output capture
- File system monitoring
- Diagnostic integration

**Strengths:**
- Autonomous agent capabilities
- Strong approval workflows
- Extensible via MCP

**Limitations:**
- Resource-intensive
- Complex architecture
- Not specialized for shell/infrastructure

**Source:** [Cline Architecture Deep Dive](https://zjy365.dev/blog/cline-source-code-analysis)

---

### 1.4 Cline Deep Dive: Agentic Architecture & Multi-Backend Support

**Task Execution Loop:**

Cline implements a sophisticated recursive conversation loop that is the core of its agentic capabilities:

```
User Task Input
    ↓
Controller.initTask() → Creates Task Instance
    ↓
┌─────────────────────────────────────────┐
│         Recursive Conversation Loop      │
│  ┌─────────────────────────────────┐    │
│  │ 1. Send request to AI provider  │    │
│  │ 2. Process response             │    │
│  │ 3. Execute tools (with approval)│    │
│  │ 4. Continue until completion    │    │
│  └─────────────────────────────────┘    │
└─────────────────────────────────────────┘
    ↓
Task Completion / User Abort
```

**Tool Approval Architecture:**

The ToolExecutor class coordinates all tool operations through a registration-based architecture:

| Approval Level | Description | Examples |
|----------------|-------------|----------|
| **Auto-Approve (Safe)** | Read-only operations | `ls`, `cat`, file reads |
| **Auto-Approve (Config)** | User-configured safe ops | Build commands, tests |
| **Require Approval** | Potentially destructive | File writes, `rm`, network |
| **YOLO Mode** | Approve all (dangerous) | Full autonomy |

**Built-in Tools:**
- `read_file`: Read file contents with line numbers
- `write_to_file`: Create/overwrite entire files
- `search_files`: Regex search across workspace
- `list_files`: Directory listing with recursion
- `execute_command`: Terminal command execution
- `browser_action`: Headless browser automation
- `use_mcp_tool`: Execute MCP server tools

**Multi-Backend Support:**

Cline supports extensive backend flexibility:

| Provider Type | Examples | Configuration |
|---------------|----------|---------------|
| **Cloud APIs** | Anthropic, OpenAI, Google Gemini | API keys |
| **Cloud Gateways** | OpenRouter, AWS Bedrock, Azure | Endpoint + auth |
| **Local Inference** | Ollama, LM Studio | `http://127.0.0.1:11434/v1` |
| **Custom** | Any OpenAI-compatible API | URL + optional key |

**Ollama Integration Details:**

```typescript
// Cline connects via OpenAI-compatible endpoint
const ollamaConfig = {
  baseUrl: "http://127.0.0.1:11434/v1",
  model: "deepseek-r1:32b",  // or any Ollama model
  apiKey: "ollama"  // placeholder, not validated
};
```

**Challenges with Local Models:**
- Complex prompts strain smaller models
- Quantization reduces tool-use capability
- Recommended: Claude 3.5 Sonnet for best results
- Local models work but may struggle with multi-step reasoning

**MCP (Model Context Protocol) Integration:**

Cline is a first-class MCP client:

```
┌─────────────────────────────────────────────────────────────┐
│                         Cline                                │
│  ┌───────────────────────────────────────────────────────┐  │
│  │                    MCP Client                          │  │
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐   │  │
│  │  │ Filesystem  │  │   GitHub    │  │   Custom    │   │  │
│  │  │   Server    │  │   Server    │  │   Server    │   │  │
│  │  │  (stdio)    │  │  (stdio)    │  │  (HTTP)     │   │  │
│  │  └─────────────┘  └─────────────┘  └─────────────┘   │  │
│  └───────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
```

**MCP Features:**
- Tools: Executable functions (file ops, API calls, DB queries)
- Resources: Data sources for context
- Prompts: Reusable interaction templates
- Self-extending: Cline can create MCP servers via conversation

**Terminal Integration (VS Code 1.93+):**

```typescript
// Cline leverages shell integration for:
// - Direct command execution
// - Real-time output capture
// - Long-running process monitoring
// - "Proceed While Running" for dev servers
```

**ShadowGit Checkpoint System:**

Cline maintains recovery points using a shadow git repository, allowing users to revert to any previous state during autonomous operations.

**Source:** [Cline DeepWiki](https://deepwiki.com/cline/cline), [Cline GitHub](https://github.com/cline/cline)

---

### 1.5 Tabby (Self-Hosted AI Coding Assistant)

**Architecture Overview:**
- Self-hosted, open-source alternative to GitHub Copilot
- No DBMS or cloud service required
- OpenAPI interface for infrastructure integration
- Consumer-grade GPU support
- RAG-based code completion with repository context

**Key Features:**
- **Code Completion**: Real-time multi-line suggestions
- **Chat**: Side-panel conversation (since v1.7, June 2024)
- **Answer Engine**: Central knowledge engine for teams (v0.13, July 2024)
- **Context Providers**: Repository, docs, and custom integrations
- **Agent** (Preview): Agentic coding capabilities

**RAG Pipeline for Code Completion:**

```
User Code Context (from IDE)
    ↓
┌─────────────────────────────────────────────────┐
│              RAG Code Completion                 │
│  ┌───────────────────────────────────────────┐  │
│  │ 1. Embed query using embedding model      │  │
│  │ 2. Semantic search (Tantivy vector index) │  │
│  │ 3. BM25 keyword search                    │  │
│  │ 4. Merge with Reciprocal Rank Fusion      │  │
│  │ 5. Insert relevant context into prompt    │  │
│  └───────────────────────────────────────────┘  │
└─────────────────────────────────────────────────┘
    ↓
LLM generates completion with project context
```

**Repository Context (Tree-sitter Based):**

Tabby uses tree-sitter to build its code index:

```toml
# ~/.tabby/config.toml
[[repositories]]
name = "my-project"
git_url = "https://github.com/user/project.git"

# Supports: Git, GitHub, GitLab (self-hosted or cloud)
```

**Context Sources:**
| Source | Description |
|--------|-------------|
| **Repositories** | Codebase, PRs, issues, commits |
| **Docs** | Developer documentation |
| **Local LSP** | Declarations from language server |
| **Recent Code** | Recently modified files |

**VS Code Extension Features:**
- Multi-line inline completion
- Auto-generated commit messages (v1.6, May 2024)
- Chat in side-panel (v1.7, June 2024)
- Multiple completion choices

**2024-2025 Release Timeline:**
| Version | Date | Key Features |
|---------|------|--------------|
| v0.9 | Mar 2024 | Full admin UI |
| v0.11 | May 2024 | Enterprise: GitHub/GitLab integration |
| v0.12 | Jun 2024 | GitLab SSO, HTTP API |
| v0.13 | Jul 2024 | Answer Engine |
| v0.20 | Nov 2024 | Multi-model chat switching |
| v0.21 | Dec 2024 | Llamafile deployment |
| v0.23 | Jan 2025 | Enhanced code browser |

**Agent Mode (Private Preview):**

Tabby is developing agent capabilities:
- Natural language to working code
- Component building (React/Vue/Svelte)
- Mockup to deployable frontend
- Complex prompt handling

**Strengths:**
- Fully self-hosted (data stays on-prem)
- RAG improves relevance with project context
- Open source with active development
- Team-friendly with predictable costs

**Limitations:**
- Requires setup/ops
- Completion quality varies by model
- Fewer out-of-box agentic features than paid tools
- Agent mode still in preview

**Source:** [Tabby GitHub](https://github.com/TabbyML/tabby), [Tabby Docs](https://tabby.tabbyml.com/docs/)

---

### 1.6 llama.cpp & llama.vscode (Local Inference)

**llama.cpp Overview:**
- C/C++ implementation for LLM inference
- Optimized for consumer hardware
- OpenAI-compatible HTTP server
- Cross-platform (macOS Metal, CUDA, CPU)

**llama.vscode Extension:**

Official VS Code extension from the llama.cpp team:

**Features:**
- Inline code completion (rivals Copilot)
- Side-panel chat
- **Agent mode** with multi-file task iteration
- Zero telemetry, full offline support
- Auto-installs llama.cpp for Mac/Windows

**Agent Mode Architecture:**

```
User: "Add authentication to this API"
    ↓
┌─────────────────────────────────────────────────┐
│              Llama Agent (Ctrl+Shift+A)          │
│  ┌───────────────────────────────────────────┐  │
│  │ Built-in Tools (9 total):                 │  │
│  │  • read_file    • write_file              │  │
│  │  • grep         • npm_install             │  │
│  │  • run_tests    • web_search              │  │
│  │  • custom_js_eval                         │  │
│  └───────────────────────────────────────────┘  │
│                      ↓                           │
│  Iterates up to N times (configurable)          │
│                      ↓                           │
│  Changes presented in diff view                 │
│  Accept/reject per file                         │
└─────────────────────────────────────────────────┘
```

**Server API Endpoints:**

| Endpoint | Description | Use Case |
|----------|-------------|----------|
| `/v1/completions` | Text completion | Code generation |
| `/v1/chat/completions` | Chat messages | Conversation |
| `/v1/models` | List loaded model | Discovery |
| `/infill` | Fill-in-the-middle | IDE completion |
| `/embedding` | Text embeddings | RAG, search |

**FIM (Fill-in-the-Middle) Endpoint:**

Critical for IDE code completion:

```json
// POST /infill
{
  "input_prefix": "def calculate_sum(",
  "input_suffix": "):\n    return result",
  "temperature": 0.2,
  "max_tokens": 128,
  "t_max_predict_ms": 500  // Timeout for FIM
}
```

**FIM Prompt Template (Qwen2.5 Coder):**
```
<|fim_prefix|>{prefix}<|fim_suffix|>{suffix}<|fim_middle|>
```

**Performance Benchmarks:**

| Hardware | Model Size | Speed |
|----------|------------|-------|
| Apple M1/M2/M3 (Metal) | 3B | ~60 tok/s |
| NVIDIA RTX 4090 | 7B Q4_0 | ~90 tok/s |
| 8-core CPU | 1.5B | ~15 tok/s |

**Configuration Options:**

```json
{
  "llama.serverPort": 8012,
  "llama.completion.maxTokens": 128,
  "llama.completion.temperature": 0.2,
  "llama.completion.contextWindow": 8192,
  "llama.agent.maxLoops": 15
}
```

**Managing Agents:**

llama.vscode supports custom agents:
```typescript
interface Agent {
  name: string;
  description: string;
  systemPrompt: string;
  tools: string[];  // Subset of built-in tools
}
```

**Strengths:**
- Zero cost, zero telemetry
- Full offline capability
- Direct hardware optimization
- Agent mode for multi-file tasks

**Limitations:**
- Requires local model download
- Quality depends on model choice
- Less polished UX than commercial tools

**Source:** [llama.cpp GitHub](https://github.com/ggml-org/llama.cpp), [llama.vscode Marketplace](https://marketplace.visualstudio.com/items?itemName=ggml-org.llama-vscode)

---

### 1.7 Cursor AI

**Architecture Overview:**
- Fork of VS Code with native AI integration
- Custom IDE (not a VS Code extension)
- Context-aware suggestions using project structure

**Key Features:**
- **Tab Completion**: Sub-second code suggestions
- **Context Variables**: `@Files`, `@Folders`, `@Code` for referencing
- **Multi-File Editing**: Composer mode for cross-file changes
- **Chat**: `Cmd+L` for project-aware conversation
- **Natural Language Editing**: Describe changes in English

**Strengths:**
- Native IDE integration
- Fast inference
- Strong project context

**Limitations:**
- Separate IDE (not VS Code extension)
- CodeCursor extension exists but limited

**Source:** [Cursor AI Overview](https://www.datacamp.com/tutorial/cursor-ai-code-editor)

---

### 1.8 Warp Terminal

**Architecture Overview:**
- Native terminal application (Rust-based)
- AI integrated directly into shell workflow
- Local natural language classifier

**Key Features:**
- **AI Command Suggestions**: `#` prefix for natural language
- **Agent Mode**: Fully autonomous task execution
- **Active AI**: Proactive suggestions based on errors
- **Dispatch Mode**: Autonomous execution with planning
- **Mixed Model Approach**: Access to OpenAI, Anthropic, Google models

**Strengths:**
- Terminal-native AI
- Proactive error assistance
- Privacy-focused (no training on user data)

**Limitations:**
- Not IDE-integrated
- Subscription for higher usage
- macOS/Linux only (Windows via WSL)

**Source:** [Warp AI Features](https://www.warp.dev/warp-ai)

---

### 1.9 Competitive Positioning Matrix

| Feature | Caro | Claude Code | Copilot | Cline | Tabby | llama.vscode | Cursor | Warp |
|---------|------|-------------|---------|-------|-------|--------------|--------|------|
| **Chat Panel** | Planned | Yes | Yes | Yes | Yes | Yes | Yes | N/A |
| **Inline Suggestions** | Planned | No | Yes | No | Yes | Yes | Yes | N/A |
| **Agent Mode** | Planned | No | Yes | Yes | Preview | Yes | Yes | Yes |
| **Terminal Integration** | Core | Yes | Yes | Yes | No | No | Yes | Native |
| **Proactive Analysis** | Planned | No | No | No | No | No | No | Yes |
| **Local/Offline** | Yes | No | No | Yes* | Yes | Yes | No | No |
| **Shell Specialized** | Core | No | No | No | No | No | No | Yes |
| **Safety Validation** | Core | No | No | Partial | No | No | No | No |
| **POSIX Focus** | Core | No | No | No | No | No | No | Partial |
| **Open Source** | Yes | No | No | Yes | Yes | Yes | No | No |
| **RAG/Context** | Planned | No | Yes | Yes | Yes | No | Yes | No |
| **MCP Support** | Planned | No | No | Yes | No | No | No | No |
| **Multi-Backend** | Yes | No | No | Yes | Yes | No | No | Yes |
| **FIM/Infill** | Planned | No | Yes | No | Yes | Yes | Yes | No |
| **Self-Hosted** | Yes | No | No | N/A | Yes | Yes | No | No |

*Cline supports local via Ollama/LM Studio

---

## Part 2: Caro VS Code Extension Vision

### 2.1 Product Philosophy

**Core Principles:**

1. **Non-Duplicative Architecture**
   - VS Code is a thin TypeScript/WebView layer
   - All logic runs in Caro's Rust core as a background process
   - User can always fall back to terminal (`caro` command)

2. **Meet Users Where They Are**
   - Multiple entry points: chat panel, context menu, inline hints
   - Graduated complexity: simple commands → proactive analysis
   - Opt-in proactive features

3. **Infrastructure-First Intelligence**
   - Specialize in shell commands, Docker, CI/CD, scripts
   - Unix philosophy: do one thing well
   - Complement general-purpose AI assistants

4. **Privacy by Design**
   - Local model inference by default
   - No data leaves machine without consent
   - Transparent about what's analyzed

---

### 2.2 User Journey Map

```
┌─────────────────────────────────────────────────────────────────────┐
│                     Caro VS Code Extension                          │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  ┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐ │
│  │   Entry Point 1 │    │  Entry Point 2  │    │  Entry Point 3  │ │
│  │   Chat Panel    │    │  Context Menu   │    │  Proactive Hints│ │
│  │                 │    │                 │    │                 │ │
│  │ "How do I find  │    │ [Select code]   │    │ 💡 Dockerfile   │ │
│  │  large files?"  │    │ → Ask Caro      │    │ uses curl+jq... │ │
│  │                 │    │ → Run Suggestion│    │ Consider using  │ │
│  │      ↓          │    │      ↓          │    │ built-in tools  │ │
│  └────────┬────────┘    └────────┬────────┘    └────────┬────────┘ │
│           │                      │                      │          │
│           └──────────────────────┼──────────────────────┘          │
│                                  │                                  │
│                                  ▼                                  │
│  ┌─────────────────────────────────────────────────────────────┐   │
│  │                      Caro Rust Core                          │   │
│  │                    (Background Process)                      │   │
│  │  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────────────┐│   │
│  │  │ Safety   │ │ Command  │ │ Context  │ │ Proactive Engine ││   │
│  │  │ Validator│ │ Generator│ │ Detector │ │ (AST + Heuristics││   │
│  │  └──────────┘ └──────────┘ └──────────┘ └──────────────────┘│   │
│  └─────────────────────────────────────────────────────────────┘   │
│                                  │                                  │
│                                  ▼                                  │
│  ┌─────────────────────────────────────────────────────────────┐   │
│  │                   VS Code Terminal                           │   │
│  │                 (Native Execution)                           │   │
│  └─────────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────────┘
```

---

### 2.3 Feature Breakdown

#### Feature 1: Chat Panel (Conversational Interface)

**Description:**
Primary interaction point for users to converse with Caro. Similar to Claude Code and Copilot Chat, but specialized for shell commands and infrastructure tasks.

**User Stories:**
- "As a developer, I want to describe what I need in natural language and receive a POSIX-compliant shell command"
- "As a developer, I want to execute the suggested command directly in VS Code's terminal"
- "As a developer, I want to see safety warnings before running destructive commands"

**UI Elements:**
- Activity Bar icon (alongside Claude, Copilot)
- Panel in Secondary Sidebar
- Input field with streaming response
- Command block with copy/execute buttons
- Safety indicator (Safe/Moderate/High/Critical)
- Explanation accordion

**Technical Requirements:**
- WebView panel with React/Vue UI
- Bidirectional message passing to extension host
- IPC to Caro Rust core
- Terminal API for command execution

---

#### Feature 2: Context Menu Integration ("Ask Caro")

**Description:**
Right-click on selected code to ask Caro for shell-related insights. Bridges code understanding with terminal expertise.

**User Stories:**
- "As a developer, I want to select a JSON parsing function and ask Caro for a `jq` equivalent"
- "As a developer, I want to select a Dockerfile and ask Caro to optimize it"
- "As a developer, I want to select a Python script and ask how to run it"

**Context Menu Actions:**
- `Ask Caro About Selection`
- `Ask Caro About Active File`
- `Generate Shell Command for This`
- `Check Script Safety`

**Technical Requirements:**
- Context menu contribution in `package.json`
- Text selection API (`editor.selection`)
- Pre-populate chat with selected context
- File type detection for specialized prompts

---

#### Feature 3: Proactive Analysis Engine

**Description:**
Background analysis of workspace files to provide intelligent suggestions. Caro identifies opportunities where shell expertise adds value.

**Target File Types:**
- Dockerfiles (COPY, RUN, apt-get)
- Shell scripts (*.sh, *.bash, *.zsh)
- Package manifests (package.json scripts, Makefile)
- CI/CD configs (.github/workflows, .gitlab-ci.yml, Jenkinsfile)
- Configuration files (docker-compose.yml, terraform, ansible)

**Proactive Suggestion Types:**
1. **Tool Recommendations**: "You're using Python to parse JSON. Consider `jq` for faster processing."
2. **Security Warnings**: "This script uses `curl | bash`. This is risky—consider downloading and inspecting first."
3. **POSIX Improvements**: "Replace `echo -e` with `printf` for portability."
4. **Performance Tips**: "Combine these 3 `grep` calls into one with `|` for efficiency."

**Technical Requirements:**
- FileSystemWatcher for workspace files
- Heuristic filtering (only analyze relevant files)
- AST parsing for script structure
- Rule engine integration
- CodeLens or diagnostic provider for inline hints
- Opt-in via settings

---

#### Feature 4: Security Analysis Mode

**Description:**
Specialized analysis for detecting risky commands and patterns in scripts. Extends Caro's 52-pattern safety validator to proactive scanning.

**Detection Categories:**
- Network data exfiltration (`curl | bash`, encoded payloads)
- Privilege escalation (`sudo` without justification)
- Destructive operations (rm -rf, dd, mkfs)
- Environment exposure (`printenv`, `env` to logs)
- Permission issues (`chmod 777`, world-readable secrets)

**UI Presentation:**
- Diagnostics panel with severity levels
- Inline CodeLens with "Fix with Caro" action
- Quick fixes suggesting safer alternatives

**Technical Requirements:**
- DiagnosticCollection API
- CodeLens provider
- Quick fix actions
- Integration with safety module in Rust core

---

#### Feature 5: Terminal Enhancement

**Description:**
Enhance VS Code's integrated terminal with Caro intelligence. Provide inline suggestions and error recovery.

**Features:**
- `#` prefix for natural language in terminal (like Warp)
- Error detection with "Ask Caro to fix" prompt
- Command history with semantic search

**Technical Requirements:**
- Terminal API integration
- Shell integration script
- Error pattern detection

---

## Part 3: Technical Architecture

### 3.1 Architecture Overview

```
┌──────────────────────────────────────────────────────────────────┐
│                        VS Code Extension                          │
│                     (TypeScript/WebView)                          │
├──────────────────────────────────────────────────────────────────┤
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐               │
│  │ Chat Panel  │  │ Context Menu│  │ CodeLens    │               │
│  │ (WebView)   │  │ Provider    │  │ Provider    │               │
│  └──────┬──────┘  └──────┬──────┘  └──────┬──────┘               │
│         │                │                │                       │
│         └────────────────┼────────────────┘                       │
│                          │                                        │
│  ┌───────────────────────▼─────────────────────────────────────┐ │
│  │                Extension Host (Node.js)                      │ │
│  │  ┌──────────────────────────────────────────────────────┐   │ │
│  │  │              Message Router                           │   │ │
│  │  │         (WebView ↔ Extension ↔ Caro)                 │   │ │
│  │  └──────────────────────────────────────────────────────┘   │ │
│  └──────────────────────────────────────────────────────────────┘ │
│                          │                                        │
│                          │ IPC (JSON-RPC over stdio)             │
│                          ▼                                        │
└──────────────────────────────────────────────────────────────────┘
                           │
                           │
┌──────────────────────────▼───────────────────────────────────────┐
│                      Caro Rust Core                               │
│                  (Background Process)                             │
├──────────────────────────────────────────────────────────────────┤
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐           │
│  │ JSON-RPC     │  │ Command      │  │ Safety       │           │
│  │ Server       │  │ Generator    │  │ Validator    │           │
│  │ (stdio)      │  │ (Backends)   │  │ (52 patterns)│           │
│  └──────┬───────┘  └──────────────┘  └──────────────┘           │
│         │                                                        │
│  ┌──────▼───────┐  ┌──────────────┐  ┌──────────────┐           │
│  │ Request      │  │ Context      │  │ Proactive    │           │
│  │ Handler      │  │ Detector     │  │ Analyzer     │           │
│  └──────────────┘  └──────────────┘  └──────────────┘           │
└──────────────────────────────────────────────────────────────────┘
```

---

### 3.2 IPC Protocol Design

**Protocol: JSON-RPC 2.0 over stdio**

The extension spawns `caro --server` as a child process and communicates via JSON-RPC messages over stdin/stdout.

**Why JSON-RPC:**
- Well-established protocol (same as LSP)
- Bidirectional (extension → core, core → extension)
- Request/response with correlation IDs
- Notification support (no response expected)

**Message Types:**

```typescript
// Extension → Core: Generate Command
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "generateCommand",
  "params": {
    "input": "find all files larger than 1GB",
    "shell": "zsh",
    "safetyLevel": "moderate",
    "context": {
      "cwd": "/home/user/project",
      "selectedText": null,
      "activeFile": "Dockerfile"
    }
  }
}

// Core → Extension: Generated Command Response
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "command": "find . -type f -size +1G",
    "explanation": "Finds files larger than 1GB in current directory",
    "safetyLevel": "safe",
    "confidence": 0.95,
    "alternatives": [
      "find . -size +1G -exec ls -lh {} \\;"
    ]
  }
}

// Core → Extension: Proactive Suggestion (Notification)
{
  "jsonrpc": "2.0",
  "method": "proactiveSuggestion",
  "params": {
    "file": "/home/user/project/install.sh",
    "line": 15,
    "type": "security",
    "severity": "high",
    "message": "curl | bash pattern detected",
    "suggestion": "Download to file first, then execute after inspection"
  }
}
```

---

### 3.3 Extension Components

#### 3.3.1 Package Structure

```
caro-vscode/
├── package.json              # Extension manifest
├── src/
│   ├── extension.ts          # Activation & lifecycle
│   ├── caro-client.ts        # JSON-RPC client to Caro core
│   ├── providers/
│   │   ├── chat-panel.ts     # WebView chat panel
│   │   ├── codelens.ts       # CodeLens for proactive hints
│   │   ├── diagnostics.ts    # Diagnostic provider for security
│   │   └── commands.ts       # Command registrations
│   └── webview/
│       ├── index.html        # Chat panel HTML
│       ├── chat.js           # Chat UI logic
│       └── styles.css        # Styling
├── media/
│   └── caro-icon.svg         # Activity bar icon
└── test/
    └── suite/
```

#### 3.3.2 Package.json Configuration

```json
{
  "name": "caro",
  "displayName": "Caro - Shell Command Intelligence",
  "description": "Convert natural language to safe POSIX shell commands",
  "version": "0.1.0",
  "publisher": "caro",
  "engines": {
    "vscode": "^1.85.0"
  },
  "categories": ["AI", "Other"],
  "activationEvents": [
    "onStartupFinished"
  ],
  "main": "./out/extension.js",
  "contributes": {
    "viewsContainers": {
      "activitybar": [
        {
          "id": "caro",
          "title": "Caro",
          "icon": "media/caro-icon.svg"
        }
      ]
    },
    "views": {
      "caro": [
        {
          "type": "webview",
          "id": "caro.chatPanel",
          "name": "Chat"
        }
      ]
    },
    "commands": [
      {
        "command": "caro.askAboutSelection",
        "title": "Ask Caro About Selection"
      },
      {
        "command": "caro.generateCommand",
        "title": "Generate Shell Command"
      },
      {
        "command": "caro.checkSafety",
        "title": "Check Script Safety"
      }
    ],
    "menus": {
      "editor/context": [
        {
          "submenu": "caro.submenu",
          "group": "caro"
        }
      ],
      "caro.submenu": [
        {
          "command": "caro.askAboutSelection",
          "when": "editorHasSelection"
        },
        {
          "command": "caro.generateCommand"
        },
        {
          "command": "caro.checkSafety",
          "when": "resourceExtname =~ /\\.(sh|bash|zsh|dockerfile)$/i"
        }
      ]
    },
    "submenus": [
      {
        "id": "caro.submenu",
        "label": "Caro"
      }
    ],
    "configuration": {
      "title": "Caro",
      "properties": {
        "caro.enableProactiveAnalysis": {
          "type": "boolean",
          "default": false,
          "description": "Enable background file analysis for suggestions"
        },
        "caro.safetyLevel": {
          "type": "string",
          "enum": ["strict", "moderate", "permissive"],
          "default": "moderate",
          "description": "Safety validation strictness"
        },
        "caro.binaryPath": {
          "type": "string",
          "default": "caro",
          "description": "Path to caro binary"
        }
      }
    }
  }
}
```

---

### 3.4 Rust Core Modifications

#### 3.4.1 New Server Mode

Add `--server` flag to `caro` binary that starts JSON-RPC server:

```rust
// src/main.rs
#[derive(Parser)]
pub struct Cli {
    #[clap(subcommand)]
    command: Option<Command>,

    /// Run as JSON-RPC server for IDE integration
    #[clap(long)]
    server: bool,
}

// src/server/mod.rs
pub struct JsonRpcServer {
    stdin: BufReader<Stdin>,
    stdout: BufWriter<Stdout>,
    backend: Arc<dyn CommandGenerator>,
    validator: SafetyValidator,
    context: ExecutionContext,
}

impl JsonRpcServer {
    pub async fn run(&mut self) -> Result<()> {
        loop {
            let request = self.read_request().await?;
            let response = self.handle_request(request).await?;
            self.write_response(response).await?;
        }
    }

    async fn handle_request(&self, req: Request) -> Response {
        match req.method.as_str() {
            "generateCommand" => self.handle_generate(req.params).await,
            "validateCommand" => self.handle_validate(req.params).await,
            "analyzeFile" => self.handle_analyze(req.params).await,
            _ => Response::error(-32601, "Method not found"),
        }
    }
}
```

#### 3.4.2 Proactive Analysis Engine

New module for file analysis:

```rust
// src/proactive/mod.rs
pub struct ProactiveAnalyzer {
    patterns: Vec<CompiledPattern>,
    file_filters: Vec<FileFilter>,
}

pub struct FileAnalysisResult {
    pub suggestions: Vec<Suggestion>,
    pub security_issues: Vec<SecurityIssue>,
    pub improvement_opportunities: Vec<Improvement>,
}

pub struct Suggestion {
    pub line: usize,
    pub column: usize,
    pub severity: Severity,
    pub message: String,
    pub suggested_fix: Option<String>,
    pub category: SuggestionCategory,
}

pub enum SuggestionCategory {
    ToolRecommendation,  // "Consider using jq"
    SecurityWarning,      // "curl | bash is risky"
    PosixCompliance,      // "Use printf over echo -e"
    Performance,          // "Combine grep calls"
    BestPractice,         // "Quote your variables"
}

impl ProactiveAnalyzer {
    pub fn analyze_file(&self, content: &str, file_type: FileType) -> FileAnalysisResult {
        // Use tree-sitter for AST parsing
        // Apply rule engine
        // Return structured suggestions
    }

    pub fn should_analyze(&self, path: &Path) -> bool {
        // Heuristic filtering
        // Check file type, size, patterns
        self.file_filters.iter().any(|f| f.matches(path))
    }
}
```

---

### 3.5 Communication Flow

```
┌────────────────────────────────────────────────────────────────────┐
│                        Startup Sequence                             │
├────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  1. VS Code loads extension                                         │
│     ↓                                                               │
│  2. Extension spawns: `caro --server`                              │
│     ↓                                                               │
│  3. Caro initializes (loads model, safety patterns)                │
│     ↓                                                               │
│  4. Caro sends: {"jsonrpc":"2.0","method":"ready"}                 │
│     ↓                                                               │
│  5. Extension registers providers (Chat, CodeLens, Diagnostics)    │
│                                                                     │
└────────────────────────────────────────────────────────────────────┘

┌────────────────────────────────────────────────────────────────────┐
│                    User Generates Command                           │
├────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  1. User types in chat: "list large files"                         │
│     ↓                                                               │
│  2. WebView posts to extension: {command: "generate", text: "..."}│
│     ↓                                                               │
│  3. Extension sends JSON-RPC to Caro core                          │
│     ↓                                                               │
│  4. Caro generates command + safety validation                     │
│     ↓                                                               │
│  5. Caro responds with command, explanation, safety level          │
│     ↓                                                               │
│  6. Extension forwards to WebView                                   │
│     ↓                                                               │
│  7. WebView renders response with Execute button                    │
│     ↓                                                               │
│  8. User clicks Execute → Terminal API runs command                │
│                                                                     │
└────────────────────────────────────────────────────────────────────┘

┌────────────────────────────────────────────────────────────────────┐
│                    Proactive Analysis                               │
├────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  1. User opens Dockerfile                                           │
│     ↓                                                               │
│  2. Extension sends file content to Caro (if enabled)              │
│     ↓                                                               │
│  3. Caro analyzes with ProactiveAnalyzer                           │
│     ↓                                                               │
│  4. Caro sends notification with suggestions                       │
│     ↓                                                               │
│  5. Extension creates DiagnosticCollection entries                 │
│     ↓                                                               │
│  6. VS Code shows squiggly lines and Problems panel entries        │
│     ↓                                                               │
│  7. User hovers → sees Caro suggestion                             │
│     ↓                                                               │
│  8. User clicks Quick Fix → applies suggestion                     │
│                                                                     │
└────────────────────────────────────────────────────────────────────┘
```

---

## Part 4: Implementation Roadmap

### Phase 1: Foundation (Weeks 1-3)

**Goal:** Basic chat panel with command generation

**Deliverables:**
- [ ] VS Code extension scaffolding
- [ ] `caro --server` mode in Rust core
- [ ] JSON-RPC protocol implementation
- [ ] Basic WebView chat panel
- [ ] Command generation via chat
- [ ] Terminal execution integration

**Success Metrics:**
- User can generate command from chat
- Command executes in VS Code terminal
- Safety level displayed

---

### Phase 2: Context Integration (Weeks 4-5)

**Goal:** Deep editor integration

**Deliverables:**
- [ ] Context menu integration
- [ ] "Ask Caro About Selection" command
- [ ] File context awareness
- [ ] Active file detection
- [ ] Shell type auto-detection

**Success Metrics:**
- User can select code and ask Caro
- Context improves command relevance

---

### Phase 3: Proactive Intelligence (Weeks 6-8)

**Goal:** Background analysis and suggestions

**Deliverables:**
- [ ] FileSystemWatcher integration
- [ ] ProactiveAnalyzer in Rust core
- [ ] DiagnosticCollection provider
- [ ] CodeLens provider
- [ ] Settings for opt-in

**Success Metrics:**
- Dockerfiles show inline suggestions
- Shell scripts highlight security issues
- User can apply quick fixes

---

### Phase 4: Security Analysis (Weeks 9-10)

**Goal:** Comprehensive security scanning

**Deliverables:**
- [ ] Extended security patterns
- [ ] Severity categorization
- [ ] "Fix with Caro" quick actions
- [ ] Security report generation

**Success Metrics:**
- High-risk patterns detected
- Actionable fixes suggested

---

### Phase 5: Polish & Performance (Weeks 11-12)

**Goal:** Production readiness

**Deliverables:**
- [ ] Performance optimization
- [ ] Error handling improvements
- [ ] Telemetry (opt-in)
- [ ] Documentation
- [ ] Marketplace preparation

**Success Metrics:**
- < 100ms startup time
- < 2s command generation
- Positive user feedback

---

## Part 5: API Contracts

### 5.1 JSON-RPC Methods

#### `generateCommand`

Generate a shell command from natural language.

**Request:**
```typescript
interface GenerateCommandParams {
  input: string;                    // Natural language description
  shell?: "bash" | "zsh" | "fish";  // Target shell
  safetyLevel?: "strict" | "moderate" | "permissive";
  context?: {
    cwd?: string;                   // Current working directory
    selectedText?: string;          // Selected text in editor
    activeFile?: string;            // Currently open file
    activeFileContent?: string;     // File content for context
  };
}
```

**Response:**
```typescript
interface GenerateCommandResult {
  command: string;                  // Generated command
  explanation: string;              // Human-readable explanation
  safetyLevel: "safe" | "moderate" | "high" | "critical";
  confidence: number;               // 0.0 - 1.0
  warnings?: string[];              // Safety warnings
  alternatives?: string[];          // Alternative commands
  estimatedImpact?: string;         // Description of effects
}
```

#### `validateCommand`

Validate a command for safety.

**Request:**
```typescript
interface ValidateCommandParams {
  command: string;
}
```

**Response:**
```typescript
interface ValidateCommandResult {
  allowed: boolean;
  riskLevel: "safe" | "moderate" | "high" | "critical";
  explanation: string;
  matchedPatterns: string[];
  suggestions?: string[];           // Safer alternatives
}
```

#### `analyzeFile`

Analyze a file for proactive suggestions.

**Request:**
```typescript
interface AnalyzeFileParams {
  path: string;
  content: string;
  fileType?: string;                // Override detected type
}
```

**Response:**
```typescript
interface AnalyzeFileResult {
  suggestions: Suggestion[];
}

interface Suggestion {
  line: number;
  column: number;
  endLine?: number;
  endColumn?: number;
  severity: "info" | "warning" | "error";
  message: string;
  category: "tool" | "security" | "posix" | "performance" | "practice";
  suggestedFix?: {
    replacement: string;
    description: string;
  };
}
```

#### Notifications (Core → Extension)

**`proactiveSuggestion`**: Sent when background analysis finds issues.

**`modelLoaded`**: Sent when local model is ready.

**`ready`**: Sent when server initialization complete.

---

## Part 6: UX Guidelines

### 6.1 Visual Design

**Icon:**
- Caro icon in Activity Bar (consistent with Claude spark, Copilot logo)
- Subtle, professional design
- Recognizable at small sizes

**Color Palette:**
- Safe: Green (#4CAF50)
- Moderate: Yellow (#FFC107)
- High: Orange (#FF9800)
- Critical: Red (#F44336)

**Chat Panel:**
- Clean, minimal interface
- Code blocks with syntax highlighting
- Streaming response indicator
- Clear execute/copy buttons

### 6.2 Interaction Patterns

**Keyboard Shortcuts:**
- `Ctrl+Shift+C`: Open Caro chat
- `Ctrl+Shift+G`: Generate command for selection

**Context Menu:**
- Grouped under "Caro" submenu
- Most common actions first
- File-type-aware options

**Proactive Hints:**
- Non-intrusive (information level by default)
- Opt-in for warnings
- Dismissible

---

## Part 7: Success Metrics

### 7.1 Adoption Metrics

- Extension installs
- Daily active users
- Commands generated per session
- Terminal execution rate

### 7.2 Quality Metrics

- Command accuracy (user acceptance rate)
- Safety validation correctness
- Proactive suggestion relevance (dismiss rate)

### 7.3 Performance Metrics

- Extension activation time (< 100ms)
- Command generation latency (< 2s)
- Memory footprint

---

## Part 8: Future Considerations

### 8.1 TUI Integration

The VS Code extension is one interface. Caro will also have:
- Terminal UI (TUI) using `ratatui`
- Same Rust core, different presentation
- Shared conversation history

### 8.2 GUI Application

Future desktop application:
- Native UI (Tauri or similar)
- System tray integration
- Global hotkey for command generation

### 8.3 Mobile Application

Long-term vision:
- iOS/Android app
- SSH to remote machines
- Command history sync

### 8.4 Chat Participant API

When VS Code Chat Participant API matures:
- Register Caro as `@caro` participant
- Integrate with Copilot ecosystem
- Benefit from shared context

---

## Appendix A: Competitive Feature Matrix

| Feature | Caro (Planned) | Claude Code | Copilot | Cline |
|---------|---------------|-------------|---------|-------|
| Chat Panel | Yes | Yes | Yes | Yes |
| Context Menu | Yes | No | No | Yes |
| Inline Suggestions | Future | No | Yes | No |
| Proactive Analysis | Yes | No | No | No |
| Security Scanning | Yes | No | No | No |
| Local Inference | Yes | No | No | No |
| Shell Specialization | Core | No | Partial | No |
| Safety Validation | Core | No | No | Partial |
| Terminal Integration | Yes | Yes | Yes | Yes |
| MCP Support | Future | No | No | Yes |

---

## Appendix B: Research Sources

### VS Code Extension Development
- [VS Code Extension API](https://code.visualstudio.com/api)
- [VS Code UX Guidelines](https://code.visualstudio.com/api/ux-guidelines/overview)
- [WebView API](https://code.visualstudio.com/api/extension-guides/webview)
- [Chat Participant API](https://code.visualstudio.com/api/extension-guides/ai/chat)

### Competitor Analysis
- [Claude Code VS Code Docs](https://code.claude.com/docs/en/vs-code)
- [GitHub Copilot Overview](https://code.visualstudio.com/docs/copilot/overview)
- [Cline Architecture](https://deepwiki.com/cline/cline/1.3-architecture-overview)
- [Cline GitHub](https://github.com/cline/cline)
- [Cline Tool Integrations](https://deepwiki.com/cline/cline/5-tool-integrations)
- [Cursor AI Guide](https://www.datacamp.com/tutorial/cursor-ai-code-editor)
- [Warp AI Features](https://www.warp.dev/warp-ai)
- [Tabby GitHub](https://github.com/TabbyML/tabby)
- [Tabby Documentation](https://tabby.tabbyml.com/docs/)
- [Tabby Context Providers](https://tabby.tabbyml.com/docs/administration/context/)

### Local Inference
- [llama.cpp GitHub](https://github.com/ggml-org/llama.cpp)
- [llama.vscode Extension](https://marketplace.visualstudio.com/items?itemName=ggml-org.llama-vscode)
- [llama.vscode Wiki](https://github.com/ggml-org/llama.vscode/wiki/How-to-use)
- [llama.cpp Server API](https://deepwiki.com/ggml-org/llama.cpp/5.2-server)

### Model Context Protocol (MCP)
- [MCP Architecture](https://modelcontextprotocol.io/docs/learn/architecture)
- [MCP GitHub](https://github.com/modelcontextprotocol)
- [MCP Introduction](https://www.philschmid.de/mcp-introduction)

### Technical References
- [Language Server Protocol](https://microsoft.github.io/language-server-protocol/)
- [tower-lsp (Rust LSP)](https://github.com/ebkalderon/tower-lsp)
- [tree-sitter](https://tree-sitter.github.io/tree-sitter/)
- [Semgrep VS Code](https://semgrep.dev/blog/2023/semgrep-vscode-extension/)

---

## Appendix C: Glossary

| Term | Definition |
|------|------------|
| **Activity Bar** | VS Code's leftmost bar with icons for views |
| **Chat Participant** | VS Code API for custom chat agents |
| **CodeLens** | Inline actionable hints above code lines |
| **Extension Host** | Node.js process running VS Code extensions |
| **IPC** | Inter-Process Communication |
| **JSON-RPC** | JSON-based Remote Procedure Call protocol |
| **LSP** | Language Server Protocol |
| **MCP** | Model Context Protocol (Anthropic) - Open protocol for LLM tool integration |
| **RAG** | Retrieval-Augmented Generation - Using search to enhance LLM context |
| **FIM** | Fill-in-the-Middle - Code completion with prefix and suffix context |
| **WebView** | Embedded web browser in VS Code |

---

*Document Version: 1.1*
*Last Updated: 2025-12-22*
*Revision Notes: Added Tabby, llama.cpp/llama.vscode, and expanded Cline analysis*
