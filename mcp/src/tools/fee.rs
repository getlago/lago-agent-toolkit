use anyhow::Result;
use rmcp::{RoleServer, handler::server::tool::Parameters, model::*, service::RequestContext};
use serde::{Deserialize, Serialize};

use lago_types::{
    filters::fee::FeeFilters,
    models::{FeePaymentStatus, FeeType, PaginationParams},
    requests::fee::{GetFeeRequest, ListFeesRequest},
};

use crate::tools::{create_lago_client, error_result, success_result};

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
    /// Filter by payment status. Valid values: 'pending', 'succeeded', 'failed', 'refunded'.
    pub payment_status: Option<String>,
    /// Filter by fee creation date (from). Format: YYYY-MM-DD or ISO 8601.
    pub created_at_from: Option<String>,
    /// Filter by fee creation date (to). Format: YYYY-MM-DD or ISO 8601.
    pub created_at_to: Option<String>,
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
pub struct FeeService;

impl FeeService {
    pub fn new() -> Self {
        Self
    }

    #[allow(clippy::collapsible_if)]
    fn build_request(&self, args: &ListFeesArgs) -> ListFeesRequest {
        let mut filters = FeeFilters::new();

        if let Some(customer_id) = &args.external_customer_id {
            filters = filters.with_customer_id(customer_id.clone());
        }

        if let Some(from) = &args.created_at_from {
            filters = filters.with_created_at_from(from.clone());
        }

        if let Some(to) = &args.created_at_to {
            filters = filters.with_created_at_to(to.clone());
        }

        if let Some(fee_type_str) = &args.fee_type {
            if let Ok(fee_type) = fee_type_str.parse::<FeeType>() {
                filters = filters.with_fee_type(fee_type);
            }
        }

        if let Some(payment_status_str) = &args.payment_status {
            if let Ok(payment_status) = payment_status_str.parse::<FeePaymentStatus>() {
                filters = filters.with_payment_status(payment_status);
            }
        }

        if let Some(code) = &args.billable_metric_code {
            filters = filters.with_billable_metric_code(code.clone());
        }

        if let Some(sub_id) = &args.external_subscription_id {
            filters = filters.with_external_subscription_id(sub_id.clone());
        }

        if let Some(currency) = &args.currency {
            filters = filters.with_currency(currency.clone());
        }

        let mut pagination = PaginationParams::default();
        if let Some(page) = args.page {
            pagination = pagination.with_page(page);
        }

        if let Some(per_page) = args.per_page {
            pagination = pagination.with_per_page(per_page);
        }

        ListFeesRequest::new()
            .with_filters(filters)
            .with_pagination(pagination)
    }
}

impl FeeService {
    pub async fn list_fees(
        &self,
        Parameters(args): Parameters<ListFeesArgs>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let client = match create_lago_client(&context).await {
            Ok(client) => client,
            Err(error_result) => return Ok(error_result),
        };
        let request = self.build_request(&args);

        match client.list_fees(Some(request)).await {
            Ok(response) => {
                let result = serde_json::json!({
                    "fees": response.fees,
                    "pagination": response.meta,
                });

                Ok(success_result(&result))
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
