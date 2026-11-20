//! Lindle Orleans — specializes generic [`AgentBackendConfig`] as [`AgentBackendKind::AgentsHttp`].

use crate::backend::{AgentBackendConfig, AgentBackendKind, AuthStyle};
use anyhow::{bail, Result};

/// Factory: Lindle Orleans Agents → generic HTTP agent backend config.
pub struct LindleOrleansBackend;

impl LindleOrleansBackend {
    /// If `AGENTS_SERVICE_URL` is set, build a required AgentsHttp backend.
    pub fn from_env() -> Result<Option<AgentBackendConfig>> {
        let Ok(base_url) = std::env::var("AGENTS_SERVICE_URL") else {
            return Ok(None);
        };
        let base_url = base_url.trim().trim_end_matches('/').to_string();
        if base_url.is_empty() {
            return Ok(None);
        }

        let auth_secret = std::env::var("AGENTS_INTERNAL_SECRET").map_err(|_| {
            anyhow::anyhow!("AGENTS_SERVICE_URL is set but AGENTS_INTERNAL_SECRET is missing")
        })?;
        if auth_secret.trim().is_empty() {
            bail!("AGENTS_INTERNAL_SECRET must not be empty");
        }

        Ok(Some(AgentBackendConfig {
            id: "lindle-orleans".into(),
            kind: AgentBackendKind::AgentsHttp,
            base_url,
            auth_secret,
            auth_style: AuthStyle::BearerAndLindleHeader,
            health_path: "/health".into(),
            default_tenant_id: "demo-tenant".into(),
            skills: vec![
                "negotiate".into(),
                "investigate".into(),
                "supplier.read".into(),
            ],
            priority: 10,
            required: true,
        }))
    }
}

/// Backward-compatible alias.
pub type LindleBackend = LindleOrleansBackend;
