//! Coop A2A — thin JSON-RPC-shaped edge for Negotiation ↔ Negotiation
//! with optional Supplier Agent consult.
//!
//! Protocol shape follows A2A ideas (Agent Card, skills, tasks, artifacts).
//! AuthN is HTTP Bearer / API key per Agent Card `security` — never in the body.

pub mod card;
pub mod client;
pub mod models;
pub mod server;

pub use card::AgentCard;
pub use client::A2aClient;
pub use models::*;
