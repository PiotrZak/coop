//! Concrete backend factories — each fills [`crate::backend::AgentBackendConfig`].
//!
//! Pattern: specialize the generic config (composition), not subclass hierarchies.

mod lindle;
mod mcp;

pub use lindle::{LindleBackend, LindleOrleansBackend};
pub use mcp::LindleMcpBackend;
