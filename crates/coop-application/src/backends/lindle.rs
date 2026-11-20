//! Lindle Orleans — one entity of the generic backend family.
//!
//! Env (minimum): `AGENTS_SERVICE_URL` + `AGENTS_INTERNAL_SECRET`.

use crate::backend::{AgentBackendConfig, AgentBackendKind, AuthStyle};
use anyhow::{bail, Result};

pub struct LindleBackend;

impl LindleBackend {
    /// If `AGENTS_SERVICE_URL` is set, build a required Lindle backend (secret mandatory).
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
            kind: AgentBackendKind::Lindle,
            base_url,
            auth_secret,
            auth_style: AuthStyle::BearerAndLindleHeader,
            health_path: "/health".into(),
            default_tenant_id: "demo-tenant".into(),
            skills: vec![
                "negotiate".into(),
                "supplier.read".into(),
                "investigate".into(),
            ],
            priority: 0,
            required: true,
        }))
    }
}
