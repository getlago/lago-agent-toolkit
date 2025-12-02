use anyhow::Result;
use rmcp::{RoleServer, handler::server::tool::Parameters, model::*, service::RequestContext};
use serde::{Deserialize, Serialize};

use lago_types::requests::customer_usage::GetCustomerCurrentUsageRequest;

use crate::tools::{create_lago_client, error_result, success_result};

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct GetCustomerCurrentUsageArgs {
    /// The external unique identifier of the customer (provided by your own application).
    pub external_customer_id: String,
    /// The unique identifier of the subscription within your application.
    pub external_subscription_id: String,
    /// Optional flag to determine if taxes should be applied. Defaults to true if not provided.
    pub apply_taxes: Option<bool>,
}

#[derive(Clone)]
pub struct CustomerUsageService;

impl CustomerUsageService {
    pub fn new() -> Self {
        Self
    }

    pub async fn get_customer_current_usage(
        &self,
        Parameters(args): Parameters<GetCustomerCurrentUsageArgs>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let client = match create_lago_client(&context).await {
            Ok(client) => client,
            Err(error_result) => return Ok(error_result),
        };

        let mut request = GetCustomerCurrentUsageRequest::new(
            args.external_customer_id,
            args.external_subscription_id,
        );

        if let Some(apply_taxes) = args.apply_taxes {
            request = request.with_apply_taxes(apply_taxes);
        }

        match client.get_customer_current_usage(request).await {
            Ok(response) => {
                let result = serde_json::json!({
                    "customer_usage": response.customer_usage,
                });

                Ok(success_result(&result))
            }
            Err(e) => {
                let error_message = format!("Failed to get customer current usage: {e}");
                tracing::error!("{error_message}");
                Ok(error_result(error_message))
            }
        }
    }
}
