use anyhow::Result;
use rmcp::{
    ErrorData as McpError, RoleServer, ServerHandler,
    handler::server::{router::tool::ToolRouter, tool::Parameters},
    model::*,
    service::RequestContext,
    tool, tool_handler, tool_router,
};
use std::future::Future;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::tools::billable_metric::BillableMetricService;
use crate::tools::customer::CustomerService;
use crate::tools::invoice::InvoiceService;

#[derive(Clone)]
pub struct ApiCredentials {
    pub api_key: Option<String>,
    pub api_url: Option<String>,
}

#[derive(Clone)]
#[allow(dead_code)]
pub struct LagoMcpServer {
    invoice_service: InvoiceService,
    customer_service: CustomerService,
    billable_metric_service: BillableMetricService,
    tool_router: ToolRouter<Self>,
    credentials: Arc<RwLock<ApiCredentials>>,
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
            credentials: Arc::new(RwLock::new(ApiCredentials {
                api_key: None,
                api_url: None,
            })),
        }
    }

    pub async fn set_credentials(&self, api_key: Option<String>, api_url: Option<String>) {
        let mut creds = self.credentials.write().await;
        creds.api_key = api_key;
        creds.api_url = api_url;
    }

    pub async fn get_credentials(&self) -> ApiCredentials {
        self.credentials.read().await.clone()
    }
}

#[tool_router]
impl LagoMcpServer {
    #[tool(description = "Get a specific invoice by its Lago ID")]
    pub async fn get_invoice(
        &self,
        parameters: Parameters<crate::tools::invoice::GetInvoiceArgs>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        self.invoice_service.get_invoice(parameters, self).await
    }

    #[tool(
        description = "List invoices from Lago with optional filtering by customer, dates, status and type"
    )]
    pub async fn list_invoices(
        &self,
        parameters: Parameters<crate::tools::invoice::ListInvoicesArgs>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        self.invoice_service.list_invoices(parameters, self).await
    }

    #[tool(description = "Get a specific customer by their external ID")]
    pub async fn get_customer(
        &self,
        parameters: Parameters<crate::tools::customer::GetCustomerArgs>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        self.customer_service.get_customer(parameters, self).await
    }

    #[tool(
        description = "List customers from Lago with optional filtering by external customer ID"
    )]
    pub async fn list_customers(
        &self,
        parameters: Parameters<crate::tools::customer::ListCustomersArgs>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        self.customer_service.list_customers(parameters, self).await
    }

    #[tool(description = "Create or update a customer in Lago")]
    pub async fn create_customer(
        &self,
        parameters: Parameters<crate::tools::customer::CreateCustomerArgs>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        self.customer_service
            .create_customer(parameters, self)
            .await
    }

    #[tool(description = "Get a specific billable metric by its code")]
    pub async fn get_billable_metric(
        &self,
        parameters: Parameters<crate::tools::billable_metric::GetBillableMetricArgs>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        self.billable_metric_service
            .get_billable_metric(parameters, self)
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
            .list_billable_metrics(parameters, self)
            .await
    }

    #[tool(description = "Create a new billable metric in Lago")]
    pub async fn create_billable_metric(
        &self,
        parameters: Parameters<crate::tools::billable_metric::CreateBillableMetricArgs>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        self.billable_metric_service
            .create_billable_metric(parameters, self)
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

    async fn initialize(
        &self,
        _request: InitializeRequestParam,
        context: RequestContext<RoleServer>,
    ) -> Result<InitializeResult, McpError> {
        if let Some(http_request_part) = context.extensions.get::<axum::http::request::Parts>() {
            let initialize_headers = &http_request_part.headers;
            let initialize_uri = &http_request_part.uri;

            let api_key = initialize_headers
                .get("X-LAGO-API-KEY")
                .and_then(|v| v.to_str().ok())
                .map(|s| s.to_string());

            let api_url = initialize_headers
                .get("X-LAGO-API-URL")
                .and_then(|v| v.to_str().ok())
                .map(|s| s.to_string())
                .or_else(|| std::env::var("LAGO_API_URL").ok());

            self.set_credentials(api_key, api_url).await;
            tracing::info!(?initialize_headers, %initialize_uri, "initialize from http server");
        }
        Ok(self.get_info())
    }
}
