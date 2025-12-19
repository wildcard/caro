// Remote backend implementations for external model providers

pub mod azure_foundry;
pub mod ollama;
pub mod vllm;

pub use azure_foundry::AzureFoundryBackend;
pub use ollama::OllamaBackend;
pub use vllm::VllmBackend;
