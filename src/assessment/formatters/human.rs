use crate::assessment::{AssessmentResult, Backend, GPUVendor};

/// Format assessment result in human-readable format with ASCII borders
pub fn format(result: &AssessmentResult) -> String {
    let mut output = String::new();

    output.push_str("═══════════════════════════════════════════════════\n");
    output.push_str("Caro System Assessment\n");
    output.push_str(&format!(
        "Generated: {}\n",
        result.timestamp.format("%Y-%m-%d %H:%M:%S UTC")
    ));
    output.push_str("═══════════════════════════════════════════════════\n\n");

    // System Information
    output.push_str("System Information:\n");
    let profile = &result.system_profile;
    output.push_str(&format!(
        "  CPU: {} ({} cores, {})\n",
        profile.cpu.model_name, profile.cpu.cores, profile.cpu.architecture
    ));
    if let Some(freq) = profile.cpu.frequency_mhz {
        output.push_str(&format!("  Frequency: {} MHz\n", freq));
    }
    output.push_str(&format!(
        "  Memory: {} MB total, {} MB available\n",
        profile.memory.total_mb, profile.memory.available_mb
    ));

    if let Some(gpu) = &profile.gpu {
        let vram = gpu
            .vram_mb
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
        profile.platform.os, profile.platform.arch
    ));

    // Recommendations
    if !result.recommendations.is_empty() {
        output.push_str("\nModel Recommendations:\n");
        for (i, rec) in result.recommendations.iter().enumerate() {
            output.push_str(&format!(
                "  {}. {} ({}) via {} backend\n",
                i + 1,
                rec.model_name,
                rec.model_size,
                match rec.backend {
                    Backend::MLX => "MLX",
                    Backend::CUDA => "CUDA",
                    Backend::CPU => "CPU",
                }
            ));
            output.push_str(&format!("     Reasoning: {}\n", rec.reasoning));
            output.push_str(&format!(
                "     Memory: ~{} MB",
                rec.estimated_memory_mb
            ));
            if let Some(quant) = &rec.quantization {
                output.push_str(&format!(", Quantization: {}", quant));
            }
            output.push_str("\n\n");
        }
    }

    // Warnings
    if !result.warnings.is_empty() {
        output.push_str("Warnings:\n");
        for warning in &result.warnings {
            output.push_str(&format!("  ⚠  {}\n", warning));
        }
        output.push('\n');
    }

    output.push_str("═══════════════════════════════════════════════════\n");

    output
}
