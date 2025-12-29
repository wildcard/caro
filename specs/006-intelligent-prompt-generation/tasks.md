# Implementation Tasks: Intelligent Prompt Generation

**Feature ID:** 006
**Status:** Ready for Implementation
**Methodology:** Test-Driven Development (TDD)
**Estimated Duration:** 6 weeks

---

## Task Organization

Tasks are organized into three phases:
- **Phase 1**: Foundation (Weeks 1-2) - Core infrastructure
- **Phase 2**: Multi-Agent System (Weeks 3-4) - Agent coordination
- **Phase 3**: Community Features (Weeks 5-6) - Extensibility

Each task follows TDD:
1. Write failing tests
2. Implement minimal code to pass
3. Refactor while maintaining green tests

---

## Phase 1: Foundation (Weeks 1-2)

### TASK-001: Platform Detection System

**Priority:** P0 (Critical)
**Estimated Effort:** 3 days
**Dependencies:** None

**Description:**
Implement platform detection logic that auto-detects OS, Unix flavor, shell, and architecture.

**Acceptance Criteria:**
- ✅ Detects macOS → sets unix_flavor=BSD
- ✅ Detects Linux → sets unix_flavor=GNU
- ✅ Detects shell from $SHELL environment variable
- ✅ Detects architecture (aarch64, x86_64)
- ✅ Falls back to sensible defaults if detection fails
- ✅ Test coverage: 100% for detection logic

**Implementation Steps:**

1. **Write Contract Tests** (`tests/contracts/platform_detector.rs`)
   ```rust
   #[test]
   fn test_detects_macos_platform() {
       #[cfg(target_os = "macos")]
       {
           let platform = PlatformDetector::detect();
           assert_eq!(platform.os, Platform::MacOS);
           assert_eq!(platform.unix_flavor, UnixFlavor::BSD);
       }
   }

   #[test]
   fn test_detects_shell_from_env() {
       std::env::set_var("SHELL", "/bin/zsh");
       let platform = PlatformDetector::detect();
       assert_eq!(platform.shell, ShellType::Zsh);
   }

   #[test]
   fn test_handles_unknown_shell_gracefully() {
       std::env::set_var("SHELL", "/bin/unknown");
       let platform = PlatformDetector::detect();
       assert_eq!(platform.shell, ShellType::Sh); // Fallback to POSIX
   }
   ```

2. **Implement Platform Detection** (`src/config/platform_detector.rs`)
   ```rust
   pub struct PlatformDetector;

   impl PlatformDetector {
       pub fn detect() -> PlatformInfo {
           PlatformInfo {
               os: Self::detect_os(),
               unix_flavor: Self::detect_unix_flavor(),
               shell: Self::detect_shell(),
               arch: Self::detect_architecture(),
           }
       }

       fn detect_os() -> Platform {
           #[cfg(target_os = "macos")]
           return Platform::MacOS;

           #[cfg(target_os = "linux")]
           return Platform::Linux;

           #[cfg(target_os = "windows")]
           return Platform::Windows;
       }

       fn detect_unix_flavor() -> UnixFlavor {
           match Self::detect_os() {
               Platform::MacOS => UnixFlavor::BSD,
               Platform::Linux => UnixFlavor::GNU,
               _ => UnixFlavor::Auto,
           }
       }

       fn detect_shell() -> ShellType {
           if let Ok(shell) = std::env::var("SHELL") {
               if shell.contains("zsh") {
                   return ShellType::Zsh;
               } else if shell.contains("bash") {
                   return ShellType::Bash;
               } else if shell.contains("fish") {
                   return ShellType::Fish;
               }
           }
           ShellType::Sh // POSIX fallback
       }

       fn detect_architecture() -> Architecture {
           #[cfg(target_arch = "aarch64")]
           return Architecture::Aarch64;

           #[cfg(target_arch = "x86_64")]
           return Architecture::X86_64;
       }
   }
   ```

3. **Add Integration Tests**
4. **Update documentation**

**Files to Create/Modify:**
- `src/config/platform_detector.rs` (new)
- `tests/contracts/platform_detector.rs` (new)
- `src/models/mod.rs` (add UnixFlavor enum)

