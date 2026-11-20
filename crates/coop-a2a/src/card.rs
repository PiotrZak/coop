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
    pub fn negotiation(url: impl Into<String>, peer_name: &str) -> Self {
        Self {
            name: peer_name.into(),
            description: "Human-owned commercial negotiation pack over A2A".into(),
            url: url.into(),
            version: "0.1.0".into(),
            skills: vec![Skill {
                id: Scope::Negotiate.skill_id().into(),
                name: "Negotiate".into(),
                description: "Exchange a negotiation ask/response; may consult Supplier Agent"
                    .into(),
                scopes: vec![Scope::Negotiate],
            }],
            security: vec![SecurityRequirement {
                scheme: AuthScheme::Bearer,
                description: Some(
                    "Authorization: Bearer <api-key|oauth-access-token>. Demo keys for local."
                        .into(),
                ),
            }],
        }
    }

    pub fn supplier(url: impl Into<String>) -> Self {
        Self {
            name: "supplier".into(),
            description: "Scoped supplier facts for a supplier_id (portfolio / leverage)".into(),
            url: url.into(),
            version: "0.1.0".into(),
            skills: vec![Skill {
                id: Scope::SupplierRead.skill_id().into(),
                name: "Supplier lookup".into(),
                description: "Return structured supplier brief for a given supplier_id".into(),
                scopes: vec![Scope::SupplierRead],
            }],
            security: vec![SecurityRequirement {
                scheme: AuthScheme::Bearer,
                description: Some("Authorization: Bearer <api-key|oauth-access-token>".into()),
            }],
        }
    }

    pub fn gateway(url: impl Into<String>, skills: Vec<Skill>) -> Self {
        Self {
            name: "coop-gateway".into(),
            description: "A2A edge gateway — skill-routed proxy to upstream agents".into(),
            url: url.into(),
            version: "0.1.0".into(),
            skills,
            security: vec![SecurityRequirement {
                scheme: AuthScheme::Bearer,
                description: Some(
                    "Authorization: Bearer <api-key|oauth-access-token>. RBAC AuthZ at gateway."
                        .into(),
                ),
            }],
        }
    }
}
