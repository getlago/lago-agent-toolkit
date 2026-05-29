use anyhow::Result;
use reqwest::Client;
use rmcp::{RoleServer, handler::server::tool::Parameters, model::*, service::RequestContext};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::tools::{error_result, get_lago_api_config, success_result};

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct ListWalletsArgs {
    /// Filter wallets by customer external ID. Use this before a wallet rollforward so all
    /// wallet IDs, balances, currencies, and current credit balances are known.
    pub external_customer_id: Option<String>,
    /// Filter wallets by ISO 4217 currency code (e.g., "USD", "EUR").
    pub currency: Option<String>,
    /// Filter wallets by billing entity codes. Each code is sent as billing_entity_codes[].
    pub billing_entity_codes: Option<Vec<String>>,
    /// Page number for pagination (default: 1).
    pub page: Option<i32>,
    /// Number of results per page (default: 20).
    pub per_page: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct GetWalletArgs {
    /// The Lago ID (UUID) of the wallet to retrieve.
    pub lago_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct ListWalletTransactionsArgs {
    /// The Lago ID (UUID) of the wallet whose transactions should be listed.
    pub lago_wallet_id: String,
    /// Filter by transaction direction. Valid values: 'inbound' for credits in and 'outbound'
    /// for credits consumed by invoices or credit notes.
    pub transaction_type: Option<String>,
    /// Filter by processing status. Valid values: 'pending', 'settled', 'failed'.
    pub status: Option<String>,
    /// Filter by business status. Valid values: 'purchased', 'granted', 'voided', 'invoiced'.
    /// Use this to separate purchased credits, granted/free credits, voided credits, and
    /// credits applied to invoices.
    pub transaction_status: Option<String>,
    /// Page number for pagination (default: 1).
    pub page: Option<i32>,
    /// Number of results per page (default: 20).
    pub per_page: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct GetWalletTransactionArgs {
    /// The Lago ID (UUID) of the wallet transaction to retrieve.
    pub lago_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct ListWalletTransactionLinksArgs {
    /// The Lago ID (UUID) of the wallet transaction. For consumptions, pass an inbound
    /// transaction to see which outbound invoice transactions consumed it. For fundings,
    /// pass an outbound transaction to see which inbound credits funded it.
    pub lago_wallet_transaction_id: String,
    /// Page number for pagination (default: 1).
    pub page: Option<i32>,
    /// Number of results per page (default: 20).
    pub per_page: Option<i32>,
}

#[derive(Clone)]
pub struct WalletService {
    http_client: Client,
}

impl WalletService {
    pub fn new() -> Self {
        Self {
            http_client: Client::new(),
        }
    }

    async fn get_json(
        &self,
        context: &RequestContext<RoleServer>,
        path: &str,
        params: Vec<(&str, String)>,
        failure_context: &str,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let config = match get_lago_api_config(context).await {
            Ok(config) => config,
            Err(error_result) => return Ok(error_result),
        };

        let url = format!("{}{}", config.base_url, path);

        match self
            .http_client
            .get(&url)
            .bearer_auth(&config.api_key)
            .query(&params)
            .send()
            .await
        {
            Ok(response) if response.status().is_success() => {
                match response.json::<Value>().await {
                    Ok(json) => Ok(success_result(&json)),
                    Err(e) => {
                        let error_message =
                            format!("Failed to parse {failure_context} response: {e}");
                        tracing::error!("{error_message}");
                        Ok(error_result(error_message))
                    }
                }
            }
            Ok(response) => {
                let status = response.status();
                let body = response
                    .text()
                    .await
                    .unwrap_or_else(|_| "Unknown error".to_string());
                let error_message = format!("Failed to {failure_context} (HTTP {status}): {body}");
                tracing::error!("{error_message}");
                Ok(error_result(error_message))
            }
            Err(e) => {
                let error_message = format!("Failed to {failure_context}: {e}");
                tracing::error!("{error_message}");
                Ok(error_result(error_message))
            }
        }
    }

    pub async fn list_wallets(
        &self,
        Parameters(args): Parameters<ListWalletsArgs>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let mut params: Vec<(&str, String)> = Vec::new();

        if let Some(external_customer_id) = args.external_customer_id {
            params.push(("external_customer_id", external_customer_id));
        }
        if let Some(currency) = args.currency {
            params.push(("currency", currency));
        }
        if let Some(page) = args.page {
            params.push(("page", page.to_string()));
        }
        if let Some(per_page) = args.per_page {
            params.push(("per_page", per_page.to_string()));
        }
        if let Some(billing_entity_codes) = args.billing_entity_codes {
            params.extend(
                billing_entity_codes
                    .into_iter()
                    .map(|code| ("billing_entity_codes[]", code)),
            );
        }

        self.get_json(&context, "/wallets", params, "list wallets")
            .await
    }

    pub async fn get_wallet(
        &self,
        Parameters(args): Parameters<GetWalletArgs>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let encoded_id = urlencoding::encode(&args.lago_id);
        let path = format!("/wallets/{encoded_id}");

        self.get_json(&context, &path, Vec::new(), "get wallet")
            .await
    }

    pub async fn list_wallet_transactions(
        &self,
        Parameters(args): Parameters<ListWalletTransactionsArgs>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let mut params: Vec<(&str, String)> = Vec::new();

        if let Some(transaction_type) = args.transaction_type {
            params.push(("transaction_type", transaction_type));
        }
        if let Some(status) = args.status {
            params.push(("status", status));
        }
        if let Some(transaction_status) = args.transaction_status {
            params.push(("transaction_status", transaction_status));
        }
        if let Some(page) = args.page {
            params.push(("page", page.to_string()));
        }
        if let Some(per_page) = args.per_page {
            params.push(("per_page", per_page.to_string()));
        }

        let encoded_id = urlencoding::encode(&args.lago_wallet_id);
        let path = format!("/wallets/{encoded_id}/wallet_transactions");

        self.get_json(&context, &path, params, "list wallet transactions")
            .await
    }

    pub async fn get_wallet_transaction(
        &self,
        Parameters(args): Parameters<GetWalletTransactionArgs>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let encoded_id = urlencoding::encode(&args.lago_id);
        let path = format!("/wallet_transactions/{encoded_id}");

        self.get_json(&context, &path, Vec::new(), "get wallet transaction")
            .await
    }

    pub async fn list_wallet_transaction_consumptions(
        &self,
        Parameters(args): Parameters<ListWalletTransactionLinksArgs>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let params = pagination_params(args.page, args.per_page);
        let encoded_id = urlencoding::encode(&args.lago_wallet_transaction_id);
        let path = format!("/wallet_transactions/{encoded_id}/consumptions");

        self.get_json(
            &context,
            &path,
            params,
            "list wallet transaction consumptions",
        )
        .await
    }

    pub async fn list_wallet_transaction_fundings(
        &self,
        Parameters(args): Parameters<ListWalletTransactionLinksArgs>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let params = pagination_params(args.page, args.per_page);
        let encoded_id = urlencoding::encode(&args.lago_wallet_transaction_id);
        let path = format!("/wallet_transactions/{encoded_id}/fundings");

        self.get_json(&context, &path, params, "list wallet transaction fundings")
            .await
    }
}

fn pagination_params(page: Option<i32>, per_page: Option<i32>) -> Vec<(&'static str, String)> {
    let mut params = Vec::new();

    if let Some(page) = page {
        params.push(("page", page.to_string()));
    }
    if let Some(per_page) = per_page {
        params.push(("per_page", per_page.to_string()));
    }

    params
}
