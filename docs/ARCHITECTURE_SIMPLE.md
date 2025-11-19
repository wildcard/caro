# cmdai Architecture - Simple Explanation

> Breaking down the system into understandable components

## The Big Picture

Think of **cmdai** as a team of specialists working together:

```
┌────────────────────────────────────────────────────────┐
│                    USER REQUEST                        │
│           "delete all temporary files"                 │
└────────────────────────────────────────────────────────┘
                         ↓
┌────────────────────────────────────────────────────────┐
│                  MASTER ORCHESTRATOR                   │
│              (main.rs - CLI entry point)               │
│                                                         │
│  Coordinates all sub-agents to safely process request  │
└────────────────────────────────────────────────────────┘
         ↓              ↓              ↓              ↓
    ┌────────┐    ┌─────────┐   ┌──────────┐   ┌─────────┐
    │Security│    │Backend  │   │Safety    │   │User     │
    │Analyst │    │Engine   │   │Validator │   │Guide    │
    └────────┘    └─────────┘   └──────────┘   └─────────┘
```

## Sub-Agent Breakdown

### 1. **Security Analyst** (Safety Module)

**Role**: "The Cautious Expert"
**Location**: `src/safety/mod.rs`

**What it does**:
```rust
// Takes a command and returns risk assessment
fn analyze_command(cmd: &str) -> RiskAssessment {
    // Check against dangerous patterns
    // Evaluate risk level
    // Provide safety recommendations
}
```

**Responsibilities**:
- ✓ Pattern matching (detects `rm -rf /`, fork bombs, etc.)
- ✓ Risk level assignment (Safe/Moderate/High/Critical)
- ✓ Threat modeling (what could go wrong?)
- ✓ Blocklist enforcement (prevent known-bad patterns)

**Skills**:
- **Threat Modeling**: Identifies potential dangers
- **Rule Engine**: Applies validation rules
- **Risk Assessment**: Quantifies danger levels

**Example**:
```rust
// Input: "rm -rf /"
// Output: RiskLevel::Critical {
//   reason: "System root deletion",
//   blocked: true,
//   explanation: "Would destroy entire filesystem"
// }
```

---

### 2. **Backend Engine** (Inference System)

**Role**: "The AI Brain"
**Location**: `src/backends/`

**What it does**:
```rust
// Converts natural language to commands
async fn generate_command(prompt: &str) -> Result<String> {
    // Send to LLM (MLX, Ollama, or vLLM)
    // Parse JSON response
    // Return generated command
}
```

**Responsibilities**:
- ✓ Natural language understanding
- ✓ Command generation (POSIX-compliant)
- ✓ Multiple backend support (MLX, Ollama, vLLM)
- ✓ Fallback handling (if one backend fails, try another)

**Skills**:
- **Model Inference**: Runs LLM locally or remotely
- **Response Parsing**: Extracts command from JSON
- **Backend Selection**: Chooses best available engine

**Backends**:
```
┌─────────────────────────────────────────┐
│  MLX Backend (Apple Silicon)            │
│  • Fastest (< 2s)                       │
│  • Uses Metal GPU                       │
│  • Local, private                       │
├─────────────────────────────────────────┤
│  Ollama Backend (Cross-platform)        │
│  • Local inference                      │
│  • Model flexibility                    │
│  • Good balance                         │
├─────────────────────────────────────────┤
│  vLLM Backend (Remote/Server)           │
│  • Scalable                             │
│  • Cloud or self-hosted                 │
│  • Highest capacity                     │
└─────────────────────────────────────────┘
```

---

### 3. **Safety Validator** (Command Checker)

**Role**: "The Quality Inspector"
**Location**: `src/safety/` (validation logic)

**What it does**:
```rust
// Validates command before execution
fn validate_command(cmd: &str) -> ValidationResult {
    // Check POSIX compliance
    // Verify path safety
    // Confirm proper quoting
    // Ensure no injection risks
}
```

**Responsibilities**:
- ✓ POSIX compliance checking
- ✓ Path validation (proper quoting, no wildcards in dangerous places)
- ✓ Syntax verification
- ✓ Injection prevention (command injection, shell expansion issues)

