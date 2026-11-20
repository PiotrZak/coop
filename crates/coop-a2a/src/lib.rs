//! Coop A2A **infrastructure / presentation** — JSON-RPC HTTP edge.
//!
//! Runtime pattern: generic [`AgentBackend`] trait; Lindle Orleans / MCP are implementations.

pub mod agent_backend;
pub mod card;
pub mod client;
pub mod http_base;
pub mod lindle;
pub mod mcp;
pub mod models;
pub mod orchestrator_http;

pub use agent_backend::{AgentBackend, BackendRegistry};
pub use card::AgentCard;
pub use client::A2aClient;
pub use models::*;
