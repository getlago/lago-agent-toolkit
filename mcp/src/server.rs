use std::{sync::Arc, future::Future};
use anyhow::Result;
use rmcp::{
    ServerHandler, 
    model::*,
    tool_handler, tool_router, tool,
    handler::server::{router::tool::ToolRouter, tool::Parameters},
};

use crate::tools::invoices::InvoiceService;

#[derive(Clone)]
pub struct LagoMcpServer {
    invoice_service: Arc<InvoiceService>,
    tool_router: ToolRouter<Self>,
}

impl LagoMcpServer {
    pub fn new() -> Self {
        let invoice_service = Arc::new(InvoiceService::new());
        
        Self {
            invoice_service,
            tool_router: Self::tool_router(),
        }
    }
}

#[tool_router]
impl LagoMcpServer {
    #[tool(
        description = "Get a specific invoice by its Lago ID"
    )]
    pub async fn get_invoice(
        &self,
        parameters: Parameters<crate::tools::invoices::GetInvoiceArgs>,
    ) -> Result<CallToolResult, rmcp::Error> {
        self.invoice_service.get_invoice(parameters).await
    }
    
    #[tool(
        description = "List invoices from Lago with optional filtering by customer, dates, status, and type"
    )]
    pub async fn list_invoices(
        &self,
        parameters: Parameters<crate::tools::invoices::ListInvoicesArgs>,
    ) -> Result<CallToolResult, rmcp::Error> {
        self.invoice_service.list_invoices(parameters).await
    }
}

#[tool_handler]
impl ServerHandler for LagoMcpServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            instructions: Some("Lago MCP server for managing invoices and other Lago resources. Use the available tools to interact with your Lago instance.".into()),
            capabilities: ServerCapabilities::builder()
            .enable_tools()
            .build(),
            ..Default::default()
        }
    }
}
