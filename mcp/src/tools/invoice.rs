use anyhow::Result;
use rmcp::{handler::server::tool::Parameters, model::*};
use serde::{Deserialize, Serialize};

use lago_types::{
    filters::invoice::InvoiceFilters,
    models::{InvoicePaymentStatus, InvoiceStatus, InvoiceType, PaginationParams},
    requests::invoice::{GetInvoiceRequest, ListInvoicesRequest},
};

use crate::tools::{create_lago_client, error_result, success_result};

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct ListInvoicesArgs {
    pub customer_external_id: Option<String>,
    pub issuing_date_from: Option<String>,
    pub issuing_date_to: Option<String>,
    pub status: Option<String>,
    pub payment_status: Option<String>,
    pub invoice_type: Option<String>,
    pub page: Option<i32>,
    pub per_page: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct GetInvoiceArgs {
    pub invoice_id: String,
}

#[derive(Clone)]
pub struct InvoiceService;

impl InvoiceService {
    pub fn new() -> Self {
        Self
    }

    fn build_request(&self, args: &ListInvoicesArgs) -> ListInvoicesRequest {
        let mut filters = InvoiceFilters::new();

        if let Some(customer_external_id) = &args.customer_external_id {
            filters.customer_filter = filters
                .customer_filter
                .with_customer_id(customer_external_id.clone());
        }

        if let Some(from_date) = &args.issuing_date_from {
            filters = filters.with_issuing_date_from(from_date.clone());
        }

        if let Some(to_date) = &args.issuing_date_to {
            filters = filters.with_issuing_date_to(to_date.clone());
        }

        if let Some(status_str) = &args.status {
            if let Ok(status) = status_str.parse::<InvoiceStatus>() {
                filters = filters.with_status(status);
            }
        }

        if let Some(payment_status_str) = &args.payment_status {
            if let Ok(payment_status) = payment_status_str.parse::<InvoicePaymentStatus>() {
                filters = filters.with_payment_status(payment_status);
            }
        }

        if let Some(invoice_type_str) = &args.invoice_type {
            if let Ok(invoice_type) = invoice_type_str.parse::<InvoiceType>() {
                filters = filters.with_invoice_type(invoice_type);
            }
        }

        let mut pagination = PaginationParams::default();
        if let Some(page) = args.page {
            pagination = pagination.with_page(page);
        }

        if let Some(per_page) = args.per_page {
            pagination = pagination.with_per_page(per_page);
        }

        ListInvoicesRequest::new()
            .with_filters(filters)
            .with_pagination(pagination)
    }
}

impl InvoiceService {
    pub async fn list_invoices(
        &self,
        Parameters(args): Parameters<ListInvoicesArgs>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let client = match create_lago_client() {
            Ok(client) => client,
            Err(error_result) => return Ok(error_result),
        };
        let request = self.build_request(&args);

        match client.list_invoices(Some(request)).await {
            Ok(response) => {
                let result = serde_json::json!({
                    "invoices": response.invoices,
                    "pagination": response.meta,
                });

                Ok(success_result(&result))
            }
            Err(e) => {
                let error_message = format!("failed to list invoices: {e}");
                Ok(error_result(error_message))
            }
        }
    }

    pub async fn get_invoice(
        &self,
        Parameters(args): Parameters<GetInvoiceArgs>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let client = match create_lago_client() {
            Ok(client) => client,
            Err(error_result) => return Ok(error_result),
        };
        let request = GetInvoiceRequest::new(args.invoice_id);

        match client.get_invoice(request).await {
            Ok(response) => {
                let result = serde_json::json!({
                    "invoice": response.invoice,
                });

                Ok(success_result(&result))
            }
            Err(e) => {
                let error_message = format!("Failed to get invoice: {e}");
                Ok(error_result(error_message))
            }
        }
    }
}
