use anyhow::Result;
use rmcp::{RoleServer, handler::server::tool::Parameters, model::*, service::RequestContext};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use lago_types::{
    filters::billable_metric::BillableMetricFilter,
    models::{
        BillableMetricAggregationType, BillableMetricFilter as BillableMetricFilterModel,
        BillableMetricRoundingFunction, BillableMetricWeightedInterval, PaginationParams,
    },
    requests::billable_metric::{
        CreateBillableMetricInput, CreateBillableMetricRequest, GetBillableMetricRequest,
        ListBillableMetricsRequest,
    },
};

use crate::tools::{create_lago_client, error_result, get_lago_api_config, success_result};

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct ListBillableMetricsArgs {
    pub aggregation_type: Option<String>,
    pub recurring: Option<bool>,
    pub page: Option<i32>,
    pub per_page: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct GetBillableMetricArgs {
    pub code: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct CreateBillableMetricArgs {
    pub name: String,
    pub code: String,
    pub aggregation_type: String,
    pub description: Option<String>,
    pub recurring: Option<bool>,
    pub rounding_function: Option<String>,
    pub rounding_precision: Option<i32>,
    pub expression: Option<String>,
    pub field_name: Option<String>,
    pub weighted_interval: Option<String>,
    pub filters: Option<Vec<BillableMetricFilterInput>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct UpdateBillableMetricArgs {
    /// The code of the billable metric to update.
    pub code: String,
    /// New name of the billable metric.
    pub name: Option<String>,
    /// New code for the billable metric.
    pub new_code: Option<String>,
    /// New description for the billable metric.
    pub description: Option<String>,
    /// Whether the metric persists across billing periods.
    pub recurring: Option<bool>,
    /// Rounding function. Possible values: round, ceil, floor.
    pub rounding_function: Option<String>,
    /// Number of decimal places for rounding.
    pub rounding_precision: Option<i32>,
    /// Expression used to calculate event units.
    pub expression: Option<String>,
    /// The property to aggregate on.
    pub field_name: Option<String>,
    /// Weighted interval for weighted_sum_agg. Possible values: seconds.
    pub weighted_interval: Option<String>,
    /// Filters for differentiated pricing.
    pub filters: Option<Vec<BillableMetricFilterInput>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct BillableMetricFilterInput {
    pub key: String,
    pub values: Vec<String>,
}

#[derive(Clone)]
pub struct BillableMetricService {
    http_client: reqwest::Client,
}

impl BillableMetricService {
    pub fn new() -> Self {
        Self {
            http_client: reqwest::Client::new(),
        }
    }

    #[allow(clippy::collapsible_if)]
    fn build_list_request(&self, params: &ListBillableMetricsArgs) -> ListBillableMetricsRequest {
        let mut filters = BillableMetricFilter::default();

        if let Some(aggregation_type) = &params.aggregation_type {
            filters.aggregation_type = Some(aggregation_type.clone());
        }

        if let Some(recurring) = params.recurring {
            filters.recurring = Some(recurring);
        }

        let mut pagination = PaginationParams::default();
        if let Some(page) = params.page {
            pagination = pagination.with_page(page);
        }

        if let Some(per_page) = params.per_page {
            pagination = pagination.with_per_page(per_page);
        }

        ListBillableMetricsRequest::new()
            .with_filters(filters)
            .with_pagination(pagination)
    }

    pub async fn list_billable_metrics(
        &self,
        Parameters(args): Parameters<ListBillableMetricsArgs>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let client = match create_lago_client(&context).await {
            Ok(client) => client,
            Err(error_result) => return Ok(error_result),
        };
        let request = self.build_list_request(&args);

        match client.list_billable_metrics(Some(request)).await {
            Ok(response) => {
                let result = serde_json::json!({
                    "billable_metrics": response.billable_metrics,
                    "pagination": response.meta,
                });

                Ok(success_result(&result))
            }
            Err(e) => {
                let error_message = format!("Failed to list billable metrics: {e}");
                tracing::error!("{error_message}");
                Ok(error_result(error_message))
            }
        }
    }

    pub async fn get_billable_metric(
        &self,
        Parameters(args): Parameters<GetBillableMetricArgs>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let client = match create_lago_client(&context).await {
            Ok(client) => client,
            Err(error_result) => return Ok(error_result),
        };
        let request = GetBillableMetricRequest::new(args.code);

        match client.get_billable_metric(request).await {
            Ok(response) => {
                let result = serde_json::json!({
                    "billable_metric": response.billable_metric,
                });

                Ok(success_result(&result))
            }
            Err(e) => {
                let error_message = format!("Failed to get billable metric: {e}");
                tracing::error!("{error_message}");
                Ok(error_result(error_message))
            }
        }
    }

    #[allow(clippy::collapsible_if)]
    pub async fn create_billable_metric(
        &self,
        Parameters(args): Parameters<CreateBillableMetricArgs>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        // Parse aggregation type
        let aggregation_type = match args
            .aggregation_type
            .parse::<BillableMetricAggregationType>()
        {
            Ok(agg_type) => agg_type,
            Err(_) => {
                return Ok(error_result(format!(
                    "Invalid aggregation_type: {}. Valid values are: count_agg, sum_agg, max_agg, unique_count_agg, weighted_sum_agg, latest_agg",
                    args.aggregation_type
                )));
            }
        };

        let mut metric_input =
            CreateBillableMetricInput::new(args.name, args.code, aggregation_type);

        if let Some(description) = args.description {
            metric_input = metric_input.with_description(description);
        }

        if let Some(recurring) = args.recurring {
            metric_input = metric_input.with_recurring(recurring);
        }

        if let Some(rounding_function_str) = args.rounding_function {
            if let Ok(rounding_function) =
                rounding_function_str.parse::<BillableMetricRoundingFunction>()
            {
                metric_input = metric_input.with_rounding_function(rounding_function);
            }
        }

        if let Some(rounding_precision) = args.rounding_precision {
            metric_input = metric_input.with_rounding_precision(rounding_precision);
        }

        if let Some(expression) = args.expression {
            metric_input = metric_input.with_expression(expression);
        }

        if let Some(field_name) = args.field_name {
            metric_input = metric_input.with_field_name(field_name);
        }

        if let Some(weighted_interval_str) = args.weighted_interval {
            if let Ok(weighted_interval) =
                weighted_interval_str.parse::<BillableMetricWeightedInterval>()
            {
                metric_input = metric_input.with_weighted_interval(weighted_interval);
            }
        }

        if let Some(filters_input) = args.filters {
            let filters: Vec<BillableMetricFilterModel> = filters_input
                .into_iter()
                .map(|f| BillableMetricFilterModel::new(f.key, f.values))
                .collect();
            metric_input = metric_input.with_filters(filters);
        }

        let request = CreateBillableMetricRequest::new(metric_input);

        let client = match create_lago_client(&context).await {
            Ok(client) => client,
            Err(error_result) => return Ok(error_result),
        };

        match client.create_billable_metric(request).await {
            Ok(response) => {
                let result = serde_json::json!({
                    "billable_metric": response.billable_metric,
                });

                Ok(success_result(&result))
            }
            Err(e) => {
                let error_message = format!("Failed to create billable metric: {e}");
                tracing::error!("{error_message}");
                Ok(error_result(error_message))
            }
        }
    }

    pub async fn update_billable_metric(
        &self,
        Parameters(args): Parameters<UpdateBillableMetricArgs>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let config = match get_lago_api_config(&context).await {
            Ok(config) => config,
            Err(error_result) => return Ok(error_result),
        };

        let mut body = serde_json::Map::new();

        if let Some(name) = args.name {
            body.insert("name".into(), Value::String(name));
        }
        if let Some(new_code) = args.new_code {
            body.insert("code".into(), Value::String(new_code));
        }
        if let Some(description) = args.description {
            body.insert("description".into(), Value::String(description));
        }
        if let Some(recurring) = args.recurring {
            body.insert("recurring".into(), Value::Bool(recurring));
        }
        if let Some(rounding_function_str) = args.rounding_function {
            if rounding_function_str
                .parse::<BillableMetricRoundingFunction>()
                .is_err()
            {
                return Ok(error_result(format!(
                    "Invalid rounding_function: {rounding_function_str}. Valid values are: round, ceil, floor"
                )));
            }
            body.insert(
                "rounding_function".into(),
                Value::String(rounding_function_str),
            );
        }
        if let Some(rounding_precision) = args.rounding_precision {
            body.insert(
                "rounding_precision".into(),
                Value::Number(rounding_precision.into()),
            );
        }
        if let Some(expression) = args.expression {
            body.insert("expression".into(), Value::String(expression));
        }
        if let Some(field_name) = args.field_name {
            body.insert("field_name".into(), Value::String(field_name));
        }
        if let Some(weighted_interval_str) = args.weighted_interval {
            if weighted_interval_str
                .parse::<BillableMetricWeightedInterval>()
                .is_err()
            {
                return Ok(error_result(format!(
                    "Invalid weighted_interval: {weighted_interval_str}. Valid values are: seconds"
                )));
            }
            body.insert(
                "weighted_interval".into(),
                Value::String(weighted_interval_str),
            );
        }
        if let Some(filters_input) = args.filters {
            let filters_json: Vec<Value> = filters_input
                .into_iter()
                .map(|f| {
                    serde_json::json!({
                        "key": f.key,
                        "values": f.values,
                    })
                })
                .collect();
            body.insert("filters".into(), Value::Array(filters_json));
        }

        let payload = serde_json::json!({ "billable_metric": Value::Object(body) });

        let encoded_code = urlencoding::encode(&args.code);
        let url = format!("{}/billable_metrics/{}", config.base_url, encoded_code);

        match self
            .http_client
            .put(&url)
            .bearer_auth(&config.api_key)
            .json(&payload)
            .send()
            .await
        {
            Ok(response) if response.status().is_success() => {
                match response.json::<Value>().await {
                    Ok(json) => Ok(success_result(&json)),
                    Err(e) => {
                        let error_message =
                            format!("Failed to parse update billable metric response: {e}");
                        tracing::error!(
                            code = %args.code,
                            error = %e,
                            "{error_message}"
                        );
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
                let error_message =
                    format!("Failed to update billable metric (HTTP {status}): {body}");
                tracing::error!(
                    code = %args.code,
                    "{error_message}"
                );
                Ok(error_result(error_message))
            }
            Err(e) => {
                let error_message = format!("Failed to update billable metric: {e}");
                tracing::error!(
                    code = %args.code,
                    error = %e,
                    "{error_message}"
                );
                Ok(error_result(error_message))
            }
        }
    }
}