---

### TASK-002: Configuration Management System

**Priority:** P0 (Critical)
**Estimated Effort:** 4 days
**Dependencies:** TASK-001

**Description:**
Implement configuration system with auto-generation, loading, saving, and user overrides.

**Acceptance Criteria:**
- ✅ Creates config file on first run with detected platform
- ✅ Loads existing config from `~/.config/cmdai/config.toml`
- ✅ Allows user to override any setting via CLI or config file
- ✅ Validates configuration values
- ✅ Provides sensible defaults for all settings
- ✅ Test coverage: 95%+

**Implementation Steps:**

1. **Write Tests** (`tests/config_manager_tests.rs`)
   ```rust
   #[tokio::test]
   async fn test_creates_config_on_first_run() {
       let temp_dir = tempdir().unwrap();
       std::env::set_var("HOME", temp_dir.path());

       let config = ConfigManager::initialize().await.unwrap();

       // Verify config file created
       let config_path = temp_dir.path().join(".config/cmdai/config.toml");
       assert!(config_path.exists());

       // Verify platform auto-detected
       assert!(!config.get().await.platform.os.to_string().is_empty());
   }

   #[tokio::test]
   async fn test_loads_existing_config() {
       let temp_dir = tempdir().unwrap();
       let config_path = temp_dir.path().join(".config/cmdai/config.toml");

       // Create config file
       std::fs::create_dir_all(config_path.parent().unwrap()).unwrap();
       std::fs::write(&config_path, r#"
           [platform]
           os = "linux"
           unix_flavor = "gnu"
       "#).unwrap();

       std::env::set_var("HOME", temp_dir.path());
       let config = ConfigManager::initialize().await.unwrap();

       assert_eq!(config.get().await.platform.os, Platform::Linux);
   }

   #[tokio::test]
   async fn test_user_override_platform() {
       let mut config = ConfigManager::initialize().await.unwrap();

       config.update(|c| {
           c.platform.target_os = Some(Platform::Linux);
           c.platform.target_unix_flavor = Some(UnixFlavor::GNU);
       }).await.unwrap();

       let updated = config.get().await;
       assert_eq!(updated.platform.target_os, Some(Platform::Linux));
   }
   ```

2. **Implement ConfigManager** (`src/config/manager.rs`)
   ```rust
   pub struct ConfigManager {
       config: Arc<RwLock<CmdaiConfig>>,
       config_path: PathBuf,
   }

   impl ConfigManager {
       pub async fn initialize() -> Result<Self> {
           let config_path = Self::config_file_path()?;

           let config = if config_path.exists() {
               Self::load_from_file(&config_path).await?
           } else {
               let detected = Self::create_default_config();
               Self::save_to_file(&config_path, &detected).await?;
               detected
           };

           Ok(Self {
               config: Arc::new(RwLock::new(config)),
               config_path,
           })
       }

       fn create_default_config() -> CmdaiConfig {
           let platform = PlatformDetector::detect();

           CmdaiConfig {
               version: env!("CARGO_PKG_VERSION").to_string(),
               platform: PlatformConfig {
                   os: platform.os,
                   unix_flavor: platform.unix_flavor,
                   shell: platform.shell,
                   arch: platform.arch,
                   target_os: None,
                   target_unix_flavor: None,
               },
               generation: GenerationConfig::default(),
               prompts: PromptConfig::default(),
               cache: CacheConfig::default(),
           }
       }

       pub async fn get(&self) -> CmdaiConfig {
           self.config.read().await.clone()
       }

       pub async fn update<F>(&self, updater: F) -> Result<()>
       where
           F: FnOnce(&mut CmdaiConfig),
       {
           let mut config = self.config.write().await;
           updater(&mut config);
           config.validate()?;
           Self::save_to_file(&self.config_path, &config).await?;
           Ok(())
       }
   }
   ```

3. **Add CLI commands for config management**
4. **Integration tests**

