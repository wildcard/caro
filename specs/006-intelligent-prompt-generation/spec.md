# Feature Specification: Intelligent Prompt Generation & Multi-Agent Validation

**Feature ID:** 006
**Status:** Draft
**Created:** 2025-11-28
**Last Updated:** 2025-11-28
**Owners:** Development Team

---

## Executive Summary

This specification defines an intelligent command generation system that addresses critical failures in platform-specific command generation through multi-agent validation, dynamic platform detection, and adaptive prompt engineering. The system learns from session failures to generate accurate, executable commands tailored to the user's specific platform and toolset.

### Problem Statement

Current command generation suffers from three critical failures:

1. **Platform Incompatibility**: Generates GNU-specific commands (e.g., `ls --sort=size`) on BSD systems (macOS), causing execution failures
2. **Weak User Intent Understanding**: Fails to interpret high-level requests (e.g., "declutter my mac") into actionable commands (e.g., finding large files)
3. **No Validation Layer**: Presents commands without verifying flag compatibility against actual system tools

### Solution Overview

A multi-agent system that:
- **Detects** platform characteristics and available tools
- **Validates** generated commands against actual man pages
- **Clarifies** ambiguous user requests through interactive dialogue
- **Adapts** prompts based on confidence scores and validation feedback
- **Learns** from community-contributed prompt templates

---

## Goals & Success Metrics

### Primary Goals

1. **Zero Platform-Incompatible Commands**: 100% of generated commands must use flags available on the target platform
2. **Intelligent Intent Recognition**: System accurately interprets user intent with 90%+ success rate
3. **Adaptive Generation**: Multi-turn flow reduces failures from 40% to <5%
4. **Community Extensibility**: Prompt templates externalized for community improvement

### Success Metrics

| Metric | Current | Target | Measurement |
|--------|---------|--------|-------------|
| Platform compatibility rate | 60% | 99% | Commands execute without flag errors |
| Single-shot success rate | 60% | 85% | First generation is valid |
| User intent accuracy | 50% | 90% | Command matches user goal |
| Validation time overhead | N/A | <100ms | 95th percentile |
| Cold start time | 2s | <5s | With man page cache miss |
| Warm start time | 2s | <2s | With man page cache hit |

### Non-Goals

- Supporting non-POSIX platforms (Windows CMD/PowerShell) in Phase 1
- Real-time streaming command generation (future enhancement)
- Natural language explanations in responses (keep JSON-only)

---

## User Stories

### US-001: Platform-Aware Command Generation

**As a** macOS user running cmdai
**I want** commands generated specifically for BSD utilities
**So that** every command executes without flag compatibility errors

**Acceptance Criteria:**
- System auto-detects macOS → selects BSD prompt template
- Generated command uses `-S` instead of `--sort=size`
- Validation agent confirms all flags exist in BSD `ls`
- Command executes successfully on first try

**Example:**
```bash
$ cmdai "list files sorted by size"
Command: ls -lhS
Explanation: Generated using BSD backend
✓ Validated against BSD ls 8.3
```

---

### US-002: Cross-Platform Command Generation

**As a** DevOps engineer working on macOS
**I want** to generate commands targeting Linux servers
**So that** I can create scripts for deployment without switching machines

**Acceptance Criteria:**
- User can configure target platform: `cmdai config set platform.target_os linux`
- System warns when target platform differs from current
- Commands validated against target platform specifications
- Provides instructions for local testing if needed

**Example:**
```bash
$ cmdai config set platform.target_os linux
$ cmdai "list files sorted by size"

⚠️  Cross-Platform Mode: macOS → Linux (GNU)

Command: ls -lh --sort=size
Explanation: Generated for Linux/GNU systems
✓ Validated against GNU coreutils ls 9.0

Note: To test locally, install 'coreutils' via Homebrew
```

---

### US-003: Ambiguity Detection & Clarification

**As a** user requesting "clean up disk space"
**I want** the system to ask clarifying questions
**So that** it generates a safe, targeted command instead of guessing

