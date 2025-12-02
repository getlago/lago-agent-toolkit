# Lago MCP Server

A Model Context Protocol (MCP) server for interacting with the Lago billing system. This server provides AI assistants with the ability to query and manage invoice and customer data from Lago through standardized MCP tools.

:warning: **This project is in active development and may change significantly.**

## What is MCP?

The Model Context Protocol (MCP) is a standardized way for AI assistants to interact with external systems and data sources. This Lago MCP server acts as a bridge between AI assistants and the Lago billing API, providing structured access to invoice and customer data.

## Features

- **Invoice Management**: Query and retrieve invoice data from Lago
- **Customer Management**: Create, retrieve, and list customers in Lago
- **Customer Usage**: Retrieve current usage data for a customer's subscription
- **Subscription Management**: Create, update, list, and delete subscriptions in Lago
- **Plan Management**: Create, update, list, and delete plans in Lago
- **Billable Metric Management**: Create, retrieve, and list billable metrics in Lago
- **Activity Log Management**: Query activity logs to track actions performed on resources
- **API Log Management**: Query API logs to monitor API requests and responses
- **Applied Coupon Management**: Apply coupons to customers and list applied coupons
- **Event Management**: Send and retrieve usage events for billing
- **Filtering Support**: Filter invoices, customers, subscriptions, plans, billable metrics, logs, and applied coupons by various criteria
- **Pagination**: Handle large result sets with built-in pagination
- **Type Safety**: Fully typed requests and responses using Rust
- **Multi-tenant Support**: Per-request client creation for handling multiple tenants
- **Environment Configuration**: Easy setup using environment variables

## Available Tools

### Invoice Tools

#### 1. `get_invoice`
Retrieve a specific invoice by its Lago ID.

**Parameters:**
- `invoice_id` (string, required): The Lago ID of the invoice to retrieve

**Example:**
```json
{
  "invoice_id": "lago_invoice_123"
}
```

#### 2. `list_invoices`
List invoices with optional filtering and pagination.

**Parameters:**
- `customer_external_id` (string, optional): Filter by customer's external ID
- `issuing_date_from` (string, optional): Filter invoices issued from this date (ISO format)
- `issuing_date_to` (string, optional): Filter invoices issued until this date (ISO format)
- `status` (string, optional): Filter by invoice status
  - Possible values: `draft`, `finalized`, `voided`, `pending`, `failed`
- `payment_status` (string, optional): Filter by payment status
  - Possible values: `pending`, `succeeded`, `failed`
- `invoice_type` (string, optional): Filter by invoice type
  - Possible values: `subscription`, `add_on`, `credit`, `one_off`, `progressive_billing`
- `page` (integer, optional): Page number for pagination (default: 1)
- `per_page` (integer, optional): Number of items per page (default: 20)

**Example:**
```json
{
  "customer_external_id": "customer_123",
  "status": "finalized",
  "payment_status": "pending",
  "page": 1,
  "per_page": 10
}
```

### Customer Tools

#### 3. `get_customer`
Retrieve a specific customer by their external ID.

**Parameters:**
- `external_customer_id` (string, required): The external ID of the customer to retrieve

**Example:**
```json
{
  "external_customer_id": "customer_123"
}
```

#### 4. `list_customers`
List customers with optional filtering and pagination.

**Parameters:**
- `external_customer_id` (string, optional): Filter by a specific customer's external ID
- `page` (integer, optional): Page number for pagination (default: 1)
- `per_page` (integer, optional): Number of items per page (default: 20)

**Example:**
```json
{
  "external_customer_id": "customer_123",
  "page": 1,
  "per_page": 10
}
```

#### 5. `create_customer`
Create or update a customer in Lago.

**Parameters:**
- `external_id` (string, required): Unique external identifier for the customer
- `name` (string, optional): Customer name
- `firstname` (string, optional): Customer first name
- `lastname` (string, optional): Customer last name
- `email` (string, optional): Customer email address
- `address_line1` (string, optional): Primary address line
- `address_line2` (string, optional): Secondary address line
- `city` (string, optional): City
- `country` (string, optional): Country
- `state` (string, optional): State or region
- `zipcode` (string, optional): ZIP or postal code
- `phone` (string, optional): Phone number
- `url` (string, optional): Customer website URL
- `legal_name` (string, optional): Legal business name
- `legal_number` (string, optional): Legal business number
- `logo_url` (string, optional): URL to customer logo
- `tax_identification_number` (string, optional): Tax ID number
- `timezone` (string, optional): Customer timezone
- `currency` (string, optional): Customer default currency (ISO 4217 code)
- `net_payment_term` (integer, optional): Payment terms in days
- `customer_type` (string, optional): Type of customer
  - Possible values: `individual`, `company`
