//! A2A peer adapter — forwards JSON-RPC to another agent.

use crate::agent_backend::AgentBackend;
use crate::client::A2aClient;
use crate::models::{JsonRpcRequest, JsonRpcResponse};
use anyhow::{bail, Context, Result};
use coop_application::AgentBackendConfig;

pub struct A2aPeerClient {
    cfg: AgentBackendConfig,
    a2a: A2aClient,
}

impl A2aPeerClient {
    pub fn new(cfg: AgentBackendConfig) -> Self {
        let a2a = A2aClient::new(&cfg.base_url, &cfg.auth_secret);
        Self { cfg, a2a }
    }
}

impl AgentBackend for A2aPeerClient {
    fn config(&self) -> &AgentBackendConfig {
        &self.cfg
    }

    fn ensure_ready(
        &self,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + Send + '_>> {
        Box::pin(async move {
            self.a2a.get_agent_card().await.context("peer agent card")?;
            tracing::info!(id = %self.cfg.id, url = %self.cfg.base_url, "A2aPeer connected");
            Ok(())
        })
    }

    fn dispatch_skill<'a>(
        &'a self,
        skill: &'a str,
        req: &'a JsonRpcRequest,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<JsonRpcResponse>> + Send + 'a>>
    {
        Box::pin(async move {
            if !self.cfg.supports(skill) {
                bail!("agent `{}` does not support skill `{skill}`", self.cfg.id);
            }
            let mut forward = req.clone();
            if let Some(obj) = forward.params.as_object_mut() {
                obj.insert("skill".into(), serde_json::json!(skill));
            }
            self.a2a
                .forward_rpc(&forward)
                .await
                .with_context(|| format!("forward to `{}`", self.cfg.id))
        })
    }
}
