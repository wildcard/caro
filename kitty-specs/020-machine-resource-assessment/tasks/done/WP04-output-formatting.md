---
work_package_id: WP04
title: "Output Formatting & Export"
priority: P2
phase: "polish"
subtasks: [T020, T021, T022, T023, T024, T025, T026]
lane: "done"
review_status: ""
reviewed_by: ""
assignee: "claude"
agent: "claude"
shell_pid: "89558"
history:
  - 2026-01-08T00:00:00Z – spec-kitty-tasks – Generated initial work package prompt
---

# Work Package 04: Output Formatting & Export

## Objective

Implement polished output formatting and export functionality. This work package adds human-readable formatting with ASCII borders, JSON export for programmatic access, and Markdown export for documentation purposes.

## Context

**User Story**: P3 - Export Assessment for Troubleshooting (All 3 acceptance scenarios)

**Why This Matters**: Users need to share assessment results with support teams or save them for reference. Multiple export formats serve different use cases: human-readable for quick viewing, JSON for scripting, Markdown for documentation.

## Implementation Guidance

### T020: Create AssessmentResult Wrapper

**Location**: `src/assessment/result.rs`

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use crate::assessment::{SystemProfile, ModelRecommendation};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssessmentResult {
    pub timestamp: DateTime<Utc>,
    pub system_profile: SystemProfile,
    pub recommendations: Vec<ModelRecommendation>,
    pub warnings: Vec<String>,
}

impl AssessmentResult {
    pub fn new(
        profile: SystemProfile,
        recommendations: Vec<ModelRecommendation>,
        warnings: Vec<String>,
    ) -> Self {
        AssessmentResult {
            timestamp: Utc::now(),
            system_profile: profile,
            recommendations,
            warnings,
        }
    }
}
```

### T021: Implement Human-Readable Formatter [P]

**Location**: `src/assessment/formatters/human.rs`

```rust
use crate::assessment::AssessmentResult;

pub fn format(result: &AssessmentResult) -> String {
    let mut output = String::new();

    output.push_str("═══════════════════════════════════════════════════\n");
    output.push_str("Caro System Assessment\n");
    output.push_str("═══════════════════════════════════════════════════\n\n");

    // System Information
    output.push_str("System Information:\n");
    let profile = &result.system_profile;
    output.push_str(&format!(
        "  CPU: {} ({} cores, {})\n",
        profile.cpu.model_name,
        profile.cpu.cores,
        profile.cpu.architecture
    ));
    output.push_str(&format!(
        "  Memory: {} MB total, {} MB available\n",
        profile.memory.total_mb,
        profile.memory.available_mb
    ));

    if let Some(gpu) = &profile.gpu {
        let vram = gpu.vram_mb
            .map(|v| format!("{} MB", v))
            .unwrap_or_else(|| "unified memory".to_string());
        output.push_str(&format!(
            "  GPU: {} {} ({})\n",
            match gpu.vendor {
                GPUVendor::Apple => "Apple",
                GPUVendor::NVIDIA => "NVIDIA",
                GPUVendor::AMD => "AMD",
                GPUVendor::Intel => "Intel",
                GPUVendor::Unknown => "Unknown",
            },
            gpu.model,
            vram
        ));
    } else {
        output.push_str("  GPU: No GPU detected\n");
    }

    output.push_str(&format!(
        "  Platform: {} ({})\n",
        profile.platform.os,
        profile.platform.arch
    ));

    // Recommendations
    if !result.recommendations.is_empty() {
        output.push_str("\nModel Recommendations:\n");
        for rec in &result.recommendations {
            output.push_str(&format!(
                "  ✓ {} ({}) via {} backend\n",
                rec.model_name,
                rec.model_size,
                match rec.backend {
                    Backend::MLX => "MLX",
                    Backend::CUDA => "CUDA",
                    Backend::CPU => "CPU",
                }
            ));
            output.push_str(&format!("    Reasoning: {}\n", rec.reasoning));
            output.push_str(&format!("    Memory: ~{} MB", rec.estimated_memory_mb));
            if let Some(quant) = &rec.quantization {
                output.push_str(&format!(", Quantization: {}", quant));
            }
            output.push_str("\n\n");
        }
    }

    // Warnings
    if !result.warnings.is_empty() {
        output.push_str("\nWarnings:\n");
        for warning in &result.warnings {
            output.push_str(&format!("  ⚠ {}\n", warning));
        }
    } else {
        output.push_str("\nWarnings:\n  (none)\n");
    }

    output.push_str("\n═══════════════════════════════════════════════════\n");

    output
}
```

### T022: Implement JSON Export [P]

**Location**: `src/assessment/formatters/json.rs`

```rust
use crate::assessment::AssessmentResult;

pub fn format(result: &AssessmentResult) -> Result<String, serde_json::Error> {
    serde_json::to_string_pretty(result)
}
```

### T023: Implement Markdown Export [P]

**Location**: `src/assessment/formatters/markdown.rs`

```rust
use crate::assessment::AssessmentResult;

