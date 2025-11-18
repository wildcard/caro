# Critical Blockers - cmdai Production Readiness

**Last Updated**: 2025-11-18
**Status**: 4 critical blockers preventing v1.0 release
**Total Estimated Effort**: 40-64 hours

---

## 🚨 Blocker Priority Matrix

| Priority | Blocker | Impact | Effort | Owner | Status |
|----------|---------|--------|--------|-------|--------|
| **P0** | Embedded Backend Tests Failing | CRITICAL - Tool non-functional | 8-12h | Unassigned | 🔴 |
| **P0** | Model Download Not Implemented | CRITICAL - Fresh installs broken | 16-24h | Unassigned | 🔴 |
| **P1** | MLX Performance Not Optimized | HIGH - Performance promise unmet | 8-16h | Unassigned | 🟡 |
| **P1** | Binary Distribution Not Set Up | HIGH - Users can't install easily | 8-12h | Unassigned | 🟡 |

**Total**: 40-64 hours to production-ready v1.0

---

## 🔴 BLOCKER 1: Embedded Backend Tests Failing (P0)

**File**: `tests/embedded_backend_contract.rs`
**Failing Tests**: 3 out of 11 tests
**Impact**: **CRITICAL - The tool cannot generate any commands**
**Effort**: 8-12 hours
**Skills**: Rust, async programming, ML inference

### The Problem

The embedded backend is completely non-functional. Three contract tests fail because `generate_command()` is not implemented:

```rust
// tests/embedded_backend_contract.rs

// FAILING TEST 1: Line 29
#[tokio::test]
async fn test_offline_operation_no_network_calls() {
    let backend = create_test_backend().expect("Failed to create backend");
    let request = CommandRequest::new("list files", ShellType::Bash)
        .with_safety(SafetyLevel::Moderate);

    let result = backend.generate_command(&request).await;

    // FAILS HERE - generate_command() returns error
    assert!(result.is_ok(), "Must work offline without network");
}

// FAILING TEST 2: Line 144
#[tokio::test]
async fn test_safety_validator_integration() {
    let backend = create_test_backend().expect("Failed to create backend");
    let dangerous_request = CommandRequest::new("delete all files", ShellType::Bash)
        .with_safety(SafetyLevel::Strict);

    let result = backend.generate_command(&dangerous_request).await;

    // FAILS - cannot validate safety without working backend
    assert!(result.is_ok() || matches!(result.unwrap_err(),
        GeneratorError::GenerationFailed { .. }));
}

// FAILING TEST 3: Line 248
#[tokio::test]
async fn test_thread_safe_concurrent_requests() {
    let backend = Arc::new(create_test_backend().expect("Failed to create backend"));

    for i in 0..5 {
        let backend_clone = Arc::clone(&backend);
        let handle = tokio::spawn(async move {
            let request = CommandRequest::new(format!("list files {}", i),
                ShellType::Bash);
            backend_clone.generate_command(&request).await
        });
        handles.push(handle);
    }

    // ALL 5 REQUESTS FAIL - backend not implemented
    for (i, handle) in handles.into_iter().enumerate() {
        let result = handle.await.expect("Task panicked");
        assert!(result.is_ok(), "Concurrent request {} must succeed", i);
    }
}
```

### Why This Is Critical

1. **Tool is completely non-functional** - Cannot generate a single command
2. **Embedded backend is the default** - 99% of users hit this immediately
3. **Remote backends require external setup** - Not a fallback for most users
4. **Blocks all downstream testing** - Cannot test safety, UX, or workflows
5. **Prevents demo/marketing** - Cannot show the tool working

### Root Cause Analysis

```rust
// src/backends/embedded/embedded_backend.rs (current state)
impl CommandGenerator for EmbeddedModelBackend {
    async fn generate_command(
        &self,
        request: &CommandRequest,
    ) -> Result<GeneratedCommand, GeneratorError> {
        // INCOMPLETE - Just validates availability
        self.ensure_available().await?;

        // TODO: Actually implement inference
        // 1. Load model (if not already loaded)
        // 2. Format prompt with system template
        // 3. Tokenize input
        // 4. Run inference (CPU or MLX)
        // 5. Decode output
        // 6. Parse JSON response
        // 7. Validate safety
        // 8. Return GeneratedCommand

        Err(GeneratorError::GenerationFailed {
            backend: "embedded".to_string(),
            message: "Not implemented yet".to_string(),
        })
    }
}
```