**Files to Create/Modify:**
- `src/config/manager.rs` (new)
- `src/config/models.rs` (new - CmdaiConfig, PlatformConfig, etc.)
- `tests/config_manager_tests.rs` (new)

---

### TASK-003: Man Page Parser & Analyzer

**Priority:** P0 (Critical)
**Estimated Effort:** 5 days
**Dependencies:** TASK-001, TASK-002

**Description:**
Implement man page parsing system that extracts flag information and creates a validation cache.

**Acceptance Criteria:**
- ✅ Parses man pages for common tools (ls, find, grep, du, etc.)
- ✅ Extracts available flags and descriptions
- ✅ Cross-validates with `--help` output
- ✅ Generates structured cache in `~/.cache/cmdai/man-pages.json`
- ✅ Detects platform-specific flags (BSD vs GNU)
- ✅ Performance: < 60s for initial cache build
- ✅ Test coverage: 90%+

**Implementation Steps:**

1. **Write Tests** (`tests/man_page_analyzer_tests.rs`)
   ```rust
   #[tokio::test]
   async fn test_parses_ls_man_page() {
       let analyzer = ManPageAnalyzer::new_for_testing().await.unwrap();
       let result = analyzer.parse_tool_info("ls").await.unwrap();

       assert_eq!(result.name, "ls");
       assert!(result.flags.contains_key("-l"));
       assert!(result.flags.contains_key("-h"));

       #[cfg(target_os = "macos")]
       {
           assert!(result.flags.contains_key("-S"));
           assert!(result.forbidden_flags.contains(&"--sort".to_string()));
       }
   }

   #[tokio::test]
   async fn test_builds_cache_for_common_tools() {
       let platform = PlatformInfo {
           os: Platform::MacOS,
           unix_flavor: UnixFlavor::BSD,
           ..Default::default()
       };

       let cache = ManPageAnalyzer::build_cache(&platform).await.unwrap();

       assert!(cache.tools.contains_key("ls"));
       assert!(cache.tools.contains_key("find"));
       assert!(cache.tools.contains_key("grep"));
       assert!(cache.tools.contains_key("du"));
   }

   #[test]
   fn test_extracts_flags_from_man_text() {
       let man_text = r#"
           -l      Use a long listing format
           -h      Print sizes in human readable format
           -S      Sort by file size
       "#;

       let flags = ManPageParser::extract_flags(man_text).unwrap();

       assert_eq!(flags.len(), 3);
       assert_eq!(flags["-l"].description, "Use a long listing format");
   }
   ```

2. **Implement Parser** (`src/agents/man_page_parser.rs`)
   ```rust
   pub struct ManPageParser;

   impl ManPageParser {
       pub fn extract_flags(man_text: &str) -> Result<HashMap<String, FlagInfo>> {
           let mut flags = HashMap::new();
           let lines: Vec<&str> = man_text.lines().collect();

           for (i, line) in lines.iter().enumerate() {
               let trimmed = line.trim();

               // Detect flag patterns: -l, --long, etc.
               if let Some(flag) = Self::parse_flag_line(trimmed) {
                   let description = Self::extract_description(trimmed);

                   flags.insert(flag.clone(), FlagInfo {
                       description,
                       requires_arg: Self::requires_argument(trimmed),
                       arg_type: Self::detect_arg_type(trimmed),
                       aliases: vec![],
                   });
               }
           }

           Ok(flags)
       }

       fn parse_flag_line(line: &str) -> Option<String> {
           // Regex to match: -l, --long, -h, etc.
           let re = Regex::new(r"^\s*(-\w+|--[\w-]+)").unwrap();

           re.captures(line)
               .and_then(|cap| cap.get(1))
               .map(|m| m.as_str().to_string())
       }

       fn detect_forbidden_flags(man_text: &str, platform: &PlatformInfo) -> Vec<String> {
           match platform.unix_flavor {
               UnixFlavor::BSD => {
                   // GNU-specific long options not available in BSD
                   vec![
                       "--sort".to_string(),
                       "--color".to_string(),
                       "--human-readable".to_string(),
                   ]
               }
               UnixFlavor::GNU => vec![],
               _ => vec![],
           }
       }
   }
   ```

