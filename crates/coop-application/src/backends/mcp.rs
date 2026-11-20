//! Lindle Backend MCP — one entity of the generic backend family.
//!
//! Env: `LINDLE_MCP_URL` + `MCP_SERVER_SECRET` (same as YCA → Lindle `/mcp`).

use crate::backend::{AgentBackendConfig, AgentBackendKind, AuthStyle};
use anyhow::{bail, Result};

pub struct LindleMcpBackend;

impl LindleMcpBackend {
    /// Optional MCP tool host. Prefer over Orleans for `supplier.read` (priority 0).
    pub fn from_env() -> Result<Option<AgentBackendConfig>> {
        let Ok(base_url) = std::env::var("LINDLE_MCP_URL") else {
            return Ok(None);
        };
        let base_url = base_url.trim().trim_end_matches('/').to_string();
        if base_url.is_empty() {
            return Ok(None);
        }

        let auth_secret = std::env::var("MCP_SERVER_SECRET").map_err(|_| {
            anyhow::anyhow!("LINDLE_MCP_URL is set but MCP_SERVER_SECRET is missing")
        })?;
        if auth_secret.trim().is_empty() {
            bail!("MCP_SERVER_SECRET must not be empty");
        }

        Ok(Some(AgentBackendConfig {
            id: "lindle-mcp".into(),
            kind: AgentBackendKind::LindleMcp,
            base_url,
            auth_secret,
            auth_style: AuthStyle::McpSecret,
            health_path: "/mcp".into(),
            default_tenant_id: "demo-tenant".into(),
            skills: vec!["supplier.read".into()],
            priority: 0,
            required: false,
        }))
    }
}
