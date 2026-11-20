//! Coop A2A **infrastructure / presentation** — JSON-RPC HTTP edge.
//!
//! Domain: [`coop_domain`]. Application: [`coop_application`]. Identity AuthN: [`coop_identity`].

pub mod card;
pub mod client;
pub mod lindle;
pub mod mcp;
pub mod models;
pub mod orchestrator_http;

pub use card::AgentCard;
pub use client::A2aClient;
pub use models::*;