3. **Implement Analyzer** (`src/agents/man_page_analyzer.rs`)
   ```rust
   pub struct ManPageAnalyzer {
       cache: Arc<ManPageCache>,
       cache_path: PathBuf,
       platform: PlatformInfo,
   }

   impl ManPageAnalyzer {
       pub async fn new(platform: PlatformInfo) -> Result<Self> {
           let cache_path = Self::cache_path()?;

           let cache = if Self::cache_is_valid(&cache_path)? {
               Self::load_cache(&cache_path).await?
           } else {
               let cache = Self::build_cache(&platform).await?;
               Self::save_cache(&cache_path, &cache).await?;
               cache
           };

           Ok(Self {
               cache: Arc::new(cache),
               cache_path,
               platform,
           })
       }

       async fn build_cache(platform: &PlatformInfo) -> Result<ManPageCache> {
           tracing::info!("Building man page cache for {:?}", platform);

           let mut cache = ManPageCache {
               version: "1.0.0".to_string(),
               platform: platform.clone(),
               generated_at: Utc::now(),
               tools: HashMap::new(),
           };

           let tools = Self::discover_tools()?;

           // Parse in parallel
           let mut tasks = Vec::new();
           for tool_path in tools {
               tasks.push(tokio::spawn(Self::parse_tool_info_static(
                   tool_path,
                   platform.clone(),
               )));
           }

           for task in tasks {
               if let Ok(Ok(tool_info)) = task.await {
                   cache.tools.insert(tool_info.name.clone(), tool_info);
               }
           }

           tracing::info!("Cache built with {} tools", cache.tools.len());

           Ok(cache)
       }

       fn discover_tools() -> Result<Vec<PathBuf>> {
           let common_tools = vec![
               "ls", "find", "grep", "du", "df", "sort", "head", "tail",
               "cat", "cut", "awk", "sed", "xargs", "tar", "gzip",
           ];

           let mut paths = Vec::new();

           for tool in common_tools {
               if let Ok(output) = std::process::Command::new("which")
                   .arg(tool)
                   .output()
               {
                   if output.status.success() {
                       if let Ok(path_str) = String::from_utf8(output.stdout) {
                           paths.push(PathBuf::from(path_str.trim()));
                       }
                   }
               }
           }

           Ok(paths)
       }
   }
   ```

4. **Add benchmarks for cache performance**
5. **Integration tests**

**Files to Create/Modify:**
- `src/agents/man_page_parser.rs` (new)
- `src/agents/man_page_analyzer.rs` (new)
- `src/cache/man_page_cache.rs` (new - data structures)
- `tests/man_page_analyzer_tests.rs` (new)
- `benches/cache_performance.rs` (new)

---

### TASK-004: Validation Agent

**Priority:** P0 (Critical)
**Estimated Effort:** 4 days
**Dependencies:** TASK-003

**Description:**
Implement validation agent that validates generated commands against man page cache and safety rules.

**Acceptance Criteria:**
- ✅ Parses commands into tools, flags, and arguments
- ✅ Validates each flag against man page cache
- ✅ Detects platform-incompatible flags
- ✅ Provides specific suggestions for corrections
- ✅ Handles piped commands correctly
- ✅ Performance: < 100ms validation time
- ✅ Test coverage: 95%+

**Implementation Steps:**

1. **Write Tests** (`tests/validation_agent_tests.rs`)
   ```rust
   #[test]
   fn test_validates_correct_command() {
       let man_analyzer = create_test_man_analyzer();
       let validator = ValidationAgent::new(Arc::new(man_analyzer));

       let result = validator.validate("ls -lhS").unwrap();

       assert!(result.is_valid);
       assert_eq!(result.issues.len(), 0);
       assert!(result.confidence > 0.9);
   }

   #[test]
   fn test_rejects_invalid_flag() {
       let man_analyzer = create_test_man_analyzer();
       let validator = ValidationAgent::new(Arc::new(man_analyzer));

       let result = validator.validate("ls --sort=size").unwrap();

       assert!(!result.is_valid);
       assert_eq!(result.issues.len(), 1);
       assert_eq!(result.issues[0].flag, Some("--sort".to_string()));
       assert!(!result.suggestions.is_empty());
       assert_eq!(result.suggestions[0].replacement, "-S");
   }

   #[test]
   fn test_validates_piped_command() {
       let man_analyzer = create_test_man_analyzer();
       let validator = ValidationAgent::new(Arc::new(man_analyzer));

       let result = validator.validate("find . -name '*.txt' | grep foo").unwrap();

       assert!(result.is_valid);
       assert_eq!(result.parsed_command.tools.len(), 2);
       assert_eq!(result.parsed_command.pipes.len(), 1);
   }
   ```

