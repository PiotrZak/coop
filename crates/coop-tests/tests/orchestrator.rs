//! Generic AgentBackendConfig — Lindle / LocalA2a are entities of that base.

use coop_application::{
    AgentBackendConfig, AgentBackendKind, AuthStyle, LocalA2aBackend, OrchestratorConfig,
};

fn lindle() -> AgentBackendConfig {
    AgentBackendConfig {
        id: "lindle-orleans".into(),
        kind: AgentBackendKind::Lindle,
        base_url: "http://127.0.0.1:8080".into(),
        auth_secret: "hunter2-do-not-leak".into(),
        auth_style: AuthStyle::BearerAndLindleHeader,
        health_path: "/health".into(),
        default_tenant_id: "t".into(),
        skills: vec![
            "negotiate".into(),
            "supplier.read".into(),
            "investigate".into(),
        ],
        priority: 0,
        required: true,
    }
}

#[test]
fn lindle_is_backend_config_entity() {
    let b = lindle();
    assert_eq!(b.kind, AgentBackendKind::Lindle);
    assert!(b.supports("negotiate"));
}

#[test]
fn skill_picks_highest_priority() {
    let mut orch = OrchestratorConfig::default();
    orch.push(LocalA2aBackend::create(
        "local-nego",
        "http://127.0.0.1:7101",
        "k",
        vec!["negotiate".into()],
    ));
    orch.push(lindle());
    let chosen = orch.backend_for_skill("negotiate").unwrap();
    assert_eq!(chosen.kind, AgentBackendKind::Lindle);
    assert_eq!(chosen.priority, 0);
}

#[test]
fn local_when_lindle_absent() {
    let mut orch = OrchestratorConfig::default();
    orch.push(LocalA2aBackend::create(
        "local",
        "http://127.0.0.1:7101",
        "k",
        vec!["negotiate".into()],
    ));
    assert_eq!(
        orch.backend_for_skill("negotiate").unwrap().kind,
        AgentBackendKind::LocalA2a
    );
}

#[test]
fn secret_redacted_in_debug() {
    let dbg = format!("{:?}", lindle());
    assert!(dbg.contains("***"));
    assert!(!dbg.contains("hunter2-do-not-leak"));
}

#[test]
fn parse_local_routes() {
    let (cfgs, router) = LocalA2aBackend::from_routes(
        "negotiate=http://127.0.0.1:7101@key,supplier.read=http://127.0.0.1:7102@key",
    )
    .unwrap();
    assert_eq!(cfgs.len(), 2);
    assert!(router.get("negotiate").is_some());
    assert_eq!(cfgs[0].kind, AgentBackendKind::LocalA2a);
}
