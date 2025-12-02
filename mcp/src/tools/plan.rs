use anyhow::Result;
use rmcp::{RoleServer, handler::server::tool::Parameters, model::*, service::RequestContext};
use serde::{Deserialize, Serialize};

use lago_types::{
    models::{ChargeModel, PaginationParams, PlanInterval},
    requests::plan::{
        CreateChargeFilterInput, CreateMinimumCommitmentInput, CreatePlanChargeInput,
        CreatePlanInput, CreatePlanRequest, CreateUsageThresholdInput, DeletePlanRequest,
        GetPlanRequest, ListPlansRequest, UpdatePlanInput, UpdatePlanRequest,
    },
};

use crate::tools::{create_lago_client, error_result, success_result};

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct ListPlansArgs {
    /// Page number for pagination (default: 1).
    pub page: Option<i32>,
    /// Number of items per page (default: 20).
    pub per_page: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct GetPlanArgs {
    /// The unique code of the plan.
    pub code: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct CreatePlanArgs {
    /// Name of the plan.
    pub name: String,
    /// Unique code for the plan.
    pub code: String,
    /// Billing interval. Possible values: weekly, monthly, quarterly, semiannual, yearly.
    pub interval: String,
    /// Base amount in cents.
    pub amount_cents: i64,
    /// Currency for the amount (e.g., USD, EUR).
    pub amount_currency: String,
    /// Display name for invoices.
    pub invoice_display_name: Option<String>,
    /// Description of the plan.
    pub description: Option<String>,
    /// Trial period in days.
    pub trial_period: Option<f64>,
    /// Whether the plan is billed in advance.
    pub pay_in_advance: Option<bool>,
    /// Whether charges are billed monthly for yearly plans.
    pub bill_charges_monthly: Option<bool>,
    /// Tax codes for this plan.
    pub tax_codes: Option<Vec<String>>,
    /// Charges for this plan.
    pub charges: Option<Vec<CreatePlanChargeArgs>>,
    /// Minimum commitment for this plan.
    pub minimum_commitment: Option<MinimumCommitmentArgs>,
    /// Usage thresholds for progressive billing.
    pub usage_thresholds: Option<Vec<UsageThresholdArgs>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct CreatePlanChargeArgs {
    /// The billable metric ID to reference.
    pub billable_metric_id: String,
    /// The charge model to use. Possible values: standard, graduated, volume, package, percentage, graduated_percentage, dynamic.
    pub charge_model: String,
    /// Whether the charge is invoiceable.
    pub invoiceable: Option<bool>,
    /// Display name for the charge on invoices.
    pub invoice_display_name: Option<String>,
    /// Whether the charge is billed in advance.
    pub pay_in_advance: Option<bool>,
    /// Whether the charge is prorated.
    pub prorated: Option<bool>,
    /// Minimum amount in cents for this charge.
    pub min_amount_cents: Option<i64>,
    /// Charge properties (model-specific configuration as JSON).
    pub properties: Option<serde_json::Value>,
    /// Tax codes for this charge.
    pub tax_codes: Option<Vec<String>>,
    /// Filters for differentiated pricing.
    pub filters: Option<Vec<ChargeFilterArgs>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct ChargeFilterArgs {
    /// Invoice display name for this filter.
    pub invoice_display_name: Option<String>,
    /// Filter properties.
    pub properties: Option<serde_json::Value>,
    /// Filter values mapping.
    pub values: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct MinimumCommitmentArgs {
    /// Minimum commitment amount in cents.
    pub amount_cents: i64,
    /// Invoice display name for the minimum commitment.
    pub invoice_display_name: Option<String>,
    /// Tax codes for the minimum commitment.
    pub tax_codes: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct UsageThresholdArgs {
    /// Threshold amount in cents.
    pub amount_cents: i64,
    /// Display name for the threshold.
    pub threshold_display_name: Option<String>,
    /// Whether the threshold is recurring.
    pub recurring: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct UpdatePlanArgs {
    /// The code of the plan to update.
    pub code: String,
    /// New name of the plan.
    pub name: Option<String>,
    /// New code for the plan.
    pub new_code: Option<String>,
    /// Billing interval. Possible values: weekly, monthly, quarterly, semiannual, yearly.
    pub interval: Option<String>,
    /// Base amount in cents.
    pub amount_cents: Option<i64>,
    /// Currency for the amount.
    pub amount_currency: Option<String>,
    /// Display name for invoices.
    pub invoice_display_name: Option<String>,
    /// Description of the plan.
    pub description: Option<String>,
    /// Trial period in days.
    pub trial_period: Option<f64>,
    /// Whether the plan is billed in advance.
    pub pay_in_advance: Option<bool>,
    /// Whether charges are billed monthly for yearly plans.
    pub bill_charges_monthly: Option<bool>,
    /// Tax codes for this plan.
    pub tax_codes: Option<Vec<String>>,
    /// Charges for this plan.
    pub charges: Option<Vec<CreatePlanChargeArgs>>,
    /// Minimum commitment for this plan.
    pub minimum_commitment: Option<MinimumCommitmentArgs>,
    /// Usage thresholds for progressive billing.
    pub usage_thresholds: Option<Vec<UsageThresholdArgs>>,
    /// Whether to cascade updates to existing subscriptions.
    pub cascade_updates: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct DeletePlanArgs {
    /// The code of the plan to delete.
    pub code: String,
}

#[derive(Clone)]
pub struct PlanService;

impl PlanService {
    pub fn new() -> Self {
        Self
    }

    fn parse_interval(interval_str: &str) -> Option<PlanInterval> {
        match interval_str.to_lowercase().as_str() {
            "weekly" => Some(PlanInterval::Weekly),
            "monthly" => Some(PlanInterval::Monthly),
            "quarterly" => Some(PlanInterval::Quarterly),
            "semiannual" => Some(PlanInterval::Semiannual),
            "yearly" => Some(PlanInterval::Yearly),
            _ => None,
        }
    }

    fn parse_charge_model(model_str: &str) -> Option<ChargeModel> {
        match model_str.to_lowercase().as_str() {
            "standard" => Some(ChargeModel::Standard),
            "graduated" => Some(ChargeModel::Graduated),
            "volume" => Some(ChargeModel::Volume),
            "package" => Some(ChargeModel::Package),
            "percentage" => Some(ChargeModel::Percentage),
            "graduated_percentage" => Some(ChargeModel::GraduatedPercentage),
            "dynamic" => Some(ChargeModel::Dynamic),
            _ => None,
        }
    }

    fn build_charge(charge_args: &CreatePlanChargeArgs) -> Option<CreatePlanChargeInput> {
        let charge_model = Self::parse_charge_model(&charge_args.charge_model)?;

        let mut charge =
            CreatePlanChargeInput::new(charge_args.billable_metric_id.clone(), charge_model);

        if let Some(invoiceable) = charge_args.invoiceable {
            charge = charge.with_invoiceable(invoiceable);
        }
        if let Some(name) = &charge_args.invoice_display_name {
            charge = charge.with_invoice_display_name(name.clone());
        }
        if let Some(pay_in_advance) = charge_args.pay_in_advance {
            charge = charge.with_pay_in_advance(pay_in_advance);
        }
        if let Some(prorated) = charge_args.prorated {
            charge = charge.with_prorated(prorated);
        }
        if let Some(min_amount) = charge_args.min_amount_cents {
            charge = charge.with_min_amount_cents(min_amount);
        }
        if let Some(properties) = &charge_args.properties {
            charge = charge.with_properties(properties.clone());
        }
        if let Some(tax_codes) = &charge_args.tax_codes {
            charge = charge.with_tax_codes(tax_codes.clone());
        }
        if let Some(filters) = &charge_args.filters {
            let filter_inputs: Vec<CreateChargeFilterInput> = filters
                .iter()
                .map(|f| CreateChargeFilterInput {
                    invoice_display_name: f.invoice_display_name.clone(),
                    properties: f.properties.clone(),
                    values: f.values.clone(),
                })
                .collect();
            charge = charge.with_filters(filter_inputs);
        }

        Some(charge)
    }

    fn build_minimum_commitment(args: &MinimumCommitmentArgs) -> CreateMinimumCommitmentInput {
        let mut commitment = CreateMinimumCommitmentInput::new(args.amount_cents);

        if let Some(name) = &args.invoice_display_name {
            commitment = commitment.with_invoice_display_name(name.clone());
        }
        if let Some(tax_codes) = &args.tax_codes {
            commitment = commitment.with_tax_codes(tax_codes.clone());
        }

        commitment
    }

    fn build_usage_threshold(args: &UsageThresholdArgs) -> CreateUsageThresholdInput {
        let mut threshold = CreateUsageThresholdInput::new(args.amount_cents);

        if let Some(name) = &args.threshold_display_name {
            threshold = threshold.with_threshold_display_name(name.clone());
        }
        if let Some(recurring) = args.recurring {
            threshold = threshold.with_recurring(recurring);
        }

        threshold
    }

    pub async fn list_plans(
        &self,
        Parameters(args): Parameters<ListPlansArgs>,
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

        let request = ListPlansRequest::new().with_pagination(pagination);

        match client.list_plans(Some(request)).await {
            Ok(response) => {
                let result = serde_json::json!({
                    "plans": response.plans,
                    "pagination": response.meta
                });

                Ok(success_result(&result))
            }
            Err(e) => {
                let error_message = format!("Failed to list plans: {e}");
                tracing::error!("{error_message}");
                Ok(error_result(error_message))
            }
        }
    }

    pub async fn get_plan(
        &self,
        Parameters(args): Parameters<GetPlanArgs>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let client = match create_lago_client(&context).await {
            Ok(client) => client,
            Err(error_result) => return Ok(error_result),
        };

        let request = GetPlanRequest::new(args.code);

        match client.get_plan(request).await {
            Ok(response) => {
                let result = serde_json::json!({
                    "plan": response.plan,
                });

                Ok(success_result(&result))
            }
            Err(e) => {
                let error_message = format!("Failed to get plan: {e}");
                tracing::error!("{error_message}");
                Ok(error_result(error_message))
            }
        }
    }

    pub async fn create_plan(
        &self,
        Parameters(args): Parameters<CreatePlanArgs>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let client = match create_lago_client(&context).await {
            Ok(client) => client,
            Err(error_result) => return Ok(error_result),
        };

        let interval = match Self::parse_interval(&args.interval) {
            Some(i) => i,
            None => {
                return Ok(error_result(format!(
                    "Invalid interval: {}. Must be one of: weekly, monthly, quarterly, semiannual, yearly",
                    args.interval
                )));
            }
        };

        let mut input = CreatePlanInput::new(
            args.name,
            args.code,
            interval,
            args.amount_cents,
            args.amount_currency,
        );

        if let Some(name) = args.invoice_display_name {
            input = input.with_invoice_display_name(name);
        }
        if let Some(description) = args.description {
            input = input.with_description(description);
        }
        if let Some(trial_period) = args.trial_period {
            input = input.with_trial_period(trial_period);
        }
        if let Some(pay_in_advance) = args.pay_in_advance {
            input = input.with_pay_in_advance(pay_in_advance);
        }
        if let Some(bill_charges_monthly) = args.bill_charges_monthly {
            input = input.with_bill_charges_monthly(bill_charges_monthly);
        }
        if let Some(tax_codes) = args.tax_codes {
            input = input.with_tax_codes(tax_codes);
        }
        if let Some(charges) = args.charges {
            let charge_inputs: Vec<CreatePlanChargeInput> =
                charges.iter().filter_map(Self::build_charge).collect();
            if !charge_inputs.is_empty() {
                input = input.with_charges(charge_inputs);
            }
        }
        if let Some(commitment) = args.minimum_commitment {
            input = input.with_minimum_commitment(Self::build_minimum_commitment(&commitment));
        }
        if let Some(thresholds) = args.usage_thresholds {
            let threshold_inputs: Vec<CreateUsageThresholdInput> =
                thresholds.iter().map(Self::build_usage_threshold).collect();
            input = input.with_usage_thresholds(threshold_inputs);
        }

        let request = CreatePlanRequest::new(input);

        match client.create_plan(request).await {
            Ok(response) => {
                let result = serde_json::json!({
                    "plan": response.plan,
                });

                Ok(success_result(&result))
            }
            Err(e) => {
                let error_message = format!("Failed to create plan: {e}");
                tracing::error!("{error_message}");
                Ok(error_result(error_message))
            }
        }
    }

    pub async fn update_plan(
        &self,
        Parameters(args): Parameters<UpdatePlanArgs>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let client = match create_lago_client(&context).await {
            Ok(client) => client,
            Err(error_result) => return Ok(error_result),
        };

        let mut input = UpdatePlanInput::new();

        if let Some(name) = args.name {
            input = input.with_name(name);
        }
        if let Some(code) = args.new_code {
            input = input.with_code(code);
        }
        if let Some(interval_str) = args.interval {
            if let Some(interval) = Self::parse_interval(&interval_str) {
                input = input.with_interval(interval);
            } else {
                return Ok(error_result(format!(
                    "Invalid interval: {}. Must be one of: weekly, monthly, quarterly, semiannual, yearly",
                    interval_str
                )));
            }
        }
        if let Some(amount_cents) = args.amount_cents {
            input = input.with_amount_cents(amount_cents);
        }
        if let Some(currency) = args.amount_currency {
            input = input.with_amount_currency(currency);
        }
        if let Some(name) = args.invoice_display_name {
            input = input.with_invoice_display_name(name);
        }
        if let Some(description) = args.description {
            input = input.with_description(description);
        }
        if let Some(trial_period) = args.trial_period {
            input = input.with_trial_period(trial_period);
        }
        if let Some(pay_in_advance) = args.pay_in_advance {
            input = input.with_pay_in_advance(pay_in_advance);
        }
        if let Some(bill_charges_monthly) = args.bill_charges_monthly {
            input = input.with_bill_charges_monthly(bill_charges_monthly);
        }
        if let Some(tax_codes) = args.tax_codes {
            input = input.with_tax_codes(tax_codes);
        }
        if let Some(charges) = args.charges {
            let charge_inputs: Vec<CreatePlanChargeInput> =
                charges.iter().filter_map(Self::build_charge).collect();
            if !charge_inputs.is_empty() {
                input = input.with_charges(charge_inputs);
            }
        }
        if let Some(commitment) = args.minimum_commitment {
            input = input.with_minimum_commitment(Self::build_minimum_commitment(&commitment));
        }
        if let Some(thresholds) = args.usage_thresholds {
            let threshold_inputs: Vec<CreateUsageThresholdInput> =
                thresholds.iter().map(Self::build_usage_threshold).collect();
            input = input.with_usage_thresholds(threshold_inputs);
        }
        if let Some(cascade_updates) = args.cascade_updates {
            input = input.with_cascade_updates(cascade_updates);
        }

        let request = UpdatePlanRequest::new(args.code, input);

        match client.update_plan(request).await {
            Ok(response) => {
                let result = serde_json::json!({
                    "plan": response.plan,
                });

                Ok(success_result(&result))
            }
            Err(e) => {
                let error_message = format!("Failed to update plan: {e}");
                tracing::error!("{error_message}");
                Ok(error_result(error_message))
            }
        }
    }

    pub async fn delete_plan(
        &self,
        Parameters(args): Parameters<DeletePlanArgs>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let client = match create_lago_client(&context).await {
            Ok(client) => client,
            Err(error_result) => return Ok(error_result),
        };

        let request = DeletePlanRequest::new(args.code);

        match client.delete_plan(request).await {
            Ok(response) => {
                let result = serde_json::json!({
                    "plan": response.plan,
                });

                Ok(success_result(&result))
            }
            Err(e) => {
                let error_message = format!("Failed to delete plan: {e}");
                tracing::error!("{error_message}");
                Ok(error_result(error_message))
            }
        }
    }
}