- `finalize_zero_amount_invoice` (string, optional): Whether to finalize zero amount invoices
  - Possible values: `inherit`, `finalize`, `skip`

**Example:**
```json
{
  "external_id": "customer_456",
  "name": "Acme Corporation",
  "email": "billing@acme.com",
  "address_line1": "123 Business St",
  "city": "San Francisco",
  "country": "US",
  "state": "CA",
  "zipcode": "94105",
  "currency": "USD",
  "customer_type": "company",
  "net_payment_term": 30
}
```

### Customer Usage Tools

#### 6. `get_customer_current_usage`
Get the current usage for a customer's subscription. This endpoint retrieves the usage-based billing data for a customer within the current billing period.

**Parameters:**
- `external_customer_id` (string, required): The external unique identifier of the customer (provided by your own application)
- `external_subscription_id` (string, required): The unique identifier of the subscription within your application
- `apply_taxes` (boolean, optional): Optional flag to determine if taxes should be applied. Defaults to true if not provided.

**Example:**
```json
{
  "external_customer_id": "customer_123",
  "external_subscription_id": "subscription_456"
}
```

**Example with taxes disabled:**
```json
{
  "external_customer_id": "customer_123",
  "external_subscription_id": "subscription_456",
  "apply_taxes": false
}
```

### Billable Metric Tools

#### 7. `get_billable_metric`
Retrieve a specific billable metric by its code.

**Parameters:**
- `code` (string, required): The unique code of the billable metric to retrieve

**Example:**
```json
{
  "code": "storage_gb"
}
```

#### 8. `list_billable_metrics`
List billable metrics with optional filtering and pagination.

**Parameters:**
- `aggregation_type` (string, optional): Filter by aggregation type
  - Possible values: `count_agg`, `sum_agg`, `max_agg`, `unique_count_agg`, `weighted_sum_agg`, `latest_agg`
- `recurring` (boolean, optional): Filter by recurring status
- `page` (integer, optional): Page number for pagination (default: 1)
- `per_page` (integer, optional): Number of items per page (default: 20)

**Example:**
```json
{
  "aggregation_type": "sum_agg",
  "recurring": false,
  "page": 1,
  "per_page": 10
}
```

#### 9. `create_billable_metric`
Create a new billable metric in Lago.

**Parameters:**
- `name` (string, required): Name of the billable metric
- `code` (string, required): Unique code for the billable metric
- `aggregation_type` (string, required): Aggregation method to use
  - Possible values: `count_agg`, `sum_agg`, `max_agg`, `unique_count_agg`, `weighted_sum_agg`, `latest_agg`
- `description` (string, optional): Description of the billable metric
- `recurring` (boolean, optional): Whether the metric is recurring
- `rounding_function` (string, optional): Rounding function to apply
  - Possible values: `ceil`, `floor`, `round`
- `rounding_precision` (integer, optional): Number of decimal places for rounding
- `expression` (string, optional): Custom expression for calculation
- `field_name` (string, optional): Field name to aggregate on from usage events
- `weighted_interval` (string, optional): Interval for weighted sum aggregation
  - Possible values: `seconds`
- `filters` (array, optional): Array of filter objects for differentiated pricing
  - Each filter object has:
    - `key` (string): Filter key to match in event properties
    - `values` (array): Array of possible filter values

**Example:**
```json
{
  "name": "Storage Usage",
  "code": "storage_gb",
  "aggregation_type": "sum_agg",
  "description": "Tracks storage usage in gigabytes",
  "field_name": "gb_used",
  "recurring": false,
  "rounding_function": "round",
  "rounding_precision": 2,
  "filters": [
    {
      "key": "region",
      "values": ["us-east-1", "eu-west-1"]
    }
  ]
}
```

### Activity Log Tools

#### 10. `get_activity_log`
Retrieve a specific activity log by its activity ID.

**Parameters:**
- `activity_id` (string, required): The unique identifier of the activity log

**Example:**
```json
{
  "activity_id": "activity_uuid_123"
}
```

#### 11. `list_activity_logs`
List activity logs with optional filtering and pagination.

**Parameters:**
- `activity_types` (array of strings, optional): Filter by activity types (e.g., "invoice.created", "billable_metric.created")
- `activity_sources` (array of strings, optional): Filter by activity sources
  - Possible values: `api`, `front`, `system`
