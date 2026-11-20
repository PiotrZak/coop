//! Identity AuthN + domain RBAC AuthZ + trust score.

use coop_domain::{IdentityProof, Principal, RbacLevel, Role, Scope, TrustScore};
use coop_identity::ApiKeyDirectory;
use http::header::{HeaderMap, HeaderValue, AUTHORIZATION};

#[test]
fn bearer_auth_and_skill_scope() {
    let dir = ApiKeyDirectory::demo();
    let mut headers = HeaderMap::new();
    headers.insert(
        AUTHORIZATION,
        HeaderValue::from_static("Bearer nego-a-demo-key"),
    );
    let p = dir.authenticate(&headers).unwrap();
    assert_eq!(p.agent_id, "negotiation-a");
    assert_eq!(p.roles, vec![Role::Operator]);
    p.authorize("negotiate").unwrap();
    p.authorize("supplier.read").unwrap();
}

#[test]
fn authenticate_and_authorize_helper() {
    let dir = ApiKeyDirectory::demo();
    let mut headers = HeaderMap::new();
    headers.insert(
        AUTHORIZATION,
        HeaderValue::from_static("Bearer nego-a-demo-key"),
    );
    let (p, trust) = dir
        .authenticate_and_authorize(&headers, "negotiate")
        .unwrap();
    assert_eq!(p.agent_id, "negotiation-a");
    assert_eq!(trust, TrustScore::Medium);
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
}
