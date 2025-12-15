# Exploration Agent Specification

## Overview

A progressive, asynchronous exploration system that provides fast initial results while optionally discovering better alternatives in the background.

---

## Design Principles

1. **Fast by Default**: Simple queries get immediate responses
2. **Progressive Enhancement**: Complex queries get initial result + background exploration
3. **User Control**: Exploration is opt-in, not forced
4. **Transparent**: User sees what's happening (exploration status)
5. **Non-blocking**: Exploration runs async, user can execute immediately

---

## User Flow

### Simple Query (Fast Path)
```
$ cmdai "list files"
🤖 Analyzing query...
✓ Simple query detected

Command:
  ls -lh

Execute? [y/n/e for explore]:
```

### Complex Query (Enhanced Path)
```
$ cmdai "show top CPU processes" --explore
🤖 Analyzing query...
⚡ Complex query detected - running exploration...

Command (initial):
  ps aux | sort -nrk 3,3 | head -5

🔍 Exploring alternatives in background...

Execute now? [y/n/w for wait]:
> w

📚 Exploration complete! Found 3 options:

  1. ps aux | sort -nrk 3,3 | head -5      [Current]
     ✓ BSD-compatible, direct approach
     
  2. top -l 1 -o cpu -n 5 | tail -6
     ✓ Native macOS tool, formatted output
     
  3. ps -Ao pid,pcpu,comm -r | head -6
     ✓ Minimal output, fastest

Select command [1-3] or Enter to use current:
```

### Exploration with File Context
```
$ cmdai "find large files" --explore --explore-files

🤖 Analyzing query...
📂 Checking if file context needed...
✓ File-related query detected

Current directory: /Users/kobi/project
Files: src/ target/ Cargo.toml README.md (120 files)

🔍 Exploring options...

Options:
  1. du -sh * | sort -hr | head -10
  2. find . -type f -size +10M -exec ls -lh {} \;
  3. gdu .  [requires: brew install gdu]

Select [1-3]:
```

---

## CLI Arguments

```bash
# Exploration control
--explore              # Enable exploration (default: OFF)
--no-explore           # Explicitly disable exploration

# File context control  
--explore-files        # Always include ls output
--explore-files=auto   # Ask model if files are relevant (default when --explore)
--explore-files=no     # Never include files (default without --explore)

# Exploration behavior
--explore-depth=N      # Number of alternative commands (default: 3)
--explore-wait         # Wait for exploration before showing prompt (default: false)

# Examples
cmdai "top processes" --explore
cmdai "large files" --explore --explore-files
cmdai "git commits" --explore --explore-depth=5
```

---

## Architecture

### Phase 0: Complexity Assessment (NEW!)

**Purpose**: Decide if exploration is worth the time

**Input**: User prompt + platform context

**Output**:
```rust
struct ComplexityAssessment {
    is_complex: bool,
    confidence: f32,
    reasoning: String,
    estimated_tools: Vec<String>,
}
```

**System Prompt**:
```
Task: Assess query complexity

Query: "{prompt}"
Platform: macOS
Shell: zsh

Is this query simple or complex?
- Simple: Single tool, obvious command
- Complex: Multiple tools, platform-specific, needs alternatives

Format:
{
  "complexity": "simple" | "complex",
  "confidence": 0.9,
  "reasoning": "Multiple ways to achieve this, platform differences",
  "likely_tools": ["ps", "top", "lsof"],
  "quick_command": "ps aux | sort -nrk 3,3"
}
```

**Decision Logic**:
```rust
if !explore_enabled {
    return generate_command_direct(prompt);
}

let assessment = assess_complexity(prompt).await?;

if assessment.is_complex || assessment.confidence < 0.8 {
    // Run exploration path
    return generate_with_exploration(prompt, assessment).await;
} else {
    // Fast path: use quick_command from assessment
    return Ok(GeneratedCommand {
        command: assessment.quick_command,
        exploration: None,
    });
}
```

---

### Phase 1: Tool Discovery

**Purpose**: Identify relevant command-line tools

**Input**: 
- User prompt
- Platform context
- File context (if enabled)

**Output**:
```rust
struct ToolSuggestion {
    tool: String,           // "ps", "top", "lsof"
    relevance: String,      // Why this tool?
    confidence: f32,        // 0.0-1.0
    platform_native: bool,  // Is it standard on this platform?
}
```

