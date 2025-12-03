use anyhow::Result;
use rmcp::{RoleServer, handler::server::tool::Parameters, model::*, service::RequestContext};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use lago_types::{
    models::PaginationParams,
    requests::payment::{
        CreatePaymentInput, CreatePaymentRequest, GetPaymentRequest, ListCustomerPaymentsRequest,
        ListPaymentsRequest,
    },
};

use crate::tools::{create_lago_client, error_result, success_result};

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct ListPaymentsArgs {
    /// Filter by external customer ID.
    pub external_customer_id: Option<String>,
    /// Filter by invoice ID (UUID format).
    pub invoice_id: Option<String>,
    /// Page number for pagination (default: 1).
    pub page: Option<i32>,
    /// Number of items per page (default: 20).
    pub per_page: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct GetPaymentArgs {
    /// The Lago ID of the payment (UUID format).
    pub lago_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct ListCustomerPaymentsArgs {
    /// The external customer ID.
    pub external_customer_id: String,
    /// Filter by invoice ID (UUID format).
    pub invoice_id: Option<String>,
    /// Page number for pagination (default: 1).
    pub page: Option<i32>,
    /// Number of items per page (default: 20).
    pub per_page: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct CreatePaymentArgs {
    /// The invoice ID to associate with the payment.
    pub invoice_id: String,
    /// The payment amount in cents.
    pub amount_cents: i64,
    /// A reference for the payment.
    pub reference: String,
    /// The date the payment was made (YYYY-MM-DD format).
    pub paid_at: Option<String>,
}

#[derive(Clone)]
pub struct PaymentService;

impl PaymentService {
    pub fn new() -> Self {
        Self
    }

    pub async fn list_payments(
        &self,
        Parameters(args): Parameters<ListPaymentsArgs>,
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

        let mut request = ListPaymentsRequest::new().with_pagination(pagination);

        if let Some(external_customer_id) = args.external_customer_id {
            request = request.with_external_customer_id(external_customer_id);
        }

        if let Some(invoice_id_str) = args.invoice_id {
            match Uuid::parse_str(&invoice_id_str) {
                Ok(invoice_id) => {
                    request = request.with_invoice_id(invoice_id);
                }
                Err(_) => {
                    return Ok(error_result(format!(
                        "Invalid invoice_id format: {}. Must be a valid UUID.",
                        invoice_id_str
                    )));
                }
            }
        }

        match client.list_payments(Some(request)).await {
            Ok(response) => {
                let result = serde_json::json!({
                    "payments": response.payments,
                    "pagination": response.meta
                });

                Ok(success_result(&result))
            }
            Err(e) => {
                let error_message = format!("Failed to list payments: {e}");
                tracing::error!("{error_message}");
                Ok(error_result(error_message))
            }
        }
    }

    pub async fn get_payment(
        &self,
        Parameters(args): Parameters<GetPaymentArgs>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let client = match create_lago_client(&context).await {
            Ok(client) => client,
            Err(error_result) => return Ok(error_result),
        };

        let lago_id = match Uuid::parse_str(&args.lago_id) {
            Ok(id) => id,
            Err(_) => {
                return Ok(error_result(format!(
                    "Invalid lago_id format: {}. Must be a valid UUID.",
                    args.lago_id
                )));
            }
        };

        let request = GetPaymentRequest::new(lago_id);

        match client.get_payment(request).await {
            Ok(response) => {
                let result = serde_json::json!({
                    "payment": response.payment,
                });

                Ok(success_result(&result))
            }
            Err(e) => {
                let error_message = format!("Failed to get payment: {e}");
                tracing::error!("{error_message}");
                Ok(error_result(error_message))
            }
        }
    }

    pub async fn list_customer_payments(
        &self,
        Parameters(args): Parameters<ListCustomerPaymentsArgs>,
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

        let mut request =
            ListCustomerPaymentsRequest::new(args.external_customer_id).with_pagination(pagination);

        if let Some(invoice_id_str) = args.invoice_id {
            match Uuid::parse_str(&invoice_id_str) {
                Ok(invoice_id) => {
                    request = request.with_invoice_id(invoice_id);
                }
                Err(_) => {
                    return Ok(error_result(format!(
                        "Invalid invoice_id format: {}. Must be a valid UUID.",
                        invoice_id_str
                    )));
                }
            }
        }

        match client.list_customer_payments(request).await {
            Ok(response) => {
                let result = serde_json::json!({
                    "payments": response.payments,
                    "pagination": response.meta
                });

                Ok(success_result(&result))
            }
            Err(e) => {
                let error_message = format!("Failed to list customer payments: {e}");
                tracing::error!("{error_message}");
                Ok(error_result(error_message))
            }
        }
    }

    pub async fn create_payment(
        &self,
        Parameters(args): Parameters<CreatePaymentArgs>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let client = match create_lago_client(&context).await {
            Ok(client) => client,
            Err(error_result) => return Ok(error_result),
        };

        let mut input = CreatePaymentInput::new(args.invoice_id, args.amount_cents, args.reference);

        if let Some(paid_at) = args.paid_at {
            input = input.with_paid_at(paid_at);
        }

        let request = CreatePaymentRequest::new(input);

        match client.create_payment(request).await {
            Ok(response) => {
                let result = serde_json::json!({
                    "payment": response.payment,
                });

                Ok(success_result(&result))
            }
            Err(e) => {
                let error_message = format!("Failed to create payment: {e}");
                tracing::error!("{error_message}");
                Ok(error_result(error_message))
            }
        }
    }
}
