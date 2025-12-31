# Skills Extension System Implementation Plan

**Last Updated**: 2025-12-31
**Status**: Draft

## Implementation Phases

### Overview

```
Phase 1 (MVP)     Phase 2          Phase 3          Phase 4
┌───────────┐    ┌───────────┐    ┌───────────┐    ┌───────────┐
│ Manifest  │    │ Git/URL   │    │ Recipes   │    │ WASM      │
│ + Local   │ -> │ Sources   │ -> │ Execution │ -> │ Execution │
│ + Context │    │ + Lockfile│    │ + Testing │    │ + Registry│
└───────────┘    └───────────┘    └───────────┘    └───────────┘
   4 weeks          3 weeks          4 weeks          4 weeks
```

---

## Phase 1: MVP Foundation

**Goal**: Knowledge packs work with local path installation

**Duration**: 4 weeks

### Week 1: Manifest System

**Deliverables**:
- [ ] `src/skills/mod.rs` - Module structure
- [ ] `src/skills/manifest.rs` - TOML manifest parsing
- [ ] `src/skills/error.rs` - Skill-specific errors
- [ ] Unit tests for manifest parsing

**Tasks**:

1. **Create skill module structure**
   ```
   src/skills/
   ├── mod.rs           # Public API exports
   ├── manifest.rs      # Manifest types + parsing
   ├── error.rs         # SkillError enum
   └── tests/
       └── manifest_test.rs
   ```

2. **Implement manifest parsing**
   ```rust
   // src/skills/manifest.rs
   pub fn parse_manifest(path: &Path) -> Result<SkillManifest, SkillError>
   pub fn validate_manifest(manifest: &SkillManifest) -> Vec<ValidationWarning>
   ```

3. **Add manifest validation**
   - Required fields present
   - Version format valid
   - API version compatible
   - Paths exist if referenced

### Week 2: Skill Loader

**Deliverables**:
- [ ] `src/skills/loader.rs` - Load skills from paths
- [ ] `src/skills/cache.rs` - Installed skills cache
- [ ] Skills directory structure (`~/.caro/skills/`)
- [ ] Integration with main CLI

**Tasks**:

1. **Implement skill loader**
   ```rust
   // src/skills/loader.rs
   pub struct SkillLoader {
       skills_dir: PathBuf,
       loaded: HashMap<String, LoadedSkill>,
   }

   impl SkillLoader {
       pub fn load_all(&mut self) -> Result<(), SkillError>
       pub fn load_skill(&mut self, id: &str) -> Result<&LoadedSkill, SkillError>
       pub fn unload_skill(&mut self, id: &str) -> Result<(), SkillError>
   }
   ```

2. **Create skills cache**
   ```rust
   // src/skills/cache.rs
   pub struct SkillsCache {
       cache_file: PathBuf,  // ~/.caro/skills.json
   }

   impl SkillsCache {
       pub fn list_installed(&self) -> Vec<InstalledSkill>
       pub fn add(&mut self, skill: InstalledSkill) -> Result<(), SkillError>
       pub fn remove(&mut self, id: &str) -> Result<(), SkillError>
   }
   ```

3. **Add CLI subcommand skeleton**
   ```rust
   // src/main.rs
   #[derive(Subcommand)]
   enum Commands {
       #[command(subcommand)]
       Skill(SkillCommands),
   }

   #[derive(Subcommand)]
   enum SkillCommands {
       List { ... },
       Add { ... },
       Remove { ... },
       Info { ... },
   }
   ```

### Week 3: Context Injection

**Deliverables**:
- [ ] `src/skills/context.rs` - Context injection
- [ ] Knowledge loading from skill directories
- [ ] Integration with agent loop
- [ ] Topic matching algorithm

**Tasks**:

1. **Implement context injection**
   ```rust
   // src/skills/context.rs
   pub struct ContextInjector {
       loader: Arc<SkillLoader>,
   }

   impl ContextInjector {
       /// Get context augmentation for a given prompt
       pub fn augment_context(
           &self,
           prompt: &str,
           max_tokens: usize,
       ) -> Result<ContextAugmentation, SkillError>

       /// Match prompt against skill topics
       fn match_skills(&self, prompt: &str) -> Vec<&LoadedSkill>

       /// Load and concatenate knowledge files
       fn load_knowledge(&self, skill: &LoadedSkill) -> Result<String, SkillError>
   }
   ```

