//! Auth hub over registered [`AgentBackend`] adapters.

pub mod agent_backend;
pub mod card;
pub mod client;
pub mod http_base;
pub mod hub;
pub mod lindle;
pub mod mcp;
pub mod models;
pub mod orchestrator_http;
pub mod peer;

pub use agent_backend::{AgentBackend, AgentRegistry};
pub use card::AgentCard;
pub use client::A2aClient;
pub use hub::HubService;
pub use models::*;
