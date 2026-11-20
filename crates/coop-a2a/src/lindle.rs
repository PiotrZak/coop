//! Lindle Orleans — concrete [`AgentBackend`] for [`AgentBackendKind::AgentsHttp`].

use crate::agent_backend::AgentBackend;
use crate::http_base::HttpBackendBase;
use crate::models::{
    completed_task, Artifact, JsonRpcRequest, JsonRpcResponse, Message, NegotiateParams, Part,
    SupplierLookupParams, Task,
};
use anyhow::{anyhow, bail, Context, Result};
use coop_application::AgentBackendConfig;
use coop_domain::Scope;
use serde_json::{json, Value};

/// Lindle Orleans HTTP adapter (specializes generic AgentsHttp).
#[derive(Clone)]
pub struct LindleOrleansClient {
    base: HttpBackendBase,
}

impl LindleOrleansClient {
    pub fn new(cfg: AgentBackendConfig) -> Self {
        Self {
            base: HttpBackendBase::new(cfg),
        }
    }

    async fn negotiate(&self, message: &Message) -> Result<Task> {
        let params: NegotiateParams = message.data_part().unwrap_or(NegotiateParams {
            counterparty: "unknown".into(),
            category: "general".into(),
            ask: "(empty)".into(),
            supplier_id: None,
        });

        let body = json!({
            "tenantId": self.base.cfg.default_tenant_id,
            "insightId": format!("coop:{}", params.supplier_id.as_deref().unwrap_or("a2a")),
            "supplierName": params.supplier_id,
            "insightSummary": params.ask,
            "strategy": params.category,
            "cashPriority": 0.5,
            "timePriority": 0.5,
            "savings": 0.0,
            "timelineDays": 30,
            "recommendation": params.ask,
            "basedOn": ["coop-a2a", "orchestrator"],
        });

        let data = self
            .post_json("/agents/negotiate", &body)
            .await
            .context("Lindle negotiate")?;

        Ok(completed_task(
            "negotiate",
            Artifact {
                name: "lindle_negotiation".into(),
                mime_type: "application/json".into(),
                data,
            },
            Message {
                role: "agent".into(),
                parts: vec![Part::Text {
                    text: "Negotiation result from Lindle Orleans Agents".into(),
                }],
            },
        ))
    }

    async fn supplier_read(&self, message: &Message) -> Result<Task> {
        let params: SupplierLookupParams = message.data_part().unwrap_or(SupplierLookupParams {
            supplier_id: "unknown".into(),
            category: None,
        });

        let body = json!({
            "tenantId": self.base.cfg.default_tenant_id,
            "jobId": format!("coop-supplier:{}", params.supplier_id),
            "chooseK": 4,
            "topN": 50,
        });

        let data = self
            .post_json("/agents/optimize-portfolio", &body)
            .await
            .context("Lindle supplier/portfolio")?;

        Ok(completed_task(
            "supplier.read",
            Artifact {
                name: "lindle_supplier_portfolio".into(),
                mime_type: "application/json".into(),
                data: json!({
                    "supplier_id": params.supplier_id,
                    "category": params.category,
                    "source": "lindle.orleans.optimize-portfolio",
                    "result": data,
                }),
            },
            Message {
                role: "agent".into(),
                parts: vec![Part::Text {
                    text: format!(
                        "Supplier/portfolio consult from Lindle for {}",
                        params.supplier_id
                    ),
                }],
            },
        ))
    }

    async fn investigate(&self, message: &Message) -> Result<Task> {
        let data = message
            .data_part::<Value>()
            .unwrap_or_else(|| json!({ "summary": "coop a2a investigate" }));

        let body = json!({
            "tenantId": data.get("tenantId").and_then(|v| v.as_str()).unwrap_or(&self.base.cfg.default_tenant_id),
            "insightId": data.get("insightId").and_then(|v| v.as_str()).unwrap_or("coop:investigate"),
            "correlationId": data.get("correlationId").cloned().unwrap_or(Value::Null),
            "ruleId": data.get("ruleId").cloned().unwrap_or(Value::Null),
            "severity": data.get("severity").and_then(|v| v.as_str()).unwrap_or("info"),
        });

        let accepted = self
            .post_json("/agents/investigate", &body)
            .await
            .context("Lindle investigate")?;

        Ok(completed_task(
            "investigate",
            Artifact {
                name: "lindle_investigate_accepted".into(),
                mime_type: "application/json".into(),
                data: accepted,
            },
            Message {
                role: "agent".into(),
                parts: vec![Part::Text {
                    text: "Investigation accepted by Lindle Orleans Agents (poll for phase)".into(),
                }],
            },
        ))
    }

    async fn post_json(&self, path: &str, body: &Value) -> Result<Value> {
        let url = format!("{}{path}", self.base.cfg.base_url);
        let resp = self
            .base
            .authed(self.base.http.post(&url))
            .json(body)
            .send()
            .await
            .with_context(|| format!("POST {path}"))?;
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        if !status.is_success() {
            return Err(anyhow!("backend {path} → HTTP {status}: {text}"));
        }
        if text.is_empty() {
            return Ok(json!({ "ok": true }));
        }
        serde_json::from_str(&text).with_context(|| format!("decode {path} body"))
    }
}

/// Alias kept for call sites / docs.
pub type LindleAgentsClient = LindleOrleansClient;

impl AgentBackend for LindleOrleansClient {
    fn config(&self) -> &AgentBackendConfig {
        self.base.config()
    }

    fn ensure_ready(&self) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + Send + '_>> {
        Box::pin(async move {
            let resp = self.base.health_get().await.context("backend health request")?;
            if !resp.status().is_success() {
                bail!(
                    "agent backend `{}` not ready at {} (health HTTP {})",
                    self.base.cfg.id,
                    self.base.cfg.base_url,
                    resp.status()
                );
            }
            tracing::info!(
                id = %self.base.cfg.id,
                kind = ?self.base.cfg.kind,
                url = %self.base.cfg.base_url,
                "AgentsHttp backend connected"
            );
            Ok(())
        })
    }

    fn dispatch_skill<'a>(
        &'a self,
        skill: &'a str,
        req: &'a JsonRpcRequest,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<JsonRpcResponse>> + Send + 'a>,
    > {
        Box::pin(async move {
            let message: Message =
                serde_json::from_value(req.params.get("message").cloned().unwrap_or(Value::Null))
                    .unwrap_or(Message {
                        role: "user".into(),
                        parts: vec![],
                    });

            let task = match Scope::from_skill(skill)? {
                Scope::Negotiate => self.negotiate(&message).await?,
                Scope::SupplierRead => self.supplier_read(&message).await?,
                Scope::Investigate => self.investigate(&message).await?,
            };

            Ok(JsonRpcResponse::ok(req.id.clone(), json!(task)))
        })
    }
}
