use crate::assessment::AssessmentResult;

/// Format assessment result as Markdown document
pub fn format(result: &AssessmentResult) -> String {
    let mut output = String::new();

    output.push_str("# Caro System Assessment\n\n");
    output.push_str(&format!(
        "**Date**: {}\n\n",
        result.timestamp.format("%Y-%m-%d %H:%M:%S UTC")
    ));

    // System Information
    output.push_str("## System Information\n\n");
    let profile = &result.system_profile;
    output.push_str(&format!(
        "- **CPU**: {} ({} cores, {})\n",
        profile.cpu.model_name, profile.cpu.cores, profile.cpu.architecture
    ));
    if let Some(freq) = profile.cpu.frequency_mhz {
        output.push_str(&format!("- **Frequency**: {} MHz\n", freq));
    }
    output.push_str(&format!(
        "- **Memory**: {} MB total, {} MB available\n",
        profile.memory.total_mb, profile.memory.available_mb
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
        profile.platform.os, profile.platform.arch
    ));

    // Recommendations
    if !result.recommendations.is_empty() {
        output.push_str("## Model Recommendations\n\n");
        for rec in &result.recommendations {
            output.push_str(&format!(
                "### {} ({})\n\n",
                rec.model_name, rec.model_size
            ));
            output.push_str(&format!("- **Backend**: {}\n", rec.backend));
            if let Some(quant) = &rec.quantization {
                output.push_str(&format!("- **Quantization**: {}\n", quant));
            }
            output.push_str(&format!(
                "- **Estimated Memory**: {} MB\n",
                rec.estimated_memory_mb
            ));
            output.push_str(&format!("- **Reasoning**: {}\n\n", rec.reasoning));
        }
    }

    // Warnings
    if !result.warnings.is_empty() {
        output.push_str("## Warnings\n\n");
        for warning in &result.warnings {
            output.push_str(&format!("- ⚠️  {}\n", warning));
        }
        output.push('\n');
    }

    output
}
