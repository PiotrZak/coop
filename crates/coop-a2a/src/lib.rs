//! Coop A2A **infrastructure / presentation** — JSON-RPC HTTP edge.
//!
//! Domain: [`coop_domain`]. Application: [`coop_application`]. Identity AuthN: [`coop_identity`].

pub mod card;
pub mod client;
pub mod lindle;
pub mod models;
pub mod orchestrator_http;
pub mod proxy;
pub mod server;

pub use card::AgentCard;
pub use client::A2aClient;
pub use coop_application::{SkillRoute, SkillRouter};
pub use models::*;
pub use proxy::{build_gateway_card, forward, merge_skills};
