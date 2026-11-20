//! Generic AgentBackendConfig — Lindle Orleans / MCP entities.

use coop_application::{AgentBackendConfig, AgentBackendKind, AuthStyle, OrchestratorConfig};

fn orleans() -> AgentBackendConfig {
    AgentBackendConfig {
        id: "lindle-orleans".into(),
        kind: AgentBackendKind::Lindle,
        base_url: "http://127.0.0.1:8080".into(),
        auth_secret: "hunter2-do-not-leak".into(),
        auth_style: AuthStyle::BearerAndLindleHeader,
        health_path: "/health".into(),
        default_tenant_id: "t".into(),
        skills: vec!["negotiate".into(), "investigate".into(), "supplier.read".into()],
        priority: 10,
        required: true,
    }
}

fn mcp() -> AgentBackendConfig {
    AgentBackendConfig {
        id: "lindle-mcp".into(),
        kind: AgentBackendKind::LindleMcp,
        base_url: "http://127.0.0.1:8000/mcp".into(),
        auth_secret: "mcp-secret".into(),
        auth_style: AuthStyle::McpSecret,
        health_path: "/mcp".into(),
        default_tenant_id: "t".into(),
        skills: vec!["supplier.read".into()],
        priority: 0,
        required: false,
    }
}

#[test]
fn mcp_preferred_for_supplier_read() {
    let mut orch = OrchestratorConfig::default();
    orch.push(orleans());
    orch.push(mcp());
    let chosen = orch.backend_for_skill("supplier.read").unwrap();
    assert_eq!(chosen.kind, AgentBackendKind::LindleMcp);
    assert_eq!(chosen.priority, 0);
}

#[test]
fn negotiate_uses_orleans() {
    let mut orch = OrchestratorConfig::default();
    orch.push(orleans());
    orch.push(mcp());
    let chosen = orch.backend_for_skill("negotiate").unwrap();
    assert_eq!(chosen.kind, AgentBackendKind::Lindle);
}

#[test]
fn secret_redacted_in_debug() {
    let dbg = format!("{:?}", orleans());
    assert!(dbg.contains("***"));
    assert!(!dbg.contains("hunter2-do-not-leak"));
}
