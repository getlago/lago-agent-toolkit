pub mod activity_log;
pub mod analytics;
pub mod api_log;
pub mod applied_coupon;
pub mod billable_metric;
pub mod coupon;
pub mod credit_note;
pub mod customer;
pub mod customer_usage;
pub mod event;
pub mod fee;
pub mod invoice;
pub mod payment;
pub mod plan;
pub mod subscription;

use axum::http::{HeaderMap, header::AUTHORIZATION, request::Parts};
use lago_client::{
    Config, Credentials, EnvironmentRegionProvider, LagoClient, Region, RegionProvider,
};
use rmcp::{
    RoleServer,
    model::{CallToolResult, ContentBlock},
    service::RequestContext,
};
use serde::Serialize;
use std::env;

pub struct LagoApiConfig {
    pub api_key: String,
    pub base_url: String,
}

/// Reads the caller's Lago API key from the request headers.
///
/// Precedence: `X-LAGO-API-KEY`, then `X-API-KEY`, then `Authorization: Bearer <key>`.
/// Empty values are treated as missing. The returned key must never be logged.
pub fn api_key_from_headers(headers: &HeaderMap) -> Option<String> {
    let non_empty = |value: &str| {
        let value = value.trim();
        (!value.is_empty()).then(|| value.to_string())
    };

    ["X-LAGO-API-KEY", "X-API-KEY"]
        .into_iter()
        .find_map(|name| {
            headers
                .get(name)
                .and_then(|v| v.to_str().ok())
                .and_then(non_empty)
        })
        .or_else(|| {
            let value = headers.get(AUTHORIZATION)?.to_str().ok()?.trim();
            let (scheme, token) = value.split_once(' ')?;
            scheme
                .eq_ignore_ascii_case("bearer")
                .then(|| non_empty(token))
                .flatten()
        })
}

/// Resolves credentials for an HTTP request. The key always comes from the
/// request headers, never from the `LAGO_API_KEY` environment variable.
fn http_api_config(parts: &Parts) -> Result<LagoApiConfig, CallToolResult> {
    let api_key = api_key_from_headers(&parts.headers).ok_or_else(|| {
        error_result(
            "Missing Lago API key: send it in the X-LAGO-API-KEY, X-API-KEY or Authorization: Bearer header",
        )
    })?;
    let base_url = match env::var("LAGO_API_URL") {
        Ok(url) => url,
        Err(_) => EnvironmentRegionProvider::new()
            .provider_region()
            .map_err(|e| error_result(format!("Failed to resolve region: {e}")))?
            .endpoint()
            .to_string(),
    };

    Ok(LagoApiConfig { api_key, base_url })
}

pub async fn get_lago_api_config(
    context: &RequestContext<RoleServer>,
) -> Result<LagoApiConfig, CallToolResult> {
    if let Some(parts) = context.extensions.get::<Parts>() {
        return http_api_config(parts);
    }

    let api_key = env::var("LAGO_API_KEY")
        .map_err(|_| error_result("LAGO_API_KEY environment variable not set"))?;
    let region = EnvironmentRegionProvider::new()
        .provider_region()
        .map_err(|e| error_result(format!("Failed to resolve region: {e}")))?;
    let base_url = region.endpoint().to_string();

    Ok(LagoApiConfig { api_key, base_url })
}

pub async fn create_lago_client(
    context: &RequestContext<RoleServer>,
) -> Result<LagoClient, CallToolResult> {
    if let Some(parts) = context.extensions.get::<Parts>() {
        let LagoApiConfig { api_key, base_url } = http_api_config(parts)?;
        let config = Config::builder()
            .credentials(Credentials::new(api_key))
            .region(Region::Custom(base_url))
            .build();
        return Ok(LagoClient::new(config));
    }

    LagoClient::from_env().map_err(|e| {
        let error_message = format!("Failed to create lago client: {e}");
        error_result(error_message)
    })
}

pub fn success_result<T: Serialize>(data: &T) -> CallToolResult {
    CallToolResult::success(vec![ContentBlock::text(
        serde_json::to_string_pretty(data)
            .unwrap_or_else(|_| "Failed to serialize result".to_string()),
    )])
}

pub fn error_result(message: impl Into<String>) -> CallToolResult {
    CallToolResult::error(vec![ContentBlock::text(message.into())])
}
