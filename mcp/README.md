# Lago MCP Server

A Model Context Protocol (MCP) server for interacting with the Lago billing system. This server provides AI assistants with the ability to query and manage invoice and customer data from Lago through standardized MCP tools.

:warning: **This project is in active development and may change significantly.**

## What is MCP?

The Model Context Protocol (MCP) is a standardized way for AI assistants to interact with external systems and data sources. This Lago MCP server acts as a bridge between AI assistants and the Lago billing API, providing structured access to invoice and customer data.

## Features

- **Invoice Management**: Query and retrieve invoice data from Lago
- **Customer Management**: Create, retrieve, and list customers in Lago
- **Billable Metric Management**: Create, retrieve, and list billable metrics in Lago
- **Filtering Support**: Filter invoices, customers, and billable metrics by various criteria
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

### Billable Metric Tools

#### 6. `get_billable_metric`
Retrieve a specific billable metric by its code.

**Parameters:**
- `code` (string, required): The unique code of the billable metric to retrieve

**Example:**
```json
{
  "code": "storage_gb"
}
```

#### 7. `list_billable_metrics`
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

#### 8. `create_billable_metric`
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

## Setup and Configuration

### Environment Variables

The server requires the following environment variables:

```bash
# Required: Lago API credentials
LAGO_API_KEY=your_lago_api_key
LAGO_API_URL=https://api.getlago.com/api/v1

# Optional: Logging level
RUST_LOG=info
```

### Installation

#### Option 1: Using Docker

1. Build the Docker image:
```bash
docker build -t lago-mcp-server .
```

2. Run the container:
```bash
docker run -e LAGO_API_KEY=your_api_key -e LAGO_API_URL=your_api_url lago-mcp-server
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

## Development

### Project Structure
```
mcp/
├── src/
│   ├── main.rs          # Application entry point
│   ├── server.rs        # MCP server implementation
│   ├── tools/           # Tool implementations
│   │   ├── invoice.rs   # Invoice-related tools
│   │   ├── customer.rs  # Customer-related tools
│   │   └── billable_metric.rs # Billable metric-related tools
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
