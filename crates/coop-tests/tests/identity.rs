//! Identity AuthN + domain RBAC AuthZ + trust score.

use coop_domain::{IdentityProof, Principal, RbacLevel, Role, Scope, TrustScore};
use coop_identity::{ApiKeyDirectory, IdentityAuth, JwtAuth, MtlsDirectory};
use http::header::{HeaderMap, HeaderValue, AUTHORIZATION};
use jsonwebtoken::{encode, EncodingKey, Header};
use serde::Serialize;
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn bearer_auth_and_skill_scope() {
    let auth = IdentityAuth::new(ApiKeyDirectory::demo(), None, None);
    let mut headers = HeaderMap::new();
    headers.insert(
        AUTHORIZATION,
        HeaderValue::from_static("Bearer nego-a-demo-key"),
    );
    let (p, proof) = auth.authenticate(&headers).unwrap();
    assert_eq!(proof, IdentityProof::SharedSecret);
    assert_eq!(p.agent_id, "negotiation-a");
    assert_eq!(p.roles, vec![Role::Operator]);
    p.authorize("negotiate").unwrap();
    p.authorize("supplier.read").unwrap();
}

#[test]
fn authenticate_and_authorize_helper() {
    let auth = IdentityAuth::new(ApiKeyDirectory::demo(), None, None);
    let mut headers = HeaderMap::new();
    headers.insert(
        AUTHORIZATION,
        HeaderValue::from_static("Bearer nego-a-demo-key"),
    );
    let (p, trust, proof) = auth
        .authenticate_and_authorize(&headers, "negotiate")
        .unwrap();
    assert_eq!(p.agent_id, "negotiation-a");
    assert_eq!(trust, TrustScore::Medium);
    assert_eq!(proof, IdentityProof::SharedSecret);
}

#[test]
fn oauth2_jwt_authn() {
    let secret = b"coop-test-hs256-secret";
    let jwt = JwtAuth::hs256("https://issuer.test", "coop-gateway", secret);
    let auth = IdentityAuth::new(ApiKeyDirectory::demo(), Some(jwt), None);

    let token = sign_jwt(
        secret,
        Claims {
            sub: "oauth-agent".into(),
            iss: "https://issuer.test".into(),
            aud: "coop-gateway".into(),
            exp: now() + 3600,
            tenant_id: Some("t1".into()),
            roles: Some(vec!["operator".into()]),
            nonce: None,
            token_use: Some("access".into()),
            scope: Some("api".into()),
        },
    );

    let mut headers = HeaderMap::new();
    headers.insert(
        AUTHORIZATION,
        HeaderValue::from_str(&format!("Bearer {token}")).unwrap(),
    );
    let (p, trust, proof) = auth
        .authenticate_and_authorize(&headers, "negotiate")
        .unwrap();
    assert_eq!(proof, IdentityProof::OAuth2);
    assert_eq!(p.agent_id, "oauth-agent");
    assert_eq!(trust, TrustScore::Medium);
}

#[test]
fn openid_connect_jwt_authn() {
    let secret = b"coop-test-hs256-secret";
    let jwt = JwtAuth::hs256("https://issuer.test", "coop-gateway", secret);
    let auth = IdentityAuth::new(ApiKeyDirectory::demo(), Some(jwt), None);

    let token = sign_jwt(
        secret,
        Claims {
            sub: "oidc-agent".into(),
            iss: "https://issuer.test".into(),
            aud: "coop-gateway".into(),
            exp: now() + 3600,
            tenant_id: Some("t1".into()),
            roles: Some(vec!["admin".into()]),
            nonce: Some("n-1".into()),
            token_use: Some("id_token".into()),
            scope: Some("openid profile".into()),
        },
    );

    let mut headers = HeaderMap::new();
    headers.insert(
        AUTHORIZATION,
        HeaderValue::from_str(&format!("Bearer {token}")).unwrap(),
    );
    let (p, trust, proof) = auth
        .authenticate_and_authorize(&headers, "negotiate")
        .unwrap();
    assert_eq!(proof, IdentityProof::OpenIdConnect);
    assert_eq!(p.agent_id, "oidc-agent");
    assert_eq!(trust, TrustScore::High);
}

