//! Hub routing — target_agent vs skill.

use coop_a2a::agent_backend::{AgentBackend, AgentRegistry};
use coop_a2a::models::{JsonRpcRequest, JsonRpcResponse};
use coop_application::{AgentBackendConfig, AgentBackendKind, AuthStyle};
use serde_json::json;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

struct StubAgent {
    cfg: AgentBackendConfig,
}

impl StubAgent {
    fn stub(id: &str, skills: &[&str], priority: u32) -> Arc<dyn AgentBackend> {
        Arc::new(Self {
            cfg: AgentBackendConfig {
                id: id.into(),
                kind: AgentBackendKind::A2aPeer,
                base_url: format!("http://{id}.test"),
                auth_secret: "x".into(),
                auth_style: AuthStyle::Bearer,
                health_path: "/".into(),
                default_tenant_id: "t".into(),
                skills: skills.iter().map(|s| (*s).to_string()).collect(),
                priority,
                required: false,
            },
        })
    }
}

impl AgentBackend for StubAgent {
    fn config(&self) -> &AgentBackendConfig {
        &self.cfg
    }

    fn ensure_ready(&self) -> Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send + '_>> {
        Box::pin(async { Ok(()) })
    }

    fn dispatch_skill<'a>(
        &'a self,
        skill: &'a str,
        req: &'a JsonRpcRequest,
    ) -> Pin<Box<dyn Future<Output = anyhow::Result<JsonRpcResponse>> + Send + 'a>> {
        Box::pin(async move {
            Ok(JsonRpcResponse::ok(
                req.id.clone(),
                json!({ "agent": self.cfg.id, "skill": skill }),
            ))
        })
    }
}

#[test]
fn get_by_target_agent() {
    let reg = AgentRegistry::new(vec![
        StubAgent::stub("agent-a", &["peer.message", "negotiate"], 10),
        StubAgent::stub("lindle-orleans", &["negotiate", "investigate"], 20),
    ]);
    assert_eq!(
        reg.resolve(Some("agent-a"), "peer.message").unwrap().id(),
        "agent-a"
    );
    assert_eq!(
        reg.resolve(Some("lindle-orleans"), "negotiate")
            .unwrap()
            .id(),
        "lindle-orleans"
    );
}

#[test]
fn skill_fallback_picks_priority() {
    let reg = AgentRegistry::new(vec![
        StubAgent::stub("lindle-orleans", &["supplier.read"], 10),
        StubAgent::stub("lindle-mcp", &["supplier.read"], 0),
    ]);
    assert_eq!(
        reg.resolve(None, "supplier.read").unwrap().id(),
        "lindle-mcp"
    );
}

#[test]
fn target_agent_rejects_unsupported_skill() {
    let reg = AgentRegistry::new(vec![StubAgent::stub("agent-a", &["peer.message"], 1)]);
    match reg.resolve(Some("agent-a"), "negotiate") {
        Err(err) => assert!(err.contains("does not support")),
        Ok(_) => panic!("expected skill mismatch"),
    }
}
