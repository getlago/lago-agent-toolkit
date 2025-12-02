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
use crate::tools::coupon::CouponService;
use crate::tools::credit_note::CreditNoteService;
use crate::tools::customer::CustomerService;
use crate::tools::customer_usage::CustomerUsageService;
use crate::tools::event::EventService;
use crate::tools::invoice::InvoiceService;
use crate::tools::plan::PlanService;
use crate::tools::subscription::SubscriptionService;

#[derive(Clone)]
#[allow(dead_code)]
pub struct LagoMcpServer {
    invoice_service: InvoiceService,
    customer_service: CustomerService,
    customer_usage_service: CustomerUsageService,
    subscription_service: SubscriptionService,
    billable_metric_service: BillableMetricService,
    activity_log_service: ActivityLogService,
    api_log_service: ApiLogService,
    applied_coupon_service: AppliedCouponService,
    coupon_service: CouponService,
    credit_note_service: CreditNoteService,
    event_service: EventService,
    plan_service: PlanService,
    tool_router: ToolRouter<Self>,
}

#[allow(dead_code)]
impl LagoMcpServer {
    pub fn new() -> Self {
        let invoice_service = InvoiceService::new();
        let customer_service = CustomerService::new();
        let customer_usage_service = CustomerUsageService::new();
        let subscription_service = SubscriptionService::new();
        let billable_metric_service = BillableMetricService::new();
        let activity_log_service = ActivityLogService::new();
        let api_log_service = ApiLogService::new();
        let applied_coupon_service = AppliedCouponService::new();
        let coupon_service = CouponService::new();
        let credit_note_service = CreditNoteService::new();
        let event_service = EventService::new();
        let plan_service = PlanService::new();

        Self {
            invoice_service,
            customer_service,
            customer_usage_service,
            subscription_service,
            billable_metric_service,
            activity_log_service,
            api_log_service,
            applied_coupon_service,
            coupon_service,
            credit_note_service,
            event_service,
            plan_service,
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

    #[tool(
        description = "Get the current usage for a customer's subscription. This endpoint retrieves the usage-based billing data for a customer within the current billing period."
    )]
    pub async fn get_customer_current_usage(
        &self,
        parameters: Parameters<crate::tools::customer_usage::GetCustomerCurrentUsageArgs>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        self.customer_usage_service
            .get_customer_current_usage(parameters, context)
            .await
    }

