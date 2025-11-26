# Phase 1: Real Model Inference - Detailed Task Breakdown

## Overview

This document breaks Phase 1 into **bite-sized, 2-4 hour tasks** that can be completed by contributors with varying experience levels. Each task is self-contained and clearly defined.

**Total Estimated Effort**: 80-120 hours (4-6 weeks solo, 2-3 weeks with team)

---

## Decision Point: Which Inference Library?

**Before starting, choose ONE approach:**

### Option A: llama-cpp-rs (RECOMMENDED for MVP)
**Pros**: ✅ Proven, ✅ Fast, ✅ Good GGUF support, ✅ Active maintenance
**Cons**: ⚠️ C++ dependency, ⚠️ Larger binary size
**Effort**: 40-60 hours
**Best For**: Solo developers, quick MVP

### Option B: candle-transformers
**Pros**: ✅ Pure Rust, ✅ Good ecosystem, ✅ Flexible
**Cons**: ⚠️ Learning curve, ⚠️ Less documentation
**Effort**: 60-80 hours
**Best For**: Teams, long-term maintainability

### Option C: Both (with feature flags)
**Pros**: ✅ Best of both worlds
**Cons**: ⚠️ 2x maintenance burden
**Effort**: 100-120 hours
**Best For**: Post-MVP (defer for now)

**Recommendation**: Start with **llama-cpp-rs** for v1.0, add Candle in v1.1.

---

## Task List: llama-cpp-rs Implementation

### Milestone 1: Setup & Proof of Concept (Day 1-3, 12 hours)

#### Task 1.1: Research llama-cpp-rs
**Effort**: 2 hours
**Difficulty**: Easy
**Assignable**: Yes

**Steps**:
1. Read llama-cpp-rs documentation
2. Review example code in repository
3. Check compatibility with Qwen2.5-Coder models
4. Document findings in GitHub Discussion

**Deliverable**: Summary doc of capabilities and limitations

**Acceptance Criteria**:
- [ ] Understand basic API
- [ ] Know how to load GGUF models
- [ ] Identified potential issues

---

#### Task 1.2: Add llama-cpp dependency
**Effort**: 1 hour
**Difficulty**: Easy
**Assignable**: Yes

**Steps**:
1. Add to Cargo.toml:
   ```toml
   [dependencies]
   llama-cpp-2 = "0.1"  # Check latest version
   ```
2. Run `cargo build` and fix any issues
3. Verify compilation on target platforms

**Deliverable**: PR with dependency added

**Acceptance Criteria**:
- [ ] Builds on macOS
- [ ] Builds on Linux
- [ ] No breaking changes to existing code

---

#### Task 1.3: Create basic inference example
**Effort**: 3 hours
**Difficulty**: Medium
**Assignable**: Yes (requires Rust knowledge)

**Steps**:
1. Create `examples/basic_inference.rs`
2. Load a sample GGUF model
3. Generate text from a simple prompt
4. Print output to console

**Code Template**:
```rust
use llama_cpp_2::context::LlamaContext;
use llama_cpp_2::model::LlamaModel;

fn main() -> anyhow::Result<()> {
    // Load model
    let model = LlamaModel::load_from_file(
        "models/qwen2.5-coder-1.5b-q4_k_m.gguf",
        Default::default(),
    )?;

    // Create context
    let mut ctx = model.create_context(Default::default())?;

    // Generate
    let prompt = "Generate a shell command to list files";
    let output = ctx.complete(prompt, Default::default())?;

    println!("{}", output);
    Ok(())
}
```

**Deliverable**: Working example file

**Acceptance Criteria**:
- [ ] Successfully loads model
- [ ] Generates coherent text
- [ ] Completes in <5 seconds
- [ ] Example documented

---

#### Task 1.4: Download test model
**Effort**: 1 hour
**Difficulty**: Easy
**Assignable**: Yes

**Steps**:
1. Download Qwen2.5-Coder-1.5B-Instruct Q4_K_M from Hugging Face
2. Place in `tests/fixtures/models/`
3. Add to .gitignore (too large for git)
4. Document download instructions

