use anyhow::Result;
use rmcp::{handler::server::tool::Parameters, model::*};
use serde::{Deserialize, Serialize};

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

use crate::tools::{create_lago_client, error_result, success_result};

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
pub struct BillableMetricFilterInput {
    pub key: String,
    pub values: Vec<String>,
}

#[derive(Clone)]
pub struct BillableMetricService;

impl BillableMetricService {
    pub fn new() -> Self {
        Self
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
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let client = match create_lago_client() {
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
                Ok(error_result(error_message))
            }
        }
    }

    pub async fn get_billable_metric(
        &self,
        Parameters(args): Parameters<GetBillableMetricArgs>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let client = match create_lago_client() {
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
                Ok(error_result(error_message))
            }
        }
    }

    #[allow(clippy::collapsible_if)]
    pub async fn create_billable_metric(
        &self,
        Parameters(args): Parameters<CreateBillableMetricArgs>,
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

        let client = match create_lago_client() {
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
                Ok(error_result(error_message))
            }
        }
    }
}
