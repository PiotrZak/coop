//! Coop auth hub — thin HTTP over [`HubService`].

use anyhow::{bail, Result};
use axum::extract::State;
use axum::http::HeaderMap;
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Json, Router};
use clap::Parser;
use coop_a2a::card::AgentCard;
use coop_a2a::hub::{HubError, HubService};
use coop_a2a::models::{JsonRpcRequest, JsonRpcResponse};
use coop_a2a::orchestrator_http::{build_card, connect_backends};
use coop_application::OrchestratorConfig;
use coop_identity::IdentityAuth;
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
    hub: HubService,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("info".parse()?))
        .init();

    let args = Args::parse();
    let orch = OrchestratorConfig::from_env()?;
    if orch.backends.is_empty() {
        bail!("set AGENTS_SERVICE_URL / LINDLE_MCP_URL / COOP_PEERS_JSON");
    }
    let agents = connect_backends(&orch).await?;
    let auth = Arc::new(IdentityAuth::from_env().await?);
    let card = build_card(&format!("http://{}", args.addr), &orch, &auth);
    let hub = HubService::new(auth, agents);

    let app = Router::new()
        .route(
            "/.well-known/agent.json",
            get(|State(s): State<AppState>| async move { Json(s.card) }),
        )
        .route(
            "/health",
            get(|| async { Json(serde_json::json!({ "ok": true, "role": "auth_hub" })) }),
        )
        .route("/a2a", post(a2a_rpc))
        .layer(CorsLayer::permissive())
        .with_state(AppState { card, hub });

    tracing::info!("coop auth hub on http://{}", args.addr);
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
    let id = req.id.clone();
    match state.hub.handle_message_send(&headers, req).await {
        Ok(resp) => Json(resp).into_response(),
        Err(HubError::Auth {
            status,
            code,
            message,
        }) => (status, Json(JsonRpcResponse::err(id, code, message))).into_response(),
        Err(HubError::BadRequest { code, message }) => (
            axum::http::StatusCode::BAD_REQUEST,
            Json(JsonRpcResponse::err(id, code, message)),
        )
            .into_response(),
        Err(HubError::Upstream { message }) => (
            axum::http::StatusCode::BAD_GATEWAY,
            Json(JsonRpcResponse::err(id, -32002, message)),
        )
            .into_response(),
    }
}
