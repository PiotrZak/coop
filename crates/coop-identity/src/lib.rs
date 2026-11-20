//! Coop Identity — transport-layer AuthN for A2A.
//!
//! A2A does **not** put identity in JSON-RPC payloads. Credentials travel in HTTP
//! headers. The Agent Card advertises which schemes are accepted.
//!
//! v0 demo scheme: shared API keys presented as `Authorization: Bearer <key>`.
//! Production path: OAuth2 / OIDC access tokens with skill scopes, or mTLS.

use anyhow::{anyhow, bail, Result};
use hmac::{Hmac, Mac};
use http::header::{HeaderMap, AUTHORIZATION};
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use std::collections::HashMap;

type HmacSha256 = Hmac<Sha256>;

/// OpenAPI-style security scheme names as advertised on an Agent Card.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuthScheme {
    /// `Authorization: Bearer <token>` — API key or opaque access token.
    Bearer,
    /// `X-API-Key: <key>` — alternate header form.
    ApiKey,
    /// OAuth2 / OIDC access token (still Bearer on the wire).
    OAuth2,
    /// Mutual TLS (identity from client cert; no Bearer needed).
    Mtls,
}

/// Authenticated caller after header validation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Principal {
    pub agent_id: String,
    pub tenant_id: String,
    /// Skill scopes this principal may invoke (e.g. `negotiate`, `supplier.read`).
    pub scopes: Vec<String>,
}

/// In-memory API-key directory for hackathon / local demos.
#[derive(Debug, Clone, Default)]
pub struct ApiKeyDirectory {
    /// raw key → principal
    keys: HashMap<String, Principal>,
}

impl ApiKeyDirectory {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&mut self, key: impl Into<String>, principal: Principal) {
        self.keys.insert(key.into(), principal);
    }

    /// Seed demo keys used by Negotiation A/B and Supplier.
    pub fn demo() -> Self {
        let mut d = Self::new();
        d.register(
            "nego-a-demo-key",
            Principal {
                agent_id: "negotiation-a".into(),
                tenant_id: "buyer-tenant".into(),
                scopes: vec!["negotiate".into(), "supplier.read".into()],
            },
        );
        d.register(
            "nego-b-demo-key",
            Principal {
                agent_id: "negotiation-b".into(),
                tenant_id: "seller-tenant".into(),
                scopes: vec!["negotiate".into(), "supplier.read".into()],
            },
        );
        d.register(
            "supplier-demo-key",
            Principal {
                agent_id: "supplier".into(),
                tenant_id: "shared".into(),
                scopes: vec!["supplier.read".into()],
            },
        );
        d
    }

    pub fn authenticate_headers(&self, headers: &HeaderMap) -> Result<Principal> {
        let token = extract_bearer(headers)?
            .or_else(|| extract_api_key(headers))
            .ok_or_else(|| anyhow!("missing credentials (Bearer or X-API-Key)"))?;

        self.keys
            .get(token)
            .cloned()
            .ok_or_else(|| anyhow!("invalid credentials"))
    }

    pub fn authorize(&self, principal: &Principal, skill: &str) -> Result<()> {
        if principal.scopes.iter().any(|s| s == skill || s == "*") {
            Ok(())
        } else {
            bail!(
                "principal {} lacks scope for skill `{skill}`",
                principal.agent_id
            )
        }
    }
}

fn extract_bearer(headers: &HeaderMap) -> Result<Option<&str>> {
    let Some(value) = headers.get(AUTHORIZATION) else {
        return Ok(None);
    };
    let s = value
        .to_str()
        .map_err(|_| anyhow!("invalid Authorization header"))?;
    Ok(s.strip_prefix("Bearer ").map(str::trim))
}

fn extract_api_key(headers: &HeaderMap) -> Option<&str> {
    headers
        .get("x-api-key")
        .and_then(|v| v.to_str().ok())
        .map(str::trim)
}

/// Optional HMAC receipt over task id + artifact digest (Identity layer, not A2A core).
pub fn sign_receipt(secret: &[u8], task_id: &str, artifact_digest: &str) -> Result<String> {
    let mut mac =
        HmacSha256::new_from_slice(secret).map_err(|_| anyhow!("invalid HMAC key length"))?;
    mac.update(task_id.as_bytes());
    mac.update(b"|");
    mac.update(artifact_digest.as_bytes());
    Ok(hex::encode(mac.finalize().into_bytes()))
}

mod hex {
    pub fn encode(bytes: impl AsRef<[u8]>) -> String {
        bytes.as_ref().iter().map(|b| format!("{b:02x}")).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use http::HeaderValue;

    #[test]
    fn bearer_auth_and_skill_scope() {
        let dir = ApiKeyDirectory::demo();
        let mut headers = HeaderMap::new();
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_static("Bearer nego-a-demo-key"),
        );
        let p = dir.authenticate_headers(&headers).unwrap();
        assert_eq!(p.agent_id, "negotiation-a");
        dir.authorize(&p, "negotiate").unwrap();
        dir.authorize(&p, "supplier.read").unwrap();
    }
}
