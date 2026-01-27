use anyhow::Result;
use rmcp::{RoleServer, handler::server::tool::Parameters, model::*, service::RequestContext};
use serde::{Deserialize, Serialize};

use lago_types::{
    filters::invoice::InvoiceFilters,
    models::{InvoicePaymentStatus, InvoiceStatus, InvoiceType, PaginationParams},
    requests::invoice::{
        BillingTime, CreateInvoiceFeeInput, CreateInvoiceInput, CreateInvoiceRequest,
        DownloadInvoiceRequest, GetInvoiceRequest, InvoicePreviewCoupon, InvoicePreviewCustomer,
        InvoicePreviewInput, InvoicePreviewRequest, InvoicePreviewSubscriptions,
        ListCustomerInvoicesRequest, ListInvoicesRequest, RefreshInvoiceRequest,
        RetryInvoicePaymentRequest, RetryInvoiceRequest, UpdateInvoiceInput,
        UpdateInvoiceMetadataInput, UpdateInvoiceRequest, VoidInvoiceRequest,
    },
};

use crate::tools::{create_lago_client, error_result, success_result};

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct ListInvoicesArgs {
    /// Search by invoice id, number, customer name, external_id or email.
    pub search_term: Option<String>,
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

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct PreviewInvoiceCouponArgs {
    pub code: String,
    pub name: Option<String>,
    pub coupon_type: Option<String>,
    pub amount_cents: Option<i64>,
    pub amount_currency: Option<String>,
    pub percentage_rate: Option<String>,
    pub frequency_duration: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct PreviewInvoiceSubscriptionsArgs {
    pub external_ids: Vec<String>,
    pub plan_code: Option<String>,
    pub terminated_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct PreviewInvoiceArgs {
    pub customer_external_id: Option<String>,
    pub customer_name: Option<String>,
    pub customer_currency: Option<String>,
    pub customer_address_line1: Option<String>,
    pub customer_address_line2: Option<String>,
    pub customer_city: Option<String>,
    pub customer_state: Option<String>,
    pub customer_country: Option<String>,
    pub customer_tax_identification_number: Option<String>,
    pub plan_code: Option<String>,
    pub subscription_at: Option<String>,
    pub billing_time: Option<String>,
    pub coupons: Option<Vec<PreviewInvoiceCouponArgs>>,
    pub subscriptions: Option<PreviewInvoiceSubscriptionsArgs>,
    pub billing_entity_code: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct CreateInvoiceFeeArgs {
    /// The code of the add-on to charge.
    pub add_on_code: String,
    /// The number of units to charge.
    pub units: f64,
    /// The price per unit in cents (optional, uses add-on default if not specified).
    pub unit_amount_cents: Option<i64>,
    /// Optional description for the fee.
    pub description: Option<String>,
    /// Optional tax codes to apply to this fee.
    pub tax_codes: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct CreateInvoiceArgs {
    /// The external customer ID to create the invoice for.
    pub external_customer_id: String,
    /// The currency for the invoice (ISO 4217 code, e.g., "USD").
    pub currency: String,
    /// The list of fees to include in the invoice.
    pub fees: Vec<CreateInvoiceFeeArgs>,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct UpdateInvoiceMetadataArgs {
    /// The ID of an existing metadata entry to update (optional for new entries).
    pub id: Option<String>,
    /// The metadata key.
    pub key: String,
    /// The metadata value.
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct UpdateInvoiceArgs {
    /// The Lago ID (UUID) of the invoice to update.
    pub lago_id: String,
    /// The payment status to set (e.g., "pending", "succeeded", "failed").
    pub payment_status: Option<String>,
    /// Custom metadata entries to set on the invoice.
    pub metadata: Option<Vec<UpdateInvoiceMetadataArgs>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct ListCustomerInvoicesArgs {
    /// The external customer ID to list invoices for.
    pub external_customer_id: String,
    /// Page number for pagination.
    pub page: Option<i32>,
    /// Number of items per page.
    pub per_page: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct RefreshInvoiceArgs {
    /// The Lago ID (UUID) of the draft invoice to refresh.
    pub lago_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct DownloadInvoiceArgs {
    /// The Lago ID (UUID) of the invoice to download.
    pub lago_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct RetryInvoiceArgs {
    /// The Lago ID (UUID) of the failed invoice to retry.
    pub lago_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct RetryInvoicePaymentArgs {
    /// The Lago ID (UUID) of the invoice to retry payment for.
    pub lago_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct VoidInvoiceArgs {
    /// The Lago ID (UUID) of the finalized invoice to void.
    pub lago_id: String,
}

#[derive(Clone)]
pub struct InvoiceService;

impl InvoiceService {
    pub fn new() -> Self {
        Self
    }

    #[allow(clippy::collapsible_if)]
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

        let mut request = ListInvoicesRequest::new()
            .with_filters(filters)
            .with_pagination(pagination);

        if let Some(search_term) = &args.search_term {
            request = request.with_search_term(search_term.clone());
        }

        request
    }
}

impl InvoiceService {
    pub async fn list_invoices(
        &self,
        Parameters(args): Parameters<ListInvoicesArgs>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let client = match create_lago_client(&context).await {
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
                let error_message = format!("Failed to list invoices: {e}");
                tracing::error!("{error_message}");
                Ok(error_result(error_message))
            }
        }
    }

    pub async fn get_invoice(
        &self,
        Parameters(args): Parameters<GetInvoiceArgs>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let client = match create_lago_client(&context).await {
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
                tracing::error!("{error_message}");
                Ok(error_result(error_message))
            }
        }
    }

    fn build_preview_request(&self, args: &PreviewInvoiceArgs) -> InvoicePreviewRequest {
        let customer = if let Some(external_id) = &args.customer_external_id {
            InvoicePreviewCustomer::with_external_id(external_id.clone())
        } else {
            let mut customer = InvoicePreviewCustomer::new();

            if let Some(name) = &args.customer_name {
                customer = customer.with_name(name.clone());
            }

            if let Some(currency) = &args.customer_currency {
                customer = customer.with_currency(currency.clone());
            }

            if args.customer_address_line1.is_some()
                || args.customer_address_line2.is_some()
                || args.customer_city.is_some()
                || args.customer_state.is_some()
                || args.customer_country.is_some()
            {
                customer = customer.with_address(
                    args.customer_address_line1.clone().unwrap_or_default(),
                    args.customer_address_line2.clone(),
                    args.customer_city.clone(),
                    args.customer_state.clone(),
                    args.customer_country.clone(),
                );
            }

            if let Some(tax_id) = &args.customer_tax_identification_number {
                customer = customer.with_tax_identification_number(tax_id.clone());
            }

            customer
        };

        let mut input = InvoicePreviewInput::new(customer);

        if let Some(plan_code) = &args.plan_code {
            input = input.with_plan_code(plan_code.clone());
        }

        if let Some(subscription_at) = &args.subscription_at {
            input = input.with_subscription_at(subscription_at.clone());
        }

        if let Some(billing_time_str) = &args.billing_time {
            let billing_time = match billing_time_str.to_lowercase().as_str() {
                "anniversary" => BillingTime::Anniversary,
                _ => BillingTime::Calendar,
            };
            input = input.with_billing_time(billing_time);
        }

        if let Some(coupon_args) = &args.coupons {
            let coupons: Vec<InvoicePreviewCoupon> = coupon_args
                .iter()
                .map(|c| {
                    let mut coupon = InvoicePreviewCoupon::new(c.code.clone());

                    if let Some(name) = &c.name {
                        coupon = coupon.with_name(name.clone());
                    }

                    if let (Some(amount_cents), Some(currency)) =
                        (c.amount_cents, &c.amount_currency)
                    {
                        coupon = coupon.with_fixed_amount(amount_cents, currency.clone());
                    } else if let Some(percentage_rate) = &c.percentage_rate {
                        coupon = coupon.with_percentage(percentage_rate.clone());
                    }

                    if let Some(duration) = c.frequency_duration {
                        coupon = coupon.with_frequency_duration(duration);
                    }

                    coupon
                })
                .collect();

            input = input.with_coupons(coupons);
        }

        if let Some(subs_args) = &args.subscriptions {
            let mut subscriptions =
                InvoicePreviewSubscriptions::new(subs_args.external_ids.clone());

            if let Some(plan_code) = &subs_args.plan_code {
                subscriptions = subscriptions.with_plan_code(plan_code.clone());
            }

            if let Some(terminated_at) = &subs_args.terminated_at {
                subscriptions = subscriptions.with_terminated_at(terminated_at.clone());
            }

            input = input.with_subscriptions(subscriptions);
        }

        if let Some(billing_entity_code) = &args.billing_entity_code {
            input = input.with_billing_entity_code(billing_entity_code.clone());
        }

        InvoicePreviewRequest::new(input)
    }

    pub async fn preview_invoice(
        &self,
        Parameters(args): Parameters<PreviewInvoiceArgs>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let client = match create_lago_client(&context).await {
            Ok(client) => client,
            Err(error_result) => return Ok(error_result),
        };

        let request = self.build_preview_request(&args);

        match client.preview_invoice(request).await {
            Ok(response) => {
                let result = serde_json::json!({
                    "invoice": response.invoice,
                });

                Ok(success_result(&result))
            }
            Err(e) => {
                let error_message = format!("Failed to preview invoice: {e}");
                tracing::error!("{error_message}");
                Ok(error_result(error_message))
            }
        }
    }

    pub async fn create_invoice(
        &self,
        Parameters(args): Parameters<CreateInvoiceArgs>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let client = match create_lago_client(&context).await {
            Ok(client) => client,
            Err(error_result) => return Ok(error_result),
        };

        let fees: Vec<CreateInvoiceFeeInput> = args
            .fees
            .into_iter()
            .map(|f| {
                let mut fee = CreateInvoiceFeeInput::new(f.add_on_code, f.units);
                if let Some(amount) = f.unit_amount_cents {
                    fee = fee.with_unit_amount_cents(amount);
                }
                if let Some(desc) = f.description {
                    fee = fee.with_description(desc);
                }
                if let Some(taxes) = f.tax_codes {
                    fee = fee.with_tax_codes(taxes);
                }
                fee
            })
            .collect();

        let input = CreateInvoiceInput::new(args.external_customer_id, args.currency, fees);
        let request = CreateInvoiceRequest::new(input);

        match client.create_invoice(request).await {
            Ok(response) => {
                let result = serde_json::json!({
                    "invoice": response.invoice,
                });

                Ok(success_result(&result))
            }
            Err(e) => {
                let error_message = format!("Failed to create invoice: {e}");
                tracing::error!("{error_message}");
                Ok(error_result(error_message))
            }
        }
    }

    pub async fn update_invoice(
        &self,
        Parameters(args): Parameters<UpdateInvoiceArgs>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let client = match create_lago_client(&context).await {
            Ok(client) => client,
            Err(error_result) => return Ok(error_result),
        };

        let mut input = UpdateInvoiceInput::new();

        if let Some(status) = args.payment_status {
            input = input.with_payment_status(status);
        }

        if let Some(metadata) = args.metadata {
            let metadata_inputs: Vec<UpdateInvoiceMetadataInput> = metadata
                .into_iter()
                .map(|m| {
                    if let Some(id) = m.id {
                        UpdateInvoiceMetadataInput::with_id(id, m.key, m.value)
                    } else {
                        UpdateInvoiceMetadataInput::new(m.key, m.value)
                    }
                })
                .collect();
            input = input.with_metadata(metadata_inputs);
        }

        let request = UpdateInvoiceRequest::new(args.lago_id, input);

        match client.update_invoice(request).await {
            Ok(response) => {
                let result = serde_json::json!({
                    "invoice": response.invoice,
                });

                Ok(success_result(&result))
            }
            Err(e) => {
                let error_message = format!("Failed to update invoice: {e}");
                tracing::error!("{error_message}");
                Ok(error_result(error_message))
            }
        }
    }

    pub async fn list_customer_invoices(
        &self,
        Parameters(args): Parameters<ListCustomerInvoicesArgs>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let client = match create_lago_client(&context).await {
            Ok(client) => client,
            Err(error_result) => return Ok(error_result),
        };

        let mut request = ListCustomerInvoicesRequest::new(args.external_customer_id);

        if args.page.is_some() || args.per_page.is_some() {
            let mut pagination = PaginationParams::default();
            if let Some(page) = args.page {
                pagination = pagination.with_page(page);
            }
            if let Some(per_page) = args.per_page {
                pagination = pagination.with_per_page(per_page);
            }
            request = request.with_pagination(pagination);
        }

        match client.list_customer_invoices(request).await {
            Ok(response) => {
                let result = serde_json::json!({
                    "invoices": response.invoices,
                    "pagination": response.meta,
                });

                Ok(success_result(&result))
            }
            Err(e) => {
                let error_message = format!("Failed to list customer invoices: {e}");
                tracing::error!("{error_message}");
                Ok(error_result(error_message))
            }
        }
    }

    pub async fn refresh_invoice(
        &self,
        Parameters(args): Parameters<RefreshInvoiceArgs>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let client = match create_lago_client(&context).await {
            Ok(client) => client,
            Err(error_result) => return Ok(error_result),
        };

        let request = RefreshInvoiceRequest::new(args.lago_id);

        match client.refresh_invoice(request).await {
            Ok(response) => {
                let result = serde_json::json!({
                    "invoice": response.invoice,
                });

                Ok(success_result(&result))
            }
            Err(e) => {
                let error_message = format!("Failed to refresh invoice: {e}");
                tracing::error!("{error_message}");
                Ok(error_result(error_message))
            }
        }
    }

    pub async fn download_invoice(
        &self,
        Parameters(args): Parameters<DownloadInvoiceArgs>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let client = match create_lago_client(&context).await {
            Ok(client) => client,
            Err(error_result) => return Ok(error_result),
        };

        let request = DownloadInvoiceRequest::new(args.lago_id);

        match client.download_invoice(request).await {
            Ok(response) => {
                let result = serde_json::json!({
                    "invoice": response.invoice,
                });

                Ok(success_result(&result))
            }
            Err(e) => {
                let error_message = format!("Failed to download invoice: {e}");
                tracing::error!("{error_message}");
                Ok(error_result(error_message))
            }
        }
    }

    pub async fn retry_invoice(
        &self,
        Parameters(args): Parameters<RetryInvoiceArgs>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let client = match create_lago_client(&context).await {
            Ok(client) => client,
            Err(error_result) => return Ok(error_result),
        };

        let request = RetryInvoiceRequest::new(args.lago_id);

        match client.retry_invoice(request).await {
            Ok(response) => {
                let result = serde_json::json!({
                    "invoice": response.invoice,
                });

                Ok(success_result(&result))
            }
            Err(e) => {
                let error_message = format!("Failed to retry invoice: {e}");
                tracing::error!("{error_message}");
                Ok(error_result(error_message))
            }
        }
    }

    pub async fn retry_invoice_payment(
        &self,
        Parameters(args): Parameters<RetryInvoicePaymentArgs>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let client = match create_lago_client(&context).await {
            Ok(client) => client,
            Err(error_result) => return Ok(error_result),
        };

        let request = RetryInvoicePaymentRequest::new(args.lago_id);

        match client.retry_invoice_payment(request).await {
            Ok(_) => {
                let result = serde_json::json!({
                    "success": true,
                    "message": "Invoice payment retry initiated successfully",
                });

                Ok(success_result(&result))
            }
            Err(e) => {
                let error_message = format!("Failed to retry invoice payment: {e}");
                tracing::error!("{error_message}");
                Ok(error_result(error_message))
            }
        }
    }

    pub async fn void_invoice(
        &self,
        Parameters(args): Parameters<VoidInvoiceArgs>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let client = match create_lago_client(&context).await {
            Ok(client) => client,
            Err(error_result) => return Ok(error_result),
        };

        let request = VoidInvoiceRequest::new(args.lago_id);

        match client.void_invoice(request).await {
            Ok(response) => {
                let result = serde_json::json!({
                    "invoice": response.invoice,
                });

                Ok(success_result(&result))
            }
            Err(e) => {
                let error_message = format!("Failed to void invoice: {e}");
                tracing::error!("{error_message}");
                Ok(error_result(error_message))
            }
        }
    }
}