- `user_emails` (array of strings, optional): Filter by user email addresses
- `external_customer_id` (string, optional): Filter by external customer ID
- `external_subscription_id` (string, optional): Filter by external subscription ID
- `resource_ids` (array of strings, optional): Filter by resource IDs
- `resource_types` (array of strings, optional): Filter by resource types (e.g., "Invoice", "BillableMetric", "Customer")
- `from_date` (string, optional): Filter logs from this date (YYYY-MM-DD format)
- `to_date` (string, optional): Filter logs until this date (YYYY-MM-DD format)
- `page` (integer, optional): Page number for pagination (default: 1)
- `per_page` (integer, optional): Number of items per page (default: 20)

**Example:**
```json
{
  "activity_types": ["invoice.created", "customer.created"],
  "activity_sources": ["api", "front"],
  "resource_types": ["Invoice"],
  "from_date": "2025-01-01",
  "to_date": "2025-01-31",
  "page": 1,
  "per_page": 10
}
```

### API Log Tools

#### 12. `get_api_log`
Retrieve a specific API log by its request ID.

**Parameters:**
- `request_id` (string, required): The unique request ID of the API log

**Example:**
```json
{
  "request_id": "request_uuid_123"
}
```

#### 13. `list_api_logs`
List API logs with optional filtering and pagination.

**Parameters:**
- `http_methods` (array of strings, optional): Filter by HTTP methods
  - Possible values: `post`, `put`, `delete`
- `http_statuses` (array of strings, optional): Filter by HTTP statuses - can be numeric codes (e.g., "200", "404", "500") or outcomes ("succeeded", "failed")
- `api_version` (string, optional): Filter by API version (e.g., "v1")
- `request_paths` (array of strings, optional): Filter by request paths (e.g., "/invoices", "/customers")
- `from_date` (string, optional): Filter logs from this date (YYYY-MM-DD format)
- `to_date` (string, optional): Filter logs until this date (YYYY-MM-DD format)
- `page` (integer, optional): Page number for pagination (default: 1)
- `per_page` (integer, optional): Number of items per page (default: 20)

**Example:**
```json
{
  "http_methods": ["post", "put"],
  "http_statuses": ["failed", "500"],
  "api_version": "v1",
  "request_paths": ["/invoices"],
  "from_date": "2025-01-01",
  "to_date": "2025-01-31",
  "page": 1,
  "per_page": 10
}
```

### Event Tools

#### 14. `get_event`
Retrieve a specific usage event by its transaction ID.

**Parameters:**
- `transaction_id` (string, required): The transaction ID of the event to retrieve (will be URL encoded automatically)

**Example:**
```json
{
  "transaction_id": "transaction_1234567890"
}
```

#### 15. `create_event`
Send a usage event to Lago. Events are used to track customer usage and are aggregated into invoice line items based on billable metrics.

**Parameters:**
- `transaction_id` (string, required): Unique identifier for this event (used for idempotency and retrieval)
- `code` (string, required): Billable metric code
- `external_customer_id` (string, optional): External customer ID - required if external_subscription_id is not provided
- `external_subscription_id` (string, optional): External subscription ID - required if external_customer_id is not provided
- `timestamp` (integer, optional): Event timestamp (Unix timestamp in seconds). If not provided, the current time is used.
- `properties` (object, optional): Custom properties/metadata for the event (e.g., {"gb": 10, "region": "us-east"})
- `precise_total_amount_cents` (integer, optional): Precise total amount in cents (e.g., 1234567 for $12,345.67)

**Example (for customer):**
```json
{
  "transaction_id": "txn_unique_123",
  "external_customer_id": "customer_123",
  "code": "api_calls",
  "properties": {"calls": 150, "region": "us-east"},
  "timestamp": 1705312200
}
```

**Example (for subscription):**
```json
{
  "transaction_id": "txn_unique_456",
  "external_subscription_id": "sub_456",
  "code": "storage_gb",
  "properties": {"gb": 50.5}
}
```

### Applied Coupon Tools

#### 17. `list_applied_coupons`
List applied coupons with optional filtering and pagination.

**Parameters:**
- `status` (string, optional): Filter by applied coupon status
  - Possible values: `active`, `terminated`
- `external_customer_id` (string, optional): Filter by customer's external ID
- `coupon_codes` (array of strings, optional): Filter by one or more coupon codes
- `page` (integer, optional): Page number for pagination (default: 1)
- `per_page` (integer, optional): Number of items per page (default: 20)

**Example:**
```json
{
  "status": "active",
  "external_customer_id": "customer_123",
  "coupon_codes": ["WELCOME10", "SUMMER20"],
  "page": 1,
  "per_page": 10
}
```

#### 18. `apply_coupon`
Apply a coupon to a customer. Use this to give discounts before or during a subscription.