**Acceptance Criteria:**
- System detects ambiguity score > 0.7
- Generates 2-4 specific clarification questions
- User provides answers interactively
- Enhanced prompt incorporates answers
- Final command matches refined intent

**Example:**
```bash
$ cmdai "clean up disk space"

🤔 Clarification needed (ambiguity: 0.82)

1. Which location?
   a) Home directory (~)
   b) Specific folder
   c) System-wide (requires sudo)

2. What criteria?
   a) Large files (>100MB)
   b) Old files (>30 days)
   c) Duplicate files

3. Action preference?
   a) List only (safe)
   b) Move to trash
   c) Permanent delete

Your choice (1a, 2a, 3a): 1a 2a 3a

Command: find ~ -type f -size +100M -mtime +30 -ls
Explanation: Lists large, old files in home directory
✓ Confidence: 0.92
```

---

### US-004: Multi-Turn Validation Loop

**As a** system administrator
**I want** incorrect commands to be automatically fixed
**So that** I don't waste time debugging flag incompatibilities

**Acceptance Criteria:**
- First generation uses base prompt
- Validation agent detects incompatible flags
- System regenerates with specific feedback
- Maximum 3 retry attempts
- Escalates to detailed prompt if retries fail
- Final command passes validation

**Example:**
```bash
$ cmdai "show disk usage sorted"

[Turn 1] Generating with base prompt...
Generated: df -h --output=source,size,used
Validation: ❌ FAIL (--output not available in BSD df)

[Turn 2] Regenerating with feedback...
Feedback: "BSD df doesn't support --output. Use standard columns."
Generated: df -h | awk '{print $1,$2,$3}'
Validation: ✅ PASS

Command: df -h | awk '{print $1,$2,$3}'
✓ Confidence: 0.78
```

---

### US-005: Community Prompt Templates

**As a** power user with specific command preferences
**I want** to create custom prompt templates
**So that** cmdai generates commands in my preferred style

**Acceptance Criteria:**
- Prompt templates stored in `~/.config/cmdai/prompts/`
- Templates use TOML format with variable substitution
- User can set active template via config
- Templates support inheritance (parent templates)
- System falls back to default if custom template fails

**Example:**
```bash
$ cat ~/.config/cmdai/prompts/modern-macos.toml
[meta]
name = "Modern macOS with Homebrew tools"
parent = "base-bsd.toml"

[prompt.preferences]
prefer_modern = true
tools = ["fd", "rg", "bat", "exa"]

[prompt.additions]
style = """
Prefer modern alternatives when available:
- fd instead of find
- rg instead of grep
- exa instead of ls
- bat instead of cat
"""

$ cmdai config set prompts.base_template modern-macos.toml
$ cmdai "search for rust files"

Command: fd -e rs
Explanation: Using fd (modern find alternative)
✓ Confidence: 0.95
```

---

## Technical Architecture

### System Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                         CLI Interface                            │
│                      (main.rs, cli/mod.rs)                       │
└─────────────────────┬───────────────────────────────────────────┘
                      │
                      ▼
┌─────────────────────────────────────────────────────────────────┐
│                   Configuration Manager                          │
│  - Platform Detection (OS, Unix Flavor, Shell)                  │
│  - User Preferences (Config File Management)                    │
│  - Man Page Cache (Tool Availability Database)                  │
└─────────────────────┬───────────────────────────────────────────┘
                      │
                      ▼
┌─────────────────────────────────────────────────────────────────┐
│                     Orchestrator Agent                           │
│  - Ambiguity Detection                                          │
│  - Agent Flow Selection (Single-Shot vs Multi-Turn)            │
│  - Confidence Scoring                                           │
│  - Prompt Template Selection                                    │
└──────┬──────────────┬──────────────┬──────────────┬─────────────┘
       │              │              │              │
       ▼              ▼              ▼              ▼
