// Remote backend implementations for external model providers

pub mod ollama;
pub mod vllm;

pub use ollama::OllamaBackend;
pub use vllm::VllmBackend;