    #[tool(
        description = "List all subscriptions from Lago with optional filtering by plan code and status"
    )]
    pub async fn list_subscriptions(
        &self,
        parameters: Parameters<crate::tools::subscription::ListSubscriptionsArgs>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        self.subscription_service
            .list_subscriptions(parameters, context)
            .await
    }

    #[tool(description = "Get a specific subscription by its external ID")]
    pub async fn get_subscription(
        &self,
        parameters: Parameters<crate::tools::subscription::GetSubscriptionArgs>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        self.subscription_service
            .get_subscription(parameters, context)
            .await
    }

    #[tool(
        description = "List all subscriptions for a specific customer with optional filtering by plan code and status"
    )]
    pub async fn list_customer_subscriptions(
        &self,
        parameters: Parameters<crate::tools::subscription::ListCustomerSubscriptionsArgs>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        self.subscription_service
            .list_customer_subscriptions(parameters, context)
            .await
    }

    #[tool(
        description = "Create a new subscription to assign a plan to a customer. You can customize the plan with overrides."
    )]
    pub async fn create_subscription(
        &self,
        parameters: Parameters<crate::tools::subscription::CreateSubscriptionArgs>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        self.subscription_service
            .create_subscription(parameters, context)
            .await
    }

    #[tool(
        description = "Update an existing subscription. You can change the name, ending date, plan, or apply plan overrides."
    )]
    pub async fn update_subscription(
        &self,
        parameters: Parameters<crate::tools::subscription::UpdateSubscriptionArgs>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        self.subscription_service
            .update_subscription(parameters, context)
            .await
    }

    #[tool(description = "Delete (terminate) a subscription by its external ID")]
    pub async fn delete_subscription(
        &self,
        parameters: Parameters<crate::tools::subscription::DeleteSubscriptionArgs>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        self.subscription_service
            .delete_subscription(parameters, context)
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

    #[tool(description = "List all coupons in Lago with optional pagination")]
    pub async fn list_coupons(
        &self,
        parameters: Parameters<crate::tools::coupon::ListCouponsArgs>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        self.coupon_service.list_coupons(parameters, context).await
    }

    #[tool(description = "Get a specific coupon by its unique code")]
    pub async fn get_coupon(
        &self,
        parameters: Parameters<crate::tools::coupon::GetCouponArgs>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        self.coupon_service.get_coupon(parameters, context).await
    }

    #[tool(
        description = "Create a new coupon in Lago. Coupons can be either fixed_amount (with amount_cents and amount_currency) or percentage (with percentage_rate). Frequency can be 'once', 'recurring', or 'forever'."
    )]
    pub async fn create_coupon(
        &self,
        parameters: Parameters<crate::tools::coupon::CreateCouponArgs>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        self.coupon_service.create_coupon(parameters, context).await
    }

    #[tool(
        description = "Update an existing coupon in Lago. Only provided fields will be updated."
    )]
    pub async fn update_coupon(
        &self,
        parameters: Parameters<crate::tools::coupon::UpdateCouponArgs>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        self.coupon_service.update_coupon(parameters, context).await
    }

    #[tool(description = "Delete a coupon by its unique code. This will terminate the coupon.")]
    pub async fn delete_coupon(
        &self,
        parameters: Parameters<crate::tools::coupon::DeleteCouponArgs>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        self.coupon_service.delete_coupon(parameters, context).await
    }

    #[tool(description = "Retrieve a specific usage event by its transaction ID")]
    pub async fn get_event(
        &self,
        parameters: Parameters<crate::tools::event::GetEventArgs>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        self.event_service.get_event(parameters, context).await
    }

    #[tool(
        description = "Send a usage event to Lago. Events are used to track customer usage and are aggregated into invoice line items based on billable metrics. Provide either external_customer_id or external_subscription_id."
    )]
    pub async fn create_event(
        &self,
        parameters: Parameters<crate::tools::event::CreateEventArgs>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        self.event_service.create_event(parameters, context).await
    }

    #[tool(
        description = "List credit notes from Lago with optional filtering by customer, dates, reason, status, and amount range"
    )]
    pub async fn list_credit_notes(
        &self,
        parameters: Parameters<crate::tools::credit_note::ListCreditNotesArgs>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        self.credit_note_service
            .list_credit_notes(parameters, context)
            .await
    }

    #[tool(description = "Get a specific credit note by its Lago ID")]
    pub async fn get_credit_note(
        &self,
        parameters: Parameters<crate::tools::credit_note::GetCreditNoteArgs>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        self.credit_note_service
            .get_credit_note(parameters, context)
            .await
    }

    #[tool(
        description = "Create a credit note for an invoice. Credit notes are used to refund or credit customers for invoices. Specify the invoice ID, reason, amounts, and line items to credit."
    )]
    pub async fn create_credit_note(
        &self,
        parameters: Parameters<crate::tools::credit_note::CreateCreditNoteArgs>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        self.credit_note_service
            .create_credit_note(parameters, context)
            .await
    }

    #[tool(
        description = "Update a credit note's refund status. Use this to mark a refund as succeeded or failed."
    )]
    pub async fn update_credit_note(
        &self,
        parameters: Parameters<crate::tools::credit_note::UpdateCreditNoteArgs>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        self.credit_note_service
            .update_credit_note(parameters, context)
            .await
    }

    #[tool(description = "List all plans from Lago with optional pagination")]
    pub async fn list_plans(
        &self,
        parameters: Parameters<crate::tools::plan::ListPlansArgs>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        self.plan_service.list_plans(parameters, context).await
    }

    #[tool(description = "Get a specific plan by its unique code")]
    pub async fn get_plan(
        &self,
        parameters: Parameters<crate::tools::plan::GetPlanArgs>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        self.plan_service.get_plan(parameters, context).await
    }

    #[tool(
        description = "Create a new plan in Lago. Plans define pricing configuration with billing interval, base amount, and optional usage-based charges."
    )]
    pub async fn create_plan(
        &self,
        parameters: Parameters<crate::tools::plan::CreatePlanArgs>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        self.plan_service.create_plan(parameters, context).await
    }

    #[tool(
        description = "Update an existing plan in Lago. You can modify the name, description, pricing, charges, and other properties. Use cascade_updates to propagate changes to existing subscriptions."
    )]
    pub async fn update_plan(
        &self,
        parameters: Parameters<crate::tools::plan::UpdatePlanArgs>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        self.plan_service.update_plan(parameters, context).await
    }

    #[tool(
        description = "Delete a plan by its unique code. Note: This plan could be associated with active subscriptions."
    )]
    pub async fn delete_plan(
        &self,
        parameters: Parameters<crate::tools::plan::DeletePlanArgs>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        self.plan_service.delete_plan(parameters, context).await
    }
}

#[tool_handler]
impl ServerHandler for LagoMcpServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            instructions: Some(
                "Lago MCP server for managing invoices, customers, customer usage, subscriptions, plans, billable metrics, coupons, applied coupons, credit notes, activity logs, API logs, events, and other Lago resources. Use the available tools to interact with the Lago API.".into()
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
