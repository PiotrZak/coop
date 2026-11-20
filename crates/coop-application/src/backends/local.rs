//! Local A2A peer — one entity of the generic backend family.

use crate::backend::{AgentBackendConfig, AgentBackendKind, AuthStyle};
use crate::routing::{SkillRoute, SkillRouter};
use anyhow::Result;

pub struct LocalA2aBackend;

impl LocalA2aBackend {
    pub fn create(
        id: impl Into<String>,
        base_url: impl Into<String>,
        auth_secret: impl Into<String>,
        skills: Vec<String>,
    ) -> AgentBackendConfig {
        AgentBackendConfig {
            id: id.into(),
            kind: AgentBackendKind::LocalA2a,
            base_url: base_url.into().trim_end_matches('/').to_string(),
            auth_secret: auth_secret.into(),
            auth_style: AuthStyle::Bearer,
            health_path: "/.well-known/agent.json".into(),
            default_tenant_id: String::new(),
            skills,
            priority: 100,
            required: false,
        }
    }

    /// Parse `skill=url@bearer,...` into configs + router.
    pub fn from_routes(raw: &str) -> Result<(Vec<AgentBackendConfig>, SkillRouter)> {
        let mut configs = Vec::new();
        let mut router = SkillRouter::new();
        for part in raw.split(',').map(str::trim).filter(|p| !p.is_empty()) {
            let (skill, rest) = part
                .split_once('=')
                .ok_or_else(|| anyhow::anyhow!("bad route `{part}` (want skill=url@bearer)"))?;
            let (url, bearer) = rest
                .rsplit_once('@')
                .ok_or_else(|| anyhow::anyhow!("bad route `{part}` (want skill=url@bearer)"))?;
            let skill = skill.trim().to_string();
            let url = url.trim().to_string();
            let bearer = bearer.trim().to_string();
            router.insert(SkillRoute {
                skill: skill.clone(),
                upstream_url: url.clone(),
                upstream_bearer: bearer.clone(),
            });
            configs.push(Self::create(
                format!("local-{skill}"),
                url,
                bearer,
                vec![skill],
            ));
        }
        Ok((configs, router))
    }

    pub fn demo_router() -> SkillRouter {
        SkillRouter::demo()
    }
}
