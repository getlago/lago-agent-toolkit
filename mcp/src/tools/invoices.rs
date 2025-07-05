use std::sync::Arc;
use anyhow::Result;
use rmcp::{
    model::*,
    handler::server::tool::Parameters,
};
use serde::{Deserialize, Serialize};

use lago_client::LagoClient;
use lago_types::{
    requests::invoice::{ListInvoicesRequest, GetInvoiceRequest},
    filters::invoice::InvoiceFilters,
    models::{
        PaginationParams,
        invoice::{InvoiceType, InvoiceStatus, InvoicePaymentStatus},
    },
};

use crate::types::invoice::{InvoiceFilterParams, InvoiceSummary};

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct ListInvoicesArgs {
    pub customer_id: Option<String>,
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
pub struct InvoiceService {
    client: Arc<LagoClient>,
}

impl InvoiceService {
    pub fn new() -> Self {
        let client = Arc::new(LagoClient::from_env().expect("Failed to create Lago client from environment"));

        Self {
            client,
        }
    }

    fn parse_filters(&self, args: &ListInvoicesArgs) -> InvoiceFilterParams {
        let mut params = InvoiceFilterParams::default();

        params.customer_id = args.customer_id.clone();
        params.customer_external_id = args.customer_external_id.clone();
        params.issuing_date_from = args.issuing_date_from.clone();
        params.issuing_date_to = args.issuing_date_to.clone();
        params.page = args.page;
        params.per_page = args.per_page;

        if let Some(status_str) = &args.status {
            params.status = match status_str.to_lowercase().as_str() {
                "draft" => Some(InvoiceStatus::Draft),
                "finalized" => Some(InvoiceStatus::Finalized),
                "voided" => Some(InvoiceStatus::Voided),
                "pending" => Some(InvoiceStatus::Pending),
                "failed" => Some(InvoiceStatus::Failed),
                _ => None,
            };
        }

        if let Some(payment_status_str) = &args.payment_status {
            params.payment_status = match payment_status_str.to_lowercase().as_str() {
                "pending" => Some(InvoicePaymentStatus::Pending),
                "succeeded" => Some(InvoicePaymentStatus::Succeeded),
                "failed" => Some(InvoicePaymentStatus::Failed),
                _ => None,
            };
        }

        if let Some(invoice_type_str) = &args.invoice_type {
            params.invoice_type = match invoice_type_str.to_lowercase().as_str() {
                "subscription" => Some(InvoiceType::Subscription),
                "add_on" => Some(InvoiceType::AddOn),
                "credit" => Some(InvoiceType::Credit),
                "one_off" => Some(InvoiceType::OneOff),
                "progressive_billing" => Some(InvoiceType::ProgressiveBilling),
                _ => None,
            };
        }

        params
    }

    fn build_request(&self, params: &InvoiceFilterParams) -> ListInvoicesRequest {
        let mut filters = InvoiceFilters::new();

        if let Some(customer_id) = &params.customer_id {
            filters.customer_filter = filters.customer_filter.with_customer_id(customer_id.clone());
        }

        if let Some(customer_external_id) = &params.customer_external_id {
            filters.customer_filter = filters.customer_filter.with_customer_id(customer_external_id.clone());
        }

        if let Some(from_date) = &params.issuing_date_from {
            filters = filters.with_issuing_date_from(from_date.clone());
        }

        if let Some(to_date) = &params.issuing_date_to {
            filters = filters.with_issuing_date_to(to_date.clone());
        }

        if let Some(status) = &params.status {
            filters.status = Some(status.clone());
        }

        if let Some(payment_status) = &params.payment_status {
            filters = filters.with_status(payment_status.clone());
        }

        if let Some(invoice_type) = &params.invoice_type {
            filters = filters.with_invoice_type(invoice_type.clone());
        }

        let mut pagination = PaginationParams::default();
        if let Some(page) = params.page {
            pagination = pagination.with_page(page);
        }
        if let Some(per_page) = params.per_page {
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
    ) -> Result<CallToolResult, rmcp::Error> {
        let params = self.parse_filters(&args);
        let request = self.build_request(&params);
        
        match self.client.list_invoices(Some(request)).await {
            Ok(response) => {
                let invoice_summaries: Vec<InvoiceSummary> = response.invoices
                    .into_iter()
                    .map(InvoiceSummary::from)
                    .collect();

                let result = serde_json::json!({
                    "invoices": invoice_summaries,
                    "pagination": {
                        "current_page": response.meta.current_page,
                        "next_page": response.meta.next_page,
                        "prev_page": response.meta.prev_page,
                        "total_pages": response.meta.total_pages,
                        "total_count": response.meta.total_count
                    }
                });

                Ok(CallToolResult::success(vec![Content::text(
                    serde_json::to_string_pretty(&result).unwrap_or_else(|_| "Failed to serialize result".to_string())
                )]))
            }
            Err(e) => {
                let error_message = format!("Failed to list invoices: {}", e);
                Ok(CallToolResult::error(vec![Content::text(error_message)]))
            }
        }
    }

    pub async fn get_invoice(
        &self,
        Parameters(args): Parameters<GetInvoiceArgs>,
    ) -> Result<CallToolResult, rmcp::Error> {
        let request = GetInvoiceRequest::new(args.invoice_id);
        
        match self.client.get_invoice(request).await {
            Ok(response) => {
                let invoice_summary = InvoiceSummary::from(response.invoice);

                let result = serde_json::json!({
                    "invoice": invoice_summary
                });

                Ok(CallToolResult::success(vec![Content::text(
                    serde_json::to_string_pretty(&result).unwrap_or_else(|_| "Failed to serialize result".to_string())
                )]))
            }
            Err(e) => {
                let error_message = format!("Failed to get invoice: {}", e);
                Ok(CallToolResult::error(vec![Content::text(error_message)]))
            }
        }
    }
}