┌──────────┐  ┌─────────────┐  ┌──────────┐  ┌──────────────┐
│ Generator│  │ Clarification│  │Validation│  │   Feedback   │
│  Agent   │  │    Agent     │  │  Agent   │  │    Agent     │
└──────────┘  └─────────────┘  └──────────┘  └──────────────┘
       │              │              │              │
       │              │              ▼              │
       │              │      ┌──────────────┐       │
       │              │      │  Man Page    │       │
       │              │      │  Analyzer    │       │
       │              │      │   (Cache)    │       │
       │              │      └──────────────┘       │
       │              │              │              │
       └──────────────┴──────────────┴──────────────┘
                      │
                      ▼
              ┌──────────────┐
              │   Backend    │
              │  (MLX/CPU)   │
              └──────────────┘
```

### Component Responsibilities

#### 1. Configuration Manager

**Location:** `src/config/`

**Responsibilities:**
- Auto-detect platform on first run
- Create/load configuration file (`~/.config/cmdai/config.toml`)
- Manage user preferences and overrides
- Initialize man page cache
- Provide platform-specific defaults

**Key APIs:**
```rust
pub struct ConfigManager {
    pub platform: PlatformConfig,
    pub generation: GenerationConfig,
    pub prompts: PromptConfig,
}

impl ConfigManager {
    pub fn initialize() -> Result<Self>;
    pub fn detect_platform() -> PlatformInfo;
    pub fn load_or_create() -> Result<Self>;
    pub fn save(&self) -> Result<()>;
}

pub struct PlatformConfig {
    pub os: Platform,              // macos, linux, windows
    pub unix_flavor: UnixFlavor,   // bsd, gnu
    pub shell: ShellType,          // zsh, bash, fish
    pub arch: Architecture,        // aarch64, x86_64
    pub target_os: Option<Platform>, // For cross-platform generation
}
```

---

#### 2. Man Page Analyzer

**Location:** `src/agents/man_page_analyzer.rs`

**Responsibilities:**
- Scan system for available tools on first run
- Parse man pages to extract available flags
- Cross-validate with `--help` output
- Generate structured cache (`~/.cache/cmdai/man-pages.json`)
- Provide flag validation API

**Key APIs:**
```rust
pub struct ManPageAnalyzer {
    cache: ManPageCache,
}

impl ManPageAnalyzer {
    pub async fn initialize_cache() -> Result<ManPageCache>;
    pub fn validate_command(&self, cmd: &str) -> ValidationResult;
    pub fn suggest_alternatives(&self, tool: &str, flag: &str) -> Vec<String>;
}

pub struct ValidationResult {
    pub is_valid: bool,
    pub invalid_flags: Vec<InvalidFlag>,
    pub suggestions: Vec<String>,
    pub confidence: f32,
}

pub struct InvalidFlag {
    pub flag: String,
    pub tool: String,
    pub reason: String,
    pub alternative: Option<String>,
}
```

**Cache Structure:**
```json
{
  "version": "1.0.0",
  "platform": {
    "os": "macos",
    "unix_flavor": "bsd"
  },
  "generated_at": "2025-11-28T12:00:00Z",
  "tools": {
    "ls": {
      "path": "/bin/ls",
      "version": "BSD ls 8.3",
      "flags": {
        "-l": {"desc": "Long format", "requires_arg": false},
        "-h": {"desc": "Human-readable sizes", "requires_arg": false},
        "-S": {"desc": "Sort by size", "requires_arg": false}
      },
      "forbidden_flags": ["--sort", "--color", "--human-readable"]
    }
  }
}
```

---

#### 3. Orchestrator Agent

**Location:** `src/agents/orchestrator.rs`

**Responsibilities:**
- Analyze user request for ambiguity
- Calculate confidence score at each stage
- Select appropriate agent flow (single-shot vs multi-turn)
- Coordinate between generator, validator, clarification agents
- Manage retry logic and prompt escalation

**Key APIs:**
```rust
pub struct Orchestrator {
    config: Arc<ConfigManager>,
    man_analyzer: Arc<ManPageAnalyzer>,
    prompt_loader: PromptLoader,
}

