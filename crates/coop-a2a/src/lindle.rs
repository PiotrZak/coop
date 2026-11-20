//! Lindle Orleans adapter — implements dispatch for [`AgentBackendKind::Lindle`].
//!
//! Config is the generic [`AgentBackendConfig`]; Lindle only specializes HTTP paths.

use crate::models::{
    completed_task, Artifact, JsonRpcRequest, JsonRpcResponse, Message, NegotiateParams, Part,
    SupplierLookupParams, Task,
};
use anyhow::{anyhow, bail, Context, Result};
use coop_application::{AgentBackendConfig, AuthStyle};
use coop_domain::Scope;
use reqwest::Client;
use serde_json::{json, Value};

#[derive(Clone)]
pub struct LindleAgentsClient {
    http: Client,
    cfg: AgentBackendConfig,
}

impl LindleAgentsClient {
    pub fn new(cfg: AgentBackendConfig) -> Self {
        Self {
            http: Client::new(),
            cfg,
        }
    }

    pub fn config(&self) -> &AgentBackendConfig {
        &self.cfg
    }

    pub async fn ensure_ready(&self) -> Result<()> {
        let url = format!("{}{}", self.cfg.base_url, self.cfg.health_path);
        let resp = self
            .authed(self.http.get(&url))
            .send()
            .await
            .context("backend health request")?;
        if !resp.status().is_success() {
            bail!(
                "agent backend `{}` not ready at {} (health HTTP {})",
                self.cfg.id,
                self.cfg.base_url,
                resp.status()
            );
        }
        tracing::info!(
            id = %self.cfg.id,
            kind = ?self.cfg.kind,
            url = %self.cfg.base_url,
            "primary agent backend connected"
        );
        Ok(())
    }

    pub async fn dispatch_skill(
        &self,
        skill: &str,
        req: &JsonRpcRequest,
    ) -> Result<JsonRpcResponse> {
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
    }

    async fn negotiate(&self, message: &Message) -> Result<Task> {
        let params: NegotiateParams = message.data_part().unwrap_or(NegotiateParams {
            counterparty: "unknown".into(),
            category: "general".into(),
            ask: "(empty)".into(),
            supplier_id: None,
        });

        let body = json!({
            "tenantId": self.cfg.default_tenant_id,
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
            "tenantId": self.cfg.default_tenant_id,
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
            "tenantId": data.get("tenantId").and_then(|v| v.as_str()).unwrap_or(&self.cfg.default_tenant_id),
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
        let url = format!("{}{path}", self.cfg.base_url);
        let resp = self
            .authed(self.http.post(&url))
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

    fn authed(&self, req: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        let req = req.header("Authorization", format!("Bearer {}", self.cfg.auth_secret));
        match self.cfg.auth_style {
            AuthStyle::Bearer | AuthStyle::McpSecret => req,
            AuthStyle::BearerAndLindleHeader => {
                req.header("X-Lindle-Agents-Secret", &self.cfg.auth_secret)
            }
        }
    }
}