**Parameters:**
- `external_customer_id` (string, required): The external ID of the customer
- `coupon_code` (string, required): The code of the coupon to apply
- `frequency` (string, optional): Frequency of coupon application
  - Possible values: `once`, `recurring`, `forever`
- `frequency_duration` (integer, optional): Number of billing periods for recurring coupons
- `amount_cents` (integer, optional): Override the coupon amount in cents (for fixed_amount coupons)
- `amount_currency` (string, optional): Currency for the amount override (required with amount_cents)
- `percentage_rate` (string, optional): Override the percentage rate (for percentage coupons)

**Example:**
```json
{
  "external_customer_id": "customer_123",
  "coupon_code": "WELCOME10",
  "frequency": "recurring",
  "frequency_duration": 6
}
```

### Subscription Tools

#### 19. `list_subscriptions`
List subscriptions with optional filtering and pagination.

**Parameters:**
- `plan_code` (string, optional): Filter by plan code
- `status` (array of strings, optional): Filter by subscription status
  - Possible values: `active`, `pending`, `canceled`, `terminated`
- `page` (integer, optional): Page number for pagination (default: 1)
- `per_page` (integer, optional): Number of items per page (default: 20)

**Example:**
```json
{
  "plan_code": "starter_plan",
  "status": ["active", "pending"],
  "page": 1,
  "per_page": 10
}
```

#### 20. `get_subscription`
Retrieve a specific subscription by its external ID.

**Parameters:**
- `external_id` (string, required): The external unique identifier of the subscription

**Example:**
```json
{
  "external_id": "sub_123"
}
```

#### 21. `list_customer_subscriptions`
List subscriptions for a specific customer with optional filtering and pagination.

**Parameters:**
- `external_customer_id` (string, required): The external unique identifier of the customer
- `plan_code` (string, optional): Filter by plan code
- `status` (array of strings, optional): Filter by subscription status
  - Possible values: `active`, `pending`, `canceled`, `terminated`
- `page` (integer, optional): Page number for pagination (default: 1)
- `per_page` (integer, optional): Number of items per page (default: 20)

**Example:**
```json
{
  "external_customer_id": "customer_123",
  "status": ["active"],
  "page": 1,
  "per_page": 10
}
```

#### 22. `create_subscription`
Create a new subscription for a customer.

**Parameters:**
- `external_customer_id` (string, required): External unique identifier for the customer
- `plan_code` (string, required): Code of the plan to assign to the subscription
- `name` (string, optional): Optional display name for the subscription
- `external_id` (string, optional): Optional external unique identifier for the subscription
- `billing_time` (string, optional): Billing time determines when recurring billing cycles occur
  - Possible values: `anniversary`, `calendar`
- `subscription_at` (string, optional): The subscription start date (ISO 8601 format)
- `ending_at` (string, optional): The subscription end date (ISO 8601 format)
- `plan_overrides` (object, optional): Plan overrides to customize the plan for this subscription
  - `amount_cents` (integer, optional): Override the base amount in cents
  - `amount_currency` (string, optional): Override the currency
  - `description` (string, optional): Override the plan description
  - `invoice_display_name` (string, optional): Override the invoice display name
  - `name` (string, optional): Override the plan name
  - `trial_period` (number, optional): Override the trial period in days

**Example:**
```json
{
  "external_customer_id": "customer_123",
  "plan_code": "starter_plan",
  "name": "My Subscription",
  "external_id": "sub_001",
  "billing_time": "calendar",
  "subscription_at": "2025-01-01T00:00:00Z",
  "plan_overrides": {
    "amount_cents": 9900,
    "trial_period": 14
  }
}
```

#### 23. `update_subscription`
Update an existing subscription.

**Parameters:**
- `external_id` (string, required): The external unique identifier of the subscription to update
- `name` (string, optional): Optional new name for the subscription
- `ending_at` (string, optional): Optional new end date for the subscription (ISO 8601 format)
- `plan_code` (string, optional): Optional new plan code (for plan changes)
- `subscription_at` (string, optional): Optional new subscription date (ISO 8601 format)
- `plan_overrides` (object, optional): Plan overrides to customize the plan for this subscription

**Example:**
```json
{
  "external_id": "sub_001",
  "name": "Updated Subscription Name",
  "ending_at": "2025-12-31T23:59:59Z"
}
```

#### 24. `delete_subscription`
Terminate a subscription.

**Parameters:**
- `external_id` (string, required): The external unique identifier of the subscription to terminate
- `status` (string, optional): Optional status to set the subscription to (defaults to terminated)

**Example:**
```json
{
  "external_id": "sub_001"
}
```

