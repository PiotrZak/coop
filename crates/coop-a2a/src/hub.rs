//! Hub: AuthN → AuthZ → route → dispatch.

use crate::agent_backend::AgentRegistry;
use crate::models::{auth_failure, JsonRpcRequest, JsonRpcResponse};
use axum::http::{HeaderMap, StatusCode};
use coop_identity::IdentityAuth;
use std::sync::Arc;

#[derive(Clone)]
pub struct HubService {
    auth: Arc<IdentityAuth>,
    agents: AgentRegistry,
}

pub enum HubError {
    Auth {
        status: StatusCode,
        code: i32,
        message: String,
    },
    BadRequest {
        code: i32,
        message: String,
    },
    Upstream {
        message: String,
    },
}

impl HubService {
    pub fn new(auth: Arc<IdentityAuth>, agents: AgentRegistry) -> Self {
        Self { auth, agents }
    }

    pub async fn handle_message_send(
        &self,
        headers: &HeaderMap,
        req: JsonRpcRequest,
    ) -> Result<JsonRpcResponse, HubError> {
        let skill = req
            .params
            .get("skill")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let target = req
            .params
            .get("target_agent")
            .and_then(|v| v.as_str())
            .map(str::to_string);

        let (principal, trust, proof) =
            match self.auth.authenticate_and_authorize(headers, &skill) {
                Ok(v) => v,
                Err(e) => {
                    let msg = e.to_string();
                    let (status, code) = auth_failure(&msg);
                    return Err(HubError::Auth {
                        status,
                        code,
                        message: format!("{status}: {msg}"),
                    });
                }
            };

        let primary = match self.agents.resolve(target.as_deref(), &skill) {
            Ok(a) => a,
            Err(message) => {
                return Err(HubError::BadRequest {
                    code: -32602,
                    message,
                })
            }
        };
        let target_id = primary.id().to_string();

        tracing::info!(
            agent = %principal.agent_id,
            skill = %skill,
            target_agent = %target_id,
            trust = %trust,
            proof = %proof.as_str(),
            "hub message/send"
        );

        let result = match primary.dispatch_skill(&skill, &req).await {
            Ok(r) => Ok(r),
            Err(e) if target.is_none() => {
                if let Some(fb) = self.agents.fallback_for_skill(&skill, primary.id()) {
                    tracing::warn!(primary = %primary.id(), fallback = %fb.id(), error = %e, "fallback");
                    fb.dispatch_skill(&skill, &req).await
                } else {
                    Err(e)
                }
            }
            Err(e) => Err(e),
        };

        match result {
            Ok(mut resp) => {
                if let Some(obj) = resp.result.as_mut().and_then(|r| r.as_object_mut()) {
                    obj.insert(
                        "coop_trust".into(),
                        serde_json::json!({
                            "band": trust.as_str(),
                            "proof": proof.as_str(),
                            "agent_id": principal.agent_id,
                            "tenant_id": principal.tenant_id,
                            "target_agent": target_id,
                        }),
                    );
                }
                Ok(resp)
            }
            Err(e) => Err(HubError::Upstream {
                message: format!("upstream error: {e}"),
            }),
        }
    }
}