**Command**:
```bash
mkdir -p tests/fixtures/models
wget https://huggingface.co/Qwen/Qwen2.5-Coder-1.5B-Instruct-GGUF/resolve/main/qwen2.5-coder-1.5b-instruct-q4_k_m.gguf \
  -O tests/fixtures/models/qwen2.5-coder-1.5b-instruct-q4_k_m.gguf
```

**Deliverable**: Model available for testing

**Acceptance Criteria**:
- [ ] Model downloaded and verified
- [ ] SHA256 checksum matches
- [ ] Git ignores model file
- [ ] README documents how to download

---

#### Task 1.5: Benchmark basic inference
**Effort**: 2 hours
**Difficulty**: Medium
**Assignable**: Yes

**Steps**:
1. Create `benches/inference_speed.rs`
2. Measure model load time
3. Measure first token time
4. Measure tokens per second
5. Compare to targets (<2s inference)

**Deliverable**: Benchmark results and comparison

**Acceptance Criteria**:
- [ ] Benchmark runs successfully
- [ ] Results documented
- [ ] Performance compared to targets
- [ ] Identified bottlenecks if any

---

### Milestone 2: Integration with Existing Backend (Day 4-7, 16 hours)

#### Task 2.1: Create LlamaCppBackend struct
**Effort**: 2 hours
**Difficulty**: Medium
**Assignable**: Yes

**Steps**:
1. Create new file: `src/backends/embedded/llama_cpp.rs`
2. Define backend structure:
   ```rust
   pub struct LlamaCppBackend {
       model: Option<LlamaModel>,
       config: LlamaCppConfig,
   }

   #[derive(Debug, Clone)]
   pub struct LlamaCppConfig {
       pub model_path: PathBuf,
       pub n_ctx: usize,
       pub n_threads: usize,
   }
   ```
3. Add basic constructor and methods

**Deliverable**: Backend skeleton

**Acceptance Criteria**:
- [ ] Struct compiles
- [ ] Basic API defined
- [ ] Follows existing backend patterns

---

#### Task 2.2: Implement CommandGenerator trait
**Effort**: 4 hours
**Difficulty**: Hard
**Assignable**: Yes (requires understanding of async)

**Steps**:
1. Implement `CommandGenerator` trait
2. Handle async inference (use `tokio::task::spawn_blocking`)
3. Parse JSON from model output
4. Handle errors gracefully

**Code Template**:
```rust
#[async_trait]
impl CommandGenerator for LlamaCppBackend {
    async fn generate_command(
        &self,
        request: &CommandRequest,
    ) -> Result<GeneratedCommand, GeneratorError> {
        // Load model if needed
        let model = self.ensure_model_loaded()?;

        // Build prompt
        let prompt = self.build_prompt(request);

        // Run inference in blocking thread
        let output = tokio::task::spawn_blocking(move || {
            model.complete(&prompt, Default::default())
        })
        .await??;

        // Parse JSON
        self.parse_response(&output)
    }

    async fn is_available(&self) -> bool {
        self.config.model_path.exists()
    }

    fn backend_info(&self) -> BackendInfo {
        BackendInfo {
            name: "llama.cpp".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            supports_streaming: false,
        }
    }
}
```

**Deliverable**: Working trait implementation

**Acceptance Criteria**:
- [ ] Trait fully implemented
- [ ] Async handling works
- [ ] No deadlocks or blocking
- [ ] Error handling comprehensive

---

#### Task 2.3: Add lazy model loading
**Effort**: 2 hours
**Difficulty**: Medium
**Assignable**: Yes

**Steps**:
1. Model should load on first inference call
2. Cache loaded model in memory
3. Add unload method for cleanup
4. Handle reload on error

**Pattern**:
```rust
fn ensure_model_loaded(&mut self) -> Result<&LlamaModel> {
    if self.model.is_none() {
        tracing::info!("Loading model from {:?}", self.config.model_path);
        let model = LlamaModel::load_from_file(
            &self.config.model_path,
            Default::default(),
        )?;
        self.model = Some(model);
    }
    Ok(self.model.as_ref().unwrap())
}
```

**Deliverable**: Lazy loading implementation

**Acceptance Criteria**:
- [ ] Model loads on demand
- [ ] Subsequent calls reuse loaded model
- [ ] Memory managed properly
- [ ] Errors handled gracefully

---

