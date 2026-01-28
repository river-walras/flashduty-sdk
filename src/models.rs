use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum EventStatus {
    Ok,
    Info,
    Warning,
    Critical,
}

#[derive(Debug, Clone, Serialize)]
pub struct Image {
    pub src: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub href: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alt: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AlertEvent {
    pub integration_key: Arc<str>,
    pub event_status: EventStatus,
    pub title_rule: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alert_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub labels: Option<HashMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub images: Option<Vec<Image>>,
}

#[derive(Debug, Deserialize)]
pub struct ApiResponse {
    #[serde(default)]
    pub error: String,
    #[serde(default)]
    pub message: String,
}
