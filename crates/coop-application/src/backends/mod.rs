//! Concrete backend entities — each specializes [`crate::backend::AgentBackendConfig`].

mod lindle;
mod local;

pub use lindle::LindleBackend;
pub use local::LocalA2aBackend;