pub fn format(result: &AssessmentResult) -> String {
    let mut output = String::new();

    output.push_str("# Caro System Assessment\n\n");
    output.push_str(&format!("**Date**: {}\n\n", result.timestamp.format("%Y-%m-%d %H:%M:%S UTC")));

    // System Information
    output.push_str("## System Information\n\n");
    let profile = &result.system_profile;
    output.push_str(&format!(
        "- **CPU**: {} ({} cores, {})\n",
        profile.cpu.model_name,
        profile.cpu.cores,
        profile.cpu.architecture
    ));
    output.push_str(&format!(
        "- **Memory**: {} MB total, {} MB available\n",
        profile.memory.total_mb,
        profile.memory.available_mb
    ));

    if let Some(gpu) = &profile.gpu {
        output.push_str(&format!("- **GPU**: {} {}\n", gpu.vendor, gpu.model));
        if let Some(vram) = gpu.vram_mb {
            output.push_str(&format!("- **VRAM**: {} MB\n", vram));
        }
    } else {
        output.push_str("- **GPU**: No GPU detected\n");
    }

    output.push_str(&format!(
        "- **Platform**: {} ({})\n\n",
        profile.platform.os,
        profile.platform.arch
    ));

    // Recommendations
    if !result.recommendations.is_empty() {
        output.push_str("## Model Recommendations\n\n");
        for rec in &result.recommendations {
            output.push_str(&format!("### {} ({})\n\n", rec.model_name, rec.model_size));
            output.push_str(&format!("- **Backend**: {}\n", rec.backend));
            if let Some(quant) = &rec.quantization {
                output.push_str(&format!("- **Quantization**: {}\n", quant));
            }
            output.push_str(&format!("- **Estimated Memory**: {} MB\n", rec.estimated_memory_mb));
            output.push_str(&format!("- **Reasoning**: {}\n\n", rec.reasoning));
        }
    }

    // Warnings
    if !result.warnings.is_empty() {
        output.push_str("## Warnings\n\n");
        for warning in &result.warnings {
            output.push_str(&format!("- {}\n", warning));
        }
    }

    output
}
```

### T024: Add CLI Flags

**Location**: `src/cli/commands/assess.rs`

Update command struct:
```rust
#[derive(Debug, Parser)]
pub struct AssessCommand {
    /// Export format (json, markdown)
    #[arg(long, value_enum)]
    export: Option<ExportFormat>,

    /// Output file path
    #[arg(long, short = 'o')]
    output: Option<PathBuf>,

    /// Show only recommendations
    #[arg(long)]
    recommendations_only: bool,
}

#[derive(Debug, Clone, clap::ValueEnum)]
enum ExportFormat {
    Json,
    Markdown,
}
```

### T025-T026: Implement Export Logic

Update execute method:
```rust
impl AssessCommand {
    pub fn execute(&self) -> anyhow::Result<()> {
        let profile = SystemProfile::detect()
            .map_err(|e| anyhow::anyhow!("Assessment failed: {}", e))?;

        let recommendations = Recommender::recommend(&profile);
        let warnings = vec![];  // Collect any warnings during detection

        let result = AssessmentResult::new(profile, recommendations, warnings);

        if let Some(format) = &self.export {
            self.export_result(&result, format)?;
        } else {
            let formatted = human::format(&result);
            println!("{}", formatted);
        }

        Ok(())
    }

    fn export_result(&self, result: &AssessmentResult, format: &ExportFormat) -> anyhow::Result<()> {
        let content = match format {
            ExportFormat::Json => json::format(result)?,
            ExportFormat::Markdown => markdown::format(result),
        };

        if let Some(path) = &self.output {
            std::fs::write(path, content)?;
            println!("Assessment exported to: {}", path.display());
        } else {
            println!("{}", content);
        }

        Ok(())
    }
}
```

## Definition of Done

- [ ] AssessmentResult wrapper struct created
- [ ] Human-readable formatter with ASCII borders
- [ ] JSON export generates valid JSON
- [ ] Markdown export generates formatted markdown
- [ ] `--export json` flag works
- [ ] `--export markdown` flag works
- [ ] `--output <file>` flag writes to file
- [ ] Default output uses human-readable format
- [ ] All formatters include timestamp, profile, recommendations

## Activity Log

- 2026-01-08T00:00:00Z – spec-kitty-tasks – Generated initial work package prompt
- 2026-01-08T18:44:10Z – claude – shell_pid=74124 – lane=doing – Starting output formatting and export implementation
- 2026-01-08T18:49:55Z – claude – shell_pid=79173 – lane=for_review – Output formatting and export complete
- 2026-01-08T18:59:27Z – claude – shell_pid=89558 – lane=done – Reviewed and approved - formatters complete
