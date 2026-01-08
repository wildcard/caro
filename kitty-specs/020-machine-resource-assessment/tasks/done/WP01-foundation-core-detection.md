---
work_package_id: WP01
title: "Foundation - Core Detection"
priority: P1
phase: "setup"
subtasks: [T001, T002, T003, T004, T005, T006, T007, T008]
lane: "done"
review_status: ""
reviewed_by: ""
assignee: ""
agent: "claude"
shell_pid: "88455"
history:
  - 2026-01-08T00:00:00Z – spec-kitty-tasks – Generated work package prompt
---

# Work Package 01: Foundation - Core Detection

## Objective

Implement the foundational system profiling infrastructure with CPU and memory detection. This work package establishes the core assessment module structure and provides basic hardware detection using the `sysinfo` crate. By the end of this work package, users can run `caro assess` to see their CPU and memory information.

## Context

**User Story**: P1 - View System Resource Assessment (Acceptance Scenarios 1, 2)

**Why This Matters**: This is the foundation for the entire machine resource assessment feature. Without accurate CPU and memory detection, we cannot provide meaningful recommendations or help users understand their system capabilities.

**Technical Approach**:
- Use `sysinfo` crate (v0.30+) for cross-platform system information
- Create new `assessment` module parallel to existing `cli`, `inference`, etc. modules
- Follow existing caro patterns (see `src/doctor/` for similar diagnostic command)
- Implement basic CLI command following clap patterns used elsewhere in caro

## Implementation Guidance

### T001: Add sysinfo Dependency

**Location**: `Cargo.toml` (repository root)

Add `sysinfo` crate to dependencies:
```toml
[dependencies]
sysinfo = "0.30"  # or latest 0.30.x
```

### T002: Create Assessment Module Structure [P]

**Location**: `src/assessment/mod.rs`

Create new assessment module:
```rust
pub mod cpu;
pub mod memory;
pub mod profile;

pub use profile::SystemProfile;
pub use cpu::CPUInfo;
pub use memory::MemoryInfo;
```

**File Structure**:
- `src/assessment/mod.rs` - Module exports
- `src/assessment/profile.rs` - SystemProfile struct
- `src/assessment/cpu.rs` - CPU detection
- `src/assessment/memory.rs` - Memory detection

### T003: Implement SystemProfile Struct

**Location**: `src/assessment/profile.rs`

Create the main data structure for system information:
```rust
use serde::{Deserialize, Serialize};
use crate::assessment::{CPUInfo, MemoryInfo};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemProfile {
    pub cpu: CPUInfo,
    pub memory: MemoryInfo,
    pub platform: PlatformInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformInfo {
    pub os: String,
    pub arch: String,
}

impl SystemProfile {
    pub fn detect() -> Result<Self, AssessmentError> {
        let cpu = CPUInfo::detect()?;
        let memory = MemoryInfo::detect()?;
        let platform = PlatformInfo {
            os: std::env::consts::OS.to_string(),
            arch: std::env::consts::ARCH.to_string(),
        };

        Ok(SystemProfile { cpu, memory, platform })
    }
}

#[derive(Debug, thiserror::Error)]
pub enum AssessmentError {
    #[error("Failed to detect system information: {0}")]
    DetectionFailed(String),
}
```

### T004: Implement CPU Detection [P]

**Location**: `src/assessment/cpu.rs`

Implement CPU information detection:
```rust
use serde::{Deserialize, Serialize};
use sysinfo::System;
use crate::assessment::profile::AssessmentError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CPUInfo {
    pub architecture: String,
    pub cores: usize,
    pub model_name: String,
    pub frequency_mhz: Option<u64>,
}

impl CPUInfo {
    pub fn detect() -> Result<Self, AssessmentError> {
        let mut sys = System::new_all();
        sys.refresh_cpu();

        // Get CPU info from sysinfo
        let cpus = sys.cpus();
        if cpus.is_empty() {
            return Err(AssessmentError::DetectionFailed(
                "No CPUs detected".to_string()
            ));
        }

        let cpu = &cpus[0];

        Ok(CPUInfo {
            architecture: std::env::consts::ARCH.to_string(),
            cores: sys.cpus().len(),
            model_name: cpu.brand().to_string(),
            frequency_mhz: Some(cpu.frequency()),
        })
    }
}
```

### T005: Implement Memory Detection [P]

**Location**: `src/assessment/memory.rs`

Implement memory information detection:
```rust
use serde::{Deserialize, Serialize};
use sysinfo::System;
use crate::assessment::profile::AssessmentError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryInfo {
    pub total_mb: u64,
    pub available_mb: u64,
}

impl MemoryInfo {
    pub fn detect() -> Result<Self, AssessmentError> {
        let mut sys = System::new_all();
        sys.refresh_memory();

        let total_mb = sys.total_memory() / 1024 / 1024;
        let available_mb = sys.available_memory() / 1024 / 1024;

        if total_mb == 0 {
            return Err(AssessmentError::DetectionFailed(
                "Could not detect system memory".to_string()
            ));
        }

        Ok(MemoryInfo {
            total_mb,
            available_mb,
        })
    }
}
```

