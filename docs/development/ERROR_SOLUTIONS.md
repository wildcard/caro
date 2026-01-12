# Error-Solution Taxonomy for AI Development

> **Purpose**: When AI encounters an error, look it up here for the canonical fix. This document maps errors to solutions for faster debugging.

## Compilation Errors

### Async/Await Errors

| Error Message | Cause | Solution |
|--------------|-------|----------|
| `future cannot be sent between threads safely` | Non-Send type in async block | Wrap in `Arc<Mutex<T>>` or use `tokio::sync::Mutex` |
| `the trait bound CommandGenerator is not satisfied` | Missing async_trait macro | Add `#[async_trait]` attribute and import |
| `cannot find macro async_trait` | Missing import | Add `use async_trait::async_trait;` |
| `use of async fn in public traits is unstable` | Trait needs async support | Use `#[async_trait]` from `async-trait` crate |

**Canonical async trait pattern:**
```rust
use async_trait::async_trait;

#[async_trait]
pub trait CommandGenerator: Send + Sync {
    async fn generate_command(&self, req: &CommandRequest) -> Result<GeneratedCommand>;
}
```

### Borrow Checker Errors

| Error Message | Cause | Solution |
|--------------|-------|----------|
| `cannot borrow as mutable` | Shared reference mutation | Use `Arc<RwLock<T>>` or `RefCell<T>` |
| `cannot move out of borrowed content` | Moving from reference | Clone the value or use reference |
| `lifetime may not live long enough` | Reference outlives scope | Use owned types or add `'static` |
| `borrowed value does not live long enough` | Reference scope too short | Extend scope or clone |

**Canonical interior mutability:**
```rust
use std::sync::{Arc, RwLock};

struct SharedState {
    data: Arc<RwLock<Data>>,
}

impl SharedState {
    fn update(&self, new_data: Data) {
        let mut guard = self.data.write().unwrap();
        *guard = new_data;
    }
}
```

### Type Errors

| Error Message | Cause | Solution |
|--------------|-------|----------|
| `expected type T, found type U` | Type mismatch | Check types match exactly |
| `the trait From<T> is not implemented` | Missing conversion | Implement `From` or use explicit conversion |
| `mismatched types` in match arms | Inconsistent return types | Ensure all arms return same type |
| `expected struct Result, found ()` | Forgot to return Result | Add `Ok(())` or `?` operator |

**Canonical type conversion:**
```rust
// Implement From for custom conversions
impl From<BackendError> for anyhow::Error {
    fn from(e: BackendError) -> Self {
        anyhow!("Backend error: {}", e)
    }
}
```

### Import/Module Errors

| Error Message | Cause | Solution |
|--------------|-------|----------|
| `unresolved import` | Missing dependency or module | Check Cargo.toml and mod.rs |
| `module not found` | Missing mod declaration | Add `mod module_name;` to parent |
| `cannot find value in scope` | Not imported or wrong path | Add `use` statement |
| `private type in public interface` | Visibility mismatch | Make type `pub` or adjust visibility |

**Canonical module structure:**
```rust
// src/lib.rs
pub mod backends;
pub mod safety;
pub mod config;

// Re-export commonly used types
pub use backends::CommandGenerator;
pub use safety::{SafetyValidator, RiskLevel};
```

## Runtime Errors

### Backend Errors

| Error | Cause | Solution |
|-------|-------|----------|
| `Backend unavailable` | Service not running | Start Ollama/vLLM or use embedded |
| `Connection refused` | Wrong URL or port | Check `base_url` in config |
| `Timeout waiting for response` | Slow model or network | Increase timeout in config |
| `Model not found` | Invalid model name | Check available models with `ollama list` |

**Canonical backend error handling:**
```rust
match backend.generate_command(&request).await {
    Ok(cmd) => Ok(cmd),
    Err(BackendError::Unavailable) => {
        log::warn!("Primary backend unavailable, trying fallback");
        fallback_backend.generate_command(&request).await
    }
    Err(BackendError::Timeout) => {
        Err(anyhow!("Request timed out after {}s", timeout.as_secs()))
    }
    Err(e) => Err(e.into()),
}
```

### JSON Parsing Errors

| Error | Cause | Solution |
|-------|-------|----------|
| `expected value at line 1` | Empty or invalid JSON | Check LLM response, use fallback |
| `missing field 'cmd'` | LLM returned wrong format | Use extraction pattern |
| `invalid type: expected string` | Wrong field type | Check GeneratedCommand schema |
| `trailing characters` | Extra text after JSON | Extract JSON with regex |