impl Orchestrator {
    pub async fn generate_command(&self, request: CommandRequest)
        -> Result<GeneratedCommand>;

    fn detect_ambiguity(&self, input: &str) -> AmbiguityScore;
    fn calculate_confidence(&self, result: &ValidationResult) -> f32;
    fn select_flow(&self, ambiguity: f32, confidence: f32) -> AgentFlow;
}

pub enum AgentFlow {
    SingleShot,                    // Direct generation
    MultiTurn { max_retries: usize }, // Generation + validation loop
    Clarification,                 // Ask questions first
    Interactive,                   // Clarification + Multi-turn
}

pub struct AmbiguityScore {
    pub score: f32,              // 0.0-1.0
    pub factors: Vec<String>,    // Reasons for ambiguity
    pub requires_clarification: bool,
}
```

---

#### 4. Generator Agent

**Location:** `src/agents/generator.rs`

**Responsibilities:**
- Load appropriate prompt template
- Inject platform context and tool availability
- Call LLM backend (MLX/Ollama/vLLM)
- Parse JSON response
- Handle generation failures

**Key APIs:**
```rust
pub struct GeneratorAgent {
    backend: Box<dyn CommandGenerator>,
    prompt_loader: PromptLoader,
}

impl GeneratorAgent {
    pub async fn generate(
        &self,
        request: &CommandRequest,
        template: &PromptTemplate,
        context: &GenerationContext,
    ) -> Result<String>;
}

pub struct GenerationContext {
    pub platform: PlatformConfig,
    pub available_tools: Vec<String>,
    pub user_input: String,
    pub clarifications: Option<HashMap<String, String>>,
    pub validation_feedback: Option<Vec<String>>,
}
```

---

#### 5. Validation Agent

**Location:** `src/agents/validator.rs`

**Responsibilities:**
- Parse generated command into tokens
- Extract tools and flags
- Validate against man page cache
- Detect forbidden patterns (platform-specific)
- Generate specific feedback for regeneration

**Key APIs:**
```rust
pub struct ValidationAgent {
    man_analyzer: Arc<ManPageAnalyzer>,
}

impl ValidationAgent {
    pub fn validate(&self, command: &str) -> ValidationResult;
    pub fn generate_feedback(&self, result: &ValidationResult) -> Vec<String>;
}

pub struct ParsedCommand {
    pub tools: Vec<ToolInvocation>,
    pub pipes: Vec<PipeInfo>,
    pub redirects: Vec<RedirectInfo>,
}

pub struct ToolInvocation {
    pub name: String,
    pub flags: Vec<String>,
    pub args: Vec<String>,
}
```

---

#### 6. Clarification Agent

**Location:** `src/agents/clarification.rs`

**Responsibilities:**
- Analyze ambiguous requests
- Generate 2-4 specific questions
- Parse user responses
- Enhance original prompt with clarifications

**Key APIs:**
```rust
pub struct ClarificationAgent {
    llm_backend: Box<dyn CommandGenerator>,
}

impl ClarificationAgent {
    pub async fn generate_questions(
        &self,
        input: &str,
        ambiguity: &AmbiguityScore,
    ) -> Result<Vec<Question>>;

    pub fn enhance_prompt(
        &self,
        original: &str,
        answers: &HashMap<String, String>,
    ) -> String;
}

pub struct Question {
    pub id: String,
    pub text: String,
    pub options: Vec<String>,
    pub default: Option<String>,
}
```

---

#### 7. Feedback Agent

**Location:** `src/agents/feedback.rs`

**Responsibilities:**
- Analyze validation failures
- Generate specific, actionable feedback
- Suggest alternative approaches
- Track retry history

**Key APIs:**
```rust
pub struct FeedbackAgent {
    man_analyzer: Arc<ManPageAnalyzer>,
}

impl FeedbackAgent {
    pub fn generate_feedback(
        &self,
        validation: &ValidationResult,
        attempt: usize,
    ) -> FeedbackMessage;
}