### T006: Create CLI Command Handler

**Location**: `src/cli/commands/assess.rs`

Create the `caro assess` command handler:
```rust
use clap::Parser;
use crate::assessment::SystemProfile;

#[derive(Debug, Parser)]
pub struct AssessCommand {
    // Future: --export, --output flags will go here
}

impl AssessCommand {
    pub fn execute(&self) -> anyhow::Result<()> {
        let profile = SystemProfile::detect()
            .map_err(|e| anyhow::anyhow!("Assessment failed: {}", e))?;

        print_profile(&profile);

        Ok(())
    }
}

fn print_profile(profile: &SystemProfile) {
    println!("═══════════════════════════════════════════════════");
    println!("Caro System Assessment");
    println!("═══════════════════════════════════════════════════");
    println!();
    println!("System Information:");
    println!("  CPU: {} ({} cores, {})",
        profile.cpu.model_name,
        profile.cpu.cores,
        profile.cpu.architecture
    );
    println!("  Memory: {} MB total, {} MB available",
        profile.memory.total_mb,
        profile.memory.available_mb
    );
    println!("  Platform: {} ({})",
        profile.platform.os,
        profile.platform.arch
    );
    println!();
    println!("═══════════════════════════════════════════════════");
}
```

### T007: Register Command in CLI

**Location**: `src/cli/mod.rs` (or wherever commands are registered)

Add the assess command to the CLI:
```rust
use crate::cli::commands::assess::AssessCommand;

#[derive(Debug, Parser)]
pub enum Command {
    // ... existing commands
    Assess(AssessCommand),
}

impl Command {
    pub fn execute(self) -> anyhow::Result<()> {
        match self {
            // ... existing command handlers
            Command::Assess(cmd) => cmd.execute(),
        }
    }
}
```

Also ensure the commands module exports assess:
```rust
pub mod assess;
```

### T008: Update Main Entry Point

**Location**: `src/main.rs`

Ensure the assessment module is imported:
```rust
mod assessment;
```

## Definition of Done

- [ ] `sysinfo` dependency added to Cargo.toml
- [ ] Assessment module structure created (`mod.rs`, `profile.rs`, `cpu.rs`, `memory.rs`)
- [ ] `SystemProfile`, `CPUInfo`, and `MemoryInfo` structs implemented
- [ ] CPU detection works and returns architecture, cores, model name
- [ ] Memory detection works and returns total/available RAM
- [ ] `caro assess` command runs without errors
- [ ] Output displays CPU and memory info in human-readable format
- [ ] Works on macOS (tested locally)
- [ ] Code follows existing caro style (checked with `cargo clippy`)

## Test Strategy

**Manual Testing**:
1. Run `cargo build` to ensure compilation
2. Run `./target/debug/caro assess` to test the command
3. Verify output shows:
   - CPU model name and core count
   - Total and available memory
   - Platform OS and architecture
4. Check output format matches the example in plan.md

**Cross-Platform Verification**:
- If possible, test on Linux or Windows via CI/CD
- Otherwise, document that testing occurred on macOS only

## Integration Points

- **sysinfo crate**: Primary dependency for system detection
- **clap**: Used for CLI argument parsing (existing pattern)
- **thiserror**: Used for error types (existing pattern)
- **serde/serde_json**: Used for serialization (existing pattern)

## Risks & Mitigation

**Risk**: sysinfo may not detect all CPU features consistently
- **Mitigation**: Focus on core fields (arch, cores, model), handle optional fields gracefully

**Risk**: Memory detection may vary by platform
- **Mitigation**: Test on multiple platforms in CI/CD

## Reviewer Guidance

When reviewing this work package, verify:
1. **Code Quality**: Follows existing caro patterns and style
2. **Error Handling**: Graceful fallbacks for missing data
3. **Output Format**: Matches specification (ASCII borders, clear labels)
4. **Documentation**: Inline comments explain platform-specific behavior
5. **Testing**: Manual testing confirms detection works

**Key Questions**:
- Does the output match the format in plan.md?
- Are errors handled gracefully (no panics)?
- Is the code consistent with existing caro modules?

## Activity Log

- 2026-01-08T00:00:00Z – spec-kitty-tasks – Generated initial work package prompt
- 2026-01-08T18:29:34Z – claude – shell_pid=61049 – lane=doing – Started implementation of foundation module
- 2026-01-08T18:36:31Z – claude – shell_pid=67748 – lane=for_review – Completed implementation - all subtasks done, build succeeds
- 2026-01-08T18:58:42Z – claude – shell_pid=88455 – lane=done – Reviewed and approved - all subtasks completed, tests passing
