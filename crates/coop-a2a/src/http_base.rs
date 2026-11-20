//! Shared HTTP + auth for agent backends (composition base).

use anyhow::Result;
use coop_application::{AgentBackendConfig, AuthStyle};
use reqwest::{Client, RequestBuilder};

/// Shared transport fields used by concrete HTTP/MCP adapters.
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

    pub async fn health_get(&self) -> Result<reqwest::Response> {
        let url = format!("{}{}", self.cfg.base_url, self.cfg.health_path);
        Ok(self.authed(self.http.get(&url)).send().await?)
    }
}
