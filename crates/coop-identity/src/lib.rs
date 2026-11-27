//! AuthN (shared secret / OAuth2 / OIDC / mTLS). AuthZ stays in domain.

mod auth;
mod directory;
mod jwt;
mod mtls;

use serde::{Deserialize, Serialize};

pub use auth::IdentityAuth;
pub use coop_domain::{IdentityProof, Principal, RbacLevel, Role, Scope, TrustScore};
pub use directory::ApiKeyDirectory;
pub use jwt::JwtAuth;
pub use mtls::MtlsDirectory;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuthScheme {
    Bearer,
    ApiKey,
    OAuth2,
    OpenIdConnect,
    Mtls,
}
