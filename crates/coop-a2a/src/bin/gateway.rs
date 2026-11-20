//! Coop orchestrator — A2A edge for foreign agents → Lindle Orleans / MCP.
//!
//! Env:
//! - Orleans: `AGENTS_SERVICE_URL`, `AGENTS_INTERNAL_SECRET`
//! - MCP (optional): `LINDLE_MCP_URL`, `MCP_SERVER_SECRET`
//!
//! Listen address defaults to `127.0.0.1:7090` (override with `--addr`).

use anyhow::{bail, Result};
use axum::extract::State;
use axum::http::HeaderMap;
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Json, Router};
use clap::Parser;
use coop_a2a::card::AgentCard;
use coop_a2a::lindle::LindleAgentsClient;
use coop_a2a::mcp::LindleMcpClient;
use coop_a2a::models::{auth_failure, JsonRpcRequest, JsonRpcResponse};
use coop_a2a::orchestrator_http::{build_card, connect_backends};
use coop_application::{AgentBackendKind, OrchestratorConfig};
use coop_identity::ApiKeyDirectory;
use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use tracing_subscriber::EnvFilter;

#[derive(Parser, Debug)]
struct Args {
    #[arg(long, default_value = "127.0.0.1:7090")]
    addr: SocketAddr,
}

#[derive(Clone)]
struct AppState {
    card: AgentCard,
    auth: Arc<ApiKeyDirectory>,
    orch: Arc<OrchestratorConfig>,
    orleans: Option<LindleAgentsClient>,
    mcp: Option<LindleMcpClient>,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("info".parse()?))
        .init();

    let args = Args::parse();
    let orch = OrchestratorConfig::from_env()?;
    if orch.backends.is_empty() {
        bail!(
            "set AGENTS_SERVICE_URL + AGENTS_INTERNAL_SECRET and/or LINDLE_MCP_URL + MCP_SERVER_SECRET"
        );
    }
    let connected = connect_backends(&orch).await?;
    let public_url = format!("http://{}", args.addr);
    let card = build_card(&public_url, &orch);

    let app = Router::new()
        .route(
            "/.well-known/agent.json",
            get(|State(s): State<AppState>| async move { Json(s.card) }),
        )
        .route(
            "/health",
            get(|| async {
                Json(serde_json::json!({ "ok": true, "role": "orchestrator" }))
            }),
        )
        .route("/a2a", post(a2a_rpc))
        .layer(CorsLayer::permissive())
        .with_state(AppState {
            card,
            auth: Arc::new(ApiKeyDirectory::demo()),
            orch: Arc::new(orch),
            orleans: connected.orleans,
            mcp: connected.mcp,
        });

    tracing::info!("coop orchestrator listening on http://{}", args.addr);
    axum::serve(tokio::net::TcpListener::bind(args.addr).await?, app).await?;
    Ok(())
}

async fn a2a_rpc(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<JsonRpcRequest>,
) -> impl IntoResponse {
    if req.method != "message/send" {
        return Json(JsonRpcResponse::err(
            req.id,
            -32601,
            format!("method not found: {}", req.method),
        ))
        .into_response();
    }

    let skill = req
        .params
        .get("skill")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    let (principal, trust) = match state.auth.authenticate_and_authorize(&headers, &skill) {
        Ok(v) => v,
        Err(e) => {
            let msg = e.to_string();
            let (status, code) = auth_failure(&msg);
            return (
                status,
                Json(JsonRpcResponse::err(req.id, code, format!("{status}: {msg}"))),
            )
                .into_response();
        }
    };

    tracing::info!(
        agent = %principal.agent_id,
        tenant = %principal.tenant_id,
        skill = %skill,
        trust = %trust,
        "a2a message/send authorized"
    );

    let Some(backend) = state.orch.backend_for_skill(&skill) else {
        return (
            axum::http::StatusCode::BAD_REQUEST,
            Json(JsonRpcResponse::err(
                req.id,
                -32602,
                format!("unknown skill `{skill}`"),
            )),
        )
            .into_response();
    };

    let result = match backend.kind {
        AgentBackendKind::LindleMcp => {
            if let Some(mcp) = &state.mcp {
                mcp.dispatch_skill(&skill, &req).await
            } else if let Some(orleans) = &state.orleans {
                tracing::warn!(skill = %skill, "MCP down — falling back to Orleans");
                orleans.dispatch_skill(&skill, &req).await
            } else {
                Err(anyhow::anyhow!("MCP backend not connected for `{skill}`"))
            }
        }
        AgentBackendKind::Lindle => match &state.orleans {
            Some(orleans) => orleans.dispatch_skill(&skill, &req).await,
            None => Err(anyhow::anyhow!("Orleans backend not connected for `{skill}`")),
        },
    };

    match result {
        Ok(mut resp) => {
            if let Some(result) = resp.result.as_mut() {
                if let Some(obj) = result.as_object_mut() {
                    obj.insert(
                        "coop_trust".into(),
                        serde_json::json!({
                            "band": trust.as_str(),
                            "agent_id": principal.agent_id,
                            "tenant_id": principal.tenant_id,
                        }),
                    );
                }
            }
            Json(resp).into_response()
        }
        Err(e) => (
            axum::http::StatusCode::BAD_GATEWAY,
            Json(JsonRpcResponse::err(req.id, -32002, format!("upstream error: {e}"))),
        )
            .into_response(),
    }
}
