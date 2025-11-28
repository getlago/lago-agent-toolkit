use anyhow::Result;
use rmcp::{RoleServer, handler::server::tool::Parameters, model::*, service::RequestContext};
use serde::{Deserialize, Serialize};

use lago_types::{
    filters::api_log::ApiLogFilters,
    models::{HttpMethod, HttpStatus, PaginationParams, StatusOutcome},
    requests::api_log::{GetApiLogRequest, ListApiLogsRequest},
};

use crate::tools::{create_lago_client, error_result, success_result};

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct ListApiLogsArgs {
    /// Filter by HTTP methods: "post", "put", or "delete"
    pub http_methods: Option<Vec<String>>,
    /// Filter by HTTP statuses: numeric codes (200, 404, 500) or outcomes ("succeeded", "failed")
    pub http_statuses: Option<Vec<String>>,
    /// Filter by API version (e.g., "v1")
    pub api_version: Option<String>,
    /// Filter by request paths (e.g., "/invoices", "/customers")
    pub request_paths: Option<Vec<String>>,
    /// Filter logs from this date (format: YYYY-MM-DD)
    pub from_date: Option<String>,
    /// Filter logs until this date (format: YYYY-MM-DD)
    pub to_date: Option<String>,
    /// Page number for pagination
    pub page: Option<i32>,
    /// Number of items per page
    pub per_page: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct GetApiLogArgs {
    /// The unique request ID of the API log
    pub request_id: String,
}

#[derive(Clone)]
pub struct ApiLogService;

impl ApiLogService {
    pub fn new() -> Self {
        Self
    }

    fn build_list_request(&self, params: &ListApiLogsArgs) -> ListApiLogsRequest {
        let mut filters = ApiLogFilters::default();

        if let Some(http_methods) = &params.http_methods {
            let methods: Vec<HttpMethod> = http_methods
                .iter()
                .filter_map(|m| m.parse::<HttpMethod>().ok())
                .collect();
            if !methods.is_empty() {
                filters = filters.with_http_methods(methods);
            }
        }

        if let Some(http_statuses) = &params.http_statuses {
            let statuses: Vec<HttpStatus> = http_statuses
                .iter()
                .filter_map(|s| {
                    // Try parsing as numeric status code first
                    if let Ok(code) = s.parse::<i32>() {
                        Some(HttpStatus::Code(code))
                    } else {
                        // Try parsing as outcome
                        s.parse::<StatusOutcome>().ok().map(HttpStatus::Outcome)
                    }
                })
                .collect();
            if !statuses.is_empty() {
                filters = filters.with_http_statuses(statuses);
            }
        }

        if let Some(api_version) = &params.api_version {
            filters = filters.with_api_version(api_version.clone());
        }

        if let Some(request_paths) = &params.request_paths {
            filters = filters.with_request_paths(request_paths.clone());
        }

        if let Some(from_date) = &params.from_date {
            filters = filters.with_from_date(from_date.clone());
        }

        if let Some(to_date) = &params.to_date {
            filters = filters.with_to_date(to_date.clone());
        }

        let mut pagination = PaginationParams::default();
        if let Some(page) = params.page {
            pagination = pagination.with_page(page);
        }
        if let Some(per_page) = params.per_page {
            pagination = pagination.with_per_page(per_page);
        }

        ListApiLogsRequest::new()
            .with_filters(filters)
            .with_pagination(pagination)
    }

    pub async fn list_api_logs(
        &self,
        Parameters(args): Parameters<ListApiLogsArgs>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let client = match create_lago_client(&context).await {
            Ok(client) => client,
            Err(error_result) => return Ok(error_result),
        };
        let request = self.build_list_request(&args);

        match client.list_api_logs(Some(request)).await {
            Ok(response) => {
                let result = serde_json::json!({
                    "api_logs": response.api_logs,
                    "pagination": response.meta,
                });

                Ok(success_result(&result))
            }
            Err(e) => {
                let error_message = format!("Failed to list API logs: {e}");
                tracing::error!("{error_message}");
                Ok(error_result(error_message))
            }
        }
    }

    pub async fn get_api_log(
        &self,
        Parameters(args): Parameters<GetApiLogArgs>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let client = match create_lago_client(&context).await {
            Ok(client) => client,
            Err(error_result) => return Ok(error_result),
        };
        let request = GetApiLogRequest::new(args.request_id);

        match client.get_api_log(request).await {
            Ok(response) => {
                let result = serde_json::json!({
                    "api_log": response.api_log,
                });

                Ok(success_result(&result))
            }
            Err(e) => {
                let error_message = format!("Failed to get API log: {e}");
                tracing::error!("{error_message}");
                Ok(error_result(error_message))
            }
        }
    }
}