**Skills**:
- **Syntax Validation**: Ensures command is well-formed
- **Compliance Checking**: Verifies POSIX standards
- **Injection Detection**: Prevents security vulnerabilities

**Example**:
```rust
// Input: "ls '/path/with spaces/file.txt'"
// Output: Valid (proper quoting)

// Input: "ls /path/with spaces/file.txt"
// Output: Invalid (missing quotes)
```

---

### 4. **User Guide** (Explanation System)

**Role**: "The Friendly Teacher"
**Location**: `src/cli/` (output formatting)

**What it does**:
```rust
// Explains command to user in plain language
fn explain_command(cmd: &Command) -> Explanation {
    // Describe what it does
    // List potential outcomes
    // Highlight risks
    // Provide alternatives if risky
}
```

**Responsibilities**:
- ✓ Plain-language explanations
- ✓ Risk communication (color-coded)
- ✓ Outcome prediction (what will happen)
- ✓ Confirmation prompts (y/N)

**Skills**:
- **Command Explanation**: Translates tech jargon to plain English
- **Risk Communication**: Shows warnings in user-friendly way
- ✓ **Interactive Guidance**: Asks for confirmation appropriately

**Output Format**:
```bash
Generated command:
  find . -name "*.tmp" -delete

What it does:
  Searches current directory for files ending in .tmp and
  permanently deletes them.

Risk Level: MODERATE ⚠️
  • This operation cannot be undone
  • Will delete files in all subdirectories
  • Consider backing up important data first

Execute this command? (y/N)
```

---

### 5. **Configuration Manager** (Settings System)

**Role**: "The Preferences Keeper"
**Location**: `src/config/mod.rs`

**What it does**:
```rust
// Loads and manages user preferences
fn load_config() -> Config {
    // Read from ~/.config/cmdai/config.toml
    // Apply defaults
    // Validate settings
}
```

**Responsibilities**:
- ✓ User preference management
- ✓ Safety level configuration (strict/moderate/permissive)
- ✓ Backend selection
- ✓ Custom pattern definitions

**Configuration Example**:
```toml
[safety]
enabled = true
level = "moderate"  # strict | moderate | permissive
require_confirmation = true
custom_patterns = ["rm -rf *"]

[backend]
primary = "mlx"  # mlx | ollama | vllm
enable_fallback = true

[output]
format = "plain"  # plain | json | yaml
use_color = true
verbose = false
```

---

### 6. **Command Auditor** (Logging System)

**Role**: "The Record Keeper"
**Location**: Future implementation

**What it will do**:
```rust
// Logs all command generation and execution
fn audit_command(cmd: &Command, result: &ExecutionResult) {
    // Record timestamp
    // Log user decision (executed/rejected)
    // Track safety assessments
    // Store for learning
}
```

**Planned Responsibilities**:
- 📅 Command history logging
- 📅 Safety incident tracking
- 📅 Pattern learning (which commands users reject)
- 📅 Community contribution (anonymous usage patterns)

---

## How They Work Together

### Example: User asks "compress all images"

```
1. USER REQUEST enters system
   ↓
2. MASTER ORCHESTRATOR receives request
   ↓
3. BACKEND ENGINE generates command
   Result: "find . -name '*.jpg' -o -name '*.png' | xargs -I {} convert {} {}.compressed.jpg"
   ↓
4. SECURITY ANALYST checks risk
   Result: RiskLevel::Moderate (modifies many files)
   ↓
5. SAFETY VALIDATOR verifies syntax
   Result: Valid (proper quoting, POSIX compliant)
   ↓
6. USER GUIDE explains to user
   "This will compress all JPG and PNG images in current directory.
    Risk: Moderate - creates new files, uses disk space.
    Execute? (y/N)"
   ↓
7. USER CONFIRMS → Command executes
   or
   USER REJECTS → Command discarded
   ↓
8. COMMAND AUDITOR logs decision (future feature)
```

---

## Code Organization

