use lago_types::models::invoice::{Invoice, InvoicePaymentStatus, InvoiceStatus, InvoiceType};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct InvoiceFilterParams {
    pub customer_id: Option<String>,
    pub customer_external_id: Option<String>,
    pub issuing_date_from: Option<String>,
    pub issuing_date_to: Option<String>,
    pub status: Option<InvoiceStatus>,
    pub payment_status: Option<InvoicePaymentStatus>,
    pub invoice_type: Option<InvoiceType>,
    pub page: Option<i32>,
    pub per_page: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvoiceSummary {
    pub lago_id: String,
    pub number: String,
    pub issuing_date: String,
    pub invoice_type: InvoiceType,
    pub status: InvoiceStatus,
    pub payment_status: InvoicePaymentStatus,
    pub currency: String,
    pub total_amount_cents: i64,
    pub customer_external_id: Option<String>,
    pub customer_name: Option<String>,
}

impl From<lago_types::models::invoice::Invoice> for InvoiceSummary {
    fn from(invoice: Invoice) -> Self {
        Self {
            lago_id: invoice.lago_id.to_string(),
            number: invoice.number,
            issuing_date: invoice.issuing_date,
            invoice_type: invoice.invoice_type,
            status: invoice.status,
            payment_status: invoice.payment_status,
            currency: invoice.currency,
            total_amount_cents: invoice.total_amount_cents,
            customer_external_id: invoice.customer.as_ref().map(|c| c.external_id.clone()),
            customer_name: invoice.customer.as_ref().and_then(|c| c.name.clone()),
        }
    }
}