#### Task 2.4: Implement prompt building
**Effort**: 3 hours
**Difficulty**: Medium
**Assignable**: Yes

**Steps**:
1. Create system prompt template
2. Format user query
3. Add few-shot examples
4. Handle special tokens properly

**Template**:
```rust
fn build_prompt(&self, request: &CommandRequest) -> String {
    format!(
        r#"<|im_start|>system
You are a helpful assistant that converts natural language to POSIX shell commands.
Output ONLY valid JSON in this format: {{"cmd": "the shell command"}}
Do not include explanations, only the JSON.<|im_end|>
<|im_start|>user
Convert to shell command: {}
Current directory: {}
Operating system: {}
<|im_end|>
<|im_start|>assistant
"#,
        request.query,
        request.context.current_directory.display(),
        request.context.platform
    )
}
```

**Deliverable**: Prompt builder function

**Acceptance Criteria**:
- [ ] Prompt follows model's format
- [ ] Includes necessary context
- [ ] Works with Qwen2.5-Coder
- [ ] Tested with various queries

---

#### Task 2.5: Implement JSON response parsing
**Effort**: 3 hours
**Difficulty**: Medium
**Assignable**: Yes

**Steps**:
1. Extract JSON from model output
2. Handle multiple extraction strategies:
   - Direct JSON parse
   - Extract from markdown code blocks
   - Regex fallback
3. Validate command before returning
4. Clear error messages

**Code Template**:
```rust
fn parse_response(&self, output: &str) -> Result<GeneratedCommand> {
    // Strategy 1: Direct JSON parse
    if let Ok(cmd) = serde_json::from_str::<CommandJson>(output) {
        return Ok(GeneratedCommand {
            command: cmd.cmd,
            explanation: None,
        });
    }

    // Strategy 2: Extract from code block
    if let Some(json) = extract_json_from_markdown(output) {
        if let Ok(cmd) = serde_json::from_str::<CommandJson>(&json) {
            return Ok(GeneratedCommand {
                command: cmd.cmd,
                explanation: None,
            });
        }
    }

    // Strategy 3: Regex extraction
    if let Some(cmd) = extract_with_regex(output) {
        return Ok(GeneratedCommand {
            command: cmd,
            explanation: None,
        });
    }

    Err(GeneratorError::JsonParseError(output.to_string()))
}
```

**Deliverable**: Robust JSON parser

**Acceptance Criteria**:
- [ ] Handles clean JSON
- [ ] Handles markdown-wrapped JSON
- [ ] Handles malformed output gracefully
- [ ] Tested with real model output

---

#### Task 2.6: Wire into CLI
**Effort**: 2 hours
**Difficulty**: Medium
**Assignable**: Yes

**Steps**:
1. Update `src/backends/mod.rs` to include llama_cpp
2. Add backend selection in CLI
3. Update default backend to use LlamaCppBackend
4. Remove mock backend from production builds

**Changes**:
```rust
// In src/backends/mod.rs
#[cfg(feature = "llama-cpp")]
pub mod llama_cpp;

// In src/main.rs
let backend: Arc<dyn CommandGenerator> = {
    #[cfg(feature = "llama-cpp")]
    {
        Arc::new(LlamaCppBackend::new(config)?)
    }
    #[cfg(not(feature = "llama-cpp"))]
    {
        compile_error!("No backend enabled")
    }
};
```

**Deliverable**: Integrated backend

**Acceptance Criteria**:
- [ ] CLI uses real backend
- [ ] Feature flags work correctly
- [ ] Compilation clean
- [ ] End-to-end flow works

---

### Milestone 3: Testing & Refinement (Day 8-12, 20 hours)

#### Task 3.1: Write unit tests
**Effort**: 4 hours
**Difficulty**: Medium
**Assignable**: Yes

**Tests to Write**:
```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_prompt_building() {
        // Test prompt format
    }

    #[test]
    fn test_json_extraction() {
        // Test various JSON formats
    }

    #[test]
    fn test_lazy_loading() {
        // Test model loads on demand
    }

    #[test]
    fn test_error_handling() {
        // Test invalid model path
    }
}
```

**Deliverable**: Comprehensive test suite

