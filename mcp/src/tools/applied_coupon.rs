use anyhow::Result;
use rmcp::{RoleServer, handler::server::tool::Parameters, model::*, service::RequestContext};
use serde::{Deserialize, Serialize};

use lago_types::{
    filters::applied_coupon::AppliedCouponFilter,
    models::{AppliedCouponFrequency, AppliedCouponStatus, PaginationParams},
    requests::applied_coupon::{ApplyCouponInput, ApplyCouponRequest, ListAppliedCouponsRequest},
};

use crate::tools::{create_lago_client, error_result, success_result};

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct ListAppliedCouponsArgs {
    pub status: Option<String>,
    pub external_customer_id: Option<String>,
    pub coupon_codes: Option<Vec<String>>,
    pub page: Option<i32>,
    pub per_page: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct ApplyCouponArgs {
    pub external_customer_id: String,
    pub coupon_code: String,
    pub frequency: Option<String>,
    pub frequency_duration: Option<i32>,
    pub amount_cents: Option<i64>,
    pub amount_currency: Option<String>,
    pub percentage_rate: Option<String>,
}

#[derive(Clone)]
pub struct AppliedCouponService;

impl AppliedCouponService {
    pub fn new() -> Self {
        Self
    }

    fn build_list_request(&self, params: &ListAppliedCouponsArgs) -> ListAppliedCouponsRequest {
        let mut filters = AppliedCouponFilter::new();

        if let Some(status_str) = &params.status
            && let Ok(status) = status_str.parse::<AppliedCouponStatus>()
        {
            filters = filters.with_status(status);
        }

        if let Some(customer_id) = &params.external_customer_id {
            filters = filters.with_external_customer_id(customer_id.clone());
        }

        if let Some(coupon_codes) = &params.coupon_codes {
            filters = filters.with_coupon_codes(coupon_codes.clone());
        }

        let mut pagination = PaginationParams::default();
        if let Some(page) = params.page {
            pagination = pagination.with_page(page);
        }

        if let Some(per_page) = params.per_page {
            pagination = pagination.with_per_page(per_page);
        }

        ListAppliedCouponsRequest::new()
            .with_filters(filters)
            .with_pagination(pagination)
    }

    pub async fn list_applied_coupons(
        &self,
        Parameters(args): Parameters<ListAppliedCouponsArgs>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let client = match create_lago_client(&context).await {
            Ok(client) => client,
            Err(error_result) => return Ok(error_result),
        };
        let request = self.build_list_request(&args);

        match client.list_applied_coupons(Some(request)).await {
            Ok(response) => {
                let result = serde_json::json!({
                    "applied_coupons": response.applied_coupons,
                    "pagination": response.meta,
                });

                Ok(success_result(&result))
            }
            Err(e) => {
                let error_message = format!("Failed to list applied coupons: {e}");
                Ok(error_result(error_message))
            }
        }
    }

    pub async fn apply_coupon(
        &self,
        Parameters(args): Parameters<ApplyCouponArgs>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let mut input =
            ApplyCouponInput::new(args.external_customer_id.clone(), args.coupon_code.clone());

        if let Some(frequency_str) = &args.frequency
            && let Ok(frequency) = frequency_str.parse::<AppliedCouponFrequency>()
        {
            input = input.with_frequency(frequency);
        }

        if let Some(duration) = args.frequency_duration {
            input = input.with_frequency_duration(duration);
        }

        if let (Some(amount_cents), Some(currency)) = (args.amount_cents, &args.amount_currency) {
            input = input.with_fixed_amount(amount_cents, currency.clone());
        }

        if let Some(rate) = &args.percentage_rate {
            input = input.with_percentage_rate(rate.clone());
        }

        let request = ApplyCouponRequest::new(input);

        let client = match create_lago_client(&context).await {
            Ok(client) => client,
            Err(error_result) => return Ok(error_result),
        };

        match client.apply_coupon(request).await {
            Ok(response) => {
                let result = serde_json::json!({
                    "applied_coupon": response.applied_coupon,
                });

                Ok(success_result(&result))
            }
            Err(e) => {
                let error_message = format!("Failed to apply coupon: {e}");
                Ok(error_result(error_message))
            }
        }
    }
}