2. **Integrate with agent loop**
   ```rust
   // src/agent/mod.rs
   impl AgentLoop {
       async fn run(&self, request: &CommandRequest) -> Result<...> {
           // Inject skill context into system prompt
           let augmented = self.context_injector.augment_context(
               &request.prompt,
               self.config.max_skill_tokens,
           )?;

           let system_prompt = format!(
               "{}\n\n{}", // Skill context first, then base prompt
               augmented.system_context,
               self.base_system_prompt,
           );
           // ...
       }
   }
   ```

3. **Implement topic matching**
   - Exact keyword matching
   - Fuzzy matching with threshold
   - Priority-based ordering

### Week 4: Core Skills + Polish

**Deliverables**:
- [ ] `core.shell` skill extracted from current code
- [ ] `core.posix` skill with platform patterns
- [ ] `caro skill list` command working
- [ ] `caro skill add --path` command working
- [ ] `caro skill remove` command working
- [ ] Documentation

**Tasks**:

1. **Create core.shell skill**
   ```
   skills/core.shell/
   ├── skill.toml
   ├── knowledge/
   │   ├── overview.md
   │   └── prompts/
   │       └── context.md  # Extracted from current system prompt
   └── tests/
   ```

2. **Create core.posix skill**
   ```
   skills/core.posix/
   ├── skill.toml
   ├── knowledge/
   │   ├── utilities.md
   │   ├── patterns/
   │   │   ├── find-xargs.md
   │   │   ├── text-processing.md
   │   │   └── macos-specifics.md
   │   └── prompts/
   │       └── posix-rules.md
   └── tests/
   ```

3. **Finalize CLI commands**
   - `caro skill list` - List installed skills
   - `caro skill add --path ./skill` - Install from local path
   - `caro skill remove skill-id` - Remove skill
   - `caro skill info skill-id` - Show skill details

4. **Write documentation**
   - Update CLAUDE.md with skill system info
   - Create skills README
   - Add contribution guide for skills

---

## Phase 2: Distribution Sources

**Goal**: Install skills from git and URL sources, with lockfile

**Duration**: 3 weeks

### Week 5: Git Source

**Deliverables**:
- [ ] `src/skills/resolver.rs` - Source resolution
- [ ] Git clone/fetch support
- [ ] `caro skill add --git <url>` working

**Tasks**:

1. **Implement resolver trait**
   ```rust
   // src/skills/resolver.rs
   #[async_trait]
   pub trait SkillSource: Send + Sync {
       async fn fetch(&self, spec: &SourceSpec) -> Result<PathBuf, SkillError>;
       async fn check_update(&self, spec: &SourceSpec) -> Result<Option<Version>, SkillError>;
   }

   pub struct GitSource {
       cache_dir: PathBuf,
   }

   impl SkillSource for GitSource {
       async fn fetch(&self, spec: &SourceSpec) -> Result<PathBuf, SkillError> {
           // Clone or fetch repository
           // Checkout specific ref
           // Return path to skill directory
       }
   }
   ```

2. **Add git2 or shell git support**
   - Clone repositories
   - Checkout tags/branches/commits
   - Handle authentication (SSH, HTTPS)

3. **Update CLI**
   ```bash
   caro skill add --git https://github.com/caro-skills/cloud-aws --ref v0.3.0
   ```

### Week 6: URL Source + Lockfile

**Deliverables**:
- [ ] Tarball/zip download support
- [ ] `skills.lock` file generation
- [ ] Integrity verification (SHA256)
- [ ] `caro skill update` command

**Tasks**:

1. **Implement URL source**
   ```rust
   // src/skills/resolver.rs
   pub struct UrlSource {
       cache_dir: PathBuf,
       client: reqwest::Client,
   }

   impl SkillSource for UrlSource {
       async fn fetch(&self, spec: &SourceSpec) -> Result<PathBuf, SkillError> {
           // Download tarball/zip
           // Verify checksum
           // Extract to cache
           // Return path
       }
   }
   ```