**System Prompt**:
```
Task: Identify tools for "{prompt}"

Platform: macOS (BSD commands)
Shell: zsh
CWD: /Users/kobi/project
Available commands: [ps, lsof, top, find, du, ...]

{FILE_CONTEXT}

List 2-4 relevant command-line tools that can solve this.
Prefer platform-native tools.

Format:
{
  "tools": [
    {
      "name": "ps",
      "reason": "Process monitoring, available on all Unix",
      "confidence": 0.95,
      "native": true
    },
    {
      "name": "top", 
      "reason": "Real-time process viewer, macOS optimized",
      "confidence": 0.85,
      "native": true
    }
  ]
}
```

**File Context Decision**:
```rust
async fn should_include_files(&self, prompt: &str) -> bool {
    match self.config.explore_files {
        ExploreFiles::Always => true,
        ExploreFiles::Never => false,
        ExploreFiles::Auto => {
            // Ask model if prompt is file-related
            let response = self.backend.generate_command(&CommandRequest {
                input: format!(
                    "Is this query about files/directories? '{prompt}'\n\
                     Answer: yes/no"
                ),
                ..Default::default()
            }).await?;
            
            response.command.to_lowercase().contains("yes")
        }
    }
}
```

---

### Phase 2: Context Enrichment

**Purpose**: Gather detailed information about suggested tools

**For each tool:**
1. Check if installed (`which <tool>`)
2. Fetch `man <tool> | head -20` (summary)
3. Fetch `<tool> --help` (options)
4. Fetch `tldr <tool>` (examples) if tldr installed
5. If tldr not installed, log recommendation once

**Output**:
```rust
struct ToolContext {
    tool: String,
    installed: bool,
    man_summary: Option<String>,
    help_text: Option<String>,
    tldr_example: Option<String>,
}

struct EnrichmentResult {
    contexts: HashMap<String, ToolContext>,
    tldr_recommended: bool,  // Recommend installing tldr?
}
```

**Implementation**:
```rust
async fn enrich_tool_context(
    &self,
    tools: &[ToolSuggestion]
) -> EnrichmentResult {
    let mut contexts = HashMap::new();
    let mut tldr_available = false;
    
    // Check tldr once
    if Command::new("which").arg("tldr").output().is_ok() {
        tldr_available = true;
    }
    
    for tool in tools {
        let mut ctx = ToolContext {
            tool: tool.tool.clone(),
            installed: self.is_installed(&tool.tool),
            ..Default::default()
        };
        
        if ctx.installed {
            // Parallel fetch with timeout
            let (man, help, tldr) = tokio::join!(
                self.fetch_man_summary(&tool.tool),
                self.fetch_help_text(&tool.tool),
                async {
                    if tldr_available {
                        self.fetch_tldr(&tool.tool).await
                    } else {
                        None
                    }
                }
            );
            
            ctx.man_summary = man;
            ctx.help_text = help;
            ctx.tldr_example = tldr;
        }
        
        contexts.insert(tool.tool.clone(), ctx);
    }
    
    EnrichmentResult {
        contexts,
        tldr_recommended: !tldr_available,
    }
}
```

---

### Phase 3: Multi-Command Generation

**Purpose**: Generate 2-3 concrete command options with explanations

**Input**:
- Original prompt
- Tool suggestions
- Enriched contexts

**Output**:
```rust
struct CommandOption {
    rank: usize,            // 1 = best, 2 = second, etc.
    command: String,        // Actual shell command
    explanation: String,    // Why this approach?
    tools_used: Vec<String>,
    pros: Vec<String>,
    cons: Vec<String>,
    estimated_safety: RiskLevel,
}

struct ExplorationResult {
    initial_command: String,      // From Phase 0
    options: Vec<CommandOption>,  // From Phase 3
    enrichment: EnrichmentResult,
}
```

