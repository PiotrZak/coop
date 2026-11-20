//! Application layer — use cases + orchestrator over [`coop_domain`].

pub mod backend;
pub mod backends;
mod negotiation;
mod orchestrator;
pub mod routing;
mod supplier;

pub use backend::{AgentBackendConfig, AgentBackendKind, AuthStyle};
pub use backends::{LindleBackend, LocalA2aBackend};
pub use negotiation::build_negotiation_pack;
pub use orchestrator::OrchestratorConfig;
pub use routing::{SkillRoute, SkillRouter};
pub use supplier::supplier_brief;
