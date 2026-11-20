//! Application layer — orchestrator over [`coop_domain`].

pub mod backend;
pub mod backends;
mod orchestrator;

pub use backend::{AgentBackendConfig, AgentBackendKind, AuthStyle};
pub use backends::{LindleBackend, LindleMcpBackend, LindleOrleansBackend};
pub use orchestrator::OrchestratorConfig;