**Missing Components**:
- Model loading logic
- Tokenizer integration
- Prompt formatting
- Inference engine integration (Candle CPU or MLX)
- JSON response parsing
- Error handling for inference failures

### Implementation Checklist

#### Phase 1: CPU Backend Integration (6-8 hours)

**Step 1.1**: Add Candle dependencies (30 minutes)
```toml
# Cargo.toml
[dependencies]
candle-core = "0.3"
candle-transformers = "0.3"
candle-nn = "0.3"
tokenizers = "0.14"
```

**Step 1.2**: Implement model loading (2-3 hours)
```rust
// src/backends/embedded/cpu.rs
use candle_core::{Device, Tensor};
use candle_transformers::models::quantized_llama as model;
use tokenizers::Tokenizer;

impl CpuBackend {
    async fn load_model(&self) -> Result<model::ModelWeights, GeneratorError> {
        let device = Device::Cpu;

        // Load quantized GGUF model
        let model = model::ModelWeights::from_gguf(
            &self.model_path,
            &device
        ).map_err(|e| GeneratorError::ModelLoadFailed {
            backend: "cpu".to_string(),
            message: format!("Failed to load GGUF model: {}", e),
        })?;

        Ok(model)
    }

    async fn load_tokenizer(&self) -> Result<Tokenizer, GeneratorError> {
        // Tokenizer should be in same directory as model
        let tokenizer_path = self.model_path
            .parent()
            .unwrap()
            .join("tokenizer.json");

        Tokenizer::from_file(tokenizer_path).map_err(|e| {
            GeneratorError::ModelLoadFailed {
                backend: "cpu".to_string(),
                message: format!("Failed to load tokenizer: {}", e),
            }
        })
    }
}
```

**Step 1.3**: Implement prompt formatting (1 hour)
```rust
// src/backends/embedded/prompts.rs (NEW FILE)
pub fn format_system_prompt(user_input: &str, shell: &ShellType) -> String {
    format!(
        r#"You are a command line expert. Generate a safe {shell} command for this task.

Rules:
- Output ONLY valid JSON in this exact format: {{"cmd": "command here"}}
- Generate POSIX-compliant commands when possible
- Quote file paths with spaces
- Avoid destructive operations
- Never use: rm -rf /, mkfs, dd to system devices
- Prefer safe alternatives

User request: {user_input}

JSON response:"#,
        shell = shell.as_str(),
        user_input = user_input
    )
}
```

**Step 1.4**: Implement inference loop (2-3 hours)
```rust
// src/backends/embedded/cpu.rs
impl InferenceBackend for CpuBackend {
    async fn generate(&self, prompt: &str) -> Result<String, GeneratorError> {
        // 1. Load model and tokenizer (cached after first load)
        let model = self.ensure_model_loaded().await?;
        let tokenizer = self.ensure_tokenizer_loaded().await?;

        // 2. Tokenize input
        let tokens = tokenizer
            .encode(prompt, true)
            .map_err(|e| GeneratorError::GenerationFailed {
                backend: "cpu".to_string(),
                message: format!("Tokenization failed: {}", e),
            })?
            .get_ids()
            .to_vec();

        // 3. Convert to tensor
        let input_tensor = Tensor::new(&tokens[..], &Device::Cpu)
            .map_err(|e| GeneratorError::GenerationFailed {
                backend: "cpu".to_string(),
                message: format!("Tensor creation failed: {}", e),
            })?;

        // 4. Run inference
        let mut output_tokens = tokens.clone();
        let max_new_tokens = 256;

        for _ in 0..max_new_tokens {
            let logits = model.forward(&input_tensor)?;
            let next_token = logits.argmax(1)?;

            output_tokens.push(next_token.to_scalar::<u32>()?);

            // Check for EOS token
            if next_token.to_scalar::<u32>()? == tokenizer.get_eos_token_id() {
                break;
            }
        }

        // 5. Decode output
        let generated_text = tokenizer
            .decode(&output_tokens[tokens.len()..], true)
            .map_err(|e| GeneratorError::GenerationFailed {
                backend: "cpu".to_string(),
                message: format!("Decoding failed: {}", e),
            })?;

        Ok(generated_text)
    }
}
```

