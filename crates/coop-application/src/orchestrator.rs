//! Hub agent registry config.

use crate::backend::AgentBackendConfig;
use crate::backends::{lindle_mcp_from_env, lindle_orleans_from_env, peers_from_env};
use anyhow::Result;

#[derive(Debug, Clone, Default)]
pub struct OrchestratorConfig {
    pub backends: Vec<AgentBackendConfig>,
}

impl OrchestratorConfig {
    pub fn from_env() -> Result<Self> {
        let mut backends = Vec::new();
        if let Some(b) = lindle_mcp_from_env()? {
            backends.push(b);
        }
        if let Some(b) = lindle_orleans_from_env()? {
            backends.push(b);
        }
        backends.extend(peers_from_env()?);
        backends.sort_by_key(|b| b.priority);
        Ok(Self { backends })
    }

    pub fn push(&mut self, backend: AgentBackendConfig) {
        self.backends.push(backend);
        self.backends.sort_by_key(|b| b.priority);
    }

    pub fn by_id(&self, agent_id: &str) -> Option<&AgentBackendConfig> {
        self.backends.iter().find(|b| b.id == agent_id)
    }

    pub fn backend_for_skill(&self, skill: &str) -> Option<&AgentBackendConfig> {
        self.backends
            .iter()
            .filter(|b| b.supports(skill))
            .min_by_key(|b| b.priority)
    }
}
