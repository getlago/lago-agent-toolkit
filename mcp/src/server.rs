use anyhow::Result;
use rmcp::{
    ServerHandler,
    handler::server::{router::tool::ToolRouter, tool::Parameters},
    model::*,
    tool, tool_handler, tool_router,
};
use std::future::Future;

use crate::tools::billable_metric::BillableMetricService;
use crate::tools::customer::CustomerService;
use crate::tools::invoice::InvoiceService;

#[derive(Clone)]
#[allow(dead_code)]
pub struct LagoMcpServer {
    invoice_service: InvoiceService,
    customer_service: CustomerService,
    billable_metric_service: BillableMetricService,
    tool_router: ToolRouter<Self>,
}

#[allow(dead_code)]
impl LagoMcpServer {
    pub fn new() -> Self {
        let invoice_service = InvoiceService::new();
        let customer_service = CustomerService::new();
        let billable_metric_service = BillableMetricService::new();

        Self {
            invoice_service,
            customer_service,
            billable_metric_service,
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
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        self.invoice_service.get_invoice(parameters).await
    }

    #[tool(
        description = "List invoices from Lago with optional filtering by customer, dates, status and type"
    )]
    pub async fn list_invoices(
        &self,
        parameters: Parameters<crate::tools::invoice::ListInvoicesArgs>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        self.invoice_service.list_invoices(parameters).await
    }

    #[tool(description = "Get a specific customer by their external ID")]
    pub async fn get_customer(
        &self,
        parameters: Parameters<crate::tools::customer::GetCustomerArgs>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        self.customer_service.get_customer(parameters).await
    }

    #[tool(
        description = "List customers from Lago with optional filtering by external customer ID"
    )]
    pub async fn list_customers(
        &self,
        parameters: Parameters<crate::tools::customer::ListCustomersArgs>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        self.customer_service.list_customers(parameters).await
    }

    #[tool(description = "Create or update a customer in Lago")]
    pub async fn create_customer(
        &self,
        parameters: Parameters<crate::tools::customer::CreateCustomerArgs>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        self.customer_service.create_customer(parameters).await
    }

    #[tool(description = "Get a specific billable metric by its code")]
    pub async fn get_billable_metric(
        &self,
        parameters: Parameters<crate::tools::billable_metric::GetBillableMetricArgs>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        self.billable_metric_service
            .get_billable_metric(parameters)
            .await
    }

    #[tool(
        description = "List billable metrics from Lago with optional filtering by aggregation type and recurring status"
    )]
    pub async fn list_billable_metrics(
        &self,
        parameters: Parameters<crate::tools::billable_metric::ListBillableMetricsArgs>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        self.billable_metric_service
            .list_billable_metrics(parameters)
            .await
    }

    #[tool(description = "Create a new billable metric in Lago")]
    pub async fn create_billable_metric(
        &self,
        parameters: Parameters<crate::tools::billable_metric::CreateBillableMetricArgs>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        self.billable_metric_service
            .create_billable_metric(parameters)
            .await
    }
}

#[tool_handler]
impl ServerHandler for LagoMcpServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            instructions: Some(
                "Lago MCP server for managing invoices, customers, billable metrics and other lago resources. Use the available tools to interact with the Lago API.".into()
            ),
            capabilities: ServerCapabilities::builder()
                .enable_tools()
                .build(),
            ..Default::default()
        }
    }
}
