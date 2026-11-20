use crate::card::AgentCard;
use crate::models::*;
use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Json, Router};
use coop_identity::{ApiKeyDirectory, Principal};
use serde_json::{json, Value};
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use uuid::Uuid;

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
    Json(state.card)
}

async fn a2a_rpc(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<JsonRpcRequest>,
) -> impl IntoResponse {
    let principal = match state.auth.authenticate_headers(&headers) {
        Ok(p) => p,
        Err(e) => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(JsonRpcResponse {
                    jsonrpc: "2.0".into(),
                    id: req.id.clone(),
                    result: None,
                    error: Some(JsonRpcError {
                        code: -32001,
                        message: format!("401 Unauthorized: {e}"),
                    }),
                }),
            )
                .into_response();
        }
    };

    if req.method != "message/send" {
        return (
            StatusCode::OK,
            Json(JsonRpcResponse {
                jsonrpc: "2.0".into(),
                id: req.id,
                result: None,
                error: Some(JsonRpcError {
                    code: -32601,
                    message: format!("method not found: {}", req.method),
                }),
            }),
        )
            .into_response();
    }

    let skill = req
        .params
        .get("skill")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    if let Err(e) = state.auth.authorize(&principal, &skill) {
        return (
            StatusCode::FORBIDDEN,
            Json(JsonRpcResponse {
                jsonrpc: "2.0".into(),
                id: req.id,
                result: None,
                error: Some(JsonRpcError {
                    code: -32003,
                    message: format!("403 Forbidden: {e}"),
                }),
            }),
        )
            .into_response();
    }

    let message: Message =
        match serde_json::from_value(req.params.get("message").cloned().unwrap_or(Value::Null)) {
            Ok(m) => m,
            Err(e) => {
                return (
                    StatusCode::OK,
                    Json(JsonRpcResponse {
                        jsonrpc: "2.0".into(),
                        id: req.id,
                        result: None,
                        error: Some(JsonRpcError {
                            code: -32602,
                            message: format!("invalid params: {e}"),
                        }),
                    }),
                )
                    .into_response();
            }
        };

    match (state.handler)(principal, skill, message).await {
        Ok(task) => (
            StatusCode::OK,
            Json(JsonRpcResponse {
                jsonrpc: "2.0".into(),
                id: req.id,
                result: Some(json!(task)),
                error: None,
            }),
        )
            .into_response(),
        Err((status, msg)) => (
            status,
            Json(JsonRpcResponse {
                jsonrpc: "2.0".into(),
                id: req.id,
                result: None,
                error: Some(JsonRpcError {
                    code: -32000,
                    message: msg,
                }),
            }),
        )
            .into_response(),
    }
}

pub fn completed_task(skill: &str, artifact: Artifact, echo: Message) -> Task {
    Task {
        id: Uuid::new_v4(),
        skill: skill.into(),
        status: TaskStatus::Completed,
        messages: vec![echo],
        artifact: Some(artifact),
    }
}
