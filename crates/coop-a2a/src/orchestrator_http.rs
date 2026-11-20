//! HTTP boot helpers for the Coop orchestrator.

use crate::card::{AgentCard, Skill};
use crate::client::A2aClient;
use crate::lindle::LindleAgentsClient;
use crate::proxy::{build_gateway_card, SkillRouter};
use anyhow::Result;
use coop_application::{AgentBackendConfig, AgentBackendKind, OrchestratorConfig};
use coop_domain::Scope;

/// Connect required backends first, then optional. Returns Lindle client if up.
pub async fn connect_backends(orch: &OrchestratorConfig) -> Result<Option<LindleAgentsClient>> {
    let mut lindle = None;

    for cfg in orch.required() {
        if cfg.kind == AgentBackendKind::Lindle {
            let client = LindleAgentsClient::new(cfg.clone());
            client
                .ensure_ready()
                .await
                .map_err(|e| anyhow::anyhow!("required backend `{}` failed: {e}", cfg.id))?;
            lindle = Some(client);
        }
    }

    if lindle.is_none() {
        if let Some(cfg) = orch.of_kind(AgentBackendKind::Lindle) {
            let client = LindleAgentsClient::new(cfg.clone());
            match client.ensure_ready().await {
                Ok(()) => lindle = Some(client),
                Err(e) => tracing::warn!(id = %cfg.id, error = %e, "optional backend down"),
            }
        }
    }

    if orch.backends.is_empty() {
        tracing::warn!("no agent backends from env — local A2A only");
    }

    Ok(lindle)
}

pub async fn build_card(
    public_url: &str,
    orch: &OrchestratorConfig,
    lindle_up: bool,
    local: &SkillRouter,
) -> AgentCard {
    let mut skills: Vec<Skill> = Vec::new();

    for cfg in &orch.backends {
        if cfg.kind == AgentBackendKind::Lindle && !lindle_up {
            continue;
        }
        push_skills(&mut skills, cfg);
    }

    let mut cards = Vec::new();
    for route in local.routes() {
        if let Ok(c) = A2aClient::new(&route.upstream_url, &route.upstream_bearer)
            .get_agent_card()
            .await
        {
            cards.push(c);
        }
    }
    if !cards.is_empty() {
        for s in build_gateway_card(public_url, cards).skills {
            if !skills.iter().any(|x| x.id == s.id) {
                skills.push(s);
            }
        }
    } else if skills.is_empty() {
        for id in local.skills() {
            skills.push(Skill {
                id: id.into(),
                name: id.into(),
                description: format!("Local A2A skill `{id}`"),
                scopes: Scope::from_skill(id).into_iter().collect(),
            });
        }
    }

    let mut card = AgentCard::gateway(public_url, skills);
    card.description =
        "Coop orchestrator — generic AgentBackendConfig; Lindle is one entity".into();
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
