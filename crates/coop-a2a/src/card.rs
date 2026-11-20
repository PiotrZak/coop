use coop_domain::Scope;
use coop_identity::AuthScheme;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentCard {
    pub name: String,
    pub description: String,
    pub url: String,
    pub version: String,
    pub skills: Vec<Skill>,
    pub security: Vec<SecurityRequirement>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Skill {
    pub id: String,
    pub name: String,
    pub description: String,
    pub scopes: Vec<Scope>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityRequirement {
    pub scheme: AuthScheme,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

impl AgentCard {
    pub fn gateway(url: impl Into<String>, skills: Vec<Skill>) -> Self {
        Self {
            name: "coop-gateway".into(),
            description:
                "A2A edge for foreign agents → Lindle (Orleans / MCP). Identity + trust score + RBAC."
                    .into(),
            url: url.into(),
            version: "0.1.0".into(),
            skills,
            security: vec![SecurityRequirement {
                scheme: AuthScheme::Bearer,
                description: Some(
                    "Authorization: Bearer <api-key|oauth-access-token>. RBAC + identity trust floor."
                        .into(),
                ),
            }],
        }
    }
}