### Plan Tools

#### 25. `list_plans`
List all plans with optional pagination.

**Parameters:**
- `page` (integer, optional): Page number for pagination (default: 1)
- `per_page` (integer, optional): Number of items per page (default: 20)

**Example:**
```json
{
  "page": 1,
  "per_page": 10
}
```

#### 26. `get_plan`
Retrieve a specific plan by its unique code.

**Parameters:**
- `code` (string, required): The unique code of the plan

**Example:**
```json
{
  "code": "starter_plan"
}
```

#### 27. `create_plan`
Create a new plan in Lago. Plans define pricing configuration with billing interval, base amount, and optional usage-based charges.

**Parameters:**
- `name` (string, required): Name of the plan
- `code` (string, required): Unique code for the plan
- `interval` (string, required): Billing interval
  - Possible values: `weekly`, `monthly`, `quarterly`, `semiannual`, `yearly`
- `amount_cents` (integer, required): Base amount in cents
- `amount_currency` (string, required): Currency for the amount (e.g., USD, EUR)
- `invoice_display_name` (string, optional): Display name for invoices
- `description` (string, optional): Description of the plan
- `trial_period` (number, optional): Trial period in days
- `pay_in_advance` (boolean, optional): Whether the plan is billed in advance
- `bill_charges_monthly` (boolean, optional): Whether charges are billed monthly for yearly plans
- `tax_codes` (array, optional): Tax codes for this plan
- `charges` (array, optional): Usage-based charges for this plan
  - Each charge has: `billable_metric_id`, `charge_model` (standard, graduated, volume, package, percentage), `properties`, etc.
- `minimum_commitment` (object, optional): Minimum commitment configuration
  - `amount_cents` (integer): Minimum commitment amount
  - `invoice_display_name` (string, optional): Display name
  - `tax_codes` (array, optional): Tax codes
- `usage_thresholds` (array, optional): Usage thresholds for progressive billing
  - Each threshold has: `amount_cents`, `threshold_display_name`, `recurring`

**Example:**
```json
{
  "name": "Starter Plan",
  "code": "starter_plan",
  "interval": "monthly",
  "amount_cents": 9900,
  "amount_currency": "USD",
  "description": "Our starter plan for small teams",
  "pay_in_advance": true,
  "trial_period": 14
}
```

**Example with charges:**
```json
{
  "name": "Usage Plan",
  "code": "usage_plan",
  "interval": "monthly",
  "amount_cents": 4900,
  "amount_currency": "USD",
  "charges": [
    {
      "billable_metric_id": "metric_lago_id",
      "charge_model": "standard",
      "invoiceable": true,
      "properties": {"amount": "0.01"}
    }
  ]
}
```

#### 28. `update_plan`
Update an existing plan in Lago.

**Parameters:**
- `code` (string, required): The code of the plan to update
- `name` (string, optional): New name of the plan
- `new_code` (string, optional): New code for the plan
- `interval` (string, optional): Billing interval
- `amount_cents` (integer, optional): Base amount in cents
- `amount_currency` (string, optional): Currency for the amount
- `invoice_display_name` (string, optional): Display name for invoices
- `description` (string, optional): Description of the plan
- `trial_period` (number, optional): Trial period in days
- `pay_in_advance` (boolean, optional): Whether the plan is billed in advance
- `bill_charges_monthly` (boolean, optional): Whether charges are billed monthly
- `tax_codes` (array, optional): Tax codes for this plan
- `charges` (array, optional): Charges for this plan
- `minimum_commitment` (object, optional): Minimum commitment configuration
- `usage_thresholds` (array, optional): Usage thresholds
- `cascade_updates` (boolean, optional): Whether to cascade updates to existing subscriptions

**Example:**
```json
{
  "code": "starter_plan",
  "name": "Updated Starter Plan",
  "amount_cents": 12900,
  "description": "Updated description",
  "cascade_updates": true
}
```

#### 29. `delete_plan`
Delete a plan by its unique code. Note: This plan could be associated with active subscriptions.

**Parameters:**
- `code` (string, required): The code of the plan to delete

**Example:**
```json
{
  "code": "starter_plan"
}
```

## Setup and Configuration

### Add LAGO_MCP_SERVER_PATH

- Add `LAGO_MCP_SERVER_PATH` to your `.bashrc` or `.zshrc` file
- Modify the `lago alias`

```bash
# This path depends on where you put lago-agent-toolkit on your computer.
export LAGO_MCP_SERVER_PATH=/home/lago/lago-agent-toolkit/mcp

alias lago="docker-compose -f $LAGO_PATH/docker-compose.dev.yml -f $LAGO_LICENSE_PATH/docker-compose.dev.yml -f $LAGO_MCP_SERVER_PATH/docker-compose.dev.yml"
```