2. **Implement lockfile**
   ```rust
   // src/skills/lockfile.rs
   pub struct SkillsLock {
       version: u32,
       skills: Vec<LockedSkill>,
   }

   pub struct LockedSkill {
       id: String,
       version: Version,
       source: String,
       integrity: String,
       installed: DateTime<Utc>,
       capabilities: GrantedCapabilities,
   }

   impl SkillsLock {
       pub fn load(path: &Path) -> Result<Self, SkillError>
       pub fn save(&self, path: &Path) -> Result<(), SkillError>
       pub fn verify_integrity(&self, skill: &LoadedSkill) -> Result<bool, SkillError>
   }
   ```

3. **Implement update command**
   ```bash
   caro skill update          # Update all
   caro skill update cloud.aws  # Update specific
   caro skill update --check    # Check only, don't install
   ```

### Week 7: Capabilities + Polish

**Deliverables**:
- [ ] `src/skills/capability.rs` - Capability enforcement
- [ ] Capability prompting on install
- [ ] `caro skill capabilities` command
- [ ] Audit logging for capability use

**Tasks**:

1. **Implement capability enforcer**
   ```rust
   // src/skills/capability.rs
   pub struct CapabilityEnforcer {
       granted: HashMap<String, GrantedCapabilities>,
       audit_log: Option<AuditLog>,
   }

   impl CapabilityEnforcer {
       pub fn check_terminal(&self, skill_id: &str, command: &str) -> CapabilityResult
       pub fn check_filesystem(&self, skill_id: &str, path: &Path, write: bool) -> CapabilityResult
       pub fn check_network(&self, skill_id: &str, url: &Url) -> CapabilityResult
       pub fn grant(&mut self, skill_id: &str, capability: Capability) -> Result<(), SkillError>
       pub fn revoke(&mut self, skill_id: &str, capability: Capability) -> Result<(), SkillError>
   }
   ```

2. **Add installation prompts**
   ```
   Installing cloud.aws v0.3.0...

   This skill requests the following capabilities:

     Terminal execution:
       Allowed commands: aws, eksctl, kubectl
       Blocked commands: rm, dd, mkfs

     Filesystem read:
       ~/.aws, ~/.kube

   Grant these capabilities? [y/N/d(etails)]
   ```

3. **Implement audit logging**
   - Log capability checks to file
   - Include timestamp, skill, capability, result
   - Configurable verbosity

---

## Phase 3: Recipe Execution

**Goal**: Skills can provide declarative workflows

**Duration**: 4 weeks

### Week 8: Recipe Parser

**Deliverables**:
- [ ] `src/skills/recipe.rs` - Recipe types + parsing
- [ ] YAML recipe format parser
- [ ] Recipe validation

**Tasks**:

1. **Implement recipe types**
   ```rust
   // src/skills/recipe.rs
   pub struct Recipe {
       pub id: String,
       pub name: String,
       pub description: String,
       pub triggers: Vec<Trigger>,
       pub parameters: Vec<Parameter>,
       pub preconditions: Vec<Precondition>,
       pub steps: Vec<Step>,
       pub verification: Vec<Verification>,
       pub rollback: Vec<RollbackStep>,
   }

   pub fn parse_recipe(path: &Path) -> Result<Recipe, SkillError>
   pub fn validate_recipe(recipe: &Recipe) -> Vec<ValidationWarning>
   ```

2. **Implement parameter system**
   - Parameter types (string, bool, number, enum)
   - Default values
   - Validation patterns
   - Interactive prompting

3. **Implement precondition checks**
   - Command existence (`command_exists`)
   - Environment variables (`env_set`)
   - File existence (`file_exists`)
   - Custom scripts (`custom`)

### Week 9: Recipe Execution

**Deliverables**:
- [ ] Recipe step execution
- [ ] Confirmation gates
- [ ] Progress display
- [ ] Error handling

**Tasks**:

1. **Implement recipe executor**
   ```rust
   // src/skills/executor.rs
   pub struct RecipeExecutor {
       capability_enforcer: Arc<CapabilityEnforcer>,
       shell: ShellType,
   }

   impl RecipeExecutor {
       pub async fn execute(
           &self,
           recipe: &Recipe,
           params: &HashMap<String, Value>,
       ) -> Result<RecipeResult, SkillError>

       async fn execute_step(&self, step: &Step, params: &HashMap<String, Value>)
           -> Result<StepOutput, SkillError>

       async fn check_preconditions(&self, preconditions: &[Precondition])
           -> Result<(), PreconditionError>

       async fn run_verification(&self, verification: &[Verification], params: &HashMap<String, Value>)
           -> Result<bool, SkillError>

       async fn run_rollback(&self, rollback: &[RollbackStep], params: &HashMap<String, Value>)
           -> Result<(), SkillError>
   }
   ```

