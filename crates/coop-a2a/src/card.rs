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

pub fn security_for_auth(jwt: bool, mtls: bool) -> Vec<SecurityRequirement> {
    let mut s = vec![SecurityRequirement {
        scheme: AuthScheme::Bearer,
        description: Some("Bearer <api-key|jwt>; optional params.target_agent".into()),
    }];
    if jwt {
        s.push(SecurityRequirement {
            scheme: AuthScheme::OAuth2,
            description: Some("Bearer JWT access token".into()),
        });
        s.push(SecurityRequirement {
            scheme: AuthScheme::OpenIdConnect,
            description: Some("Bearer OIDC JWT".into()),
        });
    }
    if mtls {
        s.push(SecurityRequirement {
            scheme: AuthScheme::Mtls,
            description: Some("Proxy client-cert headers".into()),
        });
    }
    s
}
