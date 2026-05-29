use anyhow::Result;
use reqwest::Client;
use rmcp::{RoleServer, handler::server::tool::Parameters, model::*, service::RequestContext};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use lago_types::requests::fee::GetFeeRequest;

use crate::tools::{create_lago_client, error_result, get_lago_api_config, success_result};

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct ListFeesArgs {
    /// Filter by fee type. Valid values: 'charge' (usage-based fee from a billable metric),
    /// 'add_on' (one-off fee), 'subscription' (recurring base fee from a plan),
    /// 'credit' (credit fee), 'commitment' (minimum commitment fee).
    /// For MRR calculations: 'subscription' fees are always recurring revenue; 'charge'
    /// fees are usage-based and may or may not count toward MRR depending on the underlying
    /// billable metric (e.g., 'seats' typically counts, 'api_calls' typically does not).
    pub fee_type: Option<String>,
    /// Filter by the billable metric code (e.g., "seats", "storage_gb"). Useful for
    /// isolating specific usage charges that the customer counts as recurring revenue.
    pub billable_metric_code: Option<String>,
    /// Filter by the customer external ID.
    pub external_customer_id: Option<String>,
    /// Filter by the subscription external ID.
    pub external_subscription_id: Option<String>,
    /// Filter by ISO 4217 currency code (e.g., "USD", "EUR").
    pub currency: Option<String>,
    /// Filter pay-in-advance charge fees by the source event transaction ID. Use this to jump
    /// from an event to the exact rated fee line when reconciling event-to-invoice math.
    pub event_transaction_id: Option<String>,
    /// Filter by payment status. Valid values: 'pending', 'succeeded', 'failed', 'refunded'.
    pub payment_status: Option<String>,
    /// Filter by fee creation date (from). Format: YYYY-MM-DD or ISO 8601.
    pub created_at_from: Option<String>,
    /// Filter by fee creation date (to). Format: YYYY-MM-DD or ISO 8601.
    pub created_at_to: Option<String>,
    /// Filter by succeeded date (from). Format: YYYY-MM-DD or ISO 8601.
    pub succeeded_at_from: Option<String>,
    /// Filter by succeeded date (to). Format: YYYY-MM-DD or ISO 8601.
    pub succeeded_at_to: Option<String>,
    /// Filter by failed date (from). Format: YYYY-MM-DD or ISO 8601.
    pub failed_at_from: Option<String>,
    /// Filter by failed date (to). Format: YYYY-MM-DD or ISO 8601.
    pub failed_at_to: Option<String>,
    /// Filter by refunded date (from). Format: YYYY-MM-DD or ISO 8601.
    pub refunded_at_from: Option<String>,
    /// Filter by refunded date (to). Format: YYYY-MM-DD or ISO 8601.
    pub refunded_at_to: Option<String>,
    /// Page number for pagination (default: 1).
    pub page: Option<i32>,
    /// Number of results per page (default: 20, max: 100).
    pub per_page: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct GetFeeArgs {
    /// The Lago ID (UUID) of the fee to retrieve.
    pub lago_id: String,
}

#[derive(Clone)]
pub struct FeeService {
    http_client: Client,
}

impl FeeService {
    pub fn new() -> Self {
        Self {
            http_client: Client::new(),
        }
    }

    fn build_query_params(args: ListFeesArgs) -> Vec<(&'static str, String)> {
        let mut params: Vec<(&'static str, String)> = Vec::new();

        push_param(&mut params, "fee_type", args.fee_type);
        push_param(
            &mut params,
            "billable_metric_code",
            args.billable_metric_code,
        );
        push_param(
            &mut params,
            "external_customer_id",
            args.external_customer_id,
        );
        push_param(
            &mut params,
            "external_subscription_id",
            args.external_subscription_id,
        );
        push_param(&mut params, "currency", args.currency);
        push_param(
            &mut params,
            "event_transaction_id",
            args.event_transaction_id,
        );
        push_param(&mut params, "payment_status", args.payment_status);
        push_param(&mut params, "created_at_from", args.created_at_from);
        push_param(&mut params, "created_at_to", args.created_at_to);
        push_param(&mut params, "succeeded_at_from", args.succeeded_at_from);
        push_param(&mut params, "succeeded_at_to", args.succeeded_at_to);
        push_param(&mut params, "failed_at_from", args.failed_at_from);
        push_param(&mut params, "failed_at_to", args.failed_at_to);
        push_param(&mut params, "refunded_at_from", args.refunded_at_from);
        push_param(&mut params, "refunded_at_to", args.refunded_at_to);

        if let Some(page) = args.page {
            params.push(("page", page.to_string()));
        }
        if let Some(per_page) = args.per_page {
            params.push(("per_page", per_page.to_string()));
        }

        params
    }
}

impl FeeService {
    pub async fn list_fees(
        &self,
        Parameters(args): Parameters<ListFeesArgs>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let config = match get_lago_api_config(&context).await {
            Ok(config) => config,
            Err(error_result) => return Ok(error_result),
        };

        let params = Self::build_query_params(args);
        let url = format!("{}/fees", config.base_url);

        match self
            .http_client
            .get(&url)
            .bearer_auth(&config.api_key)
            .query(&params)
            .send()
            .await
        {
            Ok(response) if response.status().is_success() => {
                match response.json::<Value>().await {
                    Ok(json) => {
                        let result = serde_json::json!({
                            "fees": json.get("fees").cloned().unwrap_or(Value::Array(vec![])),
                            "pagination": json.get("meta")
                                .or_else(|| json.get("pagination"))
                                .cloned(),
                        });

                        Ok(success_result(&result))
                    }
                    Err(e) => {
                        let error_message = format!("Failed to parse fees response: {e}");
                        tracing::error!("{error_message}");
                        Ok(error_result(error_message))
                    }
                }
            }
            Ok(response) => {
                let status = response.status();
                let body = response
                    .text()
                    .await
                    .unwrap_or_else(|_| "Unknown error".to_string());
                let error_message = format!("Failed to list fees (HTTP {status}): {body}");
                tracing::error!("{error_message}");
                Ok(error_result(error_message))
            }
            Err(e) => {
                let error_message = format!("Failed to list fees: {e}");
                tracing::error!("{error_message}");
                Ok(error_result(error_message))
            }
        }
    }

    pub async fn get_fee(
        &self,
        Parameters(args): Parameters<GetFeeArgs>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let client = match create_lago_client(&context).await {
            Ok(client) => client,
            Err(error_result) => return Ok(error_result),
        };
        let request = GetFeeRequest::new(args.lago_id);

        match client.get_fee(request).await {
            Ok(response) => {
                let result = serde_json::json!({
                    "fee": response.fee,
                });

                Ok(success_result(&result))
            }
            Err(e) => {
                let error_message = format!("Failed to get fee: {e}");
                tracing::error!("{error_message}");
                Ok(error_result(error_message))
            }
        }
    }
}

fn push_param(params: &mut Vec<(&'static str, String)>, name: &'static str, value: Option<String>) {
    if let Some(value) = value {
        params.push((name, value));
    }
}
