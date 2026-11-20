//! Application use cases.

use coop_application::{build_negotiation_pack, supplier_brief};
use coop_domain::{NegotiateParams, Principal, Role};

#[test]
fn negotiation_pack_hitl() {
    let p = Principal {
        agent_id: "caller".into(),
        tenant_id: "buyer".into(),
        roles: vec![Role::Operator],
    };
    let params = NegotiateParams {
        counterparty: "Acme".into(),
        category: "SaaS".into(),
        ask: "cut 10%".into(),
        supplier_id: Some("acme-cloud".into()),
    };
    let (artifact, msg) = build_negotiation_pack(
        "negotiation-b",
        &p,
        &params,
        Some(supplier_brief("acme-cloud", Some("SaaS"))),
    );
    assert_eq!(artifact.name, "negotiation_pack");
    assert_eq!(artifact.data["hitl"], true);
    assert_eq!(msg.role, "agent");
}

#[test]
fn supplier_brief_known() {
    assert_eq!(supplier_brief("acme-cloud", None)["name"], "Acme Cloud");
}
