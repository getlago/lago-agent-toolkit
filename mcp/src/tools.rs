pub mod activity_log;
pub mod api_log;
pub mod applied_coupon;
pub mod billable_metric;
pub mod coupon;
pub mod customer;
pub mod event;
pub mod invoice;

use lago_client::{Config, Credentials, LagoClient, Region};
use rmcp::{
    RoleServer,
    model::{CallToolResult, Content},
    service::RequestContext,
};
use serde::Serialize;
use std::env;

pub async fn create_lago_client(
    context: &RequestContext<RoleServer>,
) -> Result<LagoClient, CallToolResult> {
    let (header_key, header_url) = context
        .extensions
        .get::<axum::http::request::Parts>()
        .map(|parts| {
            let key = parts
                .headers
                .get("X-LAGO-API-KEY")
                .and_then(|v| v.to_str().ok())
                .map(|s| s.to_string());
            let url = env::var("LAGO_API_URL").ok();
            (key, url)
        })
        .unwrap_or((None, None));

    if let (Some(key), Some(url)) = (header_key, header_url) {
        let credentials = Credentials::new(key);
        let region = Region::Custom(url);
        let config = Config::builder()
            .credentials(credentials)
            .region(region)
            .build();
        return Ok(LagoClient::new(config));
    }

    LagoClient::from_env().map_err(|e| {
        let error_message = format!("Failed to create lago client: {e}");
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