pub struct FeedbackMessage {
    pub severity: FeedbackSeverity,
    pub issues: Vec<Issue>,
    pub suggestions: Vec<Suggestion>,
    pub enhanced_context: Option<String>,
}

pub enum FeedbackSeverity {
    Minor,      // Simple flag replacement
    Moderate,   // Rethink approach
    Critical,   // Completely wrong tool/approach
}
```

---

### Prompt Template System

**Location:** `~/.config/cmdai/prompts/`

**Template Format (TOML):**

```toml
[meta]
name = "BSD Default Prompt"
version = "1.0.0"
platform = "bsd"
parent = null  # Or path to parent template
confidence_threshold = 0.6

[prompt]
system = """
You are a command-line expert for {{unix_flavor}}/{{os}} systems.

CRITICAL REQUIREMENTS:
1. Generate commands compatible with {{unix_flavor}} utilities
2. NEVER use GNU-specific long options if platform is BSD
3. Current platform context:
   - OS: {{os}}
   - Unix Flavor: {{unix_flavor}}
   - Shell: {{shell}}
   - Available Tools: {{tools}}

Response Format (JSON ONLY):
{"cmd": "your_command_here"}

Request: {{user_input}}
{{#if clarifications}}
User Clarifications:
{{#each clarifications}}
  - {{@key}}: {{this}}
{{/each}}
{{/if}}

{{#if validation_feedback}}
Previous Attempt Failed:
{{#each validation_feedback}}
  - {{this}}
{{/each}}
{{/if}}
"""

[examples]
# Platform-specific examples
list_sorted_bsd = "ls -lhS"
list_sorted_gnu = "ls -lh --sort=size"
find_large = "du -sh * | sort -rh"

[validation]
required_patterns = ["^[a-z]"]
forbidden_patterns_bsd = ["--sort", "--color", "--human-readable"]
forbidden_patterns_gnu = []

[fallback]
escalation_template = "detailed-bsd.toml"
max_retries = 3
```

**Variable Substitution:**
```rust
pub struct PromptVariables {
    pub os: String,
    pub unix_flavor: String,
    pub shell: String,
    pub tools: String,
    pub user_input: String,
    pub clarifications: Option<HashMap<String, String>>,
    pub validation_feedback: Option<Vec<String>>,
}
```

---

### Agent Flow Diagrams

#### Flow 1: Single-Shot Success (Happy Path)

```
User Input: "list files sorted by size"
     │
     ▼
[Orchestrator]
  - Ambiguity: 0.1 (low)
  - Flow: SingleShot
     │
     ▼
[Generator Agent]
  - Template: base-bsd.toml
  - Context: {os: macos, tools: [ls, find, ...]}
  - Generated: "ls -lhS"
     │
     ▼
[Validation Agent]
  - Parse: ls -lhS
  - Validate flags: ✓ All valid
  - Result: PASS
     │
     ▼
[Orchestrator]
  - Confidence: 0.95
  - Action: Present to user
     │
     ▼
Output: "ls -lhS"
Time: ~1.5s
```

---

#### Flow 2: Multi-Turn Validation Loop

```
User Input: "show files sorted"
     │
     ▼
[Orchestrator]
  - Ambiguity: 0.2 (low)
  - Flow: SingleShot
     │
     ▼
[Generator] (Attempt 1)
  - Generated: "ls -lh --sort=size"
     │
     ▼
[Validation Agent]
  - Parse: ls -lh --sort=size
  - Flag check: --sort ✗ (not in BSD)
  - Result: FAIL
     │
     ▼
[Feedback Agent]
  - Issue: "--sort not available in BSD ls"
  - Suggestion: "Use -S flag for size sorting"
     │
     ▼
[Orchestrator]
  - Retry: 1/3
  - Flow: MultiTurn
     │
     ▼
[Generator] (Attempt 2)
  - Context: + validation_feedback
  - Generated: "ls -lhS"
     │
     ▼
[Validation Agent]
  - Parse: ls -lhS
  - Validate: ✓ All valid
  - Result: PASS
     │
     ▼
[Orchestrator]
  - Confidence: 0.85
  - Action: Present to user
     │
     ▼
Output: "ls -lhS"
Time: ~3.5s
```

---

#### Flow 3: Clarification Required

```
User Input: "clean up disk space"
     │
     ▼
[Orchestrator]
  - Ambiguity: 0.82 (high)
  - Flow: Clarification
     │
     ▼
[Clarification Agent]
  - Analysis: Unclear location, criteria, action
  - Questions:
    1. Location? (home/specific/system)
    2. Criteria? (size/age/duplicates)
    3. Action? (list/trash/delete)
     │
     ▼
[User Interaction]
  - Answers: "home, size >100MB, list only"
     │
     ▼
[Clarification Agent]
  - Enhanced: "Find files >100MB in home directory"
     │
     ▼
[Generator Agent]
  - Context: + clarifications
  - Generated: "find ~ -type f -size +100M -ls"
     │
     ▼
[Validation Agent]
  - Validate: ✓ PASS
     │
     ▼
[Orchestrator]
  - Confidence: 0.92
  - Action: Present to user
     │
     ▼
Output: "find ~ -type f -size +100M -ls"
Time: ~5s + user interaction
```

---

#### Flow 4: Prompt Escalation

```
User Input: "complex system query"
     │
     ▼
[Generator] (Attempt 1: base-bsd.toml)
  - Generated: <command1>
     │
     ▼
[Validation] → FAIL
     │
     ▼
[Generator] (Attempt 2: base-bsd.toml + feedback)
  - Generated: <command2>
     │
     ▼
[Validation] → FAIL
     │
     ▼
[Generator] (Attempt 3: base-bsd.toml + feedback)
  - Generated: <command3>
     │
     ▼
[Validation] → FAIL
     │
     ▼
[Orchestrator]
  - Max retries reached (3/3)
  - Action: Escalate prompt
     │
     ▼
[Generator] (Attempt 4: detailed-bsd.toml)
  - Enhanced template with:
    - Relevant man page excerpts
    - More examples
    - Stricter constraints
  - Generated: <command4>
     │
     ▼
[Validation] → PASS
     │
     ▼
[Orchestrator]
  - Confidence: 0.72
  - Action: Present to user
     │
     ▼
