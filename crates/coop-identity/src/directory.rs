//! Shared-secret / API-key AuthN.

use anyhow::{anyhow, Result};
use coop_domain::{Principal, Role};
use http::header::{HeaderMap, AUTHORIZATION};
use std::collections::HashMap;

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
}

pub(crate) fn bearer(headers: &HeaderMap) -> Option<&str> {
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
