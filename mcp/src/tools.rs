pub mod billable_metric;
pub mod customer;
pub mod invoice;

use lago_client::LagoClient;
use rmcp::model::{CallToolResult, Content};
use serde::Serialize;

pub fn create_lago_client() -> Result<LagoClient, CallToolResult> {
    LagoClient::from_env().map_err(|e| {
        let error_message = format!("Failed to create Lago client: {e}");
        error_result(error_message)
    })
}

pub fn success_result<T: Serialize>(data: &T) -> CallToolResult {
    CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(data)
            .unwrap_or_else(|_| "Failed to serialize result".to_string()),
    )])
}

pub fn error_result(message: impl Into<String>) -> CallToolResult {
    CallToolResult::error(vec![Content::text(message.into())])
}