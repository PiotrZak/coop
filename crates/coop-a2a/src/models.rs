//! Transport DTOs + domain A2A re-exports.

pub use coop_domain::{
    completed_task, Artifact, Message, NegotiateParams, Part, SupplierLookupParams, Task,
    TaskStatus,
};

use axum::http::StatusCode;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub id: String,
    pub method: String,
    pub params: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
}

impl JsonRpcResponse {
    pub fn ok(id: impl Into<String>, result: serde_json::Value) -> Self {
        Self {
            jsonrpc: "2.0".into(),
            id: id.into(),
            result: Some(result),
            error: None,
        }
    }

    pub fn err(id: impl Into<String>, code: i32, message: impl Into<String>) -> Self {
        Self {
            jsonrpc: "2.0".into(),
            id: id.into(),
            result: None,
            error: Some(JsonRpcError {
                code,
                message: message.into(),
            }),
        }
    }
}

/// Map AuthN/AuthZ failure text → HTTP status + JSON-RPC code.
pub fn auth_failure(msg: &str) -> (StatusCode, i32) {
    if msg.contains("missing credentials") || msg.contains("invalid credentials") {
        (StatusCode::UNAUTHORIZED, -32001)
    } else {
        (StatusCode::FORBIDDEN, -32003)
    }
}
