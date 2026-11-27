//! Application layer — hub agent registry config.

pub mod backend;
mod backends;
mod orchestrator;

pub use backend::{AgentBackendConfig, AgentBackendKind, AuthStyle};
pub use orchestrator::OrchestratorConfig;
