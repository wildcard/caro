pub mod cpu;
pub mod gpu;
pub mod memory;
pub mod profile;
pub mod recommender;

pub use cpu::CPUInfo;
pub use gpu::{GPUInfo, GPUVendor};
pub use memory::MemoryInfo;
pub use profile::{AssessmentError, PlatformInfo, SystemProfile};
pub use recommender::{Backend, ModelRecommendation, Recommender};

/// Run system resource assessment and display recommendations
pub async fn run_assessment() -> Result<(), AssessmentError> {
    let profile = SystemProfile::detect()?;
    let recommendations = Recommender::recommend(&profile);

    print_profile(&profile);
    print_recommendations(&recommendations);

    Ok(())
}

/// Print system profile information to stdout
fn print_profile(profile: &SystemProfile) {
    println!("═══════════════════════════════════════════════════");
    println!("Caro System Assessment");
    println!("═══════════════════════════════════════════════════");
    println!();
    println!("System Information:");
    println!(
        "  CPU: {} ({} cores, {})",
        profile.cpu.model_name, profile.cpu.cores, profile.cpu.architecture
    );
    if let Some(freq) = profile.cpu.frequency_mhz {
        println!("  Frequency: {} MHz", freq);
    }
    println!(
        "  Memory: {} MB total, {} MB available",
        profile.memory.total_mb, profile.memory.available_mb
    );
    if let Some(gpu) = &profile.gpu {
        if let Some(vram) = gpu.vram_mb {
            println!("  GPU: {} {} ({} MB VRAM)", gpu.vendor, gpu.model, vram);
        } else {
            println!("  GPU: {} {}", gpu.vendor, gpu.model);
        }
    } else {
        println!("  GPU: No GPU detected");
    }
    println!(
        "  Platform: {} ({})",
        profile.platform.os, profile.platform.arch
    );
    println!();
    println!("═══════════════════════════════════════════════════");
}

/// Print model recommendations to stdout
fn print_recommendations(recommendations: &[ModelRecommendation]) {
    println!();
    println!("Recommended Models:");
    println!("═══════════════════════════════════════════════════");
    println!();

    for (i, rec) in recommendations.iter().enumerate() {
        println!("{}. {} ({})", i + 1, rec.model_name, rec.model_size);
        println!("   Backend: {}", rec.backend);
        if let Some(quant) = &rec.quantization {
            println!("   Quantization: {}", quant);
        }
        println!("   Estimated Memory: {} MB", rec.estimated_memory_mb);
        println!("   Reason: {}", rec.reasoning);
        println!();
    }

    println!("═══════════════════════════════════════════════════");
}
