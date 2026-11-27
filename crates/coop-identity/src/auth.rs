//! AuthN: mTLS → OIDC/OAuth2 JWT → shared secret.

use crate::directory::{bearer, ApiKeyDirectory};
use crate::jwt::{is_jwt_bearer, JwtAuth};
use crate::mtls::MtlsDirectory;
use anyhow::Result;
use coop_domain::{IdentityProof, Principal, TrustScore};
use http::HeaderMap;

#[derive(Clone)]
pub struct IdentityAuth {
    keys: ApiKeyDirectory,
    jwt: Option<JwtAuth>,
    mtls: Option<MtlsDirectory>,
}

impl IdentityAuth {
    pub fn new(keys: ApiKeyDirectory, jwt: Option<JwtAuth>, mtls: Option<MtlsDirectory>) -> Self {
        Self { keys, jwt, mtls }
    }

    pub async fn from_env() -> Result<Self> {
        let jwt = JwtAuth::from_env().await?;
        let mtls = MtlsDirectory::from_env();
        if jwt.is_some() {
            tracing::info!("OIDC/OAuth2 JWT AuthN enabled");
        }
        if mtls.is_some() {
            tracing::info!("mTLS proxy-header AuthN enabled");
        }
        Ok(Self::new(ApiKeyDirectory::demo(), jwt, mtls))
    }

    pub fn jwt_enabled(&self) -> bool {
        self.jwt.is_some()
    }

    pub fn mtls_enabled(&self) -> bool {
        self.mtls.is_some()
    }

    pub fn authenticate(&self, headers: &HeaderMap) -> Result<(Principal, IdentityProof)> {
        if let Some(mtls) = &self.mtls {
            if let Some(principal) = mtls.try_authenticate(headers)? {
                return Ok((principal, IdentityProof::Mtls));
            }
        }
        if let Some(jwt) = &self.jwt {
            if let Some(token) = bearer(headers) {
                if is_jwt_bearer(token) {
                    return jwt.authenticate(token);
                }
            }
        }
        Ok((
            self.keys.authenticate(headers)?,
            IdentityProof::SharedSecret,
        ))
    }

    pub fn authenticate_and_authorize(
        &self,
        headers: &HeaderMap,
        skill: &str,
    ) -> Result<(Principal, TrustScore, IdentityProof)> {
        let (principal, proof) = self.authenticate(headers)?;
        let trust = principal.authorize_with_trust(skill, proof)?;
        Ok((principal, trust, proof))
    }
}
