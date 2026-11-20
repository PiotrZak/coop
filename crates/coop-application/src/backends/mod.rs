//! Concrete backend entities — each specializes [`crate::backend::AgentBackendConfig`].

mod lindle;
mod mcp;

pub use lindle::LindleBackend;
pub use mcp::LindleMcpBackend;
