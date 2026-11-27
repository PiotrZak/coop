//! Shared HTTP + auth for adapters.

use anyhow::{bail, Context, Result};
use coop_application::{AgentBackendConfig, AuthStyle};
use reqwest::{Client, RequestBuilder};
use serde_json::{json, Value};

#[derive(Clone)]
pub struct HttpBackendBase {
    pub http: Client,
    pub cfg: AgentBackendConfig,
}

impl HttpBackendBase {
    pub fn new(cfg: AgentBackendConfig) -> Self {
        Self {
            http: Client::new(),
            cfg,
        }
    }

    pub fn config(&self) -> &AgentBackendConfig {
        &self.cfg
    }

    pub fn authed(&self, req: RequestBuilder) -> RequestBuilder {
        let req = req.header("Authorization", format!("Bearer {}", self.cfg.auth_secret));
        match self.cfg.auth_style {
            AuthStyle::Bearer => req,
            AuthStyle::BearerAndLindleHeader => {
                req.header("X-Lindle-Agents-Secret", &self.cfg.auth_secret)
            }
            AuthStyle::McpSecret => req.header("X-MCP-Secret", &self.cfg.auth_secret),
        }
    }

    pub async fn ensure_ready(&self, label: &str) -> Result<()> {
        let url = format!("{}{}", self.cfg.base_url, self.cfg.health_path);
        let resp = self
            .authed(self.http.get(&url))
            .send()
            .await
            .context("health request")?;
        if !resp.status().is_success() {
            bail!(
                "agent `{}` not ready at {} (HTTP {})",
                self.cfg.id,
                self.cfg.base_url,
                resp.status()
            );
        }
        tracing::info!(id = %self.cfg.id, kind = ?self.cfg.kind, url = %self.cfg.base_url, "{label}");
        Ok(())
    }

    pub async fn post_json(&self, path: &str, body: &Value) -> Result<Value> {
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
            bail!("backend {path} → HTTP {status}: {text}");
        }
        if text.is_empty() {
            return Ok(json!({ "ok": true }));
        }
        serde_json::from_str(&text).with_context(|| format!("decode {path}"))
    }
}
