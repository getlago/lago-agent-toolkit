pub mod billable_metric;
pub mod customer;
pub mod invoice;

use lago_client::{Config, Credentials, LagoClient, Region};
use rmcp::model::{CallToolResult, Content};
use serde::Serialize;

use crate::server::LagoMcpServer;

pub async fn create_lago_client(server: &LagoMcpServer) -> Result<LagoClient, CallToolResult> {
    let creds = server.get_credentials().await;

    match (creds.api_key, creds.api_url) {
        (Some(key), Some(url)) => {
            let credentials = Credentials::new(key);
            let region = Region::Custom(url);

            let config = Config::builder()
                .credentials(credentials)
                .region(region)
                .build();

            Ok(LagoClient::new(config))
        }
        _ => LagoClient::from_env().map_err(|e| {
            let error_message = format!("Failed to create Lago client: {e}");
            error_result(error_message)
        }),
    }
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