**Acceptance Criteria**:
- [ ] >80% code coverage
- [ ] All edge cases covered
- [ ] Tests pass consistently
- [ ] Fast test execution

---

#### Task 3.2: Integration tests with real model
**Effort**: 4 hours
**Difficulty**: Hard
**Assignable**: Yes (requires model download)

**Tests**:
```rust
#[tokio::test]
async fn test_real_inference() {
    let backend = LlamaCppBackend::new(test_config()).unwrap();
    let request = CommandRequest {
        query: "list all files".to_string(),
        context: ExecutionContext::current(),
    };

    let result = backend.generate_command(&request).await;

    assert!(result.is_ok());
    let cmd = result.unwrap();
    assert!(cmd.command.contains("ls") || cmd.command.contains("find"));
}
```

**Deliverable**: Integration test suite

**Acceptance Criteria**:
- [ ] Tests with real model
- [ ] Covers common queries
- [ ] Tests safety integration
- [ ] CI can run tests (model caching)

---

#### Task 3.3: Performance optimization
**Effort**: 6 hours
**Difficulty**: Hard
**Assignable**: No (requires profiling expertise)

**Areas to Optimize**:
1. Model loading time
2. Context size (n_ctx parameter)
3. Thread count (n_threads parameter)
4. Generation parameters (temperature, top_p)
5. Prompt length reduction

**Tools**:
```bash
cargo flamegraph --example basic_inference
cargo bench
```

**Deliverable**: Performance improvements

**Acceptance Criteria**:
- [ ] Meets <2s inference target (or document why not)
- [ ] Memory usage <4GB
- [ ] Startup time <100ms
- [ ] Profiling results documented

---

#### Task 3.4: Error handling polish
**Effort**: 3 hours
**Difficulty**: Medium
**Assignable**: Yes

**Error Cases to Handle**:
- Model file not found
- Model file corrupted
- Inference timeout
- Out of memory
- Invalid JSON response
- Network issues (for download)

**Pattern**:
```rust
match backend.generate_command(&request).await {
    Ok(cmd) => Ok(cmd),
    Err(GeneratorError::ModelNotFound(path)) => {
        eprintln!("Model not found at: {}", path);
        eprintln!("Run: cmdai download-model");
        Err(...)
    }
    Err(GeneratorError::Timeout) => {
        eprintln!("Inference timed out after 30s");
        eprintln!("Try a simpler query or increase timeout");
        Err(...)
    }
    // ... more cases
}
```

**Deliverable**: User-friendly error messages

**Acceptance Criteria**:
- [ ] All error paths tested
- [ ] Clear, actionable error messages
- [ ] Includes suggestions for fixes
- [ ] No panics in production

---

#### Task 3.5: Documentation
**Effort**: 3 hours
**Difficulty**: Easy
**Assignable**: Yes

**Documentation Needed**:
1. API docs for LlamaCppBackend
2. Usage examples
3. Configuration guide
4. Troubleshooting section
5. Performance tuning tips

**Deliverable**: Complete documentation

**Acceptance Criteria**:
- [ ] All public APIs documented
- [ ] Examples compile and run
- [ ] README updated
- [ ] cargo doc builds successfully

---

### Milestone 4: Model Download System (Day 13-16, 16 hours)

#### Task 4.1: Implement Hugging Face download
**Effort**: 4 hours
**Difficulty**: Medium
**Assignable**: Yes

**Use hf-hub crate**:
```rust
use hf_hub::api::sync::Api;

pub fn download_model(repo_id: &str, filename: &str) -> Result<PathBuf> {
    let api = Api::new()?;
    let repo = api.model(repo_id.to_string());
    let path = repo.get(filename)?;
    Ok(path)
}
```

**Deliverable**: Model download function

**Acceptance Criteria**:
- [ ] Downloads from Hugging Face
- [ ] Caches in correct directory
- [ ] Verifies integrity
- [ ] Handles network errors

---

#### Task 4.2: Add progress indicators
**Effort**: 3 hours
**Difficulty**: Medium
**Assignable**: Yes

**Use indicatif crate**:
```rust
use indicatif::{ProgressBar, ProgressStyle};

let pb = ProgressBar::new(total_size);
pb.set_style(
    ProgressStyle::default_bar()
        .template("[{elapsed_precise}] {bar:40.cyan/blue} {bytes}/{total_bytes} {msg}")
        .unwrap()
);
```

**Deliverable**: Download progress UI

**Acceptance Criteria**:
- [ ] Shows download progress
- [ ] Shows speed (MB/s)
- [ ] Shows ETA
- [ ] Clean terminal output

---

#### Task 4.3: First-run model download
**Effort**: 4 hours
**Difficulty**: Medium
**Assignable**: Yes

**Steps**:
1. Detect if model missing on first run
2. Prompt user to download
3. Show download progress
4. Verify download completed
5. Proceed to inference

**UI Flow**:
```
$ cmdai "list files"

Model not found. Would you like to download it? (1.1 GB) [Y/n]
> y

Downloading Qwen2.5-Coder-1.5B-Instruct...
[00:02:34] ████████████████████ 1.1GB/1.1GB (7.8 MB/s)

Model downloaded successfully!
Generating command...
> ls -la
```

**Deliverable**: First-run experience

**Acceptance Criteria**:
- [ ] User prompted on first run
- [ ] Can decline download
- [ ] Progress shown clearly
- [ ] Continues after download

---

#### Task 4.4: Offline mode handling
**Effort**: 2 hours
**Difficulty**: Easy
**Assignable**: Yes

**Steps**:
1. Detect if model already cached
2. Work offline if model exists
3. Clear error if model needed but no network
4. Document offline usage

**Deliverable**: Offline support

**Acceptance Criteria**:
- [ ] Works offline with cached model
- [ ] Clear message if model needed
- [ ] Doesn't hang on network timeout
- [ ] Documented in README

---

#### Task 4.5: Update CLI for download command
**Effort**: 3 hours
**Difficulty**: Easy
**Assignable**: Yes

**Add subcommand**:
```bash
cmdai download-model [--force]
cmdai model-info
cmdai clear-cache
```

**Implementation**:
```rust
#[derive(Parser)]
enum Commands {
    DownloadModel {
        #[arg(long)]
        force: bool,
    },
    ModelInfo,
    ClearCache,
}
```

**Deliverable**: Model management commands

**Acceptance Criteria**:
- [ ] Commands work as expected
- [ ] Help text clear
- [ ] Integrated with existing CLI
- [ ] Tested manually

---

### Milestone 5: Polish & Release (Day 17-20, 16 hours)

#### Task 5.1: End-to-end testing
**Effort**: 4 hours
**Difficulty**: Medium
**Assignable**: Yes

**Test Scenarios**:
```
1. Fresh install → download model → generate command → execute
2. Cached model → fast startup → generate command
3. Network failure → graceful error
4. Corrupted model → redownload
5. Safety validation → dangerous command blocked
6. Various query types → all succeed
```

**Deliverable**: E2E test suite

**Acceptance Criteria**:
- [ ] All scenarios pass
- [ ] No regressions
- [ ] Performance acceptable
- [ ] User experience smooth

---

#### Task 5.2: Benchmark against targets
**Effort**: 3 hours
**Difficulty**: Medium
**Assignable**: Yes

**Targets to Verify**:
- [ ] Startup time: <100ms
- [ ] First inference: <2s (M1 Mac) or <5s (CPU)
- [ ] Memory usage: <4GB
- [ ] Binary size: <50MB

**Deliverable**: Performance report

**Acceptance Criteria**:
- [ ] Meets or documents deviation from targets
- [ ] Results reproducible
- [ ] Compared across platforms
- [ ] Included in release notes

---

#### Task 5.3: CI/CD integration
**Effort**: 4 hours
**Difficulty**: Hard
**Assignable**: No (requires CI/CD knowledge)

**GitHub Actions**:
```yaml
- name: Download test model
  run: |
    mkdir -p tests/fixtures/models
    wget ... # Cache this

- name: Run integration tests
  run: cargo test --test integration

- name: Run benchmarks
  run: cargo bench
```

**Deliverable**: Automated testing

**Acceptance Criteria**:
- [ ] CI runs on every PR
- [ ] Model cached to speed up builds
- [ ] Tests pass consistently
- [ ] Benchmarks tracked over time

---