**Canonical JSON extraction:**
```rust
fn extract_command(response: &str) -> Result<GeneratedCommand> {
    // Try 1: Direct parse
    if let Ok(cmd) = serde_json::from_str(response) {
        return Ok(cmd);
    }

    // Try 2: Extract from markdown code block
    let json_block_re = Regex::new(r"```(?:json)?\s*(\{[^`]+\})\s*```").unwrap();
    if let Some(caps) = json_block_re.captures(response) {
        if let Ok(cmd) = serde_json::from_str(&caps[1]) {
            return Ok(cmd);
        }
    }

    // Try 3: Find raw JSON object
    let json_re = Regex::new(r#"\{"cmd"\s*:\s*"[^"]+"\}"#).unwrap();
    if let Some(m) = json_re.find(response) {
        return serde_json::from_str(m.as_str())
            .context("Extracted JSON parse failed");
    }

    bail!("No valid command JSON in response")
}
```

### Safety Errors

| Error | Cause | Solution |
|-------|-------|----------|
| `Command blocked: Critical risk` | Matched dangerous pattern | Review command, adjust if false positive |
| `Safety validation failed` | Pattern match error | Check regex syntax |
| `Unknown risk level` | Invalid configuration | Use enum values only |

**Canonical safety handling:**
```rust
let result = safety_validator.validate(&command);

match result.risk_level {
    RiskLevel::Safe => execute_command(&command).await?,
    RiskLevel::Moderate | RiskLevel::High => {
        if user_confirmed(&command, &result) {
            execute_command(&command).await?
        } else {
            println!("Command cancelled");
        }
    }
    RiskLevel::Critical => {
        eprintln!("{}: {}", "BLOCKED".red(), result.reason);
        bail!("Command blocked for safety")
    }
}
```

### Configuration Errors

| Error | Cause | Solution |
|-------|-------|----------|
| `Config file not found` | Missing config | Use `Config::default()` |
| `Invalid TOML` | Syntax error in config | Validate TOML syntax |
| `Unknown field` | Old/new config mismatch | Update config schema |
| `Missing required field` | Incomplete config | Use defaults for missing fields |

**Canonical config loading:**
```rust
fn load_config() -> Result<Config> {
    let config_path = dirs::config_dir()
        .ok_or_else(|| anyhow!("No config directory"))?
        .join("caro")
        .join("config.toml");

    if config_path.exists() {
        let content = std::fs::read_to_string(&config_path)
            .context("Reading config file")?;
        toml::from_str(&content)
            .context("Parsing config TOML")
    } else {
        log::info!("No config found, using defaults");
        Ok(Config::default())
    }
}
```

## Logic Errors

### Platform Compatibility

| Symptom | Cause | Solution |
|---------|-------|----------|
| `ls: illegal option` on macOS | Using GNU flags on BSD | Use platform detection |
| `xargs: illegal option` | BSD vs GNU xargs | Adjust flags per platform |
| `sed: command not found` | Wrong sed syntax | Check platform-specific patterns |
| Command works locally, fails in CI | Platform difference | Test on all platforms |

**Canonical platform detection:**
```rust
let ctx = ExecutionContext::detect();

let sort_cmd = match ctx.os {
    Os::MacOS => "sort -n -r -k 3,3",   // BSD sort
    Os::Linux => "sort -n -r -k 3",      // GNU sort
    Os::Windows => "Sort-Object",         // PowerShell
};
```

### State Management

| Symptom | Cause | Solution |
|---------|-------|----------|
| Config changes not reflected | Caching stale config | Reload config on change |
| Backend state inconsistent | Race condition | Use proper synchronization |
| Test pollution | Shared mutable state | Isolate tests properly |

**Canonical test isolation:**
```rust
#[tokio::test]
async fn test_with_isolation() {
    // Create isolated environment
    let temp_dir = tempfile::tempdir().unwrap();
    let config = Config::default()
        .with_config_dir(temp_dir.path());

    // Test with isolated config
    let result = operation_under_test(&config).await;

    // temp_dir automatically cleaned up
    assert!(result.is_ok());
}
```

## Test Failures

### Common Test Errors

| Error | Cause | Solution |
|-------|-------|----------|
| `assertion failed: result.is_ok()` | Operation returned Err | Check error message for details |
| `timeout` in async test | Test took too long | Increase timeout or check for deadlock |
| `test did not panic` | Expected panic didn't occur | Verify panic condition |
| Flaky test | Non-deterministic behavior | Add proper synchronization |

**Canonical async test with timeout:**
```rust
#[tokio::test]
async fn test_with_timeout() {
    let result = tokio::time::timeout(
        Duration::from_secs(5),
        async_operation()
    ).await;

    assert!(result.is_ok(), "Operation timed out");
    assert!(result.unwrap().is_ok(), "Operation failed");
}
```

## Quick Fixes Reference

### One-Line Fixes

| Problem | Fix |
|---------|-----|
| Missing Send bound | Add `: Send + Sync` to trait |
| Missing Clone | Add `#[derive(Clone)]` |
| Borrow in async | Clone before async block |
| Optional unwrap | Use `.unwrap_or_default()` |
| Error without context | Add `.context("description")?` |

### Code Snippets for Common Fixes

```rust
// Fix: Add Send + Sync to async trait
#[async_trait]
trait MyTrait: Send + Sync { }

// Fix: Clone before async
let owned = borrowed.clone();
async move { use_owned(owned) }

// Fix: Context on errors
let value = operation()
    .context("Operation description")?;

// Fix: Default for optional
let value = optional.unwrap_or_default();

// Fix: Interior mutability
use std::sync::Arc;
let shared = Arc::new(Mutex::new(value));
```

---

*When in doubt, check this document first.*
*Last updated: 2026-01-12*
