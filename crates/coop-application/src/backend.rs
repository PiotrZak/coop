//! Generic agent-backend config — base type all concrete backends specialize.
//!
//! There is no class inheritance: factories (e.g. Lindle Orleans / Lindle MCP)
//! fill this shared shape; runtime adapters implement a dispatch trait in `coop-a2a`.

use serde::{Deserialize, Serialize};

/// Protocol family (extensible). Product-specific factories pick a kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentBackendKind {
    /// JSON HTTP agent API (Lindle Orleans is one provider).
    AgentsHttp,
    /// MCP `tools/call` (Lindle Backend `/mcp` is one provider).
    Mcp,
}

/// How the shared secret is presented on the wire.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuthStyle {
    Bearer,
    /// Bearer + `X-Lindle-Agents-Secret` (Lindle Host).
    BearerAndLindleHeader,
    /// Bearer or `X-MCP-Secret` for MCP hosts.
    McpSecret,
}

/// Generic base config for **any** agent backend.
#[derive(Clone)]
pub struct AgentBackendConfig {
    pub id: String,
    pub kind: AgentBackendKind,
    pub base_url: String,
    pub auth_secret: String,
    pub auth_style: AuthStyle,
    pub health_path: String,
    pub default_tenant_id: String,
    pub skills: Vec<String>,
    /// Lower = preferred when multiple backends match a skill.
    pub priority: u32,
    pub required: bool,
}

impl std::fmt::Debug for AgentBackendConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AgentBackendConfig")
            .field("id", &self.id)
            .field("kind", &self.kind)
            .field("base_url", &self.base_url)
            .field("auth_secret", &"***")
            .field("auth_style", &self.auth_style)
            .field("health_path", &self.health_path)
            .field("default_tenant_id", &self.default_tenant_id)
            .field("skills", &self.skills)
            .field("priority", &self.priority)
            .field("required", &self.required)
            .finish()
    }
}

impl AgentBackendConfig {
    pub fn supports(&self, skill: &str) -> bool {
        self.skills.iter().any(|s| s == skill)
    }
}
