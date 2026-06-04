pub mod activity_log;
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

use lago_client::{
    Config, Credentials, EnvironmentRegionProvider, LagoClient, Region, RegionProvider,
};
use rmcp::{
    RoleServer,
    model::{CallToolResult, Content},
    service::RequestContext,
};
use serde::Serialize;
use std::env;

pub struct LagoApiConfig {
    pub api_key: String,
    pub base_url: String,
}

/// Header carrying the per-request Lago API key (one per tenant).
const LAGO_API_KEY_HEADER: &str = "X-LAGO-API-KEY";
/// Header carrying the per-request Lago API base URL, e.g.
/// `https://api.getlago.com/api/v1` (US) or `https://api.eu.getlago.com/api/v1` (EU).
/// This is what lets a single hosted deployment serve multiple tenants across
/// multiple regions — region is resolved per request, not from server-wide env.
const LAGO_API_URL_HEADER: &str = "X-LAGO-API-URL";

/// Lago Cloud hosts permitted for the per-request URL header by default.
/// Self-hosted instances are allowed by setting `LAGO_ALLOW_ANY_API_URL=true`.
const ALLOWED_API_HOSTS: &[&str] = &["api.getlago.com", "api.eu.getlago.com"];

/// Read a trimmed, non-empty HTTP request header from the request context.
/// Returns `None` for stdio transport (no HTTP parts) or a missing/blank header.
fn request_header(context: &RequestContext<RoleServer>, name: &str) -> Option<String> {
    context
        .extensions
        .get::<axum::http::request::Parts>()
        .and_then(|parts| parts.headers.get(name))
        .and_then(|value| value.to_str().ok())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

/// Whether the operator has opted in to arbitrary (e.g. self-hosted) API URLs.
fn allow_any_api_url() -> bool {
    env::var("LAGO_ALLOW_ANY_API_URL")
        .map(|v| matches!(v.to_ascii_lowercase().as_str(), "1" | "true" | "yes"))
        .unwrap_or(false)
}

/// Validate a base URL that was supplied by an untrusted request header.
/// Guards the outbound-request (SSRF) surface: must be https, and the host must
/// be a known Lago Cloud host unless `LAGO_ALLOW_ANY_API_URL` is set.
fn validate_header_base_url(raw: &str) -> Result<(), CallToolResult> {
    let url = reqwest::Url::parse(raw)
        .map_err(|e| error_result(format!("Invalid '{LAGO_API_URL_HEADER}' URL '{raw}': {e}")))?;

    let allow_any = allow_any_api_url();
    match url.scheme() {
        "https" => {}
        "http" if allow_any => {}
        scheme => {
            return Err(error_result(format!(
                "'{LAGO_API_URL_HEADER}' must use https (got '{scheme}'): '{raw}'"
            )));
        }
    }

    if allow_any {
        return Ok(());
    }

    match url.host_str() {
        Some(host) if ALLOWED_API_HOSTS.contains(&host) => Ok(()),
        Some(host) => Err(error_result(format!(
            "'{LAGO_API_URL_HEADER}' host '{host}' is not allowed. Use \
             https://api.getlago.com/api/v1 (US) or https://api.eu.getlago.com/api/v1 (EU). \
             To allow self-hosted instances, set LAGO_ALLOW_ANY_API_URL=true on the server."
        ))),
        None => Err(error_result(format!(
            "'{LAGO_API_URL_HEADER}' has no host: '{raw}'"
        ))),
    }
}

/// Resolve the Lago API key + base URL for a single request.
///
/// Resolution order (per request, so one stateless deployment can serve many
/// tenants across US / EU / self-hosted Lago instances):
/// - API key:  `X-LAGO-API-KEY` header  ->  `LAGO_API_KEY` env
/// - Base URL: `X-LAGO-API-URL` header  ->  env region provider
///   (`LAGO_REGION` / `LAGO_API_URL`, default US `https://api.getlago.com/api/v1`)
///
/// URLs that arrive via the header are validated (https + host allowlist). URLs
/// that come from server env are trusted operator config and are not restricted.
pub async fn get_lago_api_config(
    context: &RequestContext<RoleServer>,
) -> Result<LagoApiConfig, CallToolResult> {
    let api_key = request_header(context, LAGO_API_KEY_HEADER)
        .or_else(|| env::var("LAGO_API_KEY").ok())
        .ok_or_else(|| {
            error_result(format!(
                "Missing Lago API key: send the '{LAGO_API_KEY_HEADER}' header or set LAGO_API_KEY on the server."
            ))
        })?;

    let base_url = match request_header(context, LAGO_API_URL_HEADER) {
        Some(url) => {
            validate_header_base_url(&url)?;
            url
        }
        None => EnvironmentRegionProvider::new()
            .provider_region()
            .map_err(|e| error_result(format!("Failed to resolve Lago region: {e}")))?
            .endpoint()
            .to_string(),
    };

    Ok(LagoApiConfig { api_key, base_url })
}

pub async fn create_lago_client(
    context: &RequestContext<RoleServer>,
) -> Result<LagoClient, CallToolResult> {
    let LagoApiConfig { api_key, base_url } = get_lago_api_config(context).await?;

    let config = Config::builder()
        .credentials(Credentials::new(api_key))
        .region(Region::Custom(base_url))
        .build();

    Ok(LagoClient::new(config))
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

#[cfg(test)]
mod tests {
    use super::*;

    // NOTE: these assert default behavior with LAGO_ALLOW_ANY_API_URL unset.
    #[test]
    fn accepts_us_and_eu_cloud_hosts() {
        assert!(validate_header_base_url("https://api.getlago.com/api/v1").is_ok());
        assert!(validate_header_base_url("https://api.eu.getlago.com/api/v1").is_ok());
    }

    #[test]
    fn rejects_unknown_host_by_default() {
        // SSRF guard: an arbitrary host must not be accepted without opt-in.
        assert!(validate_header_base_url("https://evil.example.com/api/v1").is_err());
    }

    #[test]
    fn rejects_non_https_by_default() {
        assert!(validate_header_base_url("http://api.getlago.com/api/v1").is_err());
    }

    #[test]
    fn rejects_malformed_url() {
        assert!(validate_header_base_url("not-a-url").is_err());
    }
}
