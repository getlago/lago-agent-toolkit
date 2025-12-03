# Lago Agent Toolkit

**Bringing agentic superpowers to Lago** 🚀

This repository contains tools and integrations that enable AI agents to interact with the Lago billing platform, making it easier than ever to build intelligent billing workflows and automations.

## What's Inside

This toolkit currently includes:

### 🤖 MCP Server (`/mcp`)
A **Model Context Protocol (MCP) server** written in Rust that provides AI assistants (like Claude) with direct access to Lago's billing data. The server acts as a bridge between AI models and the Lago API, enabling natural language queries about invoices, customers, and billing information.

**Key Features:**
- 📋 **Invoice Management**: Query and retrieve invoice data with smart filtering
- 🔍 **Advanced Search**: Filter by customer, date ranges, status, payment status, and invoice type
- 📄 **Pagination Support**: Handle large datasets efficiently
- 🛡️ **Type Safety**: Fully typed implementation in Rust
- 🐋 **Docker Ready**: Multi-architecture support (AMD64 & ARM64)

## Quick Start with Claude Desktop

The easiest way to get started is using the pre-built Docker image with Claude Desktop:

### 1. Configure Claude Desktop

Add this configuration to your Claude Desktop MCP settings:

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
        "-e", "MISTRAL_AGENT_ID=your_mistral_agent_id",
        "-e", "MISTRAL_API_KEY=your_mistral_api_key",
        "getlago/lago-mcp-server:latest"
      ]
    }
  }
}
```

### 2. Set Your Credentials

Replace `your_lago_api_key` with your actual Lago API key. You can find this in your Lago dashboard under API settings.

Replace `your_mistral_agent_id` and `your_mistral_api_key` with your actual Mistral API credentials.

### 3. Start Chatting!

Once configured, you can ask Claude natural language questions about your billing data:

- *"Show me all pending invoices from last month"*
- *"Find all failed payment invoices"*
- *"Give me the total amount of overdue invoices for the month of March 2025"*

## Available Tools

### Invoices
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

We welcome contributions! Whether it's adding new tools, improving existing functionality, or enhancing documentation, your help makes this toolkit better for everyone.

## License

MIT License - see [LICENSE](LICENSE) for details.