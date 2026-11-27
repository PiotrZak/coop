//! Env factories — each returns [`AgentBackendConfig`] for the hub registry.

use crate::backend::{AgentBackendConfig, AgentBackendKind, AuthStyle};
use anyhow::{bail, Context, Result};
use serde::Deserialize;

fn url_secret(url_var: &str, secret_var: &str) -> Result<Option<(String, String)>> {
    let Ok(url) = std::env::var(url_var) else {
        return Ok(None);
    };
    let url = url.trim().trim_end_matches('/').to_string();
    if url.is_empty() {
        return Ok(None);
    }
    let secret = std::env::var(secret_var)
        .with_context(|| format!("{url_var} set but {secret_var} missing"))?;
    if secret.trim().is_empty() {
        bail!("{secret_var} must not be empty");
    }
    Ok(Some((url, secret)))
}

pub fn lindle_orleans_from_env() -> Result<Option<AgentBackendConfig>> {
    let Some((base_url, auth_secret)) = url_secret("AGENTS_SERVICE_URL", "AGENTS_INTERNAL_SECRET")?
    else {
        return Ok(None);
    };
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

pub fn lindle_mcp_from_env() -> Result<Option<AgentBackendConfig>> {
    let Some((base_url, auth_secret)) = url_secret("LINDLE_MCP_URL", "MCP_SERVER_SECRET")? else {
        return Ok(None);
    };
    Ok(Some(AgentBackendConfig {
        id: "lindle-mcp".into(),
        kind: AgentBackendKind::Mcp,
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

#[derive(Deserialize)]
struct PeerJson {
    id: String,
    url: String,
    #[serde(default)]
    secret: String,
    #[serde(default)]
    skills: Option<Vec<String>>,
    #[serde(default)]
    priority: Option<u32>,
    #[serde(default)]
    required: Option<bool>,
}

/// `COOP_PEERS_JSON` = `[{"id":"agent-a","url":"http://…","secret":"…"}]`
pub fn peers_from_env() -> Result<Vec<AgentBackendConfig>> {
    let Ok(raw) = std::env::var("COOP_PEERS_JSON") else {
        return Ok(Vec::new());
    };
    let raw = raw.trim();
    if raw.is_empty() {
        return Ok(Vec::new());
    }
    let peers: Vec<PeerJson> = serde_json::from_str(raw).context("parse COOP_PEERS_JSON")?;
    peers
        .into_iter()
        .map(|p| {
            let id = p.id.trim().to_string();
            if id.is_empty() {
                bail!("COOP_PEERS_JSON entry missing id");
            }
            let base_url = p.url.trim().trim_end_matches('/').to_string();
            if base_url.is_empty() {
                bail!("peer `{id}` missing url");
            }
            Ok(AgentBackendConfig {
                id,
                kind: AgentBackendKind::A2aPeer,
                base_url,
                auth_secret: p.secret,
                auth_style: AuthStyle::Bearer,
                health_path: "/.well-known/agent.json".into(),
                default_tenant_id: "shared".into(),
                skills: p.skills.unwrap_or_else(|| vec!["peer.message".into()]),
                priority: p.priority.unwrap_or(50),
                required: p.required.unwrap_or(false),
            })
        })
        .collect()
}
