use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub sender: String,
    pub recipient: String,
    pub content: String,
    #[serde(default)]
    pub message_type: String,
    pub timestamp: String,
    #[serde(default)]
    pub read: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub read_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendMessageRequest {
    pub sender: String,
    pub recipient: String,
    pub content: String,
    #[serde(default)]
    pub message_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserConnection {
    pub username: String,
    pub is_online: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectUserRequest {
    pub username: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}

impl<T> ApiResponse<T> {
    pub fn ok(data: T) -> Self {
        ApiResponse {
            success: true,
            data: Some(data),
            error: None,
        }
    }

    pub fn err(error: String) -> Self {
        ApiResponse {
            success: false,
            data: None,
            error: Some(error),
        }
    }
}