2. **Implement confirmation system**
   ```rust
   pub enum ConfirmationLevel {
       Auto,     // Execute without prompting
       Prompt,   // Ask before executing
       Always,   // Always require explicit "yes"
   }

   async fn confirm_step(&self, step: &Step, level: ConfirmationLevel) -> bool
   ```

3. **Add progress display**
   - Step-by-step progress
   - Output capture and display
   - Error highlighting
   - Rollback indication

### Week 10: Recipe Integration

**Deliverables**:
- [ ] Recipe suggestion in prompts
- [ ] `caro recipe run` command
- [ ] Recipe discovery via context

**Tasks**:

1. **Integrate recipes with agent loop**
   ```rust
   // When LLM suggests a recipe, offer to run it
   if let Some(recipe_suggestion) = response.suggested_recipe {
       let recipe = self.find_recipe(&recipe_suggestion)?;
       if confirm_recipe(&recipe) {
           let params = collect_parameters(&recipe)?;
           self.executor.execute(&recipe, &params).await?;
       }
   }
   ```

2. **Add recipe CLI**
   ```bash
   caro recipe list                    # List available recipes
   caro recipe run deploy-app          # Run recipe interactively
   caro recipe run deploy-app --dry-run  # Show steps without executing
   caro recipe show deploy-app         # Show recipe details
   ```

3. **Implement trigger matching**
   - Keyword matching
   - Intent detection
   - Context conditions

### Week 11: Testing + Documentation

**Deliverables**:
- [ ] Recipe testing framework
- [ ] `caro skill test` for recipes
- [ ] Recipe authoring guide
- [ ] Example recipes for first-party skills

**Tasks**:

1. **Implement recipe test runner**
   ```rust
   // src/skills/testing.rs
   pub struct RecipeTestRunner {
       mock_executor: MockExecutor,
   }

   impl RecipeTestRunner {
       pub fn run_tests(&self, test_file: &Path) -> TestResults
   }
   ```

2. **Create test format**
   ```yaml
   # tests/recipe_test.yaml
   test_cases:
     - name: "Deploy with valid inputs"
       recipe: deploy-app
       parameters:
         cluster_name: "test"
       mock_commands:
         - command: "aws eks *"
           exit_code: 0
       expected_result: success
   ```

3. **Write documentation**
   - Recipe format reference
   - Step-by-step authoring guide
   - Example recipes with comments

---

## Phase 4: WASM Execution

**Goal**: Skills can include executable modules

**Duration**: 4 weeks

### Week 12: WASM Runtime

**Deliverables**:
- [ ] wasmtime integration
- [ ] WASM module loading
- [ ] Host function bindings

**Tasks**:

1. **Add wasmtime dependency**
   ```toml
   # Cargo.toml
   [dependencies]
   wasmtime = { version = "15", optional = true }

   [features]
   wasm-skills = ["wasmtime"]
   ```

2. **Implement WASM loader**
   ```rust
   // src/skills/wasm.rs
   pub struct WasmRuntime {
       engine: Engine,
       linker: Linker<SkillState>,
   }

   impl WasmRuntime {
       pub fn load_module(&self, path: &Path) -> Result<WasmSkill, SkillError>

       fn setup_host_functions(&mut self) -> Result<(), SkillError> {
           // Add capability-gated host functions
           self.linker.func_wrap("caro", "execute_command", ...)?;
           self.linker.func_wrap("caro", "read_file", ...)?;
           self.linker.func_wrap("caro", "http_get", ...)?;
       }
   }
   ```

3. **Define WASM interface**
   ```wit
   // skill.wit
   package caro:skill;

   interface skill {
     record tool-info {
       name: string,
       path: string,
       version: option<string>,
     }

     discover-tools: func(env: string) -> list<tool-info>;
     parse-output: func(command: string, output: string) -> string;
     plan-workflow: func(intent: string, context: string) -> string;
   }
   ```

### Week 13: Host Functions

**Deliverables**:
- [ ] Capability-gated host functions
- [ ] Terminal execution from WASM
- [ ] Filesystem access from WASM
- [ ] Network access from WASM

