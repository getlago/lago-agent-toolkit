use anyhow::Result;
use rmcp::{RoleServer, handler::server::tool::Parameters, model::*, service::RequestContext};
use serde::{Deserialize, Serialize};

use lago_types::{
    models::{CouponExpiration, CouponFrequency, PaginationParams},
    requests::coupon::{
        CreateCouponInput, CreateCouponRequest, DeleteCouponRequest, GetCouponRequest,
        ListCouponsRequest, UpdateCouponInput, UpdateCouponRequest,
    },
};

use crate::tools::{create_lago_client, error_result, success_result};

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct ListCouponsArgs {
    /// Page number for pagination
    pub page: Option<i32>,
    /// Number of items per page
    pub per_page: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct GetCouponArgs {
    /// The unique code of the coupon to retrieve
    pub code: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct CreateCouponArgs {
    /// Display name for the coupon
    pub name: String,
    /// Unique code for the coupon (used for identification)
    pub code: String,
    /// Type of coupon: "fixed_amount" or "percentage"
    pub coupon_type: String,
    /// Discount amount in cents (required for fixed_amount type)
    pub amount_cents: Option<i64>,
    /// Currency code for the discount amount (required for fixed_amount type)
    pub amount_currency: Option<String>,
    /// Discount percentage rate as string (required for percentage type, e.g., "10.5" for 10.5%)
    pub percentage_rate: Option<String>,
    /// Frequency of coupon application: "once", "recurring", or "forever"
    pub frequency: String,
    /// Number of billing periods the coupon applies to (for recurring frequency)
    pub frequency_duration: Option<i32>,
    /// Whether the coupon can be applied to multiple customers
    pub reusable: Option<bool>,
    /// Plan codes to limit the coupon to (optional)
    pub plan_codes: Option<Vec<String>>,
    /// Billable metric codes to limit the coupon to (optional)
    pub billable_metric_codes: Option<Vec<String>>,
    /// Expiration policy: "no_expiration" or "time_limit"
    pub expiration: String,
    /// Expiration timestamp in ISO 8601 format (for time_limit expiration)
    pub expiration_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct UpdateCouponArgs {
    /// The unique code of the coupon to update
    pub code: String,
    /// New display name for the coupon
    pub name: Option<String>,
    /// New type of coupon: "fixed_amount" or "percentage"
    pub coupon_type: Option<String>,
    /// New discount amount in cents (for fixed_amount type)
    pub amount_cents: Option<i64>,
    /// New currency code for the discount amount (for fixed_amount type)
    pub amount_currency: Option<String>,
    /// New discount percentage rate as string (for percentage type)
    pub percentage_rate: Option<String>,
    /// New frequency: "once", "recurring", or "forever"
    pub frequency: Option<String>,
    /// New number of billing periods (for recurring frequency)
    pub frequency_duration: Option<i32>,
    /// Whether the coupon can be applied to multiple customers
    pub reusable: Option<bool>,
    /// Plan codes to limit the coupon to (optional)
    pub plan_codes: Option<Vec<String>>,
    /// Billable metric codes to limit the coupon to (optional)
    pub billable_metric_codes: Option<Vec<String>>,
    /// New expiration policy: "no_expiration" or "time_limit"
    pub expiration: Option<String>,
    /// New expiration timestamp in ISO 8601 format (for time_limit expiration)
    pub expiration_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct DeleteCouponArgs {
    /// The unique code of the coupon to delete
    pub code: String,
}

#[derive(Clone)]
pub struct CouponService;

impl CouponService {
    pub fn new() -> Self {
        Self
    }

    fn parse_frequency(frequency_str: &str) -> Option<CouponFrequency> {
        match frequency_str.to_lowercase().as_str() {
            "once" => Some(CouponFrequency::Once),
            "recurring" => Some(CouponFrequency::Recurring),
            "forever" => Some(CouponFrequency::Forever),
            _ => None,
        }
    }

    fn parse_expiration(expiration_str: &str) -> Option<CouponExpiration> {
        match expiration_str.to_lowercase().as_str() {
            "no_expiration" => Some(CouponExpiration::NoExpiration),
            "time_limit" => Some(CouponExpiration::TimeLimit),
            _ => None,
        }
    }

    pub async fn list_coupons(
        &self,
        Parameters(args): Parameters<ListCouponsArgs>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let client = match create_lago_client(&context).await {
            Ok(client) => client,
            Err(error_result) => return Ok(error_result),
        };

        let mut pagination = PaginationParams::default();
        if let Some(page) = args.page {
            pagination = pagination.with_page(page);
        }
        if let Some(per_page) = args.per_page {
            pagination = pagination.with_per_page(per_page);
        }

        let request = ListCouponsRequest::new().with_pagination(pagination);

        match client.list_coupons(Some(request)).await {
            Ok(response) => {
                let result = serde_json::json!({
                    "coupons": response.coupons,
                    "pagination": response.meta,
                });
                Ok(success_result(&result))
            }
            Err(e) => {
                let error_message = format!("Failed to list coupons: {e}");
                Ok(error_result(error_message))
            }
        }
    }

    pub async fn get_coupon(
        &self,
        Parameters(args): Parameters<GetCouponArgs>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let client = match create_lago_client(&context).await {
            Ok(client) => client,
            Err(error_result) => return Ok(error_result),
        };

        let request = GetCouponRequest::new(args.code);

        match client.get_coupon(request).await {
            Ok(response) => {
                let result = serde_json::json!({
                    "coupon": response.coupon,
                });
                Ok(success_result(&result))
            }
            Err(e) => {
                let error_message = format!("Failed to get coupon: {e}");
                Ok(error_result(error_message))
            }
        }
    }

    pub async fn create_coupon(
        &self,
        Parameters(args): Parameters<CreateCouponArgs>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let client = match create_lago_client(&context).await {
            Ok(client) => client,
            Err(error_result) => return Ok(error_result),
        };

        let frequency = match Self::parse_frequency(&args.frequency) {
            Some(f) => f,
            None => {
                return Ok(error_result(format!(
                    "Invalid frequency: {}. Must be 'once', 'recurring', or 'forever'",
                    args.frequency
                )));
            }
        };

        let expiration = match Self::parse_expiration(&args.expiration) {
            Some(e) => e,
            None => {
                return Ok(error_result(format!(
                    "Invalid expiration: {}. Must be 'no_expiration' or 'time_limit'",
                    args.expiration
                )));
            }
        };

        let mut input = match args.coupon_type.to_lowercase().as_str() {
            "fixed_amount" => {
                let amount_cents = match args.amount_cents {
                    Some(a) => a,
                    None => {
                        return Ok(error_result(
                            "amount_cents is required for fixed_amount coupon type",
                        ));
                    }
                };
                let amount_currency = match args.amount_currency {
                    Some(c) => c,
                    None => {
                        return Ok(error_result(
                            "amount_currency is required for fixed_amount coupon type",
                        ));
                    }
                };
                CreateCouponInput::fixed_amount(
                    args.name,
                    args.code,
                    amount_cents,
                    amount_currency,
                    frequency,
                    expiration,
                )
            }
            "percentage" => {
                let percentage_rate = match args.percentage_rate {
                    Some(r) => r,
                    None => {
                        return Ok(error_result(
                            "percentage_rate is required for percentage coupon type",
                        ));
                    }
                };
                CreateCouponInput::percentage(
                    args.name,
                    args.code,
                    percentage_rate,
                    frequency,
                    expiration,
                )
            }
            _ => {
                return Ok(error_result(format!(
                    "Invalid coupon_type: {}. Must be 'fixed_amount' or 'percentage'",
                    args.coupon_type
                )));
            }
        };

        if let Some(duration) = args.frequency_duration {
            input = input.with_frequency_duration(duration);
        }

        if let Some(reusable) = args.reusable {
            input = input.with_reusable(reusable);
        }

        if let Some(plan_codes) = args.plan_codes {
            input = input.with_limited_plans(plan_codes);
        }

        if let Some(billable_metric_codes) = args.billable_metric_codes {
            input = input.with_limited_billable_metrics(billable_metric_codes);
        }

        if let Some(expiration_at) = args.expiration_at {
            input = input.with_expiration_at(expiration_at);
        }

        let request = CreateCouponRequest::new(input);

        match client.create_coupon(request).await {
            Ok(response) => {
                let result = serde_json::json!({
                    "coupon": response.coupon,
                });
                Ok(success_result(&result))
            }
            Err(e) => {
                let error_message = format!("Failed to create coupon: {e}");
                Ok(error_result(error_message))
            }
        }
    }

    pub async fn update_coupon(
        &self,
        Parameters(args): Parameters<UpdateCouponArgs>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let client = match create_lago_client(&context).await {
            Ok(client) => client,
            Err(error_result) => return Ok(error_result),
        };

        let mut input = UpdateCouponInput::new();

        if let Some(name) = args.name {
            input = input.with_name(name);
        }

        // Handle type change with amount/percentage
        if let Some(coupon_type) = &args.coupon_type {
            match coupon_type.to_lowercase().as_str() {
                "fixed_amount" => {
                    if let (Some(amount_cents), Some(amount_currency)) =
                        (args.amount_cents, args.amount_currency.clone())
                    {
                        input = input.with_fixed_amount(amount_cents, amount_currency);
                    }
                }
                "percentage" => {
                    if let Some(percentage_rate) = args.percentage_rate.clone() {
                        input = input.with_percentage_rate(percentage_rate);
                    }
                }
                _ => {
                    return Ok(error_result(format!(
                        "Invalid coupon_type: {}. Must be 'fixed_amount' or 'percentage'",
                        coupon_type
                    )));
                }
            }
        }

        if let Some(frequency_str) = args.frequency {
            if let Some(frequency) = Self::parse_frequency(&frequency_str) {
                input = input.with_frequency(frequency);
            } else {
                return Ok(error_result(format!(
                    "Invalid frequency: {}. Must be 'once', 'recurring', or 'forever'",
                    frequency_str
                )));
            }
        }

        if let Some(duration) = args.frequency_duration {
            input = input.with_frequency_duration(duration);
        }

        if let Some(reusable) = args.reusable {
            input = input.with_reusable(reusable);
        }

        if let Some(plan_codes) = args.plan_codes {
            input = input.with_limited_plans(plan_codes);
        }

        if let Some(billable_metric_codes) = args.billable_metric_codes {
            input = input.with_limited_billable_metrics(billable_metric_codes);
        }

        if let Some(expiration_str) = args.expiration {
            if let Some(expiration) = Self::parse_expiration(&expiration_str) {
                input = input.with_expiration(expiration);
            } else {
                return Ok(error_result(format!(
                    "Invalid expiration: {}. Must be 'no_expiration' or 'time_limit'",
                    expiration_str
                )));
            }
        }

        if let Some(expiration_at) = args.expiration_at {
            input = input.with_expiration_at(expiration_at);
        }

        let request = UpdateCouponRequest::new(args.code, input);

        match client.update_coupon(request).await {
            Ok(response) => {
                let result = serde_json::json!({
                    "coupon": response.coupon,
                });
                Ok(success_result(&result))
            }
            Err(e) => {
                let error_message = format!("Failed to update coupon: {e}");
                Ok(error_result(error_message))
            }
        }
    }

    pub async fn delete_coupon(
        &self,
        Parameters(args): Parameters<DeleteCouponArgs>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let client = match create_lago_client(&context).await {
            Ok(client) => client,
            Err(error_result) => return Ok(error_result),
        };

        let request = DeleteCouponRequest::new(args.code);

        match client.delete_coupon(request).await {
            Ok(response) => {
                let result = serde_json::json!({
                    "coupon": response.coupon,
                    "message": "Coupon deleted successfully"
                });
                Ok(success_result(&result))
            }
            Err(e) => {
                let error_message = format!("Failed to delete coupon: {e}");
                Ok(error_result(error_message))
            }
        }
    }
}
