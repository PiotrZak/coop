//! HTTP forwarder for gateway — uses application [`SkillRouter`].

use crate::card::{AgentCard, Skill};
use crate::client::A2aClient;
use crate::models::{JsonRpcRequest, JsonRpcResponse};
use anyhow::{anyhow, Context, Result};
pub use coop_application::{SkillRoute, SkillRouter};

pub fn merge_skills(cards: impl IntoIterator<Item = AgentCard>) -> Vec<Skill> {
    let mut out = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for card in cards {
        for skill in card.skills {
            if seen.insert(skill.id.clone()) {
                out.push(skill);
            }
        }
    }
    out.sort_by(|a, b| a.id.cmp(&b.id));
    out
}

pub fn build_gateway_card(url: &str, cards: Vec<AgentCard>) -> AgentCard {
    AgentCard::gateway(url, merge_skills(cards))
}

pub async fn forward(
    router: &SkillRouter,
    skill: &str,
    req: &JsonRpcRequest,
) -> Result<JsonRpcResponse> {
    let route = router
        .get(skill)
        .ok_or_else(|| anyhow!("no upstream for skill `{skill}`"))?;
    let mut resp = A2aClient::new(&route.upstream_url, &route.upstream_bearer)
        .forward_rpc(req)
        .await
        .context("upstream")?;
    resp.id = req.id.clone();
    Ok(resp)
}
