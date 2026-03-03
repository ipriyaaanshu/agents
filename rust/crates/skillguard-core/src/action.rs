use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A single action that a skill can perform.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillAction {
    pub name: String,
    pub description: String,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub parameters: HashMap<String, serde_json::Value>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub returns: HashMap<String, serde_json::Value>,
}

/// Status of a skill execution.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SkillStatus {
    Success,
    Error,
    Denied,
    Timeout,
}

/// Result of executing a skill action.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillResult {
    pub status: SkillStatus,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error_message: Option<String>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub metadata: HashMap<String, serde_json::Value>,
}

impl SkillResult {
    pub fn success(data: serde_json::Value) -> Self {
        Self {
            status: SkillStatus::Success,
            data: Some(data),
            error_message: None,
            metadata: HashMap::new(),
        }
    }

    pub fn error(message: impl Into<String>) -> Self {
        Self {
            status: SkillStatus::Error,
            data: None,
            error_message: Some(message.into()),
            metadata: HashMap::new(),
        }
    }

    pub fn denied(reason: impl Into<String>) -> Self {
        Self {
            status: SkillStatus::Denied,
            data: None,
            error_message: Some(reason.into()),
            metadata: HashMap::new(),
        }
    }

    pub fn timeout(seconds: u64) -> Self {
        Self {
            status: SkillStatus::Timeout,
            data: None,
            error_message: Some(format!("Execution timed out after {seconds}s")),
            metadata: HashMap::new(),
        }
    }
}