**System Prompt**:
```
Task: Generate command options for "{prompt}"

Platform: macOS (BSD commands)
Shell: zsh

Available Tools & Context:
---
Tool: ps
Installed: yes
Summary: Display process status
Help: ps [options]
  -A: all processes
  -x: include processes without controlling terminals
  aux: BSD-style options
Example (tldr): ps aux | grep firefox

Tool: top
Installed: yes
Summary: Display sorted process information
Help: top [options]
  -l num: samples to take
  -o key: sort by key (cpu, mem, etc)
  -n num: show num processes
Example (tldr): top -l 1 -o cpu -n 10

Tool: lsof
Installed: yes
Summary: List open files and processes
Help: lsof [options]
  -i: network files
  -p pid: specific process
Example (tldr): lsof -i :8080
---

Generate 2-3 different commands using these tools.
Each should be a complete, working command.
Rank by suitability (1=best).
Include pros/cons for each.

Format:
{
  "options": [
    {
      "rank": 1,
      "cmd": "ps aux | sort -nrk 3,3 | head -5",
      "explanation": "Most straightforward, uses BSD-compatible sort",
      "tools": ["ps", "sort", "head"],
      "pros": ["Fast", "Portable", "No flags needed"],
      "cons": ["Snapshot only, not real-time"]
    },
    {
      "rank": 2,
      "cmd": "top -l 1 -o cpu -n 5 | tail -6",
      "explanation": "Native macOS tool with formatted output",
      "tools": ["top", "tail"],
      "pros": ["Formatted", "Native"],
      "cons": ["macOS-specific syntax"]
    }
  ]
}
```

---

### Phase 4: Async Execution Flow

**Key Requirement**: Don't block user on exploration

```rust
pub async fn execute_with_exploration(
    &self,
    prompt: &str,
    config: ExploreConfig,
) -> Result<ExplorationResult, CliError> {
    
    // Phase 0: Assess complexity (quick, ~1s)
    let assessment = self.assess_complexity(prompt).await?;
    
    if !assessment.is_complex && assessment.confidence > 0.8 {
        // Fast path: return quick command immediately
        println!("🤖 Analyzing query...");
        println!("✓ Simple query detected\n");
        
        return Ok(ExplorationResult {
            initial_command: assessment.quick_command,
            options: vec![],
            enrichment: EnrichmentResult::default(),
        });
    }
    
    // Complex path: run exploration in background
    println!("🤖 Analyzing query...");
    println!("⚡ Complex query detected - running exploration...\n");
    
    // Show initial command immediately (from assessment)
    println!("Command (initial):");
    println!("  {}\n", assessment.quick_command);
    
    // Spawn exploration in background
    println!("🔍 Exploring alternatives in background...\n");
    
    let exploration_handle = tokio::spawn({
        let agent = self.clone();
        let prompt = prompt.to_string();
        let assessment = assessment.clone();
        
        async move {
            // Phase 1: Discover tools (~2s)
            let tools = agent.discover_tools(&prompt, config.include_files).await?;
            
            // Phase 2: Enrich contexts (~2-3s, parallel)
            let enrichment = agent.enrich_tool_context(&tools).await;
            
            // Phase 3: Generate options (~3s)
            let options = agent.generate_command_options(
                &prompt,
                &tools,
                &enrichment.contexts,
                config.depth,
            ).await?;
            
            Ok::<_, GeneratorError>((options, enrichment))
        }
    });
    
    // User can choose: execute now or wait
    if config.wait_for_exploration {
        // Wait for exploration
        let (options, enrichment) = exploration_handle.await??;
        
        Ok(ExplorationResult {
            initial_command: assessment.quick_command,
            options,
            enrichment,
        })
    } else {
        // Check if exploration finished
        tokio::select! {
            _ = tokio::time::sleep(Duration::from_millis(100)) => {
                // Still running, return initial + handle
                Ok(ExplorationResult {
                    initial_command: assessment.quick_command,
                    options: vec![],
                    enrichment: EnrichmentResult::default(),
                    // Store handle for later
                })
            },
            result = exploration_handle => {
                // Finished quickly!
                let (options, enrichment) = result??;
                Ok(ExplorationResult {
                    initial_command: assessment.quick_command,
                    options,
                    enrichment,
                })
            }
        }
    }
}
```

---

### Phase 5: Interactive Selection

**Purpose**: Let user choose from alternatives after execution fails