**Step 1.5**: Integrate with EmbeddedModelBackend (1-2 hours)
```rust
// src/backends/embedded/embedded_backend.rs
impl CommandGenerator for EmbeddedModelBackend {
    async fn generate_command(
        &self,
        request: &CommandRequest,
    ) -> Result<GeneratedCommand, GeneratorError> {
        // Ensure backend is available
        self.ensure_available().await?;

        // Format prompt
        let prompt = format_system_prompt(
            &request.user_input,
            &request.target_shell
        );

        // Generate using appropriate backend
        let backend = self.backend.lock().await;
        let raw_response = backend.as_ref()
            .ok_or_else(|| GeneratorError::BackendNotInitialized)?
            .generate(&prompt)
            .await?;

        // Parse JSON response with fallbacks
        let command = self.parse_command_from_response(&raw_response)?;

        // Validate safety
        let risk_level = self.safety_validator.assess_risk(&command)?;

        Ok(GeneratedCommand {
            command,
            backend_used: "embedded".to_string(),
            risk_level,
            raw_response: Some(raw_response),
            generation_time: None,
        })
    }
}
```

#### Phase 2: JSON Parsing with Fallbacks (1-2 hours)

```rust
// src/backends/embedded/parsing.rs (NEW FILE)
use serde_json::Value;

pub fn parse_command_from_response(response: &str) -> Result<String, GeneratorError> {
    // Strategy 1: Try direct JSON parse
    if let Ok(json) = serde_json::from_str::<Value>(response) {
        if let Some(cmd) = json.get("cmd").and_then(|v| v.as_str()) {
            return Ok(cmd.to_string());
        }
    }

    // Strategy 2: Extract JSON from markdown code block
    if let Some(json_str) = extract_json_from_markdown(response) {
        if let Ok(json) = serde_json::from_str::<Value>(&json_str) {
            if let Some(cmd) = json.get("cmd").and_then(|v| v.as_str()) {
                return Ok(cmd.to_string());
            }
        }
    }

    // Strategy 3: Look for JSON anywhere in response
    if let Some(json_str) = extract_json_from_text(response) {
        if let Ok(json) = serde_json::from_str::<Value>(&json_str) {
            if let Some(cmd) = json.get("cmd").and_then(|v| v.as_str()) {
                return Ok(cmd.to_string());
            }
        }
    }

    // Strategy 4: Extract command from backticks
    if let Some(cmd) = extract_from_backticks(response) {
        return Ok(cmd);
    }

    Err(GeneratorError::ResponseParsingFailed {
        backend: "embedded".to_string(),
        response: response.to_string(),
    })
}

fn extract_json_from_markdown(text: &str) -> Option<String> {
    // Match ```json ... ``` blocks
    let re = regex::Regex::new(r"```(?:json)?\s*(\{.*?\})\s*```").ok()?;
    re.captures(text).and_then(|cap| cap.get(1))
        .map(|m| m.as_str().to_string())
}

fn extract_json_from_text(text: &str) -> Option<String> {
    // Find first { ... } object
    let start = text.find('{')?;
    let end = text[start..].find('}')? + start + 1;
    Some(text[start..end].to_string())
}

fn extract_from_backticks(text: &str) -> Option<String> {
    // Extract command from `command` or ```command```
    let re = regex::Regex::new(r"`{1,3}([^`]+)`{1,3}").ok()?;
    re.captures(text).and_then(|cap| cap.get(1))
        .map(|m| m.as_str().trim().to_string())
}
```

#### Phase 3: Testing and Validation (2-3 hours)

```bash
# Test with actual model (requires model in cache)
cargo test --features embedded-cpu test_offline_operation_no_network_calls -- --nocapture

# Test all embedded backend contracts
cargo test --features embedded-cpu --test embedded_backend_contract

# Benchmark inference time
cargo bench --features embedded-cpu embedded_inference
```

### Acceptance Criteria

- [ ] All 3 failing tests now pass
- [ ] `test_offline_operation_no_network_calls` ✅
- [ ] `test_safety_validator_integration` ✅
- [ ] `test_thread_safe_concurrent_requests` ✅
- [ ] Total embedded backend tests: 11/11 passing
- [ ] Full test suite: 136/136 passing (100%)
- [ ] CI pipeline green
- [ ] Can generate actual commands: `cmdai "list files"`
- [ ] Inference completes in <5s on modern CPU

### Dependencies

**Blocks**:
- User testing
- Demo creation
- Marketing launch
- Community feedback

**Blocked By**:
- Model download (Blocker 2) - need model in cache for testing
- Can work around with manually placed model file

### Related Files

```
src/backends/embedded/
├── embedded_backend.rs  # Main implementation (UPDATE)
├── cpu.rs              # CPU inference (UPDATE)
├── common.rs           # Shared types (existing)
├── prompts.rs          # NEW - Prompt formatting
└── parsing.rs          # NEW - JSON parsing

tests/
└── embedded_backend_contract.rs  # Tests to fix

Cargo.toml              # Add candle dependencies
```

### Testing Strategy

```rust
// tests/embedded_backend_contract.rs

// After implementation, verify:
#[tokio::test]
async fn test_actual_command_generation() {
    let backend = EmbeddedModelBackend::new().await.unwrap();

    let request = CommandRequest::new(
        "list files in current directory",
        ShellType::Bash
    );

    let result = backend.generate_command(&request).await.unwrap();

    // Should generate something like: ls -la or ls -l
    assert!(result.command.contains("ls"));
    assert_eq!(result.backend_used, "embedded");
    assert!(matches!(result.risk_level, RiskLevel::Safe));
}
```

---

## 🔴 BLOCKER 2: Model Download Not Implemented (P0)

**File**: `src/cache/mod.rs`
**Function**: `download_model()`
**Impact**: **CRITICAL - Fresh installs completely broken**
**Effort**: 16-24 hours
**Skills**: Rust, async I/O, HTTP APIs, progress bars

### The Problem

The model download function is a placeholder that always returns an error:

```rust
// src/cache/mod.rs (line ~150)
async fn download_model(&self, model_id: &str) -> Result<PathBuf, CacheError> {
    // Placeholder: In real implementation, this would:
    // 1. Fetch model from Hugging Face Hub
    // 2. Show progress bar
    // 3. Validate checksums
    // 4. Update manifest
    Err(CacheError::DownloadFailed("Download not implemented yet".to_string()))
}
```

### Why This Is Critical

**User Experience Failure**:
```bash
# What users expect:
$ cmdai "list files"
📦 First-time setup: Downloading model (1.1GB)...
[=========>                    ] 35% (389MB/1.1GB) ETA: 2m 14s
✅ Model downloaded successfully
Generated command: ls -la

# What actually happens:
$ cmdai "list files"
Error: Model not found in cache
Please manually download qwen2.5-coder-1.5b-instruct-q4_k_m.gguf
and place it in ~/.cache/cmdai/models/

# User thinks: "This tool is broken" and uninstalls
```

**Impact**:
1. **100% of fresh installs fail** - Users cannot use the tool
2. **Manual workaround is complex** - Requires finding correct model file, downloading manually
3. **Breaks "just works" promise** - Not a simple CLI tool anymore
4. **Prevents viral adoption** - Users won't recommend broken tools
5. **Creates support burden** - Every user needs help with setup

### Implementation Checklist

#### Phase 1: Basic Download (8-10 hours)

**Step 1**: Create HF downloader module
```rust
// src/cache/hf_download.rs (NEW FILE)
use reqwest::{Client, Response};
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use std::path::PathBuf;
use indicatif::{ProgressBar, ProgressStyle};

pub struct HfDownloader {
    client: Client,
    cache_dir: PathBuf,
}

impl HfDownloader {
    pub fn new(cache_dir: PathBuf) -> Self {
        let client = Client::builder()
            .user_agent("cmdai/0.1.0")
            .timeout(Duration::from_secs(300))  // 5 min timeout
            .build()
            .expect("Failed to create HTTP client");

        Self { client, cache_dir }
    }

    pub async fn download_model(
        &self,
        repo: &str,
        filename: &str,
    ) -> Result<PathBuf, HfDownloadError> {
        let url = format!(
            "https://huggingface.co/{}/resolve/main/{}",
            repo, filename
        );

        let dest_path = self.cache_dir.join(filename);

        // Create parent directory
        if let Some(parent) = dest_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        // Start download
        println!("📦 Downloading {} from Hugging Face...", filename);
        let response = self.client.get(&url).send().await?;

        if !response.status().is_success() {
            return Err(HfDownloadError::HttpError(
                response.status().as_u16()
            ));
        }

        let total_size = response.content_length().unwrap_or(0);

        // Setup progress bar
        let pb = ProgressBar::new(total_size);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{msg}\n{bar:40.cyan/blue} {bytes}/{total_bytes} ({eta})")
                .unwrap()
                .progress_chars("=>-")
        );
        pb.set_message(format!("Downloading {}", filename));

        // Stream to file
        let mut file = File::create(&dest_path).await?;
        let mut downloaded = 0u64;
        let mut stream = response.bytes_stream();

        use futures::StreamExt;
        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            file.write_all(&chunk).await?;
            downloaded += chunk.len() as u64;
            pb.set_position(downloaded);
        }

        pb.finish_with_message(format!("✅ {} downloaded", filename));

        Ok(dest_path)
    }
}
```

**Step 2**: Add resume support (4-6 hours)
```rust
impl HfDownloader {
    pub async fn resume_download(
        &self,
        repo: &str,
        filename: &str,
    ) -> Result<PathBuf, HfDownloadError> {
        let dest_path = self.cache_dir.join(filename);

        // Check if partial download exists
        let existing_size = if dest_path.exists() {
            tokio::fs::metadata(&dest_path).await?.len()
        } else {
            0
        };

        if existing_size > 0 {
            println!("📦 Resuming download from {} bytes...", existing_size);
        }

        let url = format!(
            "https://huggingface.co/{}/resolve/main/{}",
            repo, filename
        );

        // Request with Range header
        let response = self.client
            .get(&url)
            .header("Range", format!("bytes={}-", existing_size))
            .send()
            .await?;

        // Check if server supports resume (206 Partial Content)
        if response.status().as_u16() != 206 && existing_size > 0 {
            println!("⚠️  Server doesn't support resume, starting from beginning");
            return self.download_model(repo, filename).await;
        }

        let total_size = existing_size + response.content_length().unwrap_or(0);

        // Setup progress bar starting from existing size
        let pb = ProgressBar::new(total_size);
        pb.set_position(existing_size);
        pb.set_style(/* ... */);

        // Append to existing file
        let mut file = tokio::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&dest_path)
            .await?;

