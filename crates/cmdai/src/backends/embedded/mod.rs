// Embedded model backends for offline command generation

// Platform-specific MLX backend (Apple Silicon only)
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
pub mod mlx;

// Cross-platform CPU backend
pub mod cpu;

// Common types and traits
mod common;

// Main embedded model backend
mod embedded_backend;

// Re-export common types
pub use common::{EmbeddedConfig, InferenceBackend, ModelVariant};

// Re-export main backend
pub use embedded_backend::EmbeddedModelBackend;

// Re-export platform-specific backend
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
pub use mlx::MlxBackend;

pub use cpu::CpuBackend;
