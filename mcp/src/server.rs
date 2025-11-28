use anyhow::Result;
use rmcp::{
    ErrorData as McpError, RoleServer, ServerHandler,
    handler::server::{router::tool::ToolRouter, tool::Parameters},
    model::*,
    service::RequestContext,
    tool, tool_handler, tool_router,
};
use std::future::Future;

use crate::tools::activity_log::ActivityLogService;
use crate::tools::api_log::ApiLogService;
use crate::tools::applied_coupon::AppliedCouponService;
use crate::tools::billable_metric::BillableMetricService;
use crate::tools::customer::CustomerService;
use crate::tools::invoice::InvoiceService;

#[derive(Clone)]
#[allow(dead_code)]
pub struct LagoMcpServer {
    invoice_service: InvoiceService,
    customer_service: CustomerService,
    billable_metric_service: BillableMetricService,
    activity_log_service: ActivityLogService,
    api_log_service: ApiLogService,
    applied_coupon_service: AppliedCouponService,
    tool_router: ToolRouter<Self>,
}

#[allow(dead_code)]
impl LagoMcpServer {
    pub fn new() -> Self {
        let invoice_service = InvoiceService::new();
        let customer_service = CustomerService::new();
        let billable_metric_service = BillableMetricService::new();
        let activity_log_service = ActivityLogService::new();
        let api_log_service = ApiLogService::new();
        let applied_coupon_service = AppliedCouponService::new();

        Self {
            invoice_service,
            customer_service,
            billable_metric_service,
            activity_log_service,
            api_log_service,
            applied_coupon_service,
            tool_router: Self::tool_router(),
        }
    }
}

#[tool_router]
impl LagoMcpServer {
    #[tool(description = "Get a specific invoice by its Lago ID")]
    pub async fn get_invoice(
        &self,
        parameters: Parameters<crate::tools::invoice::GetInvoiceArgs>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        self.invoice_service.get_invoice(parameters, context).await
    }

    #[tool(
        description = "List invoices from Lago with optional filtering by customer, dates, status and type"
    )]
    pub async fn list_invoices(
        &self,
        parameters: Parameters<crate::tools::invoice::ListInvoicesArgs>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        self.invoice_service
            .list_invoices(parameters, context)
            .await
    }

    #[tool(
        description = "Preview an invoice before creating it. Use this to estimate billing amounts for new subscriptions, plan upgrades, or to see the effect of coupons. You can either reference an existing customer by external_id or provide inline customer details."
    )]
    pub async fn preview_invoice(
        &self,
        parameters: Parameters<crate::tools::invoice::PreviewInvoiceArgs>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        self.invoice_service
            .preview_invoice(parameters, context)
            .await
    }

    #[tool(description = "Get a specific customer by their external ID")]
    pub async fn get_customer(
        &self,
        parameters: Parameters<crate::tools::customer::GetCustomerArgs>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        self.customer_service
            .get_customer(parameters, context)
            .await
    }

    #[tool(
        description = "List customers from Lago with optional filtering by external customer ID"
    )]
    pub async fn list_customers(
        &self,
        parameters: Parameters<crate::tools::customer::ListCustomersArgs>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        self.customer_service
            .list_customers(parameters, context)
            .await
    }

    #[tool(description = "Create or update a customer in Lago")]
    pub async fn create_customer(
        &self,
        parameters: Parameters<crate::tools::customer::CreateCustomerArgs>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        self.customer_service
            .create_customer(parameters, context)
            .await
    }

    #[tool(description = "Get a specific billable metric by its code")]
    pub async fn get_billable_metric(
        &self,
        parameters: Parameters<crate::tools::billable_metric::GetBillableMetricArgs>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        self.billable_metric_service
            .get_billable_metric(parameters, context)
            .await
    }

    #[tool(
        description = "List billable metrics from Lago with optional filtering by aggregation type and recurring status"
    )]
    pub async fn list_billable_metrics(
        &self,
        parameters: Parameters<crate::tools::billable_metric::ListBillableMetricsArgs>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        self.billable_metric_service
            .list_billable_metrics(parameters, context)
            .await
    }

    #[tool(description = "Create a new billable metric in Lago")]
    pub async fn create_billable_metric(
        &self,
        parameters: Parameters<crate::tools::billable_metric::CreateBillableMetricArgs>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        self.billable_metric_service
            .create_billable_metric(parameters, context)
            .await
    }

    #[tool(description = "Get a specific activity log by its activity ID")]
    pub async fn get_activity_log(
        &self,
        parameters: Parameters<crate::tools::activity_log::GetActivityLogArgs>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        self.activity_log_service
            .get_activity_log(parameters, context)
            .await
    }

    #[tool(
        description = "List activity logs from Lago with optional filtering by activity type, source, user email, customer, subscription, resource type and date range"
    )]
    pub async fn list_activity_logs(
        &self,
        parameters: Parameters<crate::tools::activity_log::ListActivityLogsArgs>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        self.activity_log_service
            .list_activity_logs(parameters, context)
            .await
    }

    #[tool(description = "Get a specific API log by its request ID")]
    pub async fn get_api_log(
        &self,
        parameters: Parameters<crate::tools::api_log::GetApiLogArgs>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        self.api_log_service.get_api_log(parameters, context).await
    }

    #[tool(
        description = "List API logs from Lago with optional filtering by HTTP method, status, API version, request path and date range"
    )]
    pub async fn list_api_logs(
        &self,
        parameters: Parameters<crate::tools::api_log::ListApiLogsArgs>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        self.api_log_service
            .list_api_logs(parameters, context)
            .await
    }

    #[tool(
        description = "List applied coupons from Lago with optional filtering by status, customer and coupon codes"
    )]
    pub async fn list_applied_coupons(
        &self,
        parameters: Parameters<crate::tools::applied_coupon::ListAppliedCouponsArgs>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        self.applied_coupon_service
            .list_applied_coupons(parameters, context)
            .await
    }

    #[tool(
        description = "Apply a coupon to a customer. Use this to give discounts before or during a subscription."
    )]
    pub async fn apply_coupon(
        &self,
        parameters: Parameters<crate::tools::applied_coupon::ApplyCouponArgs>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        self.applied_coupon_service
            .apply_coupon(parameters, context)
            .await
    }
}

#[tool_handler]
impl ServerHandler for LagoMcpServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            instructions: Some(
                "Lago MCP server for managing invoices, customers, billable metrics, activity logs, API logs, applied coupons and other lago resources. Use the available tools to interact with the Lago API.".into()
            ),
            capabilities: ServerCapabilities::builder()
                .enable_tools()
                .build(),
            ..Default::default()
        }
    }

    async fn initialize(
        &self,
        _request: InitializeRequestParam,
        context: RequestContext<RoleServer>,
    ) -> Result<InitializeResult, McpError> {
        if let Some(http_request_part) = context.extensions.get::<axum::http::request::Parts>() {
            let initialize_uri = &http_request_part.uri;
            tracing::info!(%initialize_uri, "initialize from http server");
        }
        Ok(self.get_info())
    }
}
