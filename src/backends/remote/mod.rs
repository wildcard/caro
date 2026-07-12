// Remote backend implementations for external model providers

pub mod ai_horde;
pub mod claude;
pub mod exo;
pub mod mesh;
pub mod ollama;
pub mod vllm;

pub use ai_horde::AiHordeBackend;
pub use claude::ClaudeBackend;
pub use exo::ExoBackend;
pub use mesh::MeshBackend;
pub use ollama::OllamaBackend;
pub use vllm::VllmBackend;