Output: <command4>
Time: ~12s
```

---

## Data Models

### Configuration Schema

```rust
// src/config/mod.rs

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CmdaiConfig {
    pub version: String,
    pub platform: PlatformConfig,
    pub generation: GenerationConfig,
    pub prompts: PromptConfig,
    pub cache: CacheConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformConfig {
    pub os: Platform,
    pub unix_flavor: UnixFlavor,
    pub shell: ShellType,
    pub arch: Architecture,
    pub target_os: Option<Platform>,
    pub target_unix_flavor: Option<UnixFlavor>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum UnixFlavor {
    BSD,
    GNU,
    Auto,  // Detect based on OS
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationConfig {
    pub confidence_threshold: f32,
    pub enable_multi_turn: bool,
    pub enable_clarification: bool,
    pub max_retries: usize,
    pub timeout_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptConfig {
    pub base_template: String,
    pub fallback_templates: Vec<String>,
    pub template_dir: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    pub cache_dir: PathBuf,
    pub man_page_ttl_days: u32,
    pub auto_refresh: bool,
}
```

### Man Page Cache Schema

```rust
// src/cache/man_page_cache.rs

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManPageCache {
    pub version: String,
    pub platform: PlatformInfo,
    pub generated_at: DateTime<Utc>,
    pub tools: HashMap<String, ToolInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformInfo {
    pub os: Platform,
    pub unix_flavor: UnixFlavor,
    pub kernel_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolInfo {
    pub path: PathBuf,
    pub version: String,
    pub flags: HashMap<String, FlagInfo>,
    pub forbidden_flags: Vec<String>,
    pub man_page_hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlagInfo {
    pub description: String,
    pub requires_arg: bool,
    pub arg_type: Option<String>,
    pub aliases: Vec<String>,
}
```

### Validation Result Schema

```rust
// src/agents/validator.rs

#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub confidence: f32,
    pub parsed_command: ParsedCommand,
    pub issues: Vec<ValidationIssue>,
    pub suggestions: Vec<Suggestion>,
}

#[derive(Debug, Clone)]
pub struct ValidationIssue {
    pub severity: IssueSeverity,
    pub tool: String,
    pub flag: Option<String>,
    pub message: String,
    pub position: Option<usize>,
}

#[derive(Debug, Clone, Copy)]
pub enum IssueSeverity {
    Error,      // Command will fail
    Warning,    // May not work as expected
    Info,       // Suggestion for improvement
}

#[derive(Debug, Clone)]
pub struct Suggestion {
    pub original: String,
    pub replacement: String,
    pub reason: String,
    pub confidence: f32,
}
```

---

## Implementation Plan

### Phase 1: Foundation (Week 1-2)

**Deliverables:**
1. Configuration system with auto-detection
2. Man page analyzer with caching
3. Basic validation agent

**Tasks:**
```
tasks/phase1/
├── task-001-config-system.md
├── task-002-platform-detection.md
├── task-003-man-page-parser.md
├── task-004-validation-agent.md
└── task-005-integration-tests.md
```

**Tests:**
- Platform detection on macOS, Linux
- Config file creation and loading
- Man page parsing for 10+ common tools
- Flag validation against cache

---

### Phase 2: Multi-Agent System (Week 3-4)

**Deliverables:**
1. Orchestrator agent with flow selection
2. Generator agent with template system
3. Clarification agent
4. Feedback agent

**Tasks:**
```
tasks/phase2/
├── task-006-orchestrator.md
├── task-007-prompt-templates.md
├── task-008-clarification-agent.md
├── task-009-feedback-agent.md
└── task-010-multi-turn-loop.md
```

**Tests:**
- Single-shot generation flow
- Multi-turn validation loop
- Clarification workflow
- Confidence scoring accuracy

---

### Phase 3: Community Features (Week 5-6)

**Deliverables:**
1. External prompt template system
2. Cross-platform generation support
3. Community template repository
4. Documentation and examples

**Tasks:**
```
tasks/phase3/
├── task-011-template-loader.md
├── task-012-cross-platform.md
├── task-013-template-examples.md
├── task-014-documentation.md
└── task-015-end-to-end-tests.md
```

**Tests:**
- Custom template loading
- Cross-platform validation
- Template inheritance
- Community contributions

---

## Security Considerations

### 1. Man Page Parsing Safety

**Risk:** Malformed man pages could cause parser crashes or injection
**Mitigation:**
- Sandbox man page parsing
- Validate UTF-8 encoding
- Limit memory usage per parse
- Timeout after 5 seconds

### 2. Command Injection Prevention

**Risk:** Generated commands could contain injection vectors
**Mitigation:**
- Validate generated commands before presentation
- Flag dangerous patterns (`;`, `&&`, backticks)
- Require explicit confirmation for destructive operations
- Never execute commands automatically

### 3. Cache Integrity

**Risk:** Corrupted cache could lead to incorrect validations
**Mitigation:**
- Cryptographic hash of man pages
- Validate cache integrity on load
- Auto-rebuild on corruption detection
- Version compatibility checks

### 4. Prompt Injection Attacks

**Risk:** User input could manipulate LLM behavior
**Mitigation:**
- Strict JSON-only output format
- Validate response structure
- Escape special characters in user input
- Rate limit generation attempts

---

## Performance Requirements

| Operation | Target | Measurement |
|-----------|--------|-------------|
| Config load | <50ms | Cold start |
| Man cache load | <100ms | From disk |
| Man cache rebuild | <60s | First run |
| Single-shot generation | <2s | 95th percentile |
| Validation | <100ms | Per command |
| Clarification | <3s | Question generation |
| Multi-turn retry | <5s | Per attempt |
| Total max time | <15s | Including retries |

---

## Testing Strategy

### Unit Tests

**Coverage Target:** 90%

```rust
// src/config/tests.rs
#[test]
fn test_platform_detection() { }

#[test]
fn test_config_serialization() { }

// src/agents/validator.rs
#[test]
fn test_flag_validation() { }

#[test]
fn test_command_parsing() { }
```

### Integration Tests

**Coverage:** End-to-end flows

```rust
// tests/generation_flow.rs
#[tokio::test]
async fn test_single_shot_success() { }

#[tokio::test]
async fn test_multi_turn_validation() { }

#[tokio::test]
async fn test_clarification_flow() { }
```

### Contract Tests

**Coverage:** Agent interfaces

```rust
// tests/contracts/validator_contract.rs
pub trait ValidatorContract {
    fn test_validates_valid_command(&self);
    fn test_rejects_invalid_flags(&self);
    fn test_provides_suggestions(&self);
}
```

### Regression Tests

**Coverage:** Known failure cases

```
tests/regression/
├── tc_001_gnu_flags_on_bsd.rs
├── tc_002_ambiguous_fallback.rs
├── tc_003_platform_mismatch.rs
└── tc_004_pipe_validation.rs
```

---

## Success Criteria

### Launch Criteria

- ✅ All Phase 1-2 tests passing
- ✅ Platform detection accuracy: 100%
- ✅ Validation accuracy: 95%+
- ✅ Single-shot success rate: 85%+
- ✅ Performance targets met
- ✅ Documentation complete

### Post-Launch Metrics (30 days)

- User-reported command failures: <5%
- Average generation time: <3s
- Clarification engagement rate: 20-40%
- Community template contributions: 5+
- Platform compatibility issues: 0

---

## Open Questions

1. **Q:** Should we support custom validation rules per user?
   **A:** Phase 3 feature - allow users to define forbidden patterns

2. **Q:** How to handle tools not in standard PATH?
   **A:** Allow users to configure additional tool directories

3. **Q:** What if man pages are missing?
   **A:** Fallback to `--help` parsing with lower confidence

4. **Q:** Support for shell-specific features (bash vs zsh)?
   **A:** Phase 2 - shell-specific prompt templates

5. **Q:** How to version prompt templates?
   **A:** Semantic versioning with compatibility checks

---

## References

- [Session Analysis](./test_cases.md)
- [POSIX Specification](https://pubs.opengroup.org/onlinepubs/9699919799/)
- [BSD man pages](https://man.freebsd.org/)
- [GNU Coreutils Documentation](https://www.gnu.org/software/coreutils/manual/)
- [Spec-Kit Methodology](https://github.com/github/spec-kit)

---

## Appendix A: Configuration File Example

```toml
# ~/.config/cmdai/config.toml

version = "1.0.0"

[platform]
os = "macos"
unix_flavor = "bsd"
shell = "zsh"
arch = "aarch64"
# target_os = "linux"  # Uncomment for cross-platform

[generation]
confidence_threshold = 0.6
enable_multi_turn = true
enable_clarification = true
max_retries = 3
timeout_seconds = 30

[prompts]
base_template = "base-bsd.toml"
fallback_templates = ["detailed-bsd.toml", "interactive-bsd.toml"]
template_dir = "~/.config/cmdai/prompts"

[cache]
cache_dir = "~/.cache/cmdai"
man_page_ttl_days = 30
auto_refresh = true
```

---

## Appendix B: Prompt Template Examples

See `prompts/` directory:
- `base-bsd.toml` - Default macOS/BSD prompt
- `base-gnu.toml` - Default Linux/GNU prompt
- `detailed-bsd.toml` - Enhanced BSD prompt with man excerpts
- `interactive-clarification.toml` - Question generation template
- `modern-macos.toml` - Example community template

---

**Document Status:** Ready for Review
**Next Steps:** Architecture review → Implementation planning → TDD development