2. **Implement Command Parser** (`src/agents/command_parser.rs`)
   ```rust
   pub struct CommandParser;

   impl CommandParser {
       pub fn parse(command: &str) -> Result<ParsedCommand> {
           let mut tools = Vec::new();
           let mut pipes = Vec::new();

           // Split by pipes
           let segments: Vec<&str> = command.split('|').map(|s| s.trim()).collect();

           for (i, segment) in segments.iter().enumerate() {
               let tokens = shell_words::split(segment)?;

               if tokens.is_empty() {
                   continue;
               }

               let tool_invocation = Self::parse_tool_invocation(&tokens)?;
               tools.push(tool_invocation);

               if i < segments.len() - 1 {
                   pipes.push(PipeInfo {
                       from_index: i,
                       to_index: i + 1,
                   });
               }
           }

           Ok(ParsedCommand {
               tools,
               pipes,
               redirects: vec![],
           })
       }

       fn parse_tool_invocation(tokens: &[String]) -> Result<ToolInvocation> {
           let tool_name = tokens[0].clone();
           let mut flags = Vec::new();
           let mut args = Vec::new();

           for token in &tokens[1..] {
               if token.starts_with('-') {
                   flags.push(token.clone());
               } else {
                   args.push(token.clone());
               }
           }

           Ok(ToolInvocation {
               name: tool_name,
               flags,
               args,
           })
       }
   }
   ```

3. **Implement Validator** (`src/agents/validator.rs`)
   ```rust
   pub struct ValidationAgent {
       man_analyzer: Arc<ManPageAnalyzer>,
   }

   impl ValidationAgent {
       pub fn validate(&self, command: &str) -> Result<ValidationResult> {
           let parsed = CommandParser::parse(command)?;
           let mut issues = Vec::new();
           let mut suggestions = Vec::new();

           for tool in &parsed.tools {
               self.validate_tool(&tool, &mut issues, &mut suggestions)?;
           }

           let is_valid = issues.iter().all(|i| i.severity != IssueSeverity::Error);
           let confidence = self.calculate_confidence(&issues);

           Ok(ValidationResult {
               is_valid,
               confidence,
               parsed_command: parsed,
               issues,
               suggestions,
           })
       }

       fn validate_tool(
           &self,
           tool: &ToolInvocation,
           issues: &mut Vec<ValidationIssue>,
           suggestions: &mut Vec<Suggestion>,
       ) -> Result<()> {
           if let Some(tool_info) = self.man_analyzer.get_tool_info(&tool.name) {
               for flag in &tool.flags {
                   if tool_info.forbidden_flags.contains(flag) {
                       issues.push(ValidationIssue {
                           severity: IssueSeverity::Error,
                           tool: tool.name.clone(),
                           flag: Some(flag.clone()),
                           message: format!("{} not available on {:?}", flag, self.man_analyzer.platform().unix_flavor),
                           position: None,
                       });

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
                           tool: tool.name.clone(),
                           flag: Some(flag.clone()),
                           message: format!("Unknown flag: {}", flag),
                           position: None,
                       });
                   }
               }
           } else {
               issues.push(ValidationIssue {
                   severity: IssueSeverity::Error,
                   tool: tool.name.clone(),
                   flag: None,
                   message: format!("Tool not found: {}", tool.name),
                   position: None,
               });
           }

           Ok(())
       }

       fn calculate_confidence(&self, issues: &[ValidationIssue]) -> f32 {
           if issues.is_empty() {
               return 1.0;
           }

           let error_count = issues.iter().filter(|i| i.severity == IssueSeverity::Error).count();
           let warning_count = issues.iter().filter(|i| i.severity == IssueSeverity::Warning).count();

           let score = 1.0 - (error_count as f32 * 0.3) - (warning_count as f32 * 0.1);
           score.max(0.0)
       }
   }
   ```

