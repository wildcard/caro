pub mod cpu;
pub mod memory;
pub mod profile;

pub use cpu::CPUInfo;
pub use memory::MemoryInfo;
pub use profile::{AssessmentError, PlatformInfo, SystemProfile};

/// Run system resource assessment and display recommendations
pub async fn run_assessment() -> Result<(), AssessmentError> {
    let profile = SystemProfile::detect()?;
    print_profile(&profile);
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
    println!(
        "  Platform: {} ({})",
        profile.platform.os, profile.platform.arch
    );
    println!();
    println!("═══════════════════════════════════════════════════");
}