**Tasks**:

1. **Implement host functions with capability checks**
   ```rust
   // Called by WASM modules, enforces capabilities
   fn host_execute_command(
       caller: Caller<'_, SkillState>,
       cmd_ptr: i32,
       cmd_len: i32,
   ) -> Result<i32, Trap> {
       let skill_id = caller.data().skill_id;
       let command = read_string_from_wasm(&caller, cmd_ptr, cmd_len)?;

       // Check capability
       if !caller.data().capabilities.can_execute(&command) {
           return Err(Trap::new("capability denied: terminal_exec"));
       }

       // Execute and return result
       let output = execute_command(&command)?;
       write_string_to_wasm(&caller, output)
   }
   ```

2. **Add rate limiting**
   - Limit calls per second
   - Limit total execution time
   - Memory limits

3. **Implement sandboxed I/O**
   - Filesystem access only to granted paths
   - Network access only to granted domains
   - No raw socket access

### Week 14: Registry + OCI

**Deliverables**:
- [ ] Skill registry client
- [ ] OCI distribution support
- [ ] `caro skill search` command
- [ ] Signature verification

**Tasks**:

1. **Implement registry client**
   ```rust
   // src/skills/registry.rs
   pub struct RegistryClient {
       base_url: Url,
       client: reqwest::Client,
   }

   impl RegistryClient {
       pub async fn search(&self, query: &str) -> Result<Vec<SkillInfo>, SkillError>
       pub async fn get_versions(&self, id: &str) -> Result<Vec<Version>, SkillError>
       pub async fn download(&self, id: &str, version: &Version) -> Result<PathBuf, SkillError>
   }
   ```

2. **Implement OCI support**
   ```rust
   // src/skills/oci.rs
   pub struct OciSource {
       client: oci_distribution::Client,
   }

   impl SkillSource for OciSource {
       async fn fetch(&self, spec: &SourceSpec) -> Result<PathBuf, SkillError> {
           // Pull OCI artifact
           // Verify signature if present
           // Extract layers
       }
   }
   ```

3. **Add signature verification**
   - GPG signature support
   - cosign/sigstore integration (optional)
   - Configurable trust policy

### Week 15: Polish + Launch

**Deliverables**:
- [ ] Performance optimization
- [ ] Error message improvements
- [ ] Comprehensive documentation
- [ ] Migration guide from current system

**Tasks**:

1. **Performance optimization**
   - Lazy skill loading
   - Context caching
   - Parallel skill loading
   - WASM module precompilation

2. **Error improvements**
   - Helpful error messages
   - Suggestion for common issues
   - Debug mode with verbose output

3. **Documentation**
   - Complete API reference
   - Skill authoring guide
   - Migration guide
   - Troubleshooting guide

4. **Launch preparation**
   - Blog post draft
   - Changelog entry
   - Release notes
   - Example skills ready

---

## Success Criteria

### Phase 1 (MVP)
- [ ] Can install skill from local path
- [ ] Context injection works
- [ ] core.shell and core.posix bundled
- [ ] <10% startup overhead

### Phase 2 (Distribution)
- [ ] Git and URL sources work
- [ ] Lockfile prevents unexpected changes
- [ ] Capabilities are enforced
- [ ] Audit trail exists

### Phase 3 (Recipes)
- [ ] Recipes can be parsed and validated
- [ ] Recipe execution with confirmations
- [ ] Verification and rollback work
- [ ] Testing framework exists

### Phase 4 (WASM)
- [ ] WASM modules load and execute
- [ ] Sandbox prevents escapes
- [ ] Registry search works
- [ ] Signatures verified

---

## Risk Mitigation

| Risk | Mitigation |
|------|------------|
| WASM complexity | Start simple, add features incrementally |
| Performance regression | Benchmark at each phase |
| Security vulnerabilities | Security review before each phase |
| Community adoption | Ship good first-party skills |
| Scope creep | Strict phase boundaries |

---

## Dependencies

### New Crate Dependencies

| Phase | Crate | Purpose |
|-------|-------|---------|
| 1 | None | Use existing deps |
| 2 | git2 or shell | Git operations |
| 2 | sha2 | Already present |
| 3 | None | Use existing deps |
| 4 | wasmtime | WASM runtime |
| 4 | oci-distribution | OCI artifacts |
