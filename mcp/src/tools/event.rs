use anyhow::Result;
use rmcp::{RoleServer, handler::server::tool::Parameters, model::*, service::RequestContext};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use lago_types::{
    models::PaginationParams,
    requests::event::{CreateEventInput, CreateEventRequest, GetEventRequest, ListEventsRequest},
};

use crate::tools::{create_lago_client, error_result, success_result};

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct ListEventsArgs {
    /// Filter by external subscription ID.
    pub external_subscription_id: Option<String>,
    /// Filter by billable metric code.
    pub code: Option<String>,
    /// Requires `external_subscription_id` to be set. Filter events by timestamp after the subscription started at datetime.
    pub timestamp_from_started_at: Option<bool>,
    /// Filter events by timestamp starting from a specific date (ISO 8601 format, e.g., "2024-01-01T00:00:00Z").
    pub timestamp_from: Option<String>,
    /// Filter events by timestamp up to a specific date (ISO 8601 format, e.g., "2024-01-31T23:59:59Z").
    pub timestamp_to: Option<String>,
    /// Page number for pagination (default: 1).
    pub page: Option<i32>,
    /// Number of items per page (default: 20).
    pub per_page: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct GetEventArgs {
    /// The transaction ID of the event to retrieve (will be URL encoded automatically)
    pub transaction_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct CreateEventArgs {
    /// Unique identifier for this event (used for idempotency and retrieval)
    pub transaction_id: String,
    /// External customer ID - required if external_subscription_id is not provided
    pub external_customer_id: Option<String>,
    /// External subscription ID - required if external_customer_id is not provided
    pub external_subscription_id: Option<String>,
    /// Billable metric code
    pub code: String,
    /// Event timestamp (Unix timestamp in seconds). If not provided, the current time is used.
    pub timestamp: Option<i64>,
    /// Custom properties/metadata for the event (e.g., {"gb": 10, "region": "us-east"})
    pub properties: Option<Value>,
    /// Precise total amount in cents (e.g., 1234567 for $12,345.67)
    pub precise_total_amount_cents: Option<i64>,
}

#[derive(Clone)]
pub struct EventService;

impl EventService {
    pub fn new() -> Self {
        Self
    }

    pub async fn get_event(
        &self,
        Parameters(args): Parameters<GetEventArgs>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let client = match create_lago_client(&context).await {
            Ok(client) => client,
            Err(error_result) => return Ok(error_result),
        };

        let request = GetEventRequest::new(args.transaction_id.clone());

        match client.get_event(request).await {
            Ok(response) => {
                let result = serde_json::json!({
                    "event": response.event,
                });

                Ok(success_result(&result))
            }
            Err(e) => {
                let error_message = format!("Failed to get event: {e}");
                tracing::error!(
                    transaction_id = %args.transaction_id,
                    error = %e,
                    "{error_message}"
                );
                Ok(error_result(error_message))
            }
        }
    }

    pub async fn create_event(
        &self,
        Parameters(args): Parameters<CreateEventArgs>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        // Validate that either external_customer_id or external_subscription_id is provided
        if args.external_customer_id.is_none() && args.external_subscription_id.is_none() {
            return Ok(error_result(
                "Either external_customer_id or external_subscription_id must be provided",
            ));
        }

        let client = match create_lago_client(&context).await {
            Ok(client) => client,
            Err(error_result) => return Ok(error_result),
        };

        // Build the event input based on whether customer or subscription is provided
        let mut event_input = if let Some(customer_id) = args.external_customer_id.clone() {
            CreateEventInput::for_customer(
                args.transaction_id.clone(),
                customer_id,
                args.code.clone(),
            )
        } else {
            // Safe to unwrap due to validation above
            let subscription_id = args.external_subscription_id.clone().unwrap();
            CreateEventInput::for_subscription(
                args.transaction_id.clone(),
                subscription_id,
                args.code.clone(),
            )
        };

        // Apply optional fields
        if let Some(timestamp) = args.timestamp {
            event_input = event_input.with_timestamp(timestamp);
        }

        if let Some(properties) = args.properties {
            event_input = event_input.with_properties(properties);
        }

        if let Some(precise_amount) = args.precise_total_amount_cents {
            event_input = event_input.with_precise_total_amount_cents(precise_amount);
        }

        let request = CreateEventRequest::new(event_input);

        match client.create_event(request).await {
            Ok(response) => {
                let result = serde_json::json!({
                    "event": response.event,
                });

                Ok(success_result(&result))
            }
            Err(e) => {
                let error_message = format!("Failed to create event: {e}");
                tracing::error!(
                    transaction_id = %args.transaction_id,
                    code = %args.code,
                    error = %e,
                    "{error_message}"
                );
                Ok(error_result(error_message))
            }
        }
    }

    pub async fn list_events(
        &self,
        Parameters(args): Parameters<ListEventsArgs>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let client = match create_lago_client(&context).await {
            Ok(client) => client,
            Err(error_result) => return Ok(error_result),
        };

        let mut pagination = PaginationParams::new();
        if let Some(page) = args.page {
            pagination = pagination.with_page(page);
        }
        if let Some(per_page) = args.per_page {
            pagination = pagination.with_per_page(per_page);
        }

        let mut request = ListEventsRequest::new().with_pagination(pagination);

        if let Some(external_subscription_id) = args.external_subscription_id {
            request = request.with_external_subscription_id(external_subscription_id);
        }

        if let Some(code) = args.code {
            request = request.with_code(code);
        }

        if let Some(timestamp_from_started_at) = args.timestamp_from_started_at {
            request = request.with_timestamp_from_started_at(timestamp_from_started_at);
        }

        if let Some(timestamp_from) = args.timestamp_from {
            request = request.with_timestamp_from(timestamp_from);
        }

        if let Some(timestamp_to) = args.timestamp_to {
            request = request.with_timestamp_to(timestamp_to);
        }

        match client.list_events(Some(request)).await {
            Ok(response) => {
                let result = serde_json::json!({
                    "events": response.events,
                    "pagination": response.meta
                });

                Ok(success_result(&result))
            }
            Err(e) => {
                let error_message = format!("Failed to list events: {e}");
                tracing::error!("{error_message}");
                Ok(error_result(error_message))
            }
        }
    }
}