```rust
async fn execute_with_fallback(
    &self,
    result: &ExplorationResult,
) -> Result<ExecutionResult, CliError> {
    
    // Execute initial command
    let exec_result = self.execute_command(&result.initial_command).await;
    
    if exec_result.is_ok() {
        return exec_result;
    }
    
    // Failed - check if we have alternatives
    if result.options.is_empty() {
        // Exploration not done yet - wait?
        println!("\n⚠️  Command failed.");
        println!("⏳ Waiting for exploration to complete...");
        
        // Wait for exploration handle
        // (stored from Phase 4)
        
        return Err(CliError::ExecutionFailed { ... });
    }
    
    // Show alternatives interactively
    println!("\n⚠️  Command failed. Alternative options:\n");
    
    for (i, option) in result.options.iter().enumerate() {
        println!("  {}. {}", i + 1, option.command);
        println!("     {}", option.explanation);
        
        if !option.pros.is_empty() {
            println!("     ✓ {}", option.pros.join(", "));
        }
        if !option.cons.is_empty() {
            println!("     ⚠ {}", option.cons.join(", "));
        }
        println!();
    }
    
    // Interactive prompt
    print!("Select command [1-{}] or Enter to abort: ", result.options.len());
    io::stdout().flush()?;
    
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    
    if let Ok(choice) = input.trim().parse::<usize>() {
        if choice > 0 && choice <= result.options.len() {
            let selected = &result.options[choice - 1];
            return self.execute_command(&selected.command).await;
        }
    }
    
    Err(CliError::UserAborted)
}
```

---

## Module Structure

```
src/
├── agent/
│   ├── mod.rs                    # Main AgentLoop (existing)
│   ├── exploration.rs            # NEW: ExplorationAgent
│   ├── complexity.rs             # NEW: Complexity assessment
│   └── enrichment.rs             # NEW: Tool context enrichment
├── cli/
│   ├── mod.rs                    # CliApp (existing)
│   ├── args.rs                   # NEW: Exploration args
│   └── interactive.rs            # NEW: Interactive selection
└── context/
    └── mod.rs                    # ExecutionContext (existing)
```

---

## Performance Targets

| Scenario | Time | Notes |
|----------|------|-------|
| Simple query (no explore) | <2s | Current behavior |
| Complex query (initial) | <2s | Show quick command |
| Exploration (background) | 5-8s | Phases 1-3 parallel |
| Total (with explore) | <10s | User can execute at 2s |

---

## Configuration

```toml
# ~/.config/cmdai/config.toml

[exploration]
enabled = false              # Default: off
depth = 3                    # Number of alternatives
wait = false                 # Don't wait for exploration
files = "auto"               # auto | always | never
show_progress = true         # Show exploration status

[exploration.enrichment]
fetch_man = true
fetch_help = true
fetch_tldr = true
recommend_tldr = true        # Recommend if not installed
```

---

## Success Criteria

### Must Have:
- ✅ Fast path: Simple queries <2s (no regression)
- ✅ Exploration runs async (non-blocking)
- ✅ User can execute initial command immediately
- ✅ Interactive alternative selection on failure
- ✅ 6/6 Vancouver demos passing

### Should Have:
- ✅ Complexity assessment accurate (>80%)
- ✅ File context auto-detection works
- ✅ tldr integration with recommendation
- ✅ Progress indicators clear

### Nice to Have:
- ✅ Exploration caching (avoid re-fetching man pages)
- ✅ Parallel context enrichment
- ✅ Smart tool ranking

---

## Testing Strategy

### Unit Tests:
- Complexity assessment accuracy
- Tool discovery parsing
- Context enrichment (mocked commands)
- Interactive selection (mocked stdin)

### Integration Tests:
- Full exploration flow
- Async execution with timeout
- Fallback selection
- File context detection

### E2E Tests:
- All Vancouver demos
- With and without --explore
- With --explore-files
- Failure + alternative selection

---

## Migration Path

### Phase 1: Foundation (2 hours)
- Create exploration.rs module
- Implement ComplexityAssessment
- Add --explore CLI flag
- Wire into existing AgentLoop

### Phase 2: Discovery (2 hours)
- Implement tool discovery
- Add file context detection
- Test with Vancouver demos

### Phase 3: Enrichment (2 hours)
- Implement context enrichment
- Add man/help/tldr fetching
- Parallel execution

### Phase 4: Multi-Generation (1.5 hours)
- Implement multi-command generation
- Parse ranked options
- Test alternatives

### Phase 5: Async + Interactive (1.5 hours)
- Async exploration execution
- Interactive selection UI
- Integration testing

**Total**: ~9 hours to full implementation

---

## Next Steps

1. Create `src/agent/exploration.rs` skeleton
2. Implement `ComplexityAssessment` (Phase 0)
3. Test complexity detection with sample queries
4. Proceed to tool discovery (Phase 1)

Ready to begin implementation?
