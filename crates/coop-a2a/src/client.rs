use crate::models::*;
use anyhow::{anyhow, Context, Result};
use reqwest::Client;
use serde_json::json;
use uuid::Uuid;

/// Thin A2A client: JSON-RPC over HTTP with Bearer AuthN.
#[derive(Clone)]
pub struct A2aClient {
    http: Client,
    base_url: String,
    bearer: String,
}

impl A2aClient {
    pub fn new(base_url: impl Into<String>, bearer: impl Into<String>) -> Self {
        Self {
            http: Client::new(),
            base_url: base_url.into().trim_end_matches('/').to_string(),
            bearer: bearer.into(),
        }
    }

    pub async fn get_agent_card(&self) -> Result<crate::card::AgentCard> {
        let url = format!("{}/.well-known/agent.json", self.base_url);
        self.http
            .get(url)
            .send()
            .await
            .context("agent card fetch")?
            .error_for_status()
            .context("agent card status")?
            .json()
            .await
            .context("agent card json")
    }

    /// POST raw JSON-RPC to `/a2a`.
    pub async fn forward_rpc(&self, req: &JsonRpcRequest) -> Result<JsonRpcResponse> {
        let url = format!("{}/a2a", self.base_url);
        self.http
            .post(url)
            .header("Authorization", format!("Bearer {}", self.bearer))
            .json(req)
            .send()
            .await
            .context("forward_rpc")?
            .error_for_status()
            .context("forward_rpc status")?
            .json()
            .await
            .context("forward_rpc json")
    }

    pub async fn message_send(&self, skill: &str, message: Message) -> Result<Task> {
        self.message_send_to(skill, message, None).await
    }

    pub async fn message_send_to(
        &self,
        skill: &str,
        message: Message,
        target_agent: Option<&str>,
    ) -> Result<Task> {
        let mut params = json!({
            "skill": skill,
            "message": message,
        });
        if let Some(id) = target_agent {
            params
                .as_object_mut()
                .unwrap()
                .insert("target_agent".into(), json!(id));
        }
        let req = JsonRpcRequest {
            jsonrpc: "2.0".into(),
            id: Uuid::new_v4().to_string(),
            method: "message/send".into(),
            params,
        };

        let resp = self.forward_rpc(&req).await?;

        if let Some(err) = resp.error {
            return Err(anyhow!("A2A error {}: {}", err.code, err.message));
        }
        let result = resp.result.ok_or_else(|| anyhow!("empty A2A result"))?;
        serde_json::from_value(result).context("decode Task")
    }

    /// Call hub `negotiate` skill (optional `target_agent` adapter id).
    pub async fn negotiate(&self, params: NegotiateParams) -> Result<Task> {
        self.negotiate_to(params, None).await
    }

    pub async fn negotiate_to(
        &self,
        params: NegotiateParams,
        target_agent: Option<&str>,
    ) -> Result<Task> {
        let message = Message {
            role: "user".into(),
            parts: vec![Part::Data {
                data: serde_json::to_value(params)?,
            }],
        };
        self.message_send_to("negotiate", message, target_agent)
            .await
    }

    /// Consult Supplier Agent for a particular supplier_id.
    pub async fn supplier_lookup(&self, params: SupplierLookupParams) -> Result<Task> {
        let message = Message {
            role: "user".into(),
            parts: vec![Part::Data {
                data: serde_json::to_value(params)?,
            }],
        };
        self.message_send("supplier.read", message).await
    }
}