```
src/
├── main.rs                    # Master Orchestrator
│                              # Coordinates all sub-agents
│
├── backends/                  # Backend Engine (Sub-Agent 2)
│   ├── mod.rs                # Trait definition
│   ├── mlx.rs                # Apple Silicon backend
│   ├── ollama.rs             # Ollama backend
│   └── vllm.rs               # vLLM backend
│
├── safety/                    # Security Analyst (Sub-Agent 1)
│   └── mod.rs                # Safety Validator (Sub-Agent 3)
│                              # Pattern matching, risk assessment
│
├── cli/                       # User Guide (Sub-Agent 4)
│   ├── mod.rs                # CLI interface
│   └── output.rs             # Formatting and explanations
│
├── config/                    # Configuration Manager (Sub-Agent 5)
│   └── mod.rs                # Settings management
│
└── execution/                 # Command execution (planned)
    └── mod.rs                # Safe execution wrapper
```

---

## Communication Flow

```rust
// Simplified pseudo-code showing sub-agent interaction

async fn process_user_request(prompt: &str) -> Result<()> {
    // 1. Master receives request
    let request = parse_prompt(prompt);

    // 2. Backend generates command
    let command = backend_engine::generate(request).await?;

    // 3. Security analyzes risk
    let risk = security_analyst::assess(&command);

    // 4. Validator checks safety
    let validation = safety_validator::check(&command)?;

    // 5. Guide explains to user
    let explanation = user_guide::explain(&command, &risk);
    println!("{}", explanation);

    // 6. Get user confirmation
    if user_guide::confirm_execution()? {
        // 7. Execute (with auditing)
        execute_command(&command)?;
        command_auditor::log(&command, ExecutionResult::Success);
    } else {
        command_auditor::log(&command, ExecutionResult::Rejected);
    }

    Ok(())
}
```

---

## Design Principles

### 1. **Separation of Concerns**
Each sub-agent has a single, clear responsibility:
- Backend → Generate commands
- Security → Assess risk
- Validator → Check compliance
- Guide → Explain to user

### 2. **Defense in Depth**
Multiple layers of protection:
```
Backend filters → Security checks → Validation → User confirmation
```

### 3. **Fail-Safe Defaults**
- Unknown patterns → Blocked
- Missing confirmation → No execution
- Invalid syntax → Rejected
- Backend unavailable → Fallback or graceful error

### 4. **User Agency**
Users always have final say:
- See exactly what will run
- Understand the risks
- Explicitly confirm dangerous operations

---

## Future: Community Sub-Agent

**Planned Addition**: Community Moderator

**Role**: "The Collective Wisdom"
**Location**: Future `src/community/`

**What it will do**:
```rust
// Aggregate community ratings and feedback
fn get_community_insight(cmd: &str) -> CommunityRating {
    // Check crowd-sourced safety ratings
    // Get usage statistics
    // Show alternative suggestions
    // Incorporate best practices
}
```

**Skills**:
- Voting and rating system
- Community feedback aggregation
- Pattern contribution
- Best practice recommendations

**Example Output**:
```
Community Insights:
  ★★★★☆ 4.2/5 safety rating (1,234 votes)
  💬 "Consider using -i flag for interactive confirmation"
  ✓ 89% of users preview with dry-run first
  ⚠️ 5% reported unexpected deletions - always backup first
```

---

## Key Takeaways

1. **Modular Design**: Each sub-agent is independent and testable
2. **Clear Interfaces**: Sub-agents communicate through well-defined APIs
3. **Safety First**: Multiple layers validate before execution
4. **User-Centric**: Final decision always with the user
5. **Extensible**: Easy to add new backends or safety rules

## For New Contributors

**Want to help? Pick a sub-agent:**

- 🛡️ **Security Analyst**: Add dangerous pattern detection
- 🧠 **Backend Engine**: Implement new LLM support
- ✅ **Safety Validator**: Improve POSIX compliance checking
- 📖 **User Guide**: Enhance explanations and guidance
- ⚙️ **Config Manager**: Add new configuration options
- 📊 **Future/Community**: Build rating and feedback system

Each sub-agent can be developed and tested independently!

---

**Simple, Safe, Modular** | **Built with Rust** | **Open Source**
