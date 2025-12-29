# Architecture: Intelligent Prompt Generation System

**Version:** 1.0.0
**Last Updated:** 2025-11-28
**Status:** Draft

---

## Table of Contents

1. [System Overview](#system-overview)
2. [Component Architecture](#component-architecture)
3. [Data Flow](#data-flow)
4. [Agent Communication Protocol](#agent-communication-protocol)
5. [State Management](#state-management)
6. [Error Handling Strategy](#error-handling-strategy)
7. [Performance Optimization](#performance-optimization)
8. [Deployment Architecture](#deployment-architecture)

---

## System Overview

### High-Level Architecture

```
┌────────────────────────────────────────────────────────────────────────┐
│                             User Layer                                  │
│                  (CLI Interface, Terminal I/O)                          │
└───────────────────────────┬────────────────────────────────────────────┘
                            │
                            │ CommandRequest
                            ▼
┌────────────────────────────────────────────────────────────────────────┐
│                        Application Layer                                │
│                                                                         │
│  ┌─────────────────────────────────────────────────────────────────┐  │
│  │                     Orchestrator Agent                          │  │
│  │  - Request routing                                              │  │
│  │  - Agent coordination                                           │  │
│  │  - Flow management                                              │  │
│  │  - Confidence scoring                                           │  │
│  └─────────────────────────────────────────────────────────────────┘  │
│                            │                                            │
│                            │ AgentMessage                               │
│                            ▼                                            │
│  ┌──────────┬─────────────┬─────────────┬────────────┬─────────────┐  │
│  │Generator │Clarification│ Validation  │  Feedback  │  Analyzer   │  │
│  │  Agent   │   Agent     │   Agent     │   Agent    │   Agent     │  │
│  └──────────┴─────────────┴─────────────┴────────────┴─────────────┘  │
└────────────────────────────────────────────────────────────────────────┘
                            │
                            │ BackendRequest
                            ▼
┌────────────────────────────────────────────────────────────────────────┐
│                       Infrastructure Layer                              │
│                                                                         │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐                 │
│  │Configuration │  │  Man Page    │  │   Prompt     │                 │
│  │   Manager    │  │   Cache      │  │   Loader     │                 │
│  └──────────────┘  └──────────────┘  └──────────────┘                 │
│                                                                         │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐                 │
│  │   Backend    │  │   Backend    │  │   Backend    │                 │
│  │     MLX      │  │    Ollama    │  │     vLLM     │                 │
│  └──────────────┘  └──────────────┘  └──────────────┘                 │
└────────────────────────────────────────────────────────────────────────┘
```

### Design Principles

1. **Separation of Concerns**: Each agent has a single, well-defined responsibility
2. **Async-First**: All I/O operations are asynchronous for performance
3. **Fail-Safe**: Graceful degradation when components fail
4. **Extensibility**: New agents can be added without modifying core
5. **Testability**: Each component can be tested in isolation
6. **Observable**: Comprehensive logging and tracing throughout

---

## Component Architecture

### 1. Configuration Manager

**Module:** `src/config/manager.rs`

```rust
pub struct ConfigManager {
    config: Arc<RwLock<CmdaiConfig>>,
    config_path: PathBuf,
    watcher: Option<ConfigWatcher>,
}

impl ConfigManager {
    /// Initialize configuration (auto-detect or load existing)
    pub async fn initialize() -> Result<Self> {
        // 1. Check if config exists
        // 2. If not, run platform detection
        // 3. Create default config
        // 4. Save to disk
        // 5. Set up file watcher for hot-reload
    }

    /// Detect current platform characteristics
    pub fn detect_platform() -> PlatformInfo {
        PlatformInfo {
            os: Self::detect_os(),
            unix_flavor: Self::detect_unix_flavor(),
            shell: Self::detect_shell(),
            arch: Self::detect_architecture(),
        }
    }

    /// Get current configuration (thread-safe)
    pub async fn get(&self) -> CmdaiConfig {
        self.config.read().await.clone()
    }

    /// Update configuration and persist
    pub async fn update<F>(&self, updater: F) -> Result<()>
    where
        F: FnOnce(&mut CmdaiConfig),
    {
        let mut config = self.config.write().await;
        updater(&mut config);
        self.save(&config).await?;
        Ok(())
    }
}
```

**Dependencies:**
- `tokio::sync::RwLock` for thread-safe access
- `notify` for file watching
- `serde` for serialization

**State:**
- Shared via `Arc<RwLock<T>>` for multi-threaded access
- Persisted to `~/.config/cmdai/config.toml`

---

### 2. Man Page Analyzer

**Module:** `src/agents/man_page_analyzer.rs`

```rust
pub struct ManPageAnalyzer {
    cache: Arc<ManPageCache>,
    cache_path: PathBuf,
    platform: PlatformInfo,
}

impl ManPageAnalyzer {
    /// Initialize or load man page cache
    pub async fn new(platform: PlatformInfo) -> Result<Self> {
        let cache_path = Self::cache_path()?;

        let cache = if cache_path.exists() {
            Self::load_cache(&cache_path).await?
        } else {
            Self::build_cache(&platform).await?
        };

        Ok(Self {
            cache: Arc::new(cache),
            cache_path,
            platform,
        })
    }

    /// Build cache by scanning system tools
    async fn build_cache(platform: &PlatformInfo) -> Result<ManPageCache> {
        let mut cache = ManPageCache::new(platform.clone());

        // Scan common tool locations
        let tool_paths = Self::discover_tools()?;

        // Parse man pages in parallel
        let mut tasks = Vec::new();
        for tool_path in tool_paths {
            tasks.push(tokio::spawn(Self::parse_tool_info(tool_path)));
        }

        for task in tasks {
            if let Ok(Ok(tool_info)) = task.await {
                cache.tools.insert(tool_info.name.clone(), tool_info);
            }
        }

        Ok(cache)
    }

    /// Parse man page for a tool
    async fn parse_tool_info(tool_path: PathBuf) -> Result<ToolInfo> {
        let tool_name = tool_path.file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| anyhow!("Invalid tool path"))?;

        // Read man page
        let man_output = Command::new("man")
            .arg(tool_name)
            .output()
            .await?;

        let man_text = String::from_utf8_lossy(&man_output.stdout);

        // Parse flags from man page
        let flags = Self::extract_flags(&man_text)?;

        // Cross-validate with --help
        let help_output = Command::new(&tool_path)
            .arg("--help")
            .output()
            .await;

        if let Ok(output) = help_output {
            let help_text = String::from_utf8_lossy(&output.stdout);
            Self::validate_against_help(&flags, &help_text);
        }

        Ok(ToolInfo {
            name: tool_name.to_string(),
            path: tool_path,
            version: Self::get_version(&tool_path).await?,
            flags,
            forbidden_flags: Self::detect_forbidden_flags(&man_text),
            man_page_hash: Self::hash_man_page(&man_text),
        })
    }

    /// Validate command against cache
    pub fn validate_command(&self, command: &str) -> ValidationResult {
        let parsed = self.parse_command(command);
        let mut issues = Vec::new();
        let mut suggestions = Vec::new();

        for tool_invocation in &parsed.tools {
            if let Some(tool_info) = self.cache.tools.get(&tool_invocation.name) {
                for flag in &tool_invocation.flags {
                    if tool_info.forbidden_flags.contains(flag) {
                        issues.push(ValidationIssue {
                            severity: IssueSeverity::Error,
                            tool: tool_invocation.name.clone(),
                            flag: Some(flag.clone()),
                            message: format!("{} not available on {}", flag, self.platform.unix_flavor),
                            position: None,
                        });

                        // Find alternative
                        if let Some(alt) = self.find_alternative(tool_info, flag) {
                            suggestions.push(Suggestion {
                                original: flag.clone(),
                                replacement: alt.clone(),
                                reason: "Platform-compatible alternative".to_string(),
                                confidence: 0.9,
                            });
                        }
                    } else if !tool_info.flags.contains_key(flag) {
                        issues.push(ValidationIssue {
                            severity: IssueSeverity::Error,
                            tool: tool_invocation.name.clone(),
                            flag: Some(flag.clone()),
                            message: format!("Unknown flag: {}", flag),
                            position: None,
                        });
                    }
                }
            } else {
                // Tool not in cache - dynamic lookup
                if let Ok(tool_info) = self.dynamic_lookup(&tool_invocation.name) {
                    // Cache for future use
                    let mut cache = self.cache.clone();
                    Arc::get_mut(&mut cache).unwrap()
                        .tools.insert(tool_invocation.name.clone(), tool_info);
                } else {
                    issues.push(ValidationIssue {
                        severity: IssueSeverity::Error,
                        tool: tool_invocation.name.clone(),
                        flag: None,
                        message: format!("Tool not found: {}", tool_invocation.name),
                        position: None,
                    });
                }
            }
        }

        ValidationResult {
            is_valid: issues.is_empty(),
            confidence: self.calculate_validation_confidence(&issues),
            parsed_command: parsed,
            issues,
            suggestions,
        }
    }
}
```

**Async Design:**
- Man page parsing runs in parallel using `tokio::spawn`
- Cache loading is async to avoid blocking startup
- Dynamic lookups use async I/O

**Caching Strategy:**
- In-memory cache wrapped in `Arc` for shared access
- Persistent cache in `~/.cache/cmdai/man-pages.json`
- TTL-based invalidation (30 days default)
- LRU eviction if cache grows too large

---

### 3. Orchestrator Agent

**Module:** `src/agents/orchestrator.rs`

```rust
pub struct Orchestrator {
    config: Arc<ConfigManager>,
    man_analyzer: Arc<ManPageAnalyzer>,
    generator: GeneratorAgent,
    validator: ValidationAgent,
    clarification: ClarificationAgent,
    feedback: FeedbackAgent,
    prompt_loader: PromptLoader,
}

impl Orchestrator {
    pub async fn generate_command(
        &self,
        request: CommandRequest,
    ) -> Result<GeneratedCommand> {
        // Create generation context
        let mut context = GenerationContext {
            platform: self.config.get().await.platform.clone(),
            available_tools: self.man_analyzer.list_tools(),
            user_input: request.input.clone(),
            clarifications: None,
            validation_feedback: None,
        };

        // Detect ambiguity
        let ambiguity = self.detect_ambiguity(&request.input).await?;

        // Decide on flow
        let flow = self.select_flow(ambiguity.score, &request);

        match flow {
            AgentFlow::SingleShot => {
                self.single_shot_generation(&request, &context).await
            }
            AgentFlow::MultiTurn { max_retries } => {
                self.multi_turn_generation(&request, &mut context, max_retries).await
            }
            AgentFlow::Clarification => {
                self.clarification_flow(&request, &mut context, &ambiguity).await
            }
            AgentFlow::Interactive => {
                // Clarification first, then multi-turn
                self.clarification_flow(&request, &mut context, &ambiguity).await?;
                self.multi_turn_generation(&request, &mut context, 3).await
            }
        }
    }

    async fn single_shot_generation(
        &self,
        request: &CommandRequest,
        context: &GenerationContext,
    ) -> Result<GeneratedCommand> {
        // Load prompt template
        let template = self.prompt_loader.load_base_template(&context.platform)?;

        // Generate command
        let command = self.generator.generate(request, &template, context).await?;

        // Validate
        let validation = self.validator.validate(&command)?;

        if validation.is_valid {
            let confidence = self.calculate_confidence(&validation, context);
            Ok(GeneratedCommand {
                command,
                explanation: "Generated using MLX backend".to_string(),
                confidence,
                ..Default::default()
            })
        } else {
            // Validation failed - escalate to multi-turn
            Err(anyhow!("Validation failed, retrying..."))
        }
    }

    async fn multi_turn_generation(
        &self,
        request: &CommandRequest,
        context: &mut GenerationContext,
        max_retries: usize,
    ) -> Result<GeneratedCommand> {
        let mut attempt = 0;
        let mut template = self.prompt_loader.load_base_template(&context.platform)?;

        while attempt < max_retries {
            attempt += 1;
            tracing::info!("Generation attempt {}/{}", attempt, max_retries);

            // Generate
            let command = self.generator.generate(request, &template, context).await?;

            // Validate
            let validation = self.validator.validate(&command)?;

            if validation.is_valid {
                let confidence = self.calculate_confidence(&validation, context);
                return Ok(GeneratedCommand {
                    command,
                    explanation: format!("Generated on attempt {}", attempt),
                    confidence,
                    ..Default::default()
                });
            }

            // Generate feedback
            let feedback_msg = self.feedback.generate_feedback(&validation, attempt)?;
            context.validation_feedback = Some(feedback_msg.to_prompt_context());

            // Check if we should escalate prompt
            if attempt == max_retries - 1 {
                tracing::warn!("Escalating to detailed prompt");
                template = self.prompt_loader.load_fallback_template(&context.platform)?;
            }
        }

        Err(anyhow!("Max retries exceeded"))
    }

    async fn clarification_flow(
        &self,
        request: &CommandRequest,
        context: &mut GenerationContext,
        ambiguity: &AmbiguityScore,
    ) -> Result<()> {
        // Generate questions
        let questions = self.clarification
            .generate_questions(&request.input, ambiguity)
            .await?;

        // Present questions to user
        println!("\n🤔 Clarification needed (ambiguity: {:.2})\n", ambiguity.score);
        let mut answers = HashMap::new();

        for (i, question) in questions.iter().enumerate() {
            println!("{}. {}", i + 1, question.text);
            for (j, option) in question.options.iter().enumerate() {
                println!("   {}) {}", (b'a' + j as u8) as char, option);
            }

            // Read user input
            let answer = read_user_input()?;
            answers.insert(question.id.clone(), answer);
        }

        // Enhance context with clarifications
        context.clarifications = Some(answers);

        Ok(())
    }

    fn detect_ambiguity(&self, input: &str) -> AmbiguityScore {
        let mut score = 0.0;
        let mut factors = Vec::new();

        // Check for vague terms
        let vague_terms = ["clean", "fix", "optimize", "delete", "remove"];
        for term in vague_terms {
            if input.to_lowercase().contains(term) {
                score += 0.2;
                factors.push(format!("Vague term: '{}'", term));
            }
        }

        // Check for missing context
        if !input.contains('/') && !input.contains('~') {
            score += 0.1;
            factors.push("No explicit path specified".to_string());
        }

        // Check for destructive operations without criteria
        let destructive = ["delete", "remove", "rm", "clean"];
        for term in destructive {
            if input.contains(term) && !input.contains("where") && !input.contains("than") {
                score += 0.3;
                factors.push(format!("Destructive operation without criteria: '{}'", term));
            }
        }

        AmbiguityScore {
            score: score.min(1.0),
            factors,
            requires_clarification: score > 0.7,
        }
    }

    fn calculate_confidence(
        &self,
        validation: &ValidationResult,
        context: &GenerationContext,
    ) -> f32 {
        let validation_score = if validation.is_valid { 1.0 } else { 0.0 };
        let ambiguity_score = if context.clarifications.is_some() { 1.0 } else { 0.7 };
        let platform_score = if context.platform.target_os.is_none() { 1.0 } else { 0.8 };
        let safety_score = 0.9; // TODO: Implement safety analysis

        validation_score * 0.4 + ambiguity_score * 0.3 + platform_score * 0.2 + safety_score * 0.1
    }
}
```

**State Machine:**
```
┌──────────┐
│   Init   │
└────┬─────┘
     │
     ▼
┌──────────────┐      ┌─────────────────┐
│  Analyze     │─────▶│  Clarification  │
│  Ambiguity   │      │    Required     │
└────┬─────────┘      └────┬────────────┘
     │                     │
     │ Low Ambiguity       │ Questions Answered
     ▼                     ▼
┌──────────────┐      ┌─────────────────┐
│  Generate    │◀─────│  Enhanced       │
│   Command    │      │   Context       │
└────┬─────────┘      └─────────────────┘
     │
     ▼
┌──────────────┐
│   Validate   │
└────┬─────────┘
     │
     ├─────Valid────────▶ [SUCCESS]
     │
     └─────Invalid──────┐
                        │
                        ▼
                 ┌──────────────┐
                 │   Feedback   │
                 │  & Retry     │
                 └────┬─────────┘
                      │
                      ├─────Retry < Max────▶ [Generate]
                      │
                      └─────Max Reached────▶ [Escalate Prompt]
```

---

### 4. Generator Agent

**Module:** `src/agents/generator.rs`

```rust
pub struct GeneratorAgent {
    backend: Arc<dyn CommandGenerator>,
    prompt_loader: PromptLoader,
}

impl GeneratorAgent {
    pub async fn generate(
        &self,
        request: &CommandRequest,
        template: &PromptTemplate,
        context: &GenerationContext,
    ) -> Result<String> {
        // Render prompt with context
        let rendered_prompt = template.render(context)?;

        // Call backend
        let start = Instant::now();
        let response = self.backend.generate(request, &rendered_prompt).await?;
        let duration = start.elapsed();

        tracing::info!(
            "Generation completed in {:?} using {} backend",
            duration,
            self.backend.name()
        );

        // Parse JSON response
        let command = self.parse_json_response(&response)?;

        Ok(command)
    }

    fn parse_json_response(&self, response: &str) -> Result<String> {
        // Try strict JSON parsing first
        if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(response) {
            if let Some(cmd) = parsed.get("cmd").and_then(|v| v.as_str()) {
                return Ok(cmd.trim().to_string());
            }
        }

        // Fallback: Extract JSON from response
        if let Some(start) = response.find('{') {
            if let Some(end) = response.rfind('}') {
                let json_part = &response[start..=end];
                if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(json_part) {
                    if let Some(cmd) = parsed.get("cmd").and_then(|v| v.as_str()) {
                        return Ok(cmd.trim().to_string());
                    }
                }
            }
        }

        Err(anyhow!("Failed to parse command from response"))
    }
}
```

---

### 5. Validation Agent

**Module:** `src/agents/validator.rs`

```rust
pub struct ValidationAgent {
    man_analyzer: Arc<ManPageAnalyzer>,
    safety_validator: SafetyValidator,
}

impl ValidationAgent {
    pub fn validate(&self, command: &str) -> Result<ValidationResult> {
        // Parse command
        let parsed = self.parse_command(command)?;

        // Validate against man pages
        let man_validation = self.man_analyzer.validate_command(command);

        // Safety validation
        let safety_validation = self.safety_validator.validate(command)?;

        // Merge results
        let mut issues = man_validation.issues;
        issues.extend(safety_validation.issues);

        Ok(ValidationResult {
            is_valid: issues.is_empty(),
            confidence: self.calculate_confidence(&issues),
            parsed_command: parsed,
            issues,
            suggestions: man_validation.suggestions,
        })
    }

    fn parse_command(&self, command: &str) -> Result<ParsedCommand> {
        let mut tools = Vec::new();
        let mut pipes = Vec::new();
        let mut redirects = Vec::new();

        // Split by pipes
        let segments: Vec<&str> = command.split('|').collect();

        for (i, segment) in segments.iter().enumerate() {
            let tokens: Vec<&str> = segment.trim().split_whitespace().collect();

            if tokens.is_empty() {
                continue;
            }

            let tool_name = tokens[0].to_string();
            let mut flags = Vec::new();
            let mut args = Vec::new();

            for token in &tokens[1..] {
                if token.starts_with('-') {
                    flags.push(token.to_string());
                } else if token.starts_with('>') || token.starts_with('<') {
                    redirects.push(RedirectInfo {
                        operator: token.chars().next().unwrap(),
                        target: tokens.get(i + 1).map(|s| s.to_string()),
                    });
                } else {
                    args.push(token.to_string());
                }
            }

            tools.push(ToolInvocation {
                name: tool_name,
                flags,
                args,
            });

            if i < segments.len() - 1 {
                pipes.push(PipeInfo {
                    from_tool: tools[i].name.clone(),
                    to_tool: tokens[0].to_string(),
                });
            }
        }

        Ok(ParsedCommand {
            tools,
            pipes,
            redirects,
        })
    }
}
```

---

### 6. Prompt Template System

**Module:** `src/prompts/loader.rs`

```rust
pub struct PromptLoader {
    template_dir: PathBuf,
    cache: RwLock<HashMap<String, PromptTemplate>>,
}

impl PromptLoader {
    pub fn load_base_template(
        &self,
        platform: &PlatformConfig,
    ) -> Result<PromptTemplate> {
        let template_name = format!("base-{}.toml", platform.unix_flavor);
        self.load_template(&template_name)
    }

    pub fn load_fallback_template(
        &self,
        platform: &PlatformConfig,
    ) -> Result<PromptTemplate> {
        let template_name = format!("detailed-{}.toml", platform.unix_flavor);
        self.load_template(&template_name)
    }

    fn load_template(&self, name: &str) -> Result<PromptTemplate> {
        // Check cache first
        {
            let cache = self.cache.read().unwrap();
            if let Some(template) = cache.get(name) {
                return Ok(template.clone());
            }
        }

        // Load from disk
        let path = self.template_dir.join(name);
        let content = std::fs::read_to_string(&path)?;
        let template: PromptTemplate = toml::from_str(&content)?;

        // Cache it
        {
            let mut cache = self.cache.write().unwrap();
            cache.insert(name.to_string(), template.clone());
        }

        Ok(template)
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct PromptTemplate {
    pub meta: TemplateMeta,
    pub prompt: PromptContent,
    pub examples: HashMap<String, String>,
    pub validation: ValidationRules,
}

impl PromptTemplate {
    pub fn render(&self, context: &GenerationContext) -> Result<String> {
        let mut handlebars = Handlebars::new();

        let template_vars = serde_json::json!({
            "os": context.platform.os.to_string(),
            "unix_flavor": context.platform.unix_flavor.to_string(),
            "shell": context.platform.shell.to_string(),
            "tools": context.available_tools.join(", "),
            "user_input": context.user_input,
            "clarifications": context.clarifications,
            "validation_feedback": context.validation_feedback,
        });

        handlebars.render_template(&self.prompt.system, &template_vars)
            .map_err(|e| anyhow!("Template rendering failed: {}", e))
    }
}
```

---

## Data Flow

### Scenario 1: Successful Single-Shot Generation

```
┌──────┐
│ User │ "list files sorted by size"
└──┬───┘
   │ CommandRequest
   ▼
┌──────────────┐
│ Orchestrator │
└──┬───────────┘
   │
   │ 1. Analyze Ambiguity
   │    ─▶ Score: 0.1 (low)
   │
   │ 2. Select Flow
   │    ─▶ AgentFlow::SingleShot
   │
   │ 3. Load Template
   │    ─▶ base-bsd.toml
   │
   ▼
┌──────────────┐
│  Generator   │
└──┬───────────┘
   │
   │ 4. Render Prompt
   │    Context: {
   │      os: "macos",
   │      unix_flavor: "bsd",
   │      tools: ["ls", "find", ...]
   │    }
   │
   │ 5. Call Backend
   │    ─▶ MLX Inference
   │
   │ 6. Response
   │    ─▶ {"cmd": "ls -lhS"}
   │
   ▼
┌──────────────┐
│  Validator   │
└──┬───────────┘
   │
   │ 7. Parse Command
   │    ─▶ Tool: ls
   │    ─▶ Flags: [-l, -h, -S]
   │
   │ 8. Validate Flags
   │    ─▶ Check against man cache
   │    ─▶ All flags valid ✓
   │
   │ 9. Safety Check
   │    ─▶ RiskLevel::Safe
   │
   ▼
┌──────────────┐
│ Orchestrator │
└──┬───────────┘
   │
   │ 10. Calculate Confidence
   │     ─▶ 0.95
   │
   │ 11. Build Response
   │
   ▼
┌──────┐
│ User │ Command: ls -lhS ✓ Confidence: 0.95
└──────┘

Total Time: ~1.5s
```

### Scenario 2: Multi-Turn with Validation Failure

```
┌──────┐
│ User │ "show files sorted"
└──┬───┘
   │
   ▼
[Orchestrator] ─▶ Flow: SingleShot
   │
   ▼
[Generator] ─▶ "ls -lh --sort=size"
   │
   ▼
[Validator]
   │
   ├─ Parse: ls -lh --sort=size
   │
   ├─ Check Flag: --sort
   │   └─▶ ❌ Not in BSD ls
   │
   └─▶ ValidationResult {
         is_valid: false,
         issues: [
           "Flag '--sort' not available in BSD ls"
         ],
         suggestions: [
           "Use '-S' for size sorting"
         ]
       }
   │
   ▼
[Orchestrator] ─▶ Escalate to MultiTurn
   │
   ▼
[Feedback] ─▶ "BSD ls doesn't support --sort. Use -S flag."
   │
   ▼
[Generator] (Attempt 2)
   │
   ├─ Enhanced Context:
   │   validation_feedback: [
   │     "Use -S instead of --sort"
   │   ]
   │
   └─▶ "ls -lhS"
   │
   ▼
[Validator] ─▶ ✓ All flags valid
   │
   ▼
[Orchestrator] ─▶ Confidence: 0.85
   │
   ▼
┌──────┐
│ User │ Command: ls -lhS ✓
└──────┘

Total Time: ~3.5s
Attempts: 2
```

---

## Agent Communication Protocol

### Message Types

```rust
pub enum AgentMessage {
    // Request messages
    GenerateRequest(GenerateRequest),
    ValidateRequest(ValidateRequest),
    ClarifyRequest(ClarifyRequest),
    FeedbackRequest(FeedbackRequest),

    // Response messages
    GenerateResponse(GenerateResponse),
    ValidateResponse(ValidateResponse),
    ClarifyResponse(ClarifyResponse),
    FeedbackResponse(FeedbackResponse),

    // Control messages
    Abort(AbortReason),
    Timeout,
}

pub struct GenerateRequest {
    pub request_id: Uuid,
    pub command_request: CommandRequest,
    pub template: PromptTemplate,
    pub context: GenerationContext,
    pub timeout: Duration,
}

pub struct GenerateResponse {
    pub request_id: Uuid,
    pub command: String,
    pub raw_response: String,
    pub duration: Duration,
    pub backend_used: String,
}

pub struct ValidateRequest {
    pub request_id: Uuid,
    pub command: String,
    pub platform: PlatformConfig,
}

pub struct ValidateResponse {
    pub request_id: Uuid,
    pub result: ValidationResult,
    pub duration: Duration,
}
```

### Communication Patterns

```rust
// Async channel-based communication
pub struct AgentChannel {
    tx: mpsc::Sender<AgentMessage>,
    rx: mpsc::Receiver<AgentMessage>,
}

// Example: Orchestrator → Generator
let (tx, rx) = mpsc::channel(100);

let generate_msg = AgentMessage::GenerateRequest(GenerateRequest {
    request_id: Uuid::new_v4(),
    command_request,
    template,
    context,
    timeout: Duration::from_secs(30),
});

tx.send(generate_msg).await?;

// Wait for response with timeout
match timeout(Duration::from_secs(30), rx.recv()).await {
    Ok(Some(AgentMessage::GenerateResponse(response))) => {
        // Process response
    }
    Ok(Some(AgentMessage::Timeout)) => {
        // Handle timeout
    }
    _ => {
        // Handle error
    }
}
```

---

## State Management

### Application State

```rust
pub struct AppState {
    config: Arc<ConfigManager>,
    man_analyzer: Arc<ManPageAnalyzer>,
    orchestrator: Arc<Orchestrator>,
    metrics: Arc<MetricsCollector>,
}

impl AppState {
    pub async fn initialize() -> Result<Self> {
        // 1. Load configuration
        let config = Arc::new(ConfigManager::initialize().await?);

        // 2. Initialize man page analyzer
        let platform = config.get().await.platform.clone();
        let man_analyzer = Arc::new(ManPageAnalyzer::new(platform).await?);

        // 3. Create orchestrator
        let orchestrator = Arc::new(Orchestrator::new(
            config.clone(),
            man_analyzer.clone(),
        )?);

        // 4. Initialize metrics
        let metrics = Arc::new(MetricsCollector::new());

        Ok(Self {
            config,
            man_analyzer,
            orchestrator,
            metrics,
        })
    }
}
```

### Request State

```rust
pub struct RequestState {
    pub request_id: Uuid,
    pub started_at: Instant,
    pub attempts: usize,
    pub max_attempts: usize,
    pub current_flow: AgentFlow,
    pub history: Vec<GenerationAttempt>,
}

pub struct GenerationAttempt {
    pub attempt_number: usize,
    pub template_used: String,
    pub command_generated: String,
    pub validation_result: ValidationResult,
    pub duration: Duration,
}
```

---

## Error Handling Strategy

### Error Types

```rust
#[derive(Debug, thiserror::Error)]
pub enum CmdaiError {
    #[error("Configuration error: {message}")]
    ConfigError { message: String },

    #[error("Platform detection failed: {reason}")]
    PlatformDetectionError { reason: String },

    #[error("Man page parsing failed for {tool}: {reason}")]
    ManPageParseError { tool: String, reason: String },

    #[error("Generation failed: {details}")]
    GenerationError { details: String },

    #[error("Validation failed: {issues:?}")]
    ValidationError { issues: Vec<ValidationIssue> },

    #[error("Agent timeout after {duration:?}")]
    TimeoutError { duration: Duration },

    #[error("Max retries exceeded ({attempts}/{max})")]
    MaxRetriesError { attempts: usize, max: usize },

    #[error("Backend unavailable: {backend}")]
    BackendUnavailable { backend: String },
}
```

### Error Recovery

```rust
impl Orchestrator {
    async fn generate_with_recovery(
        &self,
        request: CommandRequest,
    ) -> Result<GeneratedCommand> {
        match self.generate_command(request.clone()).await {
            Ok(cmd) => Ok(cmd),

            Err(CmdaiError::ValidationError { issues }) => {
                // Retry with enhanced context
                self.retry_with_feedback(request, issues).await
            }

            Err(CmdaiError::TimeoutError { .. }) => {
                // Fallback to simpler prompt
                self.fallback_generation(request).await
            }

            Err(CmdaiError::BackendUnavailable { backend }) => {
                // Try alternative backend
                self.try_alternative_backend(request, backend).await
            }

            Err(e) => Err(e),
        }
    }
}
```

---

## Performance Optimization

### 1. Caching Strategy

```rust
// Multi-level cache
pub struct CacheHierarchy {
    // L1: In-memory LRU cache
    l1_cache: Arc<RwLock<LruCache<String, CachedResult>>>,

    // L2: Persistent disk cache
    l2_cache: DiskCache,

    // L3: Man page cache
    man_cache: Arc<ManPageCache>,
}

impl CacheHierarchy {
    async fn get(&self, key: &str) -> Option<CachedResult> {
        // Check L1
        if let Some(result) = self.l1_cache.read().await.peek(key) {
            return Some(result.clone());
        }

        // Check L2
        if let Some(result) = self.l2_cache.get(key).await.ok() {
            // Promote to L1
            self.l1_cache.write().await.put(key.to_string(), result.clone());
            return Some(result);
        }

        None
    }
}
```

### 2. Parallel Processing

```rust
async fn build_cache_parallel(tools: Vec<PathBuf>) -> Result<ManPageCache> {
    let mut tasks = Vec::new();

    for tool in tools {
        tasks.push(tokio::spawn(async move {
            parse_tool_info(tool).await
        }));
    }

    let results = futures::future::join_all(tasks).await;

    // Aggregate results
    let mut cache = ManPageCache::new();
    for result in results {
        if let Ok(Ok(tool_info)) = result {
            cache.tools.insert(tool_info.name.clone(), tool_info);
        }
    }

    Ok(cache)
}
```

### 3. Lazy Loading

```rust
pub struct LazyManAnalyzer {
    cache: OnceCell<Arc<ManPageCache>>,
    platform: PlatformInfo,
}

impl LazyManAnalyzer {
    async fn get_cache(&self) -> Result<Arc<ManPageCache>> {
        self.cache.get_or_try_init(|| async {
            ManPageAnalyzer::build_cache(&self.platform)
                .await
                .map(Arc::new)
        }).await
    }
}
```

---

## Deployment Architecture

### Directory Structure

```
~/.config/cmdai/
├── config.toml                 # User configuration
├── prompts/                    # Prompt templates
│   ├── base-bsd.toml
│   ├── base-gnu.toml
│   ├── detailed-bsd.toml
│   └── custom/                 # User templates
└── logs/
    └── cmdai.log

~/.cache/cmdai/
├── man-pages.json              # Parsed man pages
├── command-cache/              # Generated commands cache
│   └── *.json
└── metrics/
    └── usage.db

/usr/local/bin/
└── cmdai                       # Binary
```

### Configuration Precedence

1. CLI arguments (highest priority)
2. Environment variables (`CMDAI_*`)
3. User config file (`~/.config/cmdai/config.toml`)
4. Default values (lowest priority)

### Logging Architecture

```rust
// Structured logging with tracing
use tracing::{info, warn, error, debug};
use tracing_subscriber::layer::SubscriberExt;

fn init_logging(config: &CmdaiConfig) -> Result<()> {
    let file_appender = tracing_appender::rolling::daily(
        config.cache.cache_dir.join("logs"),
        "cmdai.log",
    );

    let subscriber = tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .with(tracing_subscriber::fmt::layer())
        .with(tracing_subscriber::fmt::layer().with_writer(file_appender));

    tracing::subscriber::set_global_default(subscriber)?;

    Ok(())
}

// Structured logging with context
#[instrument(skip(request))]
async fn generate_command(request: CommandRequest) -> Result<GeneratedCommand> {
    info!(request_id = %request.id, "Starting generation");

    // ... generation logic ...

    info!(
        request_id = %request.id,
        duration_ms = elapsed.as_millis(),
        "Generation completed"
    );

    Ok(command)
}
```

---

## Testing Architecture

### Unit Testing

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tokio::test;

    #[test]
    async fn test_platform_detection() {
        let platform = PlatformDetector::detect();
        assert!(matches!(
            platform.os,
            Platform::MacOS | Platform::Linux
        ));
    }

    #[test]
    fn test_command_parsing() {
        let validator = ValidationAgent::new(/* ... */);
        let parsed = validator.parse_command("ls -la | grep foo").unwrap();

        assert_eq!(parsed.tools.len(), 2);
        assert_eq!(parsed.tools[0].name, "ls");
        assert_eq!(parsed.tools[0].flags, vec!["-la"]);
    }
}
```

### Integration Testing

```rust
#[tokio::test]
async fn test_end_to_end_generation() {
    let app_state = AppState::initialize().await.unwrap();

    let request = CommandRequest::new("list files", ShellType::Bash);

    let result = app_state.orchestrator
        .generate_command(request)
        .await
        .unwrap();

    assert!(!result.command.is_empty());
    assert!(result.confidence > 0.5);
}
```

### Contract Testing

```rust
#[async_trait]
pub trait ValidatorContract {
    async fn test_validates_valid_command(&self);
    async fn test_rejects_invalid_flags(&self);
    async fn test_provides_suggestions(&self);
}

impl ValidatorContract for ValidationAgent {
    async fn test_validates_valid_command(&self) {
        let result = self.validate("ls -la").unwrap();
        assert!(result.is_valid);
    }

    async fn test_rejects_invalid_flags(&self) {
        let result = self.validate("ls --sort=size").unwrap();
        assert!(!result.is_valid);
        assert!(!result.suggestions.is_empty());
    }
}
```

---

## Monitoring & Observability

### Metrics Collection

```rust
pub struct MetricsCollector {
    generation_count: AtomicU64,
    validation_failures: AtomicU64,
    average_latency: AtomicU64,
    confidence_histogram: Mutex<Vec<f32>>,
}

impl MetricsCollector {
    pub fn record_generation(&self, duration: Duration, confidence: f32) {
        self.generation_count.fetch_add(1, Ordering::SeqCst);
        self.average_latency.fetch_add(
            duration.as_millis() as u64,
            Ordering::SeqCst,
        );

        self.confidence_histogram
            .lock()
            .unwrap()
            .push(confidence);
    }

    pub fn summary(&self) -> MetricsSummary {
        let count = self.generation_count.load(Ordering::SeqCst);
        let total_latency = self.average_latency.load(Ordering::SeqCst);

        MetricsSummary {
            total_generations: count,
            average_latency_ms: total_latency / count.max(1),
            validation_failure_rate: self.validation_failures.load(Ordering::SeqCst) as f32 / count as f32,
            // ... more metrics
        }
    }
}
```

---

**Document Status:** Ready for Implementation
**Next Steps:** Phased development following implementation plan
