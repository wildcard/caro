pub mod cpu;
pub mod formatters;
pub mod gpu;
pub mod memory;
pub mod profile;
pub mod recommender;
pub mod result;

pub use cpu::CPUInfo;
pub use gpu::{GPUInfo, GPUVendor};
pub use memory::MemoryInfo;
pub use profile::{AssessmentError, PlatformInfo, SystemProfile};
pub use recommender::{Backend, ModelRecommendation, Recommender};
pub use result::AssessmentResult;