#### Task 5.4: Update documentation
**Effort**: 3 hours
**Difficulty**: Easy
**Assignable**: Yes

**Docs to Update**:
- [ ] README with real inference info
- [ ] CLAUDE.md with llama-cpp details
- [ ] CONTRIBUTING.md with setup
- [ ] User guide with examples
- [ ] API documentation

**Deliverable**: Updated docs

**Acceptance Criteria**:
- [ ] All mentions of "mock" removed
- [ ] Real inference documented
- [ ] Examples updated
- [ ] Links verified

---

#### Task 5.5: Release preparation
**Effort**: 2 hours
**Difficulty**: Easy
**Assignable**: Yes

**Checklist**:
- [ ] CHANGELOG.md updated
- [ ] Version bumped to v1.0.0
- [ ] Git tag created
- [ ] Release notes written
- [ ] Known issues documented

**Deliverable**: Release v1.0.0

**Acceptance Criteria**:
- [ ] Tag pushed
- [ ] Release published
- [ ] Binaries attached
- [ ] Announced in community

---

## Task Assignment Guide

### For Solo Developers
**Week 1**: Tasks 1.1-1.5, 2.1-2.2
**Week 2**: Tasks 2.3-2.6, 3.1-3.2
**Week 3**: Tasks 3.3-3.5, 4.1-4.3
**Week 4**: Tasks 4.4-4.5, 5.1-5.5

### For Teams (3 people)
**Developer 1** (Backend):
- Week 1-2: Milestone 1-2 (llama-cpp integration)
- Week 3: Milestone 3 (testing, optimization)

**Developer 2** (Infrastructure):
- Week 1-2: Milestone 4 (download system)
- Week 3: Milestone 5 (CI/CD, release)

**Developer 3** (Documentation):
- Week 1-2: Help with testing, write examples
- Week 3: Documentation, user guides, tutorials

### Skill Requirements

**Easy Tasks** (can be first contribution):
- 1.1, 1.2, 1.4, 2.1, 3.5, 4.4, 5.4, 5.5

**Medium Tasks** (requires Rust experience):
- 1.3, 1.5, 2.3, 2.4, 2.5, 2.6, 3.1, 3.4, 4.1, 4.2, 4.3, 4.5, 5.1, 5.2

**Hard Tasks** (requires expertise):
- 2.2, 3.2, 3.3, 5.3

---

## Progress Tracking

### Use GitHub Projects

**Columns**:
- 📋 Backlog
- 🏗️ In Progress
- 👀 Review
- ✅ Done

**Labels**:
- `phase-1` - Phase 1 task
- `easy` - Beginner friendly
- `medium` - Intermediate
- `hard` - Expert level
- `blocked` - Waiting on something
- `help-wanted` - Need assistance

**Milestones**:
- Milestone 1: Proof of Concept (Day 3)
- Milestone 2: Integration (Day 7)
- Milestone 3: Testing (Day 12)
- Milestone 4: Download System (Day 16)
- Milestone 5: Release (Day 20)

---

## Daily Standup Template

**Post in GitHub Discussions daily**:

```markdown
### Daily Update: YYYY-MM-DD

**Completed**:
- [x] Task 1.2: Added llama-cpp dependency
- [x] Task 1.3: Created basic inference example

**In Progress**:
- [ ] Task 2.1: Creating LlamaCppBackend struct (50% done)

**Blocked**:
- Task 2.2: Need help with async trait implementation

**Next**:
- Complete Task 2.1
- Start Task 2.2
- Review PR #123
```

---

## Success Criteria

Phase 1 is complete when:
- [ ] Real model inference works end-to-end
- [ ] Generates valid shell commands
- [ ] Safety validation integrated
- [ ] Performance meets targets (or documented)
- [ ] Model download system works
- [ ] All tests passing
- [ ] Documentation updated
- [ ] Ready for Phase 2 (UX polish)

---

## Questions?

- 💬 Ask in [GitHub Discussions](https://github.com/wildcard/cmdai/discussions)
- 🐛 Report issues in [GitHub Issues](https://github.com/wildcard/cmdai/issues)
- 📖 Read [ROADMAP_REALITY_CHECK.md](./ROADMAP_REALITY_CHECK.md) for context

**Let's ship Phase 1! 🚀**
