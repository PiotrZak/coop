//! Orchestrator registry — ordered list of generic [`AgentBackendConfig`] entities.

use crate::backend::{AgentBackendConfig, AgentBackendKind};
use crate::backends::LindleBackend;
use anyhow::Result;

#[derive(Debug, Clone, Default)]
pub struct OrchestratorConfig {
    pub backends: Vec<AgentBackendConfig>,
}

impl OrchestratorConfig {
    /// Loads Lindle when `AGENTS_SERVICE_URL` + `AGENTS_INTERNAL_SECRET` are set.
    pub fn from_env() -> Result<Self> {
        let mut backends = Vec::new();
        if let Some(lindle) = LindleBackend::from_env()? {
            backends.push(lindle);
        }
        backends.sort_by_key(|b| b.priority);
        Ok(Self { backends })
    }

    pub fn push(&mut self, backend: AgentBackendConfig) {
        self.backends.push(backend);
        self.backends.sort_by_key(|b| b.priority);
    }

    pub fn of_kind(&self, kind: AgentBackendKind) -> Option<&AgentBackendConfig> {
        self.backends.iter().find(|b| b.kind == kind)
    }

    pub fn backend_for_skill(&self, skill: &str) -> Option<&AgentBackendConfig> {
        self.backends
            .iter()
            .filter(|b| b.supports(skill))
            .min_by_key(|b| b.priority)
    }

    pub fn required(&self) -> impl Iterator<Item = &AgentBackendConfig> {
        self.backends.iter().filter(|b| b.required)
    }
}
