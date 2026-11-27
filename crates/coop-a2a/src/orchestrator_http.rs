//! Connect adapters + build Agent Card.

use crate::agent_backend::{AgentBackend, AgentRegistry};
use crate::card::{security_for_auth, AgentCard, Skill};
use crate::lindle::LindleOrleansClient;
use crate::mcp::LindleMcpClient;
use crate::peer::A2aPeerClient;
use anyhow::{bail, Result};
use coop_application::{AgentBackendConfig, AgentBackendKind, OrchestratorConfig};
use coop_domain::Scope;
use coop_identity::IdentityAuth;
use std::sync::Arc;

pub async fn connect_backends(orch: &OrchestratorConfig) -> Result<AgentRegistry> {
    let mut connected: Vec<Arc<dyn AgentBackend>> = Vec::new();

    for cfg in &orch.backends {
        let backend: Arc<dyn AgentBackend> = match cfg.kind {
            AgentBackendKind::AgentsHttp => Arc::new(LindleOrleansClient::new(cfg.clone())),
            AgentBackendKind::Mcp => Arc::new(LindleMcpClient::new(cfg.clone())),
            AgentBackendKind::A2aPeer => Arc::new(A2aPeerClient::new(cfg.clone())),
        };

        match backend.ensure_ready().await {
            Ok(()) => connected.push(backend),
            Err(e) if cfg.required => {
                bail!("required agent `{}` failed: {e}", cfg.id);
            }
            Err(e) => tracing::warn!(id = %cfg.id, error = %e, "optional agent down"),
        }
    }

    if connected.is_empty() {
        bail!("no agents connected — set AGENTS_SERVICE_URL, LINDLE_MCP_URL, and/or COOP_PEERS_JSON");
    }
    Ok(AgentRegistry::new(connected))
}

pub fn build_card(public_url: &str, orch: &OrchestratorConfig, auth: &IdentityAuth) -> AgentCard {
    let mut skills = Vec::new();
    for cfg in &orch.backends {
        push_skills(&mut skills, cfg);
    }
    AgentCard {
        name: "coop-gateway".into(),
        description: "Auth hub — route A2A to registered agent adapters.".into(),
        url: public_url.into(),
        version: "0.1.0".into(),
        skills,
        security: security_for_auth(auth.jwt_enabled(), auth.mtls_enabled()),
    }
}

fn push_skills(skills: &mut Vec<Skill>, cfg: &AgentBackendConfig) {
    for skill_id in &cfg.skills {
        if skills.iter().any(|s| s.id == *skill_id) {
            continue;
        }
        skills.push(Skill {
            id: skill_id.clone(),
            name: skill_id.clone(),
            description: format!("via agent `{}`", cfg.id),
            scopes: Scope::from_skill(skill_id).into_iter().collect(),
        });
    }
}