- Add `mcp.lago.dev` to your `/etc/hosts` file

### Environment Variables

The server requires the following environment variables:

```bash
# Required: Lago API credentials
LAGO_API_URL=https://api.getlago.com/api/v1

# Required: Mistral API credentials
MISTRAL_AGENT_ID=your_mistral_agent_id
MISTRAL_API_KEY=your_mistral_api_key

# Optional: When API key is not sent through headers
LAGO_API_KEY=your_lago_api_key

# Optional: Logging level
RUST_LOG=info
```

### Installation

#### Option 1: Using Docker

1. Build the Docker image:
```bash
docker build -t mcp-server_dev .
```

2. Run the container:
```bash
docker run -e LAGO_API_KEY=your_api_key -e LAGO_API_URL=your_api_url -e MISTRAL_AGENT_ID=your_mistral_agent_id -e MISTRAL_API_KEY=your_mistral_api_key mcp-server_dev
```

#### Option 2: Building from Source

1. Ensure you have Rust installed (1.80 or later)

2. Clone the repository and navigate to the MCP directory:
```bash
git clone <repository-url>
cd lago-agent-toolkit/mcp
```

3. Build the project:
```bash
cargo build --release
```

4. Run the server:
```bash
LAGO_API_KEY=your_api_key LAGO_API_URL=your_api_url ./target/release/lago-mcp-server
```

## Transport

The Lago MCP server uses the **stdio transport** for communication with AI assistants. This means:

- **Input**: The server receives MCP protocol messages via standard input (stdin)
- **Output**: The server sends responses via standard output (stdout)
- **Logging**: All logging output is directed to standard error (stderr) to avoid interfering with the MCP protocol communication

This transport method is ideal for:
- Local development and testing
- Integration with AI assistants that support subprocess communication
- Containerized deployments where the container acts as the MCP server process

The stdio transport is automatically configured and requires no additional setup - simply run the server and it will begin listening for MCP protocol messages on stdin.

## Usage with AI Assistants

### Claude Desktop

