# Lago Agent Toolkit

MCP server and agent tools for Lago. Lets AI agents read and write billing data in Lago — invoices, events, customers, payments, credit notes, coupons — via the Model Context Protocol.

Works with Claude Desktop today.

## What's inside

An MCP (Model Context Protocol) server written in Rust, distributed as a Docker image. It bridges AI agents and Lago's billing API.

**Features:**
- Full access to Lago's billing primitives as 40 MCP tools
- Type-safe Rust implementation
- Multi-architecture Docker image (AMD64 & ARM64)
- Pagination support for large datasets
- Filter invoices, customers, payments, events with rich query parameters

## The Managed Agents context

Anthropic launched Claude Managed Agents on April 8, 2026. As of that release, two observable constraints in Anthropic's Admin API:

- The Usage Report API's `group_by` parameter accepts 7 values: `api_key_id`, `workspace_id`, `model`, `service_tier`, `context_window`, `inference_geo`, `speed`. `agent_id` and `session_id` are not among them.
- Session creation accepts `agent`, `environment_id`, and `vault_ids`. No `customer_id`, `metadata`, or `external_id`.

For operators deploying Managed Agents to their own customers and needing per-session or per-customer attribution to bill those customers, the attribution data isn't exposed by Anthropic's API. The Lago Agent Toolkit surfaces Lago's billing primitives via MCP so that layer can be built on Lago rather than hand-rolled.

## Quick start (Claude Desktop)

1. Install Docker Desktop
2. Open Claude Desktop → Settings → Developer → Edit Config
3. Add the Lago MCP server:

```json
{
  "mcpServers": {
    "lago": {
      "command": "docker",
      "args": [
        "run",
        "--rm",
        "-i",
        "--name", "lago-mcp-server",
        "-e", "LAGO_API_KEY=your_lago_api_key",
        "-e", "LAGO_API_URL=https://api.getlago.com/api/v1",
        "getlago/lago-mcp-server:latest"
      ]
    }
  }
}
```

4. Replace `your_lago_api_key` with your actual Lago API key (find it in your Lago dashboard under API settings).
5. Restart Claude Desktop. The agent can now call Lago.

For self-hosted Lago, replace `LAGO_API_URL` with your instance URL.

## Example prompts

- *"Show me all pending invoices from last month"* → `list_invoices`
- *"Find all failed payment invoices"* → `list_invoices`
- *"Give me the total amount of overdue invoices for March 2025"* → `list_invoices` + agent aggregation
- *"Preview an invoice for customer X with 500 additional API-call events"* → `preview_invoice`
- *"Retry payment on invoice INV-123"* → `retry_invoice_payment`

## Available Tools

### Invoices
- **`find_invoice_by_number`**: Find an invoice by its number (e.g., "RAF-8142-202601-312") and get its Lago ID
- **`get_invoice`**: Retrieve a specific invoice by Lago ID
- **`list_invoices`**: Search and filter invoices with advanced criteria
- **`list_customer_invoices`**: List all invoices for a specific customer
- **`create_invoice`**: Create a one-off invoice with add-on fees
- **`update_invoice`**: Update an invoice's payment status or metadata
- **`preview_invoice`**: Preview an invoice before creating it
- **`refresh_invoice`**: Refresh a draft invoice to recalculate charges
- **`download_invoice`**: Download an invoice PDF
- **`retry_invoice`**: Retry generation of a failed invoice
- **`retry_invoice_payment`**: Retry payment collection for an invoice
- **`void_invoice`**: Void a finalized invoice to prevent further modifications or payments

### Customers
- **`get_customer`**: Retrieve a customer by external ID
- **`list_customers`**: List customers with optional filtering
- **`create_customer`**: Create or update a customer

### Billable Metrics
- **`get_billable_metric`**: Retrieve a billable metric by code
- **`list_billable_metrics`**: List billable metrics with optional filtering
- **`create_billable_metric`**: Create a new billable metric

### Coupons
- **`get_coupon`**: Retrieve a coupon by code
- **`list_coupons`**: List all coupons
- **`create_coupon`**: Create a new coupon
- **`update_coupon`**: Update an existing coupon
- **`delete_coupon`**: Delete a coupon

### Applied Coupons
- **`list_applied_coupons`**: List applied coupons with optional filtering
- **`apply_coupon`**: Apply a coupon to a customer

### Events
- **`get_event`**: Retrieve a usage event by transaction ID
- **`create_event`**: Send a usage event to Lago
- **`list_events`**: List usage events with optional filtering by subscription, code, and timestamp range

### Credit Notes
- **`get_credit_note`**: Retrieve a specific credit note by Lago ID
- **`list_credit_notes`**: List credit notes with optional filtering
- **`create_credit_note`**: Create a credit note for an invoice
- **`update_credit_note`**: Update a credit note's refund status

### Payments
- **`get_payment`**: Retrieve a specific payment by Lago ID
- **`list_payments`**: List all payments with optional filtering by customer and invoice
- **`list_customer_payments`**: List all payments for a specific customer
- **`create_payment`**: Create a manual payment for an invoice

### Activity Logs
- **`get_activity_log`**: Retrieve a specific activity log
- **`list_activity_logs`**: List activity logs with optional filtering

### API Logs
- **`get_api_log`**: Retrieve a specific API log
- **`list_api_logs`**: List API logs with optional filtering

## Contributing

Issues and PRs welcome.

## About Lago

Lago is the open-source billing platform for AI-native companies. Real-time metering, programmatic invoicing, self-hostable. Used in production by PayPal, Mistral AI, Groq, Synthesia, and Laravel. AI-infrastructure companies such as CoreWeave also run Lago.

- Docs: [docs.getlago.com](https://docs.getlago.com)
- Cloud: [getlago.com](https://getlago.com)
- Community: [Slack](https://getlago.com/slack)

## License

MIT License - see [LICENSE](LICENSE) for details.