4. **Add performance benchmarks**
5. **Integration tests**

**Files to Create/Modify:**
- `src/agents/command_parser.rs` (new)
- `src/agents/validator.rs` (new)
- `tests/validation_agent_tests.rs` (new)
- `benches/validation_performance.rs` (new)

---

### TASK-005: Phase 1 Integration Tests

**Priority:** P0 (Critical)
**Estimated Effort:** 2 days
**Dependencies:** TASK-001, TASK-002, TASK-003, TASK-004

**Description:**
Create comprehensive end-to-end integration tests for Phase 1 components.

**Acceptance Criteria:**
- ✅ Tests platform detection → config creation → cache build flow
- ✅ Tests validation against real man pages
- ✅ Tests error recovery scenarios
- ✅ Performance benchmarks for critical paths
- ✅ All Phase 1 tests passing on macOS and Linux

**Implementation Steps:**

1. **End-to-End Integration Tests** (`tests/integration/phase1_flow.rs`)
2. **Performance Benchmarks** (`benches/phase1_benchmarks.rs`)
3. **Cross-Platform CI Setup** (`.github/workflows/phase1-ci.yml`)

**Files to Create/Modify:**
- `tests/integration/phase1_flow.rs` (new)
- `benches/phase1_benchmarks.rs` (new)
- `.github/workflows/phase1-ci.yml` (new)

---

## Phase 2: Multi-Agent System (Weeks 3-4)

### TASK-006: Orchestrator Agent

**Priority:** P0 (Critical)
**Estimated Effort:** 5 days
**Dependencies:** TASK-004

**Description:**
Implement orchestrator agent that coordinates generation, validation, and clarification flows.

**Acceptance Criteria:**
- ✅ Detects ambiguity in user requests
- ✅ Selects appropriate agent flow (single-shot, multi-turn, clarification)
- ✅ Calculates confidence scores
- ✅ Manages retry logic with maximum attempts
- ✅ Escalates to detailed prompts when needed
- ✅ Test coverage: 90%+

**Implementation Steps:**

1. **Write Tests** (`tests/orchestrator_tests.rs`)
2. **Implement Ambiguity Detection**
3. **Implement Flow Selection**
4. **Implement Confidence Scoring**
5. **Integration tests**

**Files to Create/Modify:**
- `src/agents/orchestrator.rs` (new)
- `tests/orchestrator_tests.rs` (new)

---

### TASK-007: Prompt Template System

**Priority:** P0 (Critical)
**Estimated Effort:** 4 days
**Dependencies:** TASK-002

**Description:**
Implement external prompt template system with variable substitution and community extensibility.

**Acceptance Criteria:**
- ✅ Templates stored as TOML files
- ✅ Variable substitution using Handlebars
- ✅ Template inheritance (parent templates)
- ✅ Platform-specific templates (BSD vs GNU)
- ✅ Custom user templates supported
- ✅ Test coverage: 95%+

**Implementation Steps:**

1. **Write Tests** (`tests/prompt_loader_tests.rs`)
2. **Create Base Templates** (`prompts/base-bsd.toml`, `prompts/base-gnu.toml`)
3. **Implement Template Loader**
4. **Implement Variable Substitution**

**Files to Create/Modify:**
- `src/prompts/loader.rs` (new)
- `src/prompts/templates/base-bsd.toml` (new)
- `src/prompts/templates/base-gnu.toml` (new)
- `tests/prompt_loader_tests.rs` (new)

---

### TASK-008: Clarification Agent

**Priority:** P1 (High)
**Estimated Effort:** 4 days
**Dependencies:** TASK-006

**Description:**
Implement clarification agent that generates questions for ambiguous requests.

