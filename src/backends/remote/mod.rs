// Remote backend implementations for external model providers

pub mod claude;
pub mod exo;
pub mod ollama;
pub mod vllm;

pub use claude::ClaudeBackend;
pub use exo::ExoBackend;
pub use ollama::OllamaBackend;
pub use vllm::VllmBackend;
