//! OAuth2 + OpenID Connect JWT AuthN (HS256 secret or JWKS).

use anyhow::{anyhow, bail, Context, Result};
use coop_domain::{IdentityProof, Principal, Role};
use jsonwebtoken::jwk::Jwk;
use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
use serde::Deserialize;
use std::env;

#[derive(Clone)]
pub struct JwtAuth {
    issuer: String,
    audience: String,
    keys: Vec<DecodingKey>,
    alg: Algorithm,
}

#[derive(Debug, Deserialize)]
struct Claims {
    sub: String,
    #[serde(default)]
    tenant_id: Option<String>,
    #[serde(default)]
    tid: Option<String>,
    #[serde(default)]
    roles: Option<Vec<String>>,
    #[serde(default)]
    role: Option<String>,
    #[serde(default)]
    nonce: Option<String>,
    #[serde(default)]
    token_use: Option<String>,
    #[serde(default)]
    amr: Option<Vec<String>>,
    #[serde(default)]
    auth_time: Option<u64>,
    #[serde(default)]
    scope: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Jwks {
    keys: Vec<serde_json::Value>,
}

impl JwtAuth {
    /// `COOP_OIDC_ISSUER` + (`COOP_OIDC_HS256_SECRET` or `COOP_OIDC_JWKS_URL` / JWKS at issuer).
    pub async fn from_env() -> Result<Option<Self>> {
        let Ok(issuer) = env::var("COOP_OIDC_ISSUER") else {
            return Ok(None);
        };
        if issuer.trim().is_empty() {
            return Ok(None);
        }
        let audience =
            env::var("COOP_OIDC_AUDIENCE").unwrap_or_else(|_| "coop-gateway".into());

        if let Ok(secret) = env::var("COOP_OIDC_HS256_SECRET") {
            if !secret.is_empty() {
                return Ok(Some(Self {
                    issuer,
                    audience,
                    keys: vec![DecodingKey::from_secret(secret.as_bytes())],
                    alg: Algorithm::HS256,
                }));
            }
        }

        let jwks_url = env::var("COOP_OIDC_JWKS_URL").unwrap_or_else(|_| {
            format!("{}/.well-known/jwks.json", issuer.trim_end_matches('/'))
        });
        let keys = fetch_jwks_keys(&jwks_url)
            .await
            .with_context(|| format!("load JWKS from {jwks_url}"))?;
        if keys.is_empty() {
            bail!("JWKS at {jwks_url} contained no usable keys");
        }
        Ok(Some(Self {
            issuer,
            audience,
            keys,
            alg: Algorithm::RS256,
        }))
    }

    /// Test / local helper: single HS256 secret.
    pub fn hs256(issuer: impl Into<String>, audience: impl Into<String>, secret: &[u8]) -> Self {
        Self {
            issuer: issuer.into(),
            audience: audience.into(),
            keys: vec![DecodingKey::from_secret(secret)],
            alg: Algorithm::HS256,
        }
    }

    pub fn authenticate(&self, token: &str) -> Result<(Principal, IdentityProof)> {
        let mut validation = Validation::new(self.alg);
        validation.set_issuer(&[&self.issuer]);
        validation.set_audience(&[&self.audience]);

        let mut last_err = None;
        let data = {
            let mut decoded = None;
            for key in &self.keys {
                match decode::<Claims>(token, key, &validation) {
                    Ok(d) => {
                        decoded = Some(d);
                        break;
                    }
                    Err(e) => last_err = Some(e),
                }
            }
            decoded.ok_or_else(|| {
                anyhow!(
                    "JWT validation failed: {}",
                    last_err
                        .map(|e| e.to_string())
                        .unwrap_or_else(|| "no keys".into())
                )
            })?
        };

        let claims = data.claims;
        let proof = classify_proof(&claims);
        let roles = parse_roles(&claims);
        if roles.is_empty() {
            bail!("JWT for sub={} has no mappable roles", claims.sub);
        }
        let tenant_id = claims
            .tenant_id
            .or(claims.tid)
            .unwrap_or_else(|| "shared".into());

        Ok((
            Principal {
                agent_id: claims.sub,
                tenant_id,
                roles,
            },
            proof,
        ))
    }
}

fn classify_proof(claims: &Claims) -> IdentityProof {
    let token_use = claims.token_use.as_deref().unwrap_or("");
    let scope = claims.scope.as_deref().unwrap_or("");
    let oidc = token_use == "id_token"
        || claims.nonce.is_some()
        || claims.auth_time.is_some()
        || claims.amr.is_some()
        || scope.split_whitespace().any(|s| s == "openid");
    if oidc {
        IdentityProof::OpenIdConnect
    } else {
        IdentityProof::OAuth2
    }
}

fn parse_roles(claims: &Claims) -> Vec<Role> {
    let mut out = Vec::new();
    if let Some(roles) = &claims.roles {
        for r in roles {
            if let Some(role) = role_from_str(r) {
                out.push(role);
            }
        }
    }
    if out.is_empty() {
        if let Some(r) = claims.role.as_deref().and_then(role_from_str) {
            out.push(r);
        }
    }
    out
}

fn role_from_str(s: &str) -> Option<Role> {
    match s.trim().to_ascii_lowercase().as_str() {
        "reader" | "read" => Some(Role::Reader),
        "contributor" | "consult" => Some(Role::Contributor),
        "operator" | "act" => Some(Role::Operator),
        "admin" => Some(Role::Admin),
        "service" => Some(Role::Service),
        _ => None,
    }
}

fn looks_like_jwt(token: &str) -> bool {
    let mut parts = token.split('.');
    parts.next().is_some() && parts.next().is_some() && parts.next().is_some() && parts.next().is_none()
}

pub(crate) fn is_jwt_bearer(token: &str) -> bool {
    looks_like_jwt(token)
}

async fn fetch_jwks_keys(url: &str) -> Result<Vec<DecodingKey>> {
    let jwks: Jwks = reqwest::Client::new()
        .get(url)
        .send()
        .await
        .context("JWKS HTTP request")?
        .error_for_status()
        .context("JWKS HTTP status")?
        .json()
        .await
        .context("JWKS JSON")?;

    let mut keys = Vec::new();
    for jwk_val in jwks.keys {
        let jwk: Jwk = match serde_json::from_value(jwk_val) {
            Ok(j) => j,
            Err(e) => {
                tracing::warn!(error = %e, "skipping unparseable JWKS key");
                continue;
            }
        };
        match DecodingKey::from_jwk(&jwk) {
            Ok(k) => keys.push(k),
            Err(e) => tracing::warn!(error = %e, "skipping JWKS key"),
        }
    }
    Ok(keys)
}