**Acceptance Criteria:**
- ✅ Generates 2-4 specific questions for ambiguous input
- ✅ Parses user responses
- ✅ Enhances prompt with clarifications
- ✅ Interactive CLI for question/answer flow
- ✅ Test coverage: 90%+

**Implementation Steps:**

1. **Write Tests**
2. **Implement Question Generation**
3. **Implement Response Parsing**
4. **Implement Interactive CLI**

**Files to Create/Modify:**
- `src/agents/clarification.rs` (new)
- `tests/clarification_agent_tests.rs` (new)

---

### TASK-009: Feedback Agent

**Priority:** P1 (High)
**Estimated Effort:** 3 days
**Dependencies:** TASK-004

**Description:**
Implement feedback agent that analyzes validation failures and generates actionable feedback.

**Acceptance Criteria:**
- ✅ Generates specific feedback from validation issues
- ✅ Provides alternative approaches
- ✅ Tracks retry history
- ✅ Suggests prompt escalation when appropriate
- ✅ Test coverage: 90%+

**Implementation Steps:**

1. **Write Tests**
2. **Implement Feedback Generation**
3. **Implement Suggestion System**

**Files to Create/Modify:**
- `src/agents/feedback.rs` (new)
- `tests/feedback_agent_tests.rs` (new)

---

### TASK-010: Multi-Turn Generation Loop

**Priority:** P0 (Critical)
**Estimated Effort:** 5 days
**Dependencies:** TASK-006, TASK-008, TASK-009

**Description:**
Implement multi-turn generation loop with validation, feedback, and retry logic.

**Acceptance Criteria:**
- ✅ Single-shot generation as default
- ✅ Automatic retry on validation failure (max 3 attempts)
- ✅ Feedback incorporated into retry attempts
- ✅ Prompt escalation after max retries
- ✅ Timeout handling (30s per attempt)
- ✅ Test coverage: 95%+

**Implementation Steps:**

1. **Write End-to-End Tests** (`tests/multi_turn_flow.rs`)
2. **Implement Single-Shot Flow**
3. **Implement Multi-Turn Loop**
4. **Implement Clarification Flow**
5. **Implement Interactive Flow (Clarification + Multi-Turn)**

**Files to Create/Modify:**
- `tests/multi_turn_flow.rs` (new)
- Update `src/agents/orchestrator.rs`

---

## Phase 3: Community Features (Weeks 5-6)

### TASK-011: External Template Loader

**Priority:** P2 (Medium)
**Estimated Effort:** 3 days
**Dependencies:** TASK-007

**Description:**
Enable loading custom prompt templates from user config directory.

**Acceptance Criteria:**
- ✅ Templates loaded from `~/.config/cmdai/prompts/`
- ✅ User can set active template via config
- ✅ Validation of custom templates
- ✅ Fallback to default on template error

**Files to Create/Modify:**
- Update `src/prompts/loader.rs`
- `tests/custom_template_tests.rs` (new)

---

### TASK-012: Cross-Platform Generation

**Priority:** P2 (Medium)
**Estimated Effort:** 4 days
**Dependencies:** TASK-006

**Description:**
Support generating commands for a different platform than the current one.

**Acceptance Criteria:**
- ✅ User can set target platform different from current
- ✅ System warns when platforms differ
- ✅ Commands validated against target platform
- ✅ Provides testing instructions

**Files to Create/Modify:**
- Update `src/config/models.rs`
- Update `src/agents/orchestrator.rs`
- `tests/cross_platform_tests.rs` (new)

---

### TASK-013: Community Template Examples

**Priority:** P3 (Low)
**Estimated Effort:** 2 days
**Dependencies:** TASK-011

**Description:**
Create example community templates with documentation.

**Deliverables:**
- `prompts/examples/modern-macos.toml`
- `prompts/examples/detailed-bsd.toml`
- `prompts/examples/interactive-clarification.toml`
- `docs/PROMPT_TEMPLATES.md`

---

### TASK-014: Documentation

**Priority:** P2 (Medium)
**Estimated Effort:** 3 days
**Dependencies:** All previous tasks

