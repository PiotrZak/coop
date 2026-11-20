//! Generic runtime agent backend — trait composition (not class inheritance).
//!
//! Concrete adapters (Lindle Orleans HTTP, Lindle MCP, …) implement [`AgentBackend`].
//! The gateway only talks to this trait + [`BackendRegistry`].

use crate::models::{JsonRpcRequest, JsonRpcResponse};
use anyhow::Result;
use coop_application::AgentBackendConfig;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

/// Runtime adapter for one [`AgentBackendConfig`].
pub trait AgentBackend: Send + Sync {
    fn config(&self) -> &AgentBackendConfig;

    fn id(&self) -> &str {
        &self.config().id
    }

    fn ensure_ready(&self) -> BoxFuture<'_, Result<()>>;

    fn dispatch_skill<'a>(
        &'a self,
        skill: &'a str,
        req: &'a JsonRpcRequest,
    ) -> BoxFuture<'a, Result<JsonRpcResponse>>;
}

/// Connected backends ordered by priority; dispatch by skill without product match arms.
#[derive(Clone, Default)]
pub struct BackendRegistry {
    backends: Vec<Arc<dyn AgentBackend>>,
}

impl BackendRegistry {
    pub fn new(backends: Vec<Arc<dyn AgentBackend>>) -> Self {
        let mut backends = backends;
        backends.sort_by_key(|b| b.config().priority);
        Self { backends }
    }

    pub fn is_empty(&self) -> bool {
        self.backends.is_empty()
    }

    pub fn for_skill(&self, skill: &str) -> Option<Arc<dyn AgentBackend>> {
        self.backends
            .iter()
            .filter(|b| b.config().supports(skill))
            .min_by_key(|b| b.config().priority)
            .cloned()
    }

    pub fn fallback_for_skill(
        &self,
        skill: &str,
        exclude_id: &str,
    ) -> Option<Arc<dyn AgentBackend>> {
        self.backends
            .iter()
            .filter(|b| b.config().supports(skill) && b.id() != exclude_id)
            .min_by_key(|b| b.config().priority)
            .cloned()
    }
}
