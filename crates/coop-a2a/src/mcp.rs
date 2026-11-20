//! Lindle Backend MCP client — `tools/call` for A2A skills.

use crate::models::{
    completed_task, Artifact, JsonRpcRequest, JsonRpcResponse, Message, Part, SupplierLookupParams,
    Task,
};
use anyhow::{anyhow, bail, Context, Result};
use coop_application::AgentBackendConfig;
use reqwest::Client;
use serde_json::{json, Value};

#[derive(Clone)]
pub struct LindleMcpClient {
    http: Client,
    cfg: AgentBackendConfig,
}

impl LindleMcpClient {
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
        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "protocolVersion": "2024-11-05",
                "capabilities": {},
                "clientInfo": { "name": "coop-gateway", "version": "0.1.0" }
            }
        });
        let resp = self
            .post_mcp(&body)
            .await
            .context("MCP initialize")?;
        if resp.get("error").is_some() {
            bail!("MCP initialize error: {resp}");
        }
        tracing::info!(
            id = %self.cfg.id,
            url = %self.cfg.base_url,
            "Lindle MCP backend connected"
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

        let task = match skill {
            "supplier.read" => self.supplier_lookup(&message).await?,
            other => bail!("MCP backend does not handle skill `{other}`"),
        };

        Ok(JsonRpcResponse::ok(req.id.clone(), json!(task)))
    }

    async fn supplier_lookup(&self, message: &Message) -> Result<Task> {
        let params: SupplierLookupParams = message.data_part().unwrap_or(SupplierLookupParams {
            supplier_id: "unknown".into(),
            category: None,
        });

        let args = json!({
            "vendor_name": params.supplier_id,
        });

        let data = self
            .tools_call("supplier-lookup", args)
            .await
            .context("MCP supplier-lookup")?;

        Ok(completed_task(
            "supplier.read",
            Artifact {
                name: "lindle_mcp_supplier_lookup".into(),
                mime_type: "application/json".into(),
                data: json!({
                    "supplier_id": params.supplier_id,
                    "category": params.category,
                    "source": "lindle.mcp.supplier-lookup",
                    "result": data,
                }),
            },
            Message {
                role: "agent".into(),
                parts: vec![Part::Text {
                    text: format!(
                        "Supplier lookup via Lindle MCP for {}",
                        params.supplier_id
                    ),
                }],
            },
        ))
    }

    async fn tools_call(&self, name: &str, arguments: Value) -> Result<Value> {
        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "tools/call",
            "params": {
                "name": name,
                "arguments": arguments
            }
        });
        let resp = self.post_mcp(&body).await?;
        if let Some(err) = resp.get("error") {
            return Err(anyhow!("MCP tools/call error: {err}"));
        }
        Ok(resp
            .get("result")
            .cloned()
            .unwrap_or(resp))
    }

    async fn post_mcp(&self, body: &Value) -> Result<Value> {
        let url = if self.cfg.base_url.ends_with("/mcp") {
            self.cfg.base_url.clone()
        } else {
            format!("{}/mcp", self.cfg.base_url)
        };

        let resp = self
            .http
            .post(&url)
            .header("Content-Type", "application/json")
            .header("Accept", "application/json, text/event-stream")
            .header("X-Tenant-ID", &self.cfg.default_tenant_id)
            .header("Authorization", format!("Bearer {}", self.cfg.auth_secret))
            .header("X-MCP-Secret", &self.cfg.auth_secret)
            .json(body)
            .send()
            .await
            .with_context(|| format!("POST {url}"))?;
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        if !status.is_success() {
            return Err(anyhow!("MCP {url} → HTTP {status}: {text}"));
        }
        if text.is_empty() {
            return Ok(json!({ "ok": true }));
        }
        // Streamable HTTP may return SSE; take first JSON object if needed
        if let Ok(v) = serde_json::from_str::<Value>(&text) {
            return Ok(v);
        }
        for line in text.lines() {
            let line = line.trim();
            if let Some(data) = line.strip_prefix("data:") {
                if let Ok(v) = serde_json::from_str::<Value>(data.trim()) {
                    return Ok(v);
                }
            }
        }
        Err(anyhow!("MCP decode failed: {text}"))
    }
}
