//! Shared config for one registered agent adapter.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentBackendKind {
    AgentsHttp,
    Mcp,
    A2aPeer,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuthStyle {
    Bearer,
    BearerAndLindleHeader,
    McpSecret,
}

/// `id` is the hub `target_agent`.
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
            .field("skills", &self.skills)
            .field("priority", &self.priority)
            .finish()
    }
}

impl AgentBackendConfig {
    pub fn supports(&self, skill: &str) -> bool {
        self.skills.iter().any(|s| s == skill)
    }
}
