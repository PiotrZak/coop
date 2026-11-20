//! Boot helpers: connect generic backends into a [`BackendRegistry`].

use crate::agent_backend::{AgentBackend, BackendRegistry};
use crate::card::{AgentCard, Skill};
use crate::lindle::LindleOrleansClient;
use crate::mcp::LindleMcpClient;
use anyhow::{bail, Result};
use coop_application::{AgentBackendConfig, AgentBackendKind, OrchestratorConfig};
use coop_domain::Scope;
use std::sync::Arc;

/// Build runtime adapters from config and connect (required must succeed).
pub async fn connect_backends(orch: &OrchestratorConfig) -> Result<BackendRegistry> {
    let mut connected: Vec<Arc<dyn AgentBackend>> = Vec::new();

    for cfg in &orch.backends {
        let backend: Arc<dyn AgentBackend> = match cfg.kind {
            AgentBackendKind::AgentsHttp => Arc::new(LindleOrleansClient::new(cfg.clone())),
            AgentBackendKind::Mcp => Arc::new(LindleMcpClient::new(cfg.clone())),
        };

        match backend.ensure_ready().await {
            Ok(()) => connected.push(backend),
            Err(e) if cfg.required => {
                return Err(anyhow::anyhow!(
                    "required backend `{}` failed: {e}",
                    cfg.id
                ));
            }
            Err(e) => {
                tracing::warn!(id = %cfg.id, error = %e, "optional backend down");
            }
        }
    }

    if connected.is_empty() {
        bail!("no agent backends connected — set AGENTS_SERVICE_URL and/or LINDLE_MCP_URL");
    }

    Ok(BackendRegistry::new(connected))
}

pub fn build_card(public_url: &str, orch: &OrchestratorConfig) -> AgentCard {
    let mut skills: Vec<Skill> = Vec::new();
    for cfg in &orch.backends {
        push_skills(&mut skills, cfg);
    }
    let mut card = AgentCard::gateway(public_url, skills);
    card.description =
        "Coop A2A edge — foreign agents → generic AgentBackend adapters (Lindle Orleans / MCP)"
            .into();
    card
}

fn push_skills(skills: &mut Vec<Skill>, cfg: &AgentBackendConfig) {
    for skill_id in &cfg.skills {
        if skills.iter().any(|s| s.id == *skill_id) {
            continue;
        }
        skills.push(Skill {
            id: skill_id.clone(),
            name: skill_id.clone(),
            description: format!("via backend `{}` ({:?})", cfg.id, cfg.kind),
            scopes: Scope::from_skill(skill_id).into_iter().collect(),
        });
    }
}