#[test]
fn mtls_proxy_headers_authn() {
    let auth =
        IdentityAuth::new(ApiKeyDirectory::demo(), None, Some(MtlsDirectory::demo()));
    let mut headers = HeaderMap::new();
    headers.insert("x-ssl-client-verify", HeaderValue::from_static("SUCCESS"));
    headers.insert(
        "x-ssl-client-s-dn",
        HeaderValue::from_static("CN=nego-a.agents.local,O=Coop"),
    );
    let (p, trust, proof) = auth
        .authenticate_and_authorize(&headers, "negotiate")
        .unwrap();
    assert_eq!(proof, IdentityProof::Mtls);
    assert_eq!(p.agent_id, "negotiation-a");
    assert_eq!(trust, TrustScore::High);
}

#[test]
fn trust_score_from_identity() {
    let reader = Principal {
        agent_id: "r".into(),
        tenant_id: "t".into(),
        roles: vec![Role::Reader],
    };
    assert_eq!(
        reader.trust_score(IdentityProof::SharedSecret),
        TrustScore::Low
    );
    assert!(reader
        .authorize_with_trust("negotiate", IdentityProof::SharedSecret)
        .is_err());
    assert!(reader
        .authorize_with_trust("supplier.read", IdentityProof::SharedSecret)
        .is_ok());
    assert_eq!(
        reader.trust_score(IdentityProof::OpenIdConnect),
        TrustScore::Medium
    );

    let admin = Principal {
        agent_id: "a".into(),
        tenant_id: "t".into(),
        roles: vec![Role::Admin],
    };
    assert_eq!(
        admin.trust_score(IdentityProof::SharedSecret),
        TrustScore::High
    );
    assert_eq!(admin.trust_score(IdentityProof::Mtls), TrustScore::High);
    assert!(admin
        .authorize_with_trust("peer.message", IdentityProof::SharedSecret)
        .is_ok());
}

#[test]
fn reader_cannot_negotiate() {
    let p = Principal {
        agent_id: "low".into(),
        tenant_id: "t".into(),
        roles: vec![Role::Reader],
    };
    assert!(!p.can_invoke(Scope::Negotiate));
    assert!(p.can_invoke(Scope::SupplierRead));
    assert!(p.authorize("negotiate").is_err());
}

#[test]
fn admin_can_negotiate() {
    let p = Principal {
        agent_id: "hi".into(),
        tenant_id: "t".into(),
        roles: vec![Role::Admin],
    };
    assert!(p.can_invoke(Scope::Negotiate));
    assert_eq!(p.level_for(Scope::Negotiate), Some(RbacLevel::Admin));
    p.authorize("negotiate").unwrap();
}

#[test]
fn role_levels_are_generic() {
    assert_eq!(Role::Reader.max_level(), RbacLevel::Read);
    assert_eq!(Role::Contributor.max_level(), RbacLevel::Consult);
    assert_eq!(Role::Operator.max_level(), RbacLevel::Act);
    assert_eq!(Role::Admin.max_level(), RbacLevel::Admin);
    assert_eq!(Role::Service.max_level(), RbacLevel::Admin);
}

#[test]
fn scope_serde() {
    assert_eq!(
        serde_json::to_string(&Scope::SupplierRead).unwrap(),
        "\"supplier.read\""
    );
    assert_eq!(
        serde_json::to_string(&Scope::PeerMessage).unwrap(),
        "\"peer.message\""
    );
}

#[derive(Serialize)]
struct Claims {
    sub: String,
    iss: String,
    aud: String,
    exp: u64,
    tenant_id: Option<String>,
    roles: Option<Vec<String>>,
    nonce: Option<String>,
    token_use: Option<String>,
    scope: Option<String>,
}

fn sign_jwt(secret: &[u8], claims: Claims) -> String {
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret),
    )
    .unwrap()
}

fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}
