//! Coop orchestrator — A2A edge over generic [`AgentBackendConfig`] entities.
//!
//! Env (minimum for Lindle): `AGENTS_SERVICE_URL`, `AGENTS_INTERNAL_SECRET`.
//! Listen address defaults to `127.0.0.1:7090` (override with `--addr`).

use anyhow::Result;
use axum::extract::State;
use axum::http::HeaderMap;
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Json, Router};
use clap::Parser;
use coop_a2a::card::AgentCard;
use coop_a2a::lindle::LindleAgentsClient;
use coop_a2a::models::{auth_failure, JsonRpcRequest, JsonRpcResponse};
use coop_a2a::orchestrator_http::{build_card, connect_backends};
use coop_a2a::proxy::{forward, SkillRouter};
use coop_application::{AgentBackendKind, LocalA2aBackend, OrchestratorConfig};
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
    local: Arc<SkillRouter>,
    orch: Arc<OrchestratorConfig>,
    lindle: Option<LindleAgentsClient>,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("info".parse()?))
        .init();

    let args = Args::parse();
    let orch = OrchestratorConfig::from_env()?;
    let lindle = connect_backends(&orch).await?;
    let local = LocalA2aBackend::demo_router();
    let public_url = format!("http://{}", args.addr);
    let card = build_card(&public_url, &orch, lindle.is_some(), &local).await;

    let app = Router::new()
        .route(
            "/.well-known/agent.json",
            get(|State(s): State<AppState>| async move { Json(s.card) }),
        )
        .route(
            "/health",
            get(|| async { Json(serde_json::json!({ "ok": true, "role": "orchestrator" })) }),
        )
        .route("/a2a", post(a2a_rpc))
        .layer(CorsLayer::permissive())
        .with_state(AppState {
            card,
            auth: Arc::new(ApiKeyDirectory::demo()),
            local: Arc::new(local),
            orch: Arc::new(orch),
            lindle,
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

    if let Err(e) = state.auth.authenticate_and_authorize(&headers, &skill) {
        let msg = e.to_string();
        let (status, code) = auth_failure(&msg);
        return (
            status,
            Json(JsonRpcResponse::err(req.id, code, format!("{status}: {msg}"))),
        )
            .into_response();
    }

    let result = match state.orch.backend_for_skill(&skill).map(|b| b.kind) {
        Some(AgentBackendKind::Lindle) if state.lindle.is_some() => {
            state
                .lindle
                .as_ref()
                .unwrap()
                .dispatch_skill(&skill, &req)
                .await
        }
        _ if state.local.get(&skill).is_some() => forward(&state.local, &skill, &req).await,
        Some(AgentBackendKind::Lindle) => {
            Err(anyhow::anyhow!("Lindle backend not connected for `{skill}`"))
        }
        _ => Err(anyhow::anyhow!("no upstream for skill `{skill}`")),
    };

    match result {
        Ok(resp) => Json(resp).into_response(),
        Err(e) => (
            axum::http::StatusCode::BAD_GATEWAY,
            Json(JsonRpcResponse::err(req.id, -32002, format!("upstream error: {e}"))),
        )
            .into_response(),
    }
}
