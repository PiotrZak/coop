//! HTTP boot helpers for the Coop orchestrator.

use crate::card::{AgentCard, Skill};
use crate::lindle::LindleAgentsClient;
use crate::mcp::LindleMcpClient;
use anyhow::{bail, Result};
use coop_application::{AgentBackendConfig, AgentBackendKind, OrchestratorConfig};
use coop_domain::Scope;

pub struct ConnectedBackends {
    pub orleans: Option<LindleAgentsClient>,
    pub mcp: Option<LindleMcpClient>,
}

/// Connect required Orleans first; MCP is optional.
pub async fn connect_backends(orch: &OrchestratorConfig) -> Result<ConnectedBackends> {
    let mut orleans = None;
    let mut mcp = None;

    for cfg in orch.required() {
        match cfg.kind {
            AgentBackendKind::Lindle => {
                let client = LindleAgentsClient::new(cfg.clone());
                client
                    .ensure_ready()
                    .await
                    .map_err(|e| anyhow::anyhow!("required backend `{}` failed: {e}", cfg.id))?;
                orleans = Some(client);
            }
            AgentBackendKind::LindleMcp => {
                bail!("MCP backend should not be marked required");
            }
        }
    }

    if orleans.is_none() {
        if let Some(cfg) = orch.of_kind(AgentBackendKind::Lindle) {
            let client = LindleAgentsClient::new(cfg.clone());
            client
                .ensure_ready()
                .await
                .map_err(|e| anyhow::anyhow!("Orleans backend `{}` failed: {e}", cfg.id))?;
            orleans = Some(client);
        }
    }

    if let Some(cfg) = orch.of_kind(AgentBackendKind::LindleMcp) {
        let client = LindleMcpClient::new(cfg.clone());
        match client.ensure_ready().await {
            Ok(()) => mcp = Some(client),
            Err(e) => tracing::warn!(id = %cfg.id, error = %e, "optional MCP backend down"),
        }
    }

    if orleans.is_none() && mcp.is_none() {
        bail!("no Lindle backends connected — set AGENTS_SERVICE_URL and/or LINDLE_MCP_URL");
    }

    Ok(ConnectedBackends { orleans, mcp })
}

pub fn build_card(public_url: &str, orch: &OrchestratorConfig) -> AgentCard {
    let mut skills: Vec<Skill> = Vec::new();
    for cfg in &orch.backends {
        push_skills(&mut skills, cfg);
    }
    let mut card = AgentCard::gateway(public_url, skills);
    card.description =
        "Coop A2A edge — foreign agents → Lindle Orleans / MCP (identity + trust score)".into();
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