**Description:**
Comprehensive documentation for the feature.

**Deliverables:**
- User guide for configuration
- Template authoring guide
- Troubleshooting guide
- API documentation

---

### TASK-015: End-to-End Testing

**Priority:** P0 (Critical)
**Estimated Effort:** 4 days
**Dependencies:** All previous tasks

**Description:**
Comprehensive end-to-end tests covering all user scenarios.

**Test Scenarios:**
- TC-001: Platform-specific command syntax
- TC-002: Inconsistent fallback behavior
- TC-003: Platform detection & configuration
- TC-004: Tool availability validation
- TC-005: Ambiguity detection & clarification
- TC-006: Multi-turn agent flow
- TC-007: Prompt template system
- TC-008: Man page analysis agent
- TC-009: Confidence scoring system
- TC-010: Cross-platform generation

**Files to Create/Modify:**
- `tests/e2e/` (all test case files)

---

## Task Dependencies Graph

```
TASK-001: Platform Detection
    │
    ├──▶ TASK-002: Config Management
    │       │
    │       └──▶ TASK-007: Prompt Templates
    │               │
    │               └──▶ TASK-011: External Templates
    │
    └──▶ TASK-003: Man Page Analyzer
            │
            └──▶ TASK-004: Validation Agent
                    │
                    ├──▶ TASK-005: Phase 1 Integration Tests
                    │
                    ├──▶ TASK-006: Orchestrator
                    │       │
                    │       ├──▶ TASK-008: Clarification Agent
                    │       │
                    │       └──▶ TASK-012: Cross-Platform
                    │
                    ├──▶ TASK-009: Feedback Agent
                    │
                    └──▶ TASK-010: Multi-Turn Loop
                            │
                            ├──▶ TASK-013: Template Examples
                            │
                            ├──▶ TASK-014: Documentation
                            │
                            └──▶ TASK-015: End-to-End Tests
```

---

## Testing Checklist

### Unit Tests
- [ ] Platform detection (TASK-001)
- [ ] Config management (TASK-002)
- [ ] Man page parsing (TASK-003)
- [ ] Command validation (TASK-004)
- [ ] Orchestrator logic (TASK-006)
- [ ] Prompt templates (TASK-007)
- [ ] Clarification questions (TASK-008)
- [ ] Feedback generation (TASK-009)

### Integration Tests
- [ ] Phase 1 integration (TASK-005)
- [ ] Multi-turn flow (TASK-010)
- [ ] Cross-platform (TASK-012)

### End-to-End Tests
- [ ] All 10 test cases from test_cases.md (TASK-015)

### Performance Tests
- [ ] Cache build time < 60s
- [ ] Validation time < 100ms
- [ ] Single-shot generation < 2s
- [ ] Multi-turn generation < 5s per attempt

---

## Acceptance Criteria for Phase Completion

### Phase 1 Complete When:
- [ ] All TASK-001 through TASK-005 tests passing
- [ ] Platform detection working on macOS and Linux
- [ ] Man page cache built successfully
- [ ] Validation agent detecting platform-specific issues
- [ ] Performance benchmarks met

### Phase 2 Complete When:
- [ ] All TASK-006 through TASK-010 tests passing
- [ ] Single-shot generation working
- [ ] Multi-turn retry logic functional
- [ ] Clarification flow interactive
- [ ] Confidence scoring accurate

### Phase 3 Complete When:
- [ ] All TASK-011 through TASK-015 tests passing
- [ ] Custom templates loadable
- [ ] Cross-platform generation working
- [ ] Documentation complete
- [ ] All E2E test scenarios passing

---

## Definition of Done

A task is considered "done" when:

1. ✅ All tests written (TDD - tests first)
2. ✅ Implementation complete and tests passing
3. ✅ Code reviewed and approved
4. ✅ Documentation updated
5. ✅ Integration tests passing
6. ✅ Performance benchmarks met
7. ✅ No regressions in existing tests

---

**Status:** Ready for Implementation
**Next Step:** Begin TASK-001 (Platform Detection System)
