use anyhow::Result;
use rmcp::{RoleServer, handler::server::tool::Parameters, model::*, service::RequestContext};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use lago_types::requests::event::{CreateEventInput, CreateEventRequest, GetEventRequest};

use crate::tools::{create_lago_client, error_result, success_result};

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
}
