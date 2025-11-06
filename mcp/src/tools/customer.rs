use anyhow::Result;
use rmcp::{
    handler::server::tool::Parameters,
    service::RequestContext,
    RoleServer,
    model::*
};
use serde::{Deserialize, Serialize};

use lago_types::{
    filters::customer::CustomerFilter,
    models::{CustomerFinalizeZeroAmountInvoice, CustomerType, PaginationParams},
    requests::customer::{
        CreateCustomerInput, CreateCustomerRequest, GetCustomerRequest, ListCustomersRequest,
    },
};

use crate::tools::{create_lago_client, error_result, success_result};

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct ListCustomersArgs {
    pub external_customer_id: Option<String>,
    pub page: Option<i32>,
    pub per_page: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct GetCustomerArgs {
    pub external_customer_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct CreateCustomerArgs {
    pub external_id: String,
    pub name: Option<String>,
    pub firstname: Option<String>,
    pub lastname: Option<String>,
    pub email: Option<String>,
    pub address_line1: Option<String>,
    pub address_line2: Option<String>,
    pub city: Option<String>,
    pub country: Option<String>,
    pub state: Option<String>,
    pub zipcode: Option<String>,
    pub phone: Option<String>,
    pub url: Option<String>,
    pub legal_name: Option<String>,
    pub legal_number: Option<String>,
    pub logo_url: Option<String>,
    pub tax_identification_number: Option<String>,
    pub timezone: Option<String>,
    pub currency: Option<String>,
    pub net_payment_term: Option<i32>,
    pub customer_type: Option<String>,
    pub finalize_zero_amount_invoice: Option<String>,
}

#[derive(Clone)]
pub struct CustomerService;

impl CustomerService {
    pub fn new() -> Self {
        Self
    }

    fn build_request(&self, params: &ListCustomersArgs) -> ListCustomersRequest {
        let mut filters = CustomerFilter::new();

        if let Some(external_customer_id) = &params.external_customer_id {
            filters = filters.with_customer_id(external_customer_id.clone());
        }

        let mut pagination = PaginationParams::default();
        if let Some(page) = params.page {
            pagination = pagination.with_page(page);
        }

        if let Some(per_page) = params.per_page {
            pagination = pagination.with_per_page(per_page);
        }

        ListCustomersRequest::new()
            .with_filters(filters)
            .with_pagination(pagination)
    }

    pub async fn list_customers(
        &self,
        Parameters(args): Parameters<ListCustomersArgs>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let client = match create_lago_client(&context).await {
            Ok(client) => client,
            Err(error_result) => return Ok(error_result),
        };
        let request = self.build_request(&args);

        match client.list_customers(Some(request)).await {
            Ok(response) => {
                let result = serde_json::json!({
                    "customers": response.customers,
                    "pagination": response.meta,
                });

                Ok(success_result(&result))
            }
            Err(e) => {
                let error_message = format!("failed to list customers: {e}");
                Ok(error_result(error_message))
            }
        }
    }

    pub async fn get_customer(
        &self,
        Parameters(args): Parameters<GetCustomerArgs>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let client = match create_lago_client(&context).await {
            Ok(client) => client,
            Err(error_result) => return Ok(error_result),
        };
        let request = GetCustomerRequest::new(args.external_customer_id);

        match client.get_customer(request).await {
            Ok(response) => {
                let result = serde_json::json!({
                    "customer": response.customer,
                });

                Ok(success_result(&result))
            }
            Err(e) => {
                let error_message = format!("Failed to get customer: {e}");
                Ok(error_result(error_message))
            }
        }
    }

    #[allow(clippy::collapsible_if)]
    pub async fn create_customer(
        &self,
        Parameters(args): Parameters<CreateCustomerArgs>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let mut customer_input = CreateCustomerInput::new(args.external_id);

        if let Some(name) = args.name {
            customer_input = customer_input.with_name(name);
        }

        if let Some(firstname) = args.firstname {
            customer_input = customer_input.with_firstname(firstname);
        }

        if let Some(lastname) = args.lastname {
            customer_input = customer_input.with_lastname(lastname);
        }

        if let Some(email) = args.email {
            customer_input = customer_input.with_email(email);
        }

        if args.address_line1.is_some()
            || args.address_line2.is_some()
            || args.city.is_some()
            || args.country.is_some()
            || args.state.is_some()
            || args.zipcode.is_some()
        {
            customer_input = customer_input.with_address(
                args.address_line1.unwrap_or_default(),
                args.address_line2,
                args.city,
                args.country,
                args.state,
                args.zipcode,
            );
        }

        if let Some(phone) = args.phone {
            customer_input = customer_input.with_phone(phone);
        }

        if let Some(url) = args.url {
            customer_input = customer_input.with_url(url);
        }

        if args.legal_name.is_some() || args.legal_number.is_some() {
            customer_input = customer_input
                .with_legal_info(args.legal_name.unwrap_or_default(), args.legal_number);
        }

        if let Some(logo_url) = args.logo_url {
            customer_input.logo_url = Some(logo_url);
        }

        if let Some(tax_identification_number) = args.tax_identification_number {
            customer_input =
                customer_input.with_tax_identification_number(tax_identification_number);
        }

        if let Some(timezone) = args.timezone {
            customer_input = customer_input.with_timezone(timezone);
        }

        if let Some(currency) = args.currency {
            customer_input = customer_input.with_currency(currency);
        }

        if let Some(net_payment_term) = args.net_payment_term {
            customer_input = customer_input.with_net_payment_term(net_payment_term);
        }

        if let Some(customer_type_str) = args.customer_type {
            if let Ok(customer_type) = customer_type_str.parse::<CustomerType>() {
                customer_input = customer_input.with_customer_type(customer_type);
            }
        }

        if let Some(finalize_str) = args.finalize_zero_amount_invoice {
            if let Ok(finalize) = finalize_str.parse::<CustomerFinalizeZeroAmountInvoice>() {
                customer_input = customer_input.with_finalize_zero_amount_invoice(finalize);
            }
        }

        let request = CreateCustomerRequest::new(customer_input);

        let client = match create_lago_client(&context).await {
            Ok(client) => client,
            Err(error_result) => return Ok(error_result),
        };

        match client.create_customer(request).await {
            Ok(response) => {
                let result = serde_json::json!({
                    "customer": response.customer,
                });

                Ok(success_result(&result))
            }
            Err(e) => {
                let error_message = format!("Failed to create customer: {e}");
                Ok(error_result(error_message))
            }
        }
    }
}
