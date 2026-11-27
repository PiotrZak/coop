//! Runtime agent adapter trait + registry.

use crate::models::{JsonRpcRequest, JsonRpcResponse};
use anyhow::Result;
use coop_application::AgentBackendConfig;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

type BoxFut<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

pub trait AgentBackend: Send + Sync {
    fn config(&self) -> &AgentBackendConfig;
    fn id(&self) -> &str {
        &self.config().id
    }
    fn ensure_ready(&self) -> BoxFut<'_, Result<()>>;
    fn dispatch_skill<'a>(
        &'a self,
        skill: &'a str,
        req: &'a JsonRpcRequest,
    ) -> BoxFut<'a, Result<JsonRpcResponse>>;
}

#[derive(Clone, Default)]
pub struct AgentRegistry {
    agents: Vec<Arc<dyn AgentBackend>>,
}

impl AgentRegistry {
    pub fn new(agents: Vec<Arc<dyn AgentBackend>>) -> Self {
        let mut agents = agents;
        agents.sort_by_key(|b| b.config().priority);
        Self { agents }
    }

    pub fn is_empty(&self) -> bool {
        self.agents.is_empty()
    }

    pub fn get(&self, agent_id: &str) -> Option<Arc<dyn AgentBackend>> {
        self.agents.iter().find(|a| a.id() == agent_id).cloned()
    }

    pub fn for_skill(&self, skill: &str) -> Option<Arc<dyn AgentBackend>> {
        self.agents
            .iter()
            .filter(|b| b.config().supports(skill))
            .min_by_key(|b| b.config().priority)
            .cloned()
    }

    pub fn fallback_for_skill(
        &self,
        skill: &str,
        exclude_id: &str,
    ) -> Option<Arc<dyn AgentBackend>> {
        self.agents
            .iter()
            .filter(|b| b.config().supports(skill) && b.id() != exclude_id)
            .min_by_key(|b| b.config().priority)
            .cloned()
    }

    /// `target_agent` if set, else skill priority.
    pub fn resolve(
        &self,
        target_agent: Option<&str>,
        skill: &str,
    ) -> Result<Arc<dyn AgentBackend>, String> {
        if let Some(id) = target_agent {
            let agent = self
                .get(id)
                .ok_or_else(|| format!("unknown target_agent `{id}`"))?;
            if !agent.config().supports(skill) {
                return Err(format!("agent `{id}` does not support skill `{skill}`"));
            }
            return Ok(agent);
        }
        self.for_skill(skill)
            .ok_or_else(|| format!("no registered agent for skill `{skill}`"))
    }
}
