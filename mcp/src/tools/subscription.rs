use anyhow::Result;
use rmcp::{RoleServer, handler::server::tool::Parameters, model::*, service::RequestContext};
use serde::{Deserialize, Serialize};

use lago_types::{
    filters::subscription::SubscriptionFilters,
    models::{PaginationParams, SubscriptionBillingTime, SubscriptionStatus},
    requests::subscription::{
        CreateSubscriptionInput, CreateSubscriptionRequest, DeleteSubscriptionRequest,
        GetSubscriptionRequest, ListCustomerSubscriptionsRequest, ListSubscriptionsRequest,
        SubscriptionPlanOverrides, UpdateSubscriptionInput, UpdateSubscriptionRequest,
    },
};

use crate::tools::{create_lago_client, error_result, success_result};

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct ListSubscriptionsArgs {
    /// Filter by plan code.
    pub plan_code: Option<String>,
    /// Filter by subscription status. Possible values: active, pending, canceled, terminated.
    pub status: Option<Vec<String>>,
    /// Page number for pagination (default: 1).
    pub page: Option<i32>,
    /// Number of items per page (default: 20).
    pub per_page: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct GetSubscriptionArgs {
    /// The external unique identifier of the subscription.
    pub external_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct ListCustomerSubscriptionsArgs {
    /// The external unique identifier of the customer.
    pub external_customer_id: String,
    /// Filter by plan code.
    pub plan_code: Option<String>,
    /// Filter by subscription status. Possible values: active, pending, canceled, terminated.
    pub status: Option<Vec<String>>,
    /// Page number for pagination (default: 1).
    pub page: Option<i32>,
    /// Number of items per page (default: 20).
    pub per_page: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct CreateSubscriptionArgs {
    /// External unique identifier for the customer.
    pub external_customer_id: String,
    /// Code of the plan to assign to the subscription.
    pub plan_code: String,
    /// Optional display name for the subscription.
    pub name: Option<String>,
    /// Optional external unique identifier for the subscription.
    pub external_id: Option<String>,
    /// Billing time determines when recurring billing cycles occur. Possible values: anniversary, calendar.
    pub billing_time: Option<String>,
    /// The subscription start date (ISO 8601 format).
    pub subscription_at: Option<String>,
    /// The subscription end date (ISO 8601 format).
    pub ending_at: Option<String>,
    /// Plan overrides to customize the plan for this subscription.
    pub plan_overrides: Option<PlanOverridesInput>,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct PlanOverridesInput {
    /// Override the base amount in cents.
    pub amount_cents: Option<i64>,
    /// Override the currency.
    pub amount_currency: Option<String>,
    /// Override the plan description.
    pub description: Option<String>,
    /// Override the invoice display name.
    pub invoice_display_name: Option<String>,
    /// Override the plan name.
    pub name: Option<String>,
    /// Override the trial period in days.
    pub trial_period: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct UpdateSubscriptionArgs {
    /// The external unique identifier of the subscription to update.
    pub external_id: String,
    /// Optional new name for the subscription.
    pub name: Option<String>,
    /// Optional new end date for the subscription (ISO 8601 format).
    pub ending_at: Option<String>,
    /// Optional new plan code (for plan changes).
    pub plan_code: Option<String>,
    /// Optional new subscription date (ISO 8601 format).
    pub subscription_at: Option<String>,
    /// Plan overrides to customize the plan for this subscription.
    pub plan_overrides: Option<PlanOverridesInput>,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct DeleteSubscriptionArgs {
    /// The external unique identifier of the subscription to terminate.
    pub external_id: String,
    /// Optional status to set the subscription to (defaults to terminated).
    pub status: Option<String>,
}

#[derive(Clone)]
pub struct SubscriptionService;

impl SubscriptionService {
    pub fn new() -> Self {
        Self
    }

    fn parse_status(status_str: &str) -> Option<SubscriptionStatus> {
        match status_str.to_lowercase().as_str() {
            "active" => Some(SubscriptionStatus::Active),
            "pending" => Some(SubscriptionStatus::Pending),
            "canceled" => Some(SubscriptionStatus::Canceled),
            "terminated" => Some(SubscriptionStatus::Terminated),
            _ => None,
        }
    }

    fn parse_billing_time(billing_time_str: &str) -> Option<SubscriptionBillingTime> {
        match billing_time_str.to_lowercase().as_str() {
            "anniversary" => Some(SubscriptionBillingTime::Anniversary),
            "calendar" => Some(SubscriptionBillingTime::Calendar),
            _ => None,
        }
    }

    fn build_filters(
        plan_code: Option<String>,
        status: Option<Vec<String>>,
    ) -> SubscriptionFilters {
        let mut filters = SubscriptionFilters::new();

        if let Some(plan_code) = plan_code {
            filters = filters.with_plan_code(plan_code);
        }

        if let Some(statuses) = status {
            let parsed_statuses: Vec<SubscriptionStatus> = statuses
                .iter()
                .filter_map(|s| Self::parse_status(s))
                .collect();
            if !parsed_statuses.is_empty() {
                filters = filters.with_statuses(parsed_statuses);
            }
        }

        filters
    }

    fn build_plan_overrides(
        input: Option<PlanOverridesInput>,
    ) -> Option<SubscriptionPlanOverrides> {
        input.map(|overrides| {
            let mut plan_overrides = SubscriptionPlanOverrides::new();

            if let Some(amount_cents) = overrides.amount_cents {
                plan_overrides = plan_overrides.with_amount_cents(amount_cents);
            }
            if let Some(currency) = overrides.amount_currency {
                plan_overrides = plan_overrides.with_amount_currency(currency);
            }
            if let Some(description) = overrides.description {
                plan_overrides = plan_overrides.with_description(description);
            }
            if let Some(invoice_display_name) = overrides.invoice_display_name {
                plan_overrides = plan_overrides.with_invoice_display_name(invoice_display_name);
            }
            if let Some(name) = overrides.name {
                plan_overrides = plan_overrides.with_name(name);
            }
            if let Some(trial_period) = overrides.trial_period {
                plan_overrides = plan_overrides.with_trial_period(trial_period);
            }

            plan_overrides
        })
    }

    pub async fn list_subscriptions(
        &self,
        Parameters(args): Parameters<ListSubscriptionsArgs>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let client = match create_lago_client(&context).await {
            Ok(client) => client,
            Err(error_result) => return Ok(error_result),
        };

        let filters = Self::build_filters(args.plan_code, args.status);

        let mut pagination = PaginationParams::new();
        if let Some(page) = args.page {
            pagination = pagination.with_page(page);
        }
        if let Some(per_page) = args.per_page {
            pagination = pagination.with_per_page(per_page);
        }

        let request = ListSubscriptionsRequest::new()
            .with_filters(filters)
            .with_pagination(pagination);

        match client.list_subscriptions(Some(request)).await {
            Ok(response) => {
                let result = serde_json::json!({
                    "subscriptions": response.subscriptions,
                    "pagination": response.meta
                });

                Ok(success_result(&result))
            }
            Err(e) => {
                let error_message = format!("Failed to list subscriptions: {e}");
                tracing::error!("{error_message}");
                Ok(error_result(error_message))
            }
        }
    }

    pub async fn get_subscription(
        &self,
        Parameters(args): Parameters<GetSubscriptionArgs>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let client = match create_lago_client(&context).await {
            Ok(client) => client,
            Err(error_result) => return Ok(error_result),
        };

        let request = GetSubscriptionRequest::new(args.external_id);

        match client.get_subscription(request).await {
            Ok(response) => {
                let result = serde_json::json!({
                    "subscription": response.subscription,
                });

                Ok(success_result(&result))
            }
            Err(e) => {
                let error_message = format!("Failed to get subscription: {e}");
                tracing::error!("{error_message}");
                Ok(error_result(error_message))
            }
        }
    }

    pub async fn list_customer_subscriptions(
        &self,
        Parameters(args): Parameters<ListCustomerSubscriptionsArgs>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let client = match create_lago_client(&context).await {
            Ok(client) => client,
            Err(error_result) => return Ok(error_result),
        };

        let filters = Self::build_filters(args.plan_code, args.status);

        let mut pagination = PaginationParams::new();
        if let Some(page) = args.page {
            pagination = pagination.with_page(page);
        }
        if let Some(per_page) = args.per_page {
            pagination = pagination.with_per_page(per_page);
        }

        let request = ListCustomerSubscriptionsRequest::new(args.external_customer_id)
            .with_filters(filters)
            .with_pagination(pagination);

        match client.list_customer_subscriptions(request).await {
            Ok(response) => {
                let result = serde_json::json!({
                    "subscriptions": response.subscriptions,
                    "pagination": response.meta
                });

                Ok(success_result(&result))
            }
            Err(e) => {
                let error_message = format!("Failed to list customer subscriptions: {e}");
                tracing::error!("{error_message}");
                Ok(error_result(error_message))
            }
        }
    }

    pub async fn create_subscription(
        &self,
        Parameters(args): Parameters<CreateSubscriptionArgs>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let client = match create_lago_client(&context).await {
            Ok(client) => client,
            Err(error_result) => return Ok(error_result),
        };

        let mut input = CreateSubscriptionInput::new(args.external_customer_id, args.plan_code);

        if let Some(name) = args.name {
            input = input.with_name(name);
        }
        if let Some(external_id) = args.external_id {
            input = input.with_external_id(external_id);
        }
        if let Some(billing_time_str) = args.billing_time
            && let Some(billing_time) = Self::parse_billing_time(&billing_time_str)
        {
            input = input.with_billing_time(billing_time);
        }
        if let Some(subscription_at) = args.subscription_at {
            input = input.with_subscription_at(subscription_at);
        }
        if let Some(ending_at) = args.ending_at {
            input = input.with_ending_at(ending_at);
        }
        if let Some(plan_overrides) = Self::build_plan_overrides(args.plan_overrides) {
            input = input.with_plan_overrides(plan_overrides);
        }

        let request = CreateSubscriptionRequest::new(input);

        match client.create_subscription(request).await {
            Ok(response) => {
                let result = serde_json::json!({
                    "subscription": response.subscription,
                });

                Ok(success_result(&result))
            }
            Err(e) => {
                let error_message = format!("Failed to create subscription: {e}");
                tracing::error!("{error_message}");
                Ok(error_result(error_message))
            }
        }
    }

    pub async fn update_subscription(
        &self,
        Parameters(args): Parameters<UpdateSubscriptionArgs>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let client = match create_lago_client(&context).await {
            Ok(client) => client,
            Err(error_result) => return Ok(error_result),
        };

        let mut input = UpdateSubscriptionInput::new();

        if let Some(name) = args.name {
            input = input.with_name(name);
        }
        if let Some(ending_at) = args.ending_at {
            input = input.with_ending_at(ending_at);
        }
        if let Some(plan_code) = args.plan_code {
            input = input.with_plan_code(plan_code);
        }
        if let Some(subscription_at) = args.subscription_at {
            input = input.with_subscription_at(subscription_at);
        }
        if let Some(plan_overrides) = Self::build_plan_overrides(args.plan_overrides) {
            input = input.with_plan_overrides(plan_overrides);
        }

        let request = UpdateSubscriptionRequest::new(args.external_id, input);

        match client.update_subscription(request).await {
            Ok(response) => {
                let result = serde_json::json!({
                    "subscription": response.subscription,
                });

                Ok(success_result(&result))
            }
            Err(e) => {
                let error_message = format!("Failed to update subscription: {e}");
                tracing::error!("{error_message}");
                Ok(error_result(error_message))
            }
        }
    }

    pub async fn delete_subscription(
        &self,
        Parameters(args): Parameters<DeleteSubscriptionArgs>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let client = match create_lago_client(&context).await {
            Ok(client) => client,
            Err(error_result) => return Ok(error_result),
        };

        let mut request = DeleteSubscriptionRequest::new(args.external_id);

        if let Some(status) = args.status {
            request = request.with_status(status);
        }

        match client.delete_subscription(request).await {
            Ok(response) => {
                let result = serde_json::json!({
                    "subscription": response.subscription,
                });

                Ok(success_result(&result))
            }
            Err(e) => {
                let error_message = format!("Failed to delete subscription: {e}");
                tracing::error!("{error_message}");
                Ok(error_result(error_message))
            }
        }
    }
}
