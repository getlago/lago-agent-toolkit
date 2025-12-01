use anyhow::Result;
use rmcp::{RoleServer, handler::server::tool::Parameters, model::*, service::RequestContext};
use serde::{Deserialize, Serialize};

use lago_types::filters::credit_note::CreditNoteFilter;
use lago_types::models::{CreditNoteReason, CreditNoteRefundStatus};
use lago_types::requests::credit_note::{
    CreateCreditNoteInput, CreateCreditNoteItemInput, CreateCreditNoteRequest,
    GetCreditNoteRequest, ListCreditNotesRequest, UpdateCreditNoteInput, UpdateCreditNoteRequest,
};

use crate::tools::{create_lago_client, error_result, success_result};

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct ListCreditNotesArgs {
    /// Page number for pagination (starting from 1)
    pub page: Option<i32>,
    /// Number of items per page (max 100)
    pub per_page: Option<i32>,
    /// Filter by external customer ID
    pub external_customer_id: Option<String>,
    /// Filter by issuing date from (ISO 8601 date, e.g., "2024-01-01")
    pub issuing_date_from: Option<String>,
    /// Filter by issuing date to (ISO 8601 date, e.g., "2024-12-31")
    pub issuing_date_to: Option<String>,
    /// Search by id, number, customer name, external_id or email
    pub search_term: Option<String>,
    /// Filter by currency (ISO 4217 code, e.g., "USD")
    pub currency: Option<String>,
    /// Filter by reason (duplicated_charge, product_unsatisfactory, order_change, order_cancellation, fraudulent_charge, other)
    pub reason: Option<String>,
    /// Filter by credit status (available, consumed, voided)
    pub credit_status: Option<String>,
    /// Filter by refund status (pending, succeeded, failed)
    pub refund_status: Option<String>,
    /// Filter by invoice number
    pub invoice_number: Option<String>,
    /// Filter by minimum amount in cents
    pub amount_from: Option<i64>,
    /// Filter by maximum amount in cents
    pub amount_to: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct GetCreditNoteArgs {
    /// The Lago ID of the credit note to retrieve
    pub lago_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct CreditNoteItemArg {
    /// The Lago ID of the fee to credit
    pub fee_id: String,
    /// The amount to credit for this fee in cents
    pub amount_cents: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct CreateCreditNoteArgs {
    /// The Lago ID of the invoice to credit
    pub invoice_id: String,
    /// The reason for the credit note (duplicated_charge, product_unsatisfactory, order_change, order_cancellation, fraudulent_charge, other)
    pub reason: String,
    /// Optional description for the credit note
    pub description: Option<String>,
    /// The amount to be credited in cents
    pub credit_amount_cents: i64,
    /// The amount to be refunded in cents
    pub refund_amount_cents: i64,
    /// The line items for the credit note
    pub items: Vec<CreditNoteItemArg>,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct UpdateCreditNoteArgs {
    /// The Lago ID of the credit note to update
    pub lago_id: String,
    /// The new refund status (pending, succeeded, failed)
    pub refund_status: String,
}

#[derive(Clone)]
pub struct CreditNoteService;

impl CreditNoteService {
    pub fn new() -> Self {
        Self
    }

    pub async fn list_credit_notes(
        &self,
        Parameters(args): Parameters<ListCreditNotesArgs>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let client = match create_lago_client(&context).await {
            Ok(client) => client,
            Err(error_result) => return Ok(error_result),
        };

        let mut request = ListCreditNotesRequest::new();

        // Apply pagination
        if let Some(page) = args.page {
            request.pagination.page = Some(page);
        }
        if let Some(per_page) = args.per_page {
            request.pagination.per_page = Some(per_page);
        }

        // Build filter
        let mut filter = CreditNoteFilter::new();

        if let Some(customer_id) = args.external_customer_id {
            filter = filter.with_external_customer_id(customer_id);
        }
        if let Some(from) = args.issuing_date_from {
            filter = filter.with_issuing_date_from(from);
        }
        if let Some(to) = args.issuing_date_to {
            filter = filter.with_issuing_date_to(to);
        }
        if let Some(term) = args.search_term {
            filter = filter.with_search_term(term);
        }
        if let Some(currency) = args.currency {
            filter = filter.with_currency(currency);
        }
        if let Some(reason_str) = args.reason
            && let Ok(reason) = reason_str.parse::<CreditNoteReason>()
        {
            filter = filter.with_reason(reason);
        }
        if let Some(status_str) = &args.credit_status
            && let Ok(status) = status_str.parse::<lago_types::models::CreditNoteCreditStatus>()
        {
            filter = filter.with_credit_status(status);
        }
        if let Some(status_str) = &args.refund_status
            && let Ok(status) = status_str.parse::<CreditNoteRefundStatus>()
        {
            filter = filter.with_refund_status(status);
        }
        if let Some(number) = args.invoice_number {
            filter = filter.with_invoice_number(number);
        }
        if let Some(amount) = args.amount_from {
            filter = filter.with_amount_from(amount);
        }
        if let Some(amount) = args.amount_to {
            filter = filter.with_amount_to(amount);
        }

        request = request.with_filters(filter);

        match client.list_credit_notes(Some(request)).await {
            Ok(response) => {
                let result = serde_json::json!({
                    "credit_notes": response.credit_notes,
                    "meta": response.meta,
                });

                Ok(success_result(&result))
            }
            Err(e) => {
                let error_message = format!("Failed to list credit notes: {e}");
                tracing::error!(error = %e, "{error_message}");
                Ok(error_result(error_message))
            }
        }
    }

    pub async fn get_credit_note(
        &self,
        Parameters(args): Parameters<GetCreditNoteArgs>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let client = match create_lago_client(&context).await {
            Ok(client) => client,
            Err(error_result) => return Ok(error_result),
        };

        let request = GetCreditNoteRequest::new(args.lago_id.clone());

        match client.get_credit_note(request).await {
            Ok(response) => {
                let result = serde_json::json!({
                    "credit_note": response.credit_note,
                });

                Ok(success_result(&result))
            }
            Err(e) => {
                let error_message = format!("Failed to get credit note: {e}");
                tracing::error!(
                    lago_id = %args.lago_id,
                    error = %e,
                    "{error_message}"
                );
                Ok(error_result(error_message))
            }
        }
    }

    pub async fn create_credit_note(
        &self,
        Parameters(args): Parameters<CreateCreditNoteArgs>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let client = match create_lago_client(&context).await {
            Ok(client) => client,
            Err(error_result) => return Ok(error_result),
        };

        // Parse reason
        let reason = match args.reason.parse::<CreditNoteReason>() {
            Ok(r) => r,
            Err(_) => {
                return Ok(error_result(format!(
                    "Invalid reason '{}'. Must be one of: duplicated_charge, product_unsatisfactory, order_change, order_cancellation, fraudulent_charge, other",
                    args.reason
                )));
            }
        };

        // Convert items
        let items: Vec<CreateCreditNoteItemInput> = args
            .items
            .into_iter()
            .map(|item| CreateCreditNoteItemInput::new(item.fee_id, item.amount_cents))
            .collect();

        let mut input = CreateCreditNoteInput::new(
            args.invoice_id.clone(),
            reason,
            args.credit_amount_cents,
            args.refund_amount_cents,
            items,
        );

        if let Some(description) = args.description {
            input = input.with_description(description);
        }

        let request = CreateCreditNoteRequest::new(input);

        match client.create_credit_note(request).await {
            Ok(response) => {
                let result = serde_json::json!({
                    "credit_note": response.credit_note,
                });

                Ok(success_result(&result))
            }
            Err(e) => {
                let error_message = format!("Failed to create credit note: {e}");
                tracing::error!(
                    invoice_id = %args.invoice_id,
                    error = %e,
                    "{error_message}"
                );
                Ok(error_result(error_message))
            }
        }
    }

    pub async fn update_credit_note(
        &self,
        Parameters(args): Parameters<UpdateCreditNoteArgs>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let client = match create_lago_client(&context).await {
            Ok(client) => client,
            Err(error_result) => return Ok(error_result),
        };

        // Parse refund status
        let refund_status = match args.refund_status.parse::<CreditNoteRefundStatus>() {
            Ok(s) => s,
            Err(_) => {
                return Ok(error_result(format!(
                    "Invalid refund_status '{}'. Must be one of: pending, succeeded, failed",
                    args.refund_status
                )));
            }
        };

        let input = UpdateCreditNoteInput::new().with_refund_status(refund_status);
        let request = UpdateCreditNoteRequest::new(args.lago_id.clone(), input);

        match client.update_credit_note(request).await {
            Ok(response) => {
                let result = serde_json::json!({
                    "credit_note": response.credit_note,
                });

                Ok(success_result(&result))
            }
            Err(e) => {
                let error_message = format!("Failed to update credit note: {e}");
                tracing::error!(
                    lago_id = %args.lago_id,
                    error = %e,
                    "{error_message}"
                );
                Ok(error_result(error_message))
            }
        }
    }
}
