//! A2A domain types — tasks, messages, skill inputs.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: Uuid,
    pub skill: String,
    pub status: TaskStatus,
    pub messages: Vec<Message>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub artifact: Option<Artifact>,
}

impl Task {
    pub fn completed(skill: &str, artifact: Artifact, echo: Message) -> Self {
        Self {
            id: Uuid::new_v4(),
            skill: skill.into(),
            status: TaskStatus::Completed,
            messages: vec![echo],
            artifact: Some(artifact),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
    Submitted,
    Working,
    Completed,
    Failed,
    InputRequired,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: String,
    pub parts: Vec<Part>,
}

impl Message {
    pub fn data_part<T: for<'de> Deserialize<'de>>(&self) -> Option<T> {
        self.parts.iter().find_map(|p| match p {
            Part::Data { data } => serde_json::from_value(data.clone()).ok(),
            _ => None,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Part {
    Text { text: String },
    Data { data: serde_json::Value },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Artifact {
    pub name: String,
    pub mime_type: String,
    pub data: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NegotiateParams {
    pub counterparty: String,
    pub category: String,
    pub ask: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub supplier_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupplierLookupParams {
    pub supplier_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
}

/// Back-compat alias used by bins/server.
pub fn completed_task(skill: &str, artifact: Artifact, echo: Message) -> Task {
    Task::completed(skill, artifact, echo)
}
