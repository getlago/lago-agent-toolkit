# Lago MCP Server

A Model Context Protocol (MCP) server for interacting with the Lago billing system. This server provides AI assistants with the ability to query and retrieve invoice data from Lago through standardized MCP tools.

:warning: **This project is in active development and may change significantly.**

## What is MCP?

The Model Context Protocol (MCP) is a standardized way for AI assistants to interact with external systems and data sources. This Lago MCP server acts as a bridge between AI assistants and the Lago billing API, providing structured access to invoice data.

## Features

- **Invoice Management**: Query and retrieve invoice data from Lago
- **Filtering Support**: Filter invoices by customer, date ranges, status, and type
- **Pagination**: Handle large result sets with built-in pagination
- **Type Safety**: Fully typed requests and responses using Rust
- **Environment Configuration**: Easy setup using environment variables

## Available Tools

### 1. `get_invoice`
Retrieve a specific invoice by its Lago ID.

**Parameters:**
- `invoice_id` (string, required): The Lago ID of the invoice to retrieve

**Example:**
```json
{
  "invoice_id": "lago_invoice_123"
}
```

### 2. `list_invoices`
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

## Development

### Project Structure
```
mcp/
├── src/
│   ├── main.rs          # Application entry point
│   ├── server.rs        # MCP server implementation
│   ├── tools/           # Tool implementations
│   │   └── invoice.rs   # Invoice-related tools
│   └── types/           # Type definitions
│       └── invoice.rs   # Invoice type definitions
├── Cargo.toml           # Rust dependencies
└── Dockerfile           # Docker configuration
```

### Adding New Tools

1. Create a new module in `src/tools/`
2. Implement the tool functions with proper MCP annotations
3. Add the tool to the `LagoMcpServer` router in `src/server.rs`
4. Update this README with the new tool documentation

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
