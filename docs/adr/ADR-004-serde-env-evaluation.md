# ADR-004: Environment Variable Deserialization with serde-env

| **Status**     | Proposed                            |
|----------------|-------------------------------------|
| **Date**       | January 2026                        |
| **Authors**    | Caro Maintainers                    |
| **Target**     | Community                           |

## Table of Contents

1. [Executive Summary](#executive-summary)
2. [Context and Problem Statement](#context-and-problem-statement)
3. [Decision Drivers](#decision-drivers)
4. [Candidate Evaluation](#candidate-evaluation)
5. [Recommendation](#recommendation)
6. [Implementation Notes](#implementation-notes)
7. [Consequences](#consequences)

---

## Executive Summary

This document evaluates whether to adopt `serde-env` or an alternative crate for deserializing environment variables into Caro's configuration structures. After analysis, we **recommend NOT adopting serde-env** for the current caro codebase, instead maintaining the existing manual approach with targeted improvements.

**Key Findings:**
- Caro's `UserConfiguration` struct is flat (no nesting), eliminating serde-env's primary advantage
- The current 5 environment variables are well-served by explicit manual parsing
- serde-env's `_` separator conflicts with Caro's `CARO_` prefix convention
- Manual parsing provides better error messages and validation control

---

## Context and Problem Statement

### Current Implementation

Caro currently uses manual environment variable parsing in `src/config/mod.rs:136-163`:

```rust
pub fn merge_with_env(&self) -> Result<UserConfiguration, ConfigError> {
    let mut config = self.load()?;

    if let Ok(safety_str) = std::env::var("CARO_SAFETY_LEVEL") {
        config.safety_level = safety_str.parse().map_err(ConfigError::ValidationError)?;
    }

    if let Ok(shell_str) = std::env::var("CARO_DEFAULT_SHELL") {
        config.default_shell = Some(shell_str.parse().map_err(ConfigError::ValidationError)?);
    }

    // ... 3 more env vars
    Ok(config)
}
```

**Supported Environment Variables:**
| Variable | Type | Purpose |
|----------|------|---------|
| `CARO_SAFETY_LEVEL` | SafetyLevel enum | Safety validation strictness |
| `CARO_DEFAULT_SHELL` | ShellType enum | Target shell for commands |
| `CARO_LOG_LEVEL` | LogLevel enum | Logging verbosity |
| `CARO_DEFAULT_MODEL` | String | Preferred LLM model |
| `CARO_CACHE_MAX_SIZE_GB` | u64 | Cache size limit |

### The Question

Should we replace manual parsing with a serde-based deserialization crate like `serde-env` to:
1. Reduce boilerplate code?
2. Automatically handle new configuration fields?
3. Leverage serde's type system for parsing?

---

## Decision Drivers

### Primary Considerations

1. **Simplicity**: Avoid unnecessary dependencies for solved problems
2. **Error Messages**: Users need clear feedback on invalid env var values
3. **Naming Conventions**: Maintain `CARO_` prefix for all env vars
4. **Type Safety**: Parse custom enums (SafetyLevel, ShellType, LogLevel) correctly
5. **Validation**: Enforce business rules (cache size 1-1000GB, rotation days 1-365)

### Secondary Considerations

- Dependency footprint (binary size, compile time)
- Maintenance burden of manual code vs. library updates
- Consistency with existing TOML configuration approach

---

## Candidate Evaluation

### Candidate 1: serde-env

**Repository**: [crates.io/crates/serde-env](https://crates.io/crates/serde-env)
**Version**: 0.2.0
**License**: Apache-2.0

**Key Feature**: Deserializes `_` separated environment variables into nested structs.

```rust
// serde-env treats CARGO_HOME as { cargo: { home: "..." } }
#[derive(Deserialize)]
struct Config {
    cargo: CargoConfig,
}

#[derive(Deserialize)]
struct CargoConfig {
    home: String,
}

let config: Config = serde_env::from_env()?;
```

**API:**
- `from_env<T>()` - Deserialize from all env vars
- `from_env_with_prefix<T>("PREFIX")` - Filter by prefix
- `from_iter<T>(items)` - From iterator of key-value pairs

**Pros:**
- Clean serde integration
- Prefix support via `from_env_with_prefix`
- Small dependency footprint (serde, anyhow)

**Cons:**
- **Critical**: Uses `_` as separator, conflicting with `CARO_DEFAULT_SHELL` (would parse as nested `caro.default.shell`)
- No nested struct support with prefix (prefix is stripped before parsing)
- Less mature (0.2.0 version)
- 85% documentation coverage

**Compatibility Issue Example:**
```rust
// Current: CARO_DEFAULT_SHELL -> config.default_shell
// With serde-env: CARO_DEFAULT_SHELL -> parsed as caro.default.shell (3 levels!)

// Would require changing env vars to:
// CARO_DEFAULTSHELL (awkward)
// or CARO__DEFAULT_SHELL (double underscore prefix)
```

---

### Candidate 2: envy

**Repository**: [github.com/softprops/envy](https://github.com/softprops/envy)
**Version**: 0.4.x
**License**: MIT
**Stars**: 957 | **Dependents**: ~5,100

**Key Feature**: Flat mapping of env vars to struct fields.

```rust
#[derive(Deserialize)]
struct Config {
    default_shell: Option<ShellType>,  // reads DEFAULT_SHELL
    safety_level: SafetyLevel,          // reads SAFETY_LEVEL
}

let config: Config = envy::from_env()?;
let config: Config = envy::prefixed("CARO_").from_env()?;
```

**Pros:**
- Very mature and widely used
- Flat mapping matches Caro's struct
- Good prefix support
- Handles `Option<T>` for missing vars
- Comma-separated `Vec` support

**Cons:**
- No nested struct support (not needed for Caro)
- Field names must match env var names (case-insensitive conversion)
- Generic serde errors, not custom messages

---

### Candidate 3: envious

**Repository**: [crates.io/crates/envious](https://crates.io/crates/envious)
**Version**: 0.3.x
**License**: MIT/Apache-2.0

**Key Feature**: Configurable separator (`__` by default) for nested structs.

```rust
let config: Config = envious::Config::new()
    .with_prefix("CARO")
    .with_separator("__")
    .case_sensitive(false)
    .build_from_env()?;
```

**Pros:**
- Configurable separator (avoids `_` conflict)
- Prefix stripping
- Case sensitivity control
- Better nested struct handling than serde-env

**Cons:**
- More complex API
- Additional dependency complexity
- Overkill for flat configuration

---

### Candidate 4: Manual Approach (Current)

**Key Feature**: Explicit parsing with custom error handling.

**Pros:**
- Full control over error messages
- Clear validation points
- No additional dependencies
- Explicit env var documentation
- Type-specific parsing (custom FromStr implementations)

**Cons:**
- More boilerplate (5 if-let blocks currently)
- Must update when adding fields
- No automatic serde integration

---

## Recommendation

**Decision: Maintain manual approach with targeted improvements.**

### Rationale

1. **serde-env is not suitable** due to `_` separator conflict with `CARO_*` naming convention
2. **envy could work** but provides minimal benefit for 5 env vars
3. **Current implementation is clear** and provides excellent error messages:
   ```
   Invalid safety level 'foo'. Valid values: strict, moderate, permissive
   ```
4. **No nested configuration** - serde-env's primary advantage is irrelevant
5. **Dependency philosophy** - Caro prioritizes minimal dependencies (see ADR-001)

### When to Reconsider

Adopt a serde-based approach if:
- Environment variables grow beyond 10-15
- Nested configuration becomes necessary (e.g., `CARO_BACKEND_OLLAMA_URL`)
- Multiple configuration sources need unified serde-based loading

---

## Implementation Notes

### Potential Improvements to Current Approach

If the manual approach is retained, consider these enhancements:

#### 1. Extract env var reading to a macro (optional)

```rust
macro_rules! read_env {
    ($config:expr, $field:ident, $env_var:expr) => {
        if let Ok(val) = std::env::var($env_var) {
            $config.$field = val.parse().map_err(ConfigError::ValidationError)?;
        }
    };
}
```

#### 2. Document all env vars in one place

Create a constants module:
```rust
pub mod env_vars {
    pub const SAFETY_LEVEL: &str = "CARO_SAFETY_LEVEL";
    pub const DEFAULT_SHELL: &str = "CARO_DEFAULT_SHELL";
    pub const LOG_LEVEL: &str = "CARO_LOG_LEVEL";
    pub const DEFAULT_MODEL: &str = "CARO_DEFAULT_MODEL";
    pub const CACHE_MAX_SIZE_GB: &str = "CARO_CACHE_MAX_SIZE_GB";
}
```

#### 3. Add `--env-help` CLI flag

Show all environment variables with descriptions:
```
$ caro --env-help
Environment Variables:
  CARO_SAFETY_LEVEL      Safety validation level (strict|moderate|permissive)
  CARO_DEFAULT_SHELL     Target shell (bash|zsh|fish|sh|powershell|cmd)
  CARO_LOG_LEVEL         Log verbosity (debug|info|warn|error)
  CARO_DEFAULT_MODEL     Preferred LLM model identifier
  CARO_CACHE_MAX_SIZE_GB Maximum cache size in gigabytes (1-1000)
```

### If envy is adopted later

Migration path:
```rust
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(default)]
struct EnvConfig {
    safety_level: Option<SafetyLevel>,
    default_shell: Option<ShellType>,
    log_level: Option<LogLevel>,
    default_model: Option<String>,
    cache_max_size_gb: Option<u64>,
}

impl ConfigManager {
    pub fn merge_with_env(&self) -> Result<UserConfiguration, ConfigError> {
        let mut config = self.load()?;

        let env_config: EnvConfig = envy::prefixed("CARO_")
            .from_env()
            .map_err(|e| ConfigError::ValidationError(e.to_string()))?;

        if let Some(level) = env_config.safety_level {
            config.safety_level = level;
        }
        // ... apply other fields

        config.validate().map_err(ConfigError::ValidationError)?;
        Ok(config)
    }
}
```

---

## Consequences

### Benefits of This Decision

1. **No new dependencies** - keeps binary size minimal
2. **Clear error messages** - maintained custom validation feedback
3. **Explicit documentation** - env vars remain clearly documented in code
4. **No naming conflicts** - `CARO_DEFAULT_SHELL` works as expected
5. **Validation control** - business rules enforced at parse time

### Trade-offs

1. **Manual maintenance** - new env vars require code additions
2. **No serde integration** - env loading separate from TOML loading
3. **Boilerplate** - 5 similar if-let blocks

### Risks

1. **Scaling risk**: If env vars grow significantly, manual approach becomes unwieldy
   - **Mitigation**: Adopt envy if >10 env vars needed

2. **Inconsistency risk**: Env parsing differs from TOML parsing patterns
   - **Mitigation**: Document the precedence chain clearly (CLI > env > file > defaults)

---

## References

### Crates Evaluated

- [serde-env](https://crates.io/crates/serde-env) - Environment variable deserialization with `_` separator
- [envy](https://github.com/softprops/envy) - Flat environment variable deserialization
- [envious](https://crates.io/crates/envious) - Configurable environment variable deserialization
- [serde-envfile](https://docs.rs/serde-envfile) - .env file integration

### Related ADRs

- [ADR-001: LLM Inference Architecture](./001-llm-inference-architecture.md) - Dependency philosophy
- Current config implementation: `src/config/mod.rs`
- UserConfiguration model: `src/models/mod.rs`

---

## Revision History

| Date | Author | Changes |
|------|--------|---------|
| 2026-01-02 | Claude Code | Initial draft |
