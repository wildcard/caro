// Sandbox execution backends for command evaluation

pub mod sandbox;
pub mod local;
pub mod docker;
pub mod executor;

pub use sandbox::{Sandbox, SandboxError, ExecutionContext, ExecutionOutput};
pub use local::LocalSandbox;
pub use docker::DockerSandbox;
pub use executor::SandboxExecutor;
