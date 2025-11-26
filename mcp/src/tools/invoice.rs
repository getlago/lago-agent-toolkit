use anyhow::Result;
use rmcp::{RoleServer, handler::server::tool::Parameters, model::*, service::RequestContext};
use serde::{Deserialize, Serialize};

use lago_types::{
    filters::invoice::InvoiceFilters,
    models::{InvoicePaymentStatus, InvoiceStatus, InvoiceType, PaginationParams},
    requests::invoice::{
        BillingTime, GetInvoiceRequest, InvoicePreviewCoupon, InvoicePreviewCustomer,
        InvoicePreviewInput, InvoicePreviewRequest, InvoicePreviewSubscriptions,
        ListInvoicesRequest,
    },
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

        ListInvoicesRequest::new()
            .with_filters(filters)
            .with_pagination(pagination)
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
                let error_message = format!("failed to list invoices: {e}");
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
                Ok(error_result(error_message))
            }
        }
    }
}
