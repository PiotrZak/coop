//! Identity infrastructure — AuthN only (headers → [`Principal`]).
//! AuthZ stays on [`Principal::authorize`] in the domain.

use anyhow::{anyhow, Result};
use http::header::{HeaderMap, AUTHORIZATION};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub use coop_domain::{IdentityProof, Principal, RbacLevel, Role, Scope, TrustScore};

/// Schemes advertised on Agent Cards (OpenAPI-aligned).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuthScheme {
    Bearer,
    ApiKey,
    OAuth2,
    Mtls,
}

#[derive(Debug, Clone, Default)]
pub struct ApiKeyDirectory {
    keys: HashMap<String, Principal>,
}

impl ApiKeyDirectory {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&mut self, key: impl Into<String>, principal: Principal) {
        self.keys.insert(key.into(), principal);
    }

    pub fn demo() -> Self {
        let mut d = Self::new();
        d.register(
            "nego-a-demo-key",
            Principal {
                agent_id: "negotiation-a".into(),
                tenant_id: "buyer-tenant".into(),
                roles: vec![Role::Operator],
            },
        );
        d.register(
            "nego-b-demo-key",
            Principal {
                agent_id: "negotiation-b".into(),
                tenant_id: "seller-tenant".into(),
                roles: vec![Role::Admin],
            },
        );
        d.register(
            "supplier-demo-key",
            Principal {
                agent_id: "supplier".into(),
                tenant_id: "shared".into(),
                roles: vec![Role::Reader],
            },
        );
        d.register(
            "gateway-service-key",
            Principal {
                agent_id: "coop-gateway".into(),
                tenant_id: "shared".into(),
                roles: vec![Role::Service],
            },
        );
        d
    }

    pub fn authenticate(&self, headers: &HeaderMap) -> Result<Principal> {
        let token = bearer(headers)
            .or_else(|| api_key(headers))
            .ok_or_else(|| anyhow!("missing credentials (Bearer or X-API-Key)"))?;
        self.keys
            .get(token)
            .cloned()
            .ok_or_else(|| anyhow!("invalid credentials"))
    }

    /// AuthN then domain AuthZ + identity trust floor.
    ///
    /// Demo directory credentials use [`IdentityProof::SharedSecret`].
    pub fn authenticate_and_authorize(
        &self,
        headers: &HeaderMap,
        skill: &str,
    ) -> Result<(Principal, TrustScore)> {
        self.authenticate_authorize_with_proof(headers, skill, IdentityProof::SharedSecret)
    }

    pub fn authenticate_authorize_with_proof(
        &self,
        headers: &HeaderMap,
        skill: &str,
        proof: IdentityProof,
    ) -> Result<(Principal, TrustScore)> {
        let principal = self.authenticate(headers)?;
        let trust = principal.authorize_with_trust(skill, proof)?;
        Ok((principal, trust))
    }
}


fn bearer(headers: &HeaderMap) -> Option<&str> {
    headers
        .get(AUTHORIZATION)?
        .to_str()
        .ok()?
        .strip_prefix("Bearer ")
        .map(str::trim)
}

fn api_key(headers: &HeaderMap) -> Option<&str> {
    headers.get("x-api-key")?.to_str().ok().map(str::trim)
}
