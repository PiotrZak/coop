use crate::card::AgentCard;
use crate::models::*;
use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Json, Router};
use coop_domain::Principal;
use coop_identity::ApiKeyDirectory;
use serde_json::{json, Value};
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use tower_http::cors::CorsLayer;

pub type SkillFuture = Pin<Box<dyn Future<Output = Result<Task, (StatusCode, String)>> + Send>>;
pub type SkillHandler = Arc<dyn Fn(Principal, String, Message) -> SkillFuture + Send + Sync>;

#[derive(Clone)]
pub struct AppState {
    pub card: AgentCard,
    pub auth: Arc<ApiKeyDirectory>,
    pub handler: SkillHandler,
}

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/.well-known/agent.json", get(agent_card))
        .route("/a2a", post(a2a_rpc))
        .layer(CorsLayer::permissive())
        .with_state(state)
}

async fn agent_card(State(state): State<AppState>) -> Json<AgentCard> {
    Json(state.card.clone())
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

    let principal = match state.auth.authenticate_and_authorize(&headers, &skill) {
        Ok(p) => p,
        Err(e) => {
            let msg = e.to_string();
            let (status, code) = auth_failure(&msg);
            return (
                status,
                Json(JsonRpcResponse::err(
                    req.id,
                    code,
                    format!("{status}: {msg}"),
                )),
            )
                .into_response();
        }
    };

    let message: Message =
        match serde_json::from_value(req.params.get("message").cloned().unwrap_or(Value::Null)) {
            Ok(m) => m,
            Err(e) => {
                return Json(JsonRpcResponse::err(
                    req.id,
                    -32602,
                    format!("invalid params: {e}"),
                ))
                .into_response();
            }
        };

    match (state.handler)(principal, skill, message).await {
        Ok(task) => Json(JsonRpcResponse::ok(req.id, json!(task))).into_response(),
        Err((status, msg)) => {
            (status, Json(JsonRpcResponse::err(req.id, -32000, msg))).into_response()
        }
    }
}

pub use coop_domain::completed_task;