Add the following to your Claude Desktop MCP configuration:

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
        "getlago/lago-mcp-server"
      ]
    }
  }
}
```

The server provides comprehensive tools for managing invoices, customers, and billable metrics in Lago, with support for filtering, pagination, and full CRUD operations.

### Other MCP-Compatible Assistants

The server communicates via stdin/stdout using the MCP protocol. Refer to your AI assistant's documentation for specific integration instructions.

## Response Format

All tools return JSON responses with the following structure:

### Invoice Data
```json
{
  "lago_id": "uuid",
  "number": "invoice_number",
  "issuing_date": "2024-01-15",
  "invoice_type": "subscription",
  "status": "finalized",
  "payment_status": "pending",
  "currency": "USD",
  "total_amount_cents": 10000,
  "customer": {
    "external_id": "customer_123",
    "name": "Customer Name"
  }
}
```

### Customer Data
```json
{
  "lago_id": "uuid",
  "external_id": "customer_123",
  "name": "Acme Corporation",
  "email": "billing@acme.com",
  "created_at": "2024-01-15T10:30:00Z",
  "country": "US",
  "currency": "USD",
  "timezone": "America/New_York",
  "applicable_timezone": "America/New_York",
  "billing_configuration": {
    "invoice_grace_period": 3,
    "payment_provider": "stripe",
    "provider_customer_id": "cus_stripe123"
  }
}
```

### Customer Usage Data
```json
{
  "customer_usage": {
    "from_datetime": "2024-01-01T00:00:00Z",
    "to_datetime": "2024-01-31T23:59:59Z",
    "issuing_date": "2024-02-01",
    "lago_invoice_id": null,
    "currency": "USD",
    "amount_cents": 15000,
    "taxes_amount_cents": 1500,
    "total_amount_cents": 16500,
    "charges_usage": [
      {
        "units": "150.0",
        "events_count": 45,
        "amount_cents": 15000,
        "amount_currency": "USD",
        "charge": {
          "lago_id": "uuid",
          "charge_model": "standard",
          "invoice_display_name": "API Calls"
        },
        "billable_metric": {
          "lago_id": "uuid",
          "name": "API Calls",
          "code": "api_calls",
          "aggregation_type": "count_agg"
        },
        "filters": [],
        "grouped_usage": []
      }
    ]
  }
}
```

### Billable Metric Data
```json
{
  "lago_id": "uuid",
  "name": "Storage Usage",
  "code": "storage_gb",
  "description": "Tracks storage usage in gigabytes",
  "aggregation_type": "sum_agg",
  "recurring": false,
  "rounding_function": "round",
  "rounding_precision": 2,
  "created_at": "2024-01-15T10:30:00Z",
  "expression": null,
  "field_name": "gb_used",
  "weighted_interval": null,
  "filters": [
    {
      "key": "region",
      "values": ["us-east-1", "eu-west-1"]
    }
  ]
}
```

### List Response
```json
{
  "invoices": [
    // Array of invoice objects
  ],
  "pagination": {
    "current_page": 1,
    "total_pages": 5,
    "total_count": 100,
    "next_page": 2,
    "prev_page": null
  }
}
```

**For customer lists:**
```json
{
  "customers": [
    // Array of customer objects  
  ],
  "pagination": {
    "current_page": 1,
    "total_pages": 3,
    "total_count": 50,
    "next_page": 2,
    "prev_page": null
  }
}
```

**For billable metric lists:**
```json
{
  "billable_metrics": [
    // Array of billable metric objects
  ],
  "pagination": {
    "current_page": 1,
    "total_pages": 2,
    "total_count": 25,
    "next_page": 2,
    "prev_page": null
  }
}
```

### Activity Log Data
```json
{
  "activity_id": "uuid",
  "activity_type": "invoice.created",
  "activity_source": "api",
  "logged_at": "2025-01-15T10:30:00Z",
  "created_at": "2025-01-15T10:30:00Z",
  "user_email": "admin@example.com",
  "resource_id": "resource_uuid",
  "resource_type": "Invoice",
  "external_customer_id": "customer_123",
  "external_subscription_id": null,
  "activity_object": {
    // Activity-specific data
  }
}
```

**For activity log lists:**
```json
{
  "activity_logs": [
    // Array of activity log objects
  ],
  "pagination": {
    "current_page": 1,
    "total_pages": 5,
    "total_count": 100,
    "next_page": 2,
    "prev_page": null
  }
}
```

### API Log Data
```json
{
  "request_id": "uuid",
  "api_version": "v1",
  "client": "lago-ruby-client",
  "http_method": "post",
  "http_status": 200,
  "request_origin": "https://example.com",
  "request_path": "/api/v1/invoices",
  "request_body": {
    // Request body data
  },
  "request_response": {
    // Response data
  },
  "logged_at": "2025-01-15T10:30:00Z",
  "created_at": "2025-01-15T10:30:00Z"
}
```

**For API log lists:**
```json
{
  "api_logs": [
    // Array of API log objects
  ],
  "pagination": {
    "current_page": 1,
    "total_pages": 10,
    "total_count": 200,
    "next_page": 2,
    "prev_page": null
  }
}
```

### Applied Coupon Data
```json
{
  "lago_id": "uuid",
  "lago_coupon_id": "uuid",
  "coupon_code": "WELCOME10",
  "coupon_name": "Welcome Discount",
  "lago_customer_id": "uuid",
  "external_customer_id": "customer_123",
  "status": "active",
  "frequency": "recurring",
  "amount_cents": 1000,
  "amount_cents_remaining": 500,
  "amount_currency": "USD",
  "percentage_rate": null,
  "frequency_duration": 6,
  "frequency_duration_remaining": 3,
  "expiration_at": "2025-12-31T23:59:59Z",
  "created_at": "2025-01-15T10:30:00Z",
  "terminated_at": null
}
```

**For applied coupon lists:**
```json
{
  "applied_coupons": [
    // Array of applied coupon objects
  ],
  "pagination": {
    "current_page": 1,
    "total_pages": 3,
    "total_count": 50,
    "next_page": 2,
    "prev_page": null
  }
}
```

### Event Data
```json
{
  "lago_id": "uuid",
  "transaction_id": "transaction_1234567890",
  "lago_customer_id": "uuid",
  "code": "api_calls",
  "timestamp": "2024-01-15T10:30:00Z",
  "lago_subscription_id": "uuid",
  "external_subscription_id": "sub_123",
  "created_at": "2024-01-15T10:30:00Z",
  "precise_total_amount_cents": 123456,
  "properties": {"calls": 150}
}
```

**For created events (acknowledgment):**
```json
{
  "event": {
    "transaction_id": "transaction_1234567890",
    "external_customer_id": "customer_123",
    "external_subscription_id": null,
    "code": "api_calls"
  }
}
```

### Subscription Data
```json
{
  "subscription": {
    "lago_id": "uuid",
    "external_id": "sub_001",
    "lago_customer_id": "uuid",
    "external_customer_id": "customer_123",
    "billing_time": "calendar",
    "name": "My Subscription",
    "plan_code": "starter_plan",
    "status": "active",
    "created_at": "2024-01-15T10:30:00Z",
    "canceled_at": null,
    "started_at": "2024-01-15T10:30:00Z",
    "ending_at": null,
    "subscription_at": "2024-01-15T10:30:00Z",
    "terminated_at": null,
    "previous_plan_code": null,
    "next_plan_code": null,
    "downgrade_plan_date": null,
    "trial_ended_at": null,
    "current_billing_period_started_at": "2024-01-01T00:00:00Z",
    "current_billing_period_ending_at": "2024-01-31T23:59:59Z",
    "plan": {
      "lago_id": "uuid",
      "name": "Starter Plan",
      "invoice_display_name": null,
      "created_at": "2024-01-01T00:00:00Z",
      "code": "starter_plan",
      "interval": "monthly",
      "description": null,
      "amount_cents": 9900,
      "amount_currency": "USD",
      "trial_period": 0.0,
      "pay_in_advance": true,
      "bill_charges_monthly": null,
      "active_subscriptions_count": 10,
      "draft_invoices_count": 0,
      "parent_id": null,
      "taxes": []
    }
  }
}
```

**For subscription lists:**
```json
{
  "subscriptions": [
    // Array of subscription objects
  ],
  "pagination": {
    "current_page": 1,
    "total_pages": 5,
    "total_count": 100,
    "next_page": 2,
    "prev_page": null
  }
}
```

### Plan Data
```json
{
  "plan": {
    "lago_id": "uuid",
    "name": "Starter Plan",
    "invoice_display_name": null,
    "created_at": "2024-01-01T00:00:00Z",
    "code": "starter_plan",
    "interval": "monthly",
    "description": "Our starter plan for small teams",
    "amount_cents": 9900,
    "amount_currency": "USD",
    "trial_period": 14.0,
    "pay_in_advance": true,
    "bill_charges_monthly": null,
    "active_subscriptions_count": 10,
    "draft_invoices_count": 0,
    "parent_id": null,
    "charges": [],
    "taxes": [],
    "minimum_commitment": null,
    "usage_thresholds": []
  }
}
```

**For plan lists:**
```json
{
  "plans": [
    // Array of plan objects
  ],
  "pagination": {
    "current_page": 1,
    "total_pages": 3,
    "total_count": 25,
    "next_page": 2,
    "prev_page": null
  }
}
```

## Development

### Project Structure
```
mcp/
├── src/
│   ├── main.rs          # Application entry point
│   ├── server.rs        # MCP server implementation
│   ├── tools/           # Tool implementations
│   │   ├── activity_log.rs    # Activity log-related tools
│   │   ├── api_log.rs         # API log-related tools
│   │   ├── applied_coupon.rs  # Applied coupon-related tools
│   │   ├── billable_metric.rs # Billable metric-related tools
│   │   ├── coupon.rs          # Coupon-related tools
│   │   ├── customer.rs        # Customer-related tools
│   │   ├── customer_usage.rs  # Customer usage-related tools
│   │   ├── event.rs           # Event-related tools
│   │   ├── invoice.rs         # Invoice-related tools
│   │   ├── plan.rs            # Plan-related tools
│   │   └── subscription.rs    # Subscription-related tools
│   └── tools.rs         # Shared utilities and client creation
├── Cargo.toml           # Rust dependencies
└── Dockerfile           # Docker configuration
```

### Adding New Tools

1. Create a new module in `src/tools/` or add to existing modules
2. Implement the tool functions with proper MCP annotations
3. Add the tool to the `LagoMcpServer` router in `src/server.rs`
4. Use the centralized `create_lago_client()` helper from `tools.rs` for client creation
5. Update this README with the new tool documentation

### Architecture Notes

- **Multi-tenant Support**: Each tool request creates a fresh `LagoClient` instance, allowing the server to handle multiple tenants
- **Centralized Client Creation**: All tools use the `create_lago_client()` utility function to avoid code duplication
- **Type Safety**: Direct usage of `lago-types` ensures type safety and consistency with the Lago API
- **Error Handling**: Standardized error responses using helper functions

## Error Handling

The server returns structured error responses when operations fail:

```json
{
  "error": "Failed to retrieve invoice: Invoice not found"
}
```

## Logging

The server uses structured logging with configurable levels:
- `RUST_LOG=debug` - Detailed debug information
- `RUST_LOG=info` - General information (default)
- `RUST_LOG=warn` - Warning messages only
- `RUST_LOG=error` - Error messages only

## License

This project is licensed under the MIT License - see the [LICENSE](../LICENSE) file for details.