        let mut downloaded = existing_size;
        let mut stream = response.bytes_stream();

        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            file.write_all(&chunk).await?;
            downloaded += chunk.len() as u64;
            pb.set_position(downloaded);
        }

        pb.finish_with_message("✅ Download complete");
        Ok(dest_path)
    }
}
```

**Step 3**: Add checksum validation (2-3 hours)
```rust
use sha2::{Sha256, Digest};
use tokio::io::AsyncReadExt;

impl HfDownloader {
    pub async fn verify_checksum(
        &self,
        path: &PathBuf,
        expected: &str,
    ) -> Result<bool, HfDownloadError> {
        let mut file = File::open(path).await?;
        let mut hasher = Sha256::new();
        let mut buffer = vec![0u8; 8192];

        loop {
            let n = file.read(&mut buffer).await?;
            if n == 0 {
                break;
            }
            hasher.update(&buffer[..n]);
        }

        let hash = format!("{:x}", hasher.finalize());
        Ok(hash == expected)
    }

    pub async fn download_with_verification(
        &self,
        repo: &str,
        filename: &str,
        expected_checksum: Option<&str>,
    ) -> Result<PathBuf, HfDownloadError> {
        let path = self.resume_download(repo, filename).await?;

        if let Some(checksum) = expected_checksum {
            println!("🔍 Verifying checksum...");
            if !self.verify_checksum(&path, checksum).await? {
                // Delete corrupted file
                tokio::fs::remove_file(&path).await?;
                return Err(HfDownloadError::ChecksumMismatch);
            }
            println!("✅ Checksum verified");
        }

        Ok(path)
    }
}
```

#### Phase 2: Integration (2-4 hours)

```rust
// src/cache/mod.rs (UPDATE)
impl CacheManager {
    async fn download_model(&self, model_id: &str) -> Result<PathBuf, CacheError> {
        let downloader = HfDownloader::new(self.cache_dir.clone());

        // Map model_id to repo and filename
        let (repo, filename) = match model_id {
            "qwen2.5-coder-1.5b" => (
                "Qwen/Qwen2.5-Coder-1.5B-Instruct-GGUF",
                "qwen2.5-coder-1.5b-instruct-q4_k_m.gguf"
            ),
            "qwen2.5-coder-0.5b" => (
                "Qwen/Qwen2.5-Coder-0.5B-Instruct-GGUF",
                "qwen2.5-coder-0.5b-instruct-q4_k_m.gguf"
            ),
            _ => return Err(CacheError::UnsupportedModel(model_id.to_string())),
        };

        // Download with resume support
        let model_path = downloader
            .resume_download(repo, filename)
            .await
            .map_err(|e| CacheError::DownloadFailed(e.to_string()))?;

        // Update manifest
        self.add_to_manifest(model_id, &model_path).await?;

        Ok(model_path)
    }
}

// src/backends/embedded/embedded_backend.rs (UPDATE)
impl EmbeddedModelBackend {
    pub async fn ensure_model_available(&self) -> Result<PathBuf> {
        let model_path = self.model_path.clone();

        if !model_path.exists() {
            println!("\n📦 First-time setup: cmdai needs to download a language model");
            println!("   Model: Qwen2.5-Coder 1.5B (Instruct, quantized)");
            println!("   Size: ~1.1GB (one-time download)");
            println!("   Purpose: Enable offline command generation\n");

            let cache = CacheManager::new()?;
            cache.download_model("qwen2.5-coder-1.5b").await?;

            println!("\n✅ Setup complete! Model cached for offline use.\n");
        }

        Ok(model_path)
    }
}
```

#### Phase 3: Error Handling (1-2 hours)

```rust
#[derive(Debug, thiserror::Error)]
pub enum HfDownloadError {
    #[error("HTTP error: status {0}")]
    HttpError(u16),

    #[error("Network error: {0}")]
    NetworkError(#[from] reqwest::Error),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Checksum mismatch - downloaded file is corrupted")]
    ChecksumMismatch,

    #[error("Download interrupted - you can resume with the same command")]
    Interrupted,
}

// Retry logic for transient failures
impl HfDownloader {
    async fn download_with_retry(
        &self,
        repo: &str,
        filename: &str,
        max_attempts: u32,
    ) -> Result<PathBuf, HfDownloadError> {
        let mut attempts = 0;
        let mut backoff = Duration::from_secs(1);

        loop {
            attempts += 1;

            match self.resume_download(repo, filename).await {
                Ok(path) => return Ok(path),
                Err(e) if attempts >= max_attempts => return Err(e),
                Err(e) => {
                    println!("⚠️  Download failed (attempt {}/{}): {}",
                             attempts, max_attempts, e);
                    println!("   Retrying in {:?}...", backoff);
                    tokio::time::sleep(backoff).await;
                    backoff *= 2;  // Exponential backoff
                }
            }
        }
    }
}
```

### Dependencies Required

```toml
# Cargo.toml
[dependencies]
reqwest = { version = "0.11", features = ["stream"] }
tokio = { version = "1.0", features = ["fs", "io-util"] }
tokio-util = { version = "0.7", features = ["io"] }
futures = "0.3"
indicatif = "0.17"
sha2 = "0.10"
```

### Acceptance Criteria

- [ ] Fresh install downloads model automatically
- [ ] Download shows progress bar with ETA
- [ ] Download can be interrupted and resumed
- [ ] Checksum validation prevents corrupted models
- [ ] Network failures trigger retry with backoff
- [ ] Clear error messages for all failure modes
- [ ] Downloaded model works with embedded backend
- [ ] Manifest updated after successful download
- [ ] First-run UX is clear and reassuring

### Testing

```bash
# Unit test with mock HTTP server
cargo test --package cmdai --lib cache::hf_download

# Integration test with local file server
cargo test --test hf_download_integration

# Manual testing
rm -rf ~/.cache/cmdai  # Clear cache
cargo run -- "list files"  # Should trigger download
# Interrupt with Ctrl+C, run again to test resume
```

---

## 🟡 BLOCKER 3: MLX Performance Not Optimized (P1)

**File**: `src/backends/embedded/mlx.rs`
**Impact**: HIGH - Performance promise unmet (<2s inference)
**Effort**: 8-16 hours
**Skills**: Rust, Python/Swift FFI, Apple Silicon optimization

### The Problem

MLX backend is a placeholder - no actual implementation exists.

**Current State**:
```rust
// src/backends/embedded/mlx.rs
pub struct MlxBackend {
    model_path: PathBuf,
    model_state: Arc<Mutex<Option<MlxModelState>>>,
}

// PLACEHOLDER - No actual inference
```

**Performance Gap**:
- **Promise**: First inference <2s on M1 Mac
- **Current**: Unknown (not implemented)
- **Risk**: May need optimization tuning

### Quick Win: Use MLX Python Bindings

See PROJECT_STATUS.md "Blocker 3" section for full implementation using PyO3.

**Estimated effort**: 8-12 hours for Python approach

---

## 🟡 BLOCKER 4: Binary Distribution Not Set Up (P1)

**Files**: `.github/workflows/release.yml`, Homebrew formula
**Impact**: HIGH - Users cannot easily install
**Effort**: 8-12 hours
**Skills**: GitHub Actions, Homebrew, package management

### The Problem

While CI builds binaries, there's no automated release process.

**Current State**:
- ✅ CI builds binaries for 5 platforms
- ❌ No GitHub releases on tags
- ❌ No Homebrew tap
- ❌ No installation scripts

### Implementation

See PROJECT_STATUS.md "Blocker 4" section for complete implementation guide.

**Phases**:
1. Automated GitHub releases (3-4 hours)
2. Homebrew formula (4-6 hours)
3. Installation documentation (1-2 hours)

---

## 📊 Blocker Resolution Timeline

### Week 1: Core Functionality
**Days 1-2**: Blocker 1 (Embedded Backend) - **CRITICAL**
- Implement CPU inference with Candle
- Add JSON parsing
- Fix 3 failing tests

**Days 3-5**: Blocker 2 (Model Download) - **CRITICAL**
- Implement HTTP download
- Add resume support
- Integrate with backend

### Week 2: Polish & Distribution
**Days 1-2**: Blocker 3 (MLX Performance) - **HIGH PRIORITY**
- Add PyO3 MLX bindings
- Benchmark and optimize

**Days 3-4**: Blocker 4 (Distribution) - **HIGH PRIORITY**
- Setup GitHub releases
- Create Homebrew formula

**Day 5**: Testing & Documentation
- Full integration testing
- Update documentation
- Prepare for launch

---

## 🎯 Success Metrics

**Blocker 1 Resolved**:
- ✅ All 136/136 tests passing
- ✅ Can generate commands: `cmdai "list files"` works

**Blocker 2 Resolved**:
- ✅ Fresh install downloads model automatically
- ✅ Download completes successfully on 3/3 test machines

**Blocker 3 Resolved**:
- ✅ First inference <2s on M1 Mac
- ✅ Performance benchmarks passing

**Blocker 4 Resolved**:
- ✅ `brew install wildcard/tap/cmdai` works
- ✅ Linux installation script successful

**All Blockers Resolved** = **Ready for v1.0 Launch** 🚀

---

## 📞 Getting Help

**Need Help Resolving a Blocker?**

1. Comment on the tracking issue
2. Check implementation guides in PROJECT_STATUS.md
3. Review existing code for patterns
4. Ask in GitHub Discussions

**Want to Claim a Blocker?**

1. Comment on this file or tracking issue
2. Provide estimated timeline
3. Request assignment
4. Submit progress updates

---

**Last Updated**: 2025-11-18
**Next Review**: After each blocker resolution
