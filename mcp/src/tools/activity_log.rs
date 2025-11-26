use anyhow::Result;
use rmcp::{RoleServer, handler::server::tool::Parameters, model::*, service::RequestContext};
use serde::{Deserialize, Serialize};

use lago_types::{
    filters::activity_log::ActivityLogFilters,
    models::{ActivitySource, PaginationParams},
    requests::activity_log::{GetActivityLogRequest, ListActivityLogsRequest},
};

use crate::tools::{create_lago_client, error_result, success_result};

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct ListActivityLogsArgs {
    /// Filter by activity types (e.g., "invoice.created", "billable_metric.created")
    pub activity_types: Option<Vec<String>>,
    /// Filter by activity sources: "api", "front", or "system"
    pub activity_sources: Option<Vec<String>>,
    /// Filter by user emails
    pub user_emails: Option<Vec<String>>,
    /// Filter by external customer ID
    pub external_customer_id: Option<String>,
    /// Filter by external subscription ID
    pub external_subscription_id: Option<String>,
    /// Filter by resource IDs
    pub resource_ids: Option<Vec<String>>,
    /// Filter by resource types (e.g., "Invoice", "BillableMetric", "Customer")
    pub resource_types: Option<Vec<String>>,
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
pub struct GetActivityLogArgs {
    /// The unique identifier of the activity log
    pub activity_id: String,
}

#[derive(Clone)]
pub struct ActivityLogService;

impl ActivityLogService {
    pub fn new() -> Self {
        Self
    }

    fn build_list_request(&self, params: &ListActivityLogsArgs) -> ListActivityLogsRequest {
        let mut filters = ActivityLogFilters::default();

        if let Some(activity_types) = &params.activity_types {
            filters = filters.with_activity_types(activity_types.clone());
        }

        if let Some(activity_sources) = &params.activity_sources {
            let sources: Vec<ActivitySource> = activity_sources
                .iter()
                .filter_map(|s| s.parse::<ActivitySource>().ok())
                .collect();
            if !sources.is_empty() {
                filters = filters.with_activity_sources(sources);
            }
        }

        if let Some(user_emails) = &params.user_emails {
            filters = filters.with_user_emails(user_emails.clone());
        }

        if let Some(external_customer_id) = &params.external_customer_id {
            filters = filters.with_external_customer_id(external_customer_id.clone());
        }

        if let Some(external_subscription_id) = &params.external_subscription_id {
            filters = filters.with_external_subscription_id(external_subscription_id.clone());
        }

        if let Some(resource_ids) = &params.resource_ids {
            filters = filters.with_resource_ids(resource_ids.clone());
        }

        if let Some(resource_types) = &params.resource_types {
            filters = filters.with_resource_types(resource_types.clone());
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

        ListActivityLogsRequest::new()
            .with_filters(filters)
            .with_pagination(pagination)
    }

    pub async fn list_activity_logs(
        &self,
        Parameters(args): Parameters<ListActivityLogsArgs>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let client = match create_lago_client(&context).await {
            Ok(client) => client,
            Err(error_result) => return Ok(error_result),
        };
        let request = self.build_list_request(&args);

        match client.list_activity_logs(Some(request)).await {
            Ok(response) => {
                let result = serde_json::json!({
                    "activity_logs": response.activity_logs,
                    "pagination": response.meta,
                });

                Ok(success_result(&result))
            }
            Err(e) => {
                let error_message = format!("Failed to list activity logs: {e}");
                Ok(error_result(error_message))
            }
        }
    }

    pub async fn get_activity_log(
        &self,
        Parameters(args): Parameters<GetActivityLogArgs>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let client = match create_lago_client(&context).await {
            Ok(client) => client,
            Err(error_result) => return Ok(error_result),
        };
        let request = GetActivityLogRequest::new(args.activity_id);

        match client.get_activity_log(request).await {
            Ok(response) => {
                let result = serde_json::json!({
                    "activity_log": response.activity_log,
                });

                Ok(success_result(&result))
            }
            Err(e) => {
                let error_message = format!("Failed to get activity log: {e}");
                Ok(error_result(error_message))
            }
        }
    }
}
