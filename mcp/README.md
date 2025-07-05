# Lago MCP Server

A Model Context Protocol (MCP) server for managing Lago invoices. This server provides tools to interact with your Lago instance through the MCP protocol.

## Features

- **get_invoice**: Get a specific invoice by its Lago ID
- **list_invoices**: List and filter invoices from your Lago instance with comprehensive filtering options

## Quick Start

1. **Set up environment variables:**
   ```bash
   export LAGO_API_KEY="your-lago-api-key"
   export LAGO_API_URL="https://api.getlago.com/api/v1"  # Optional
   ```

2. **Build and run the server:**
   ```bash
   cargo build --release
   ./target/release/lago-mcp-server
   ```

3. **Use with an MCP client:**
   - Configure your MCP client to use this server
   - Available tools: `get_invoice`, `list_invoices`

## Prerequisites

- Rust (latest stable version)
- A Lago instance with API access
- Lago API key

## Installation

1. Clone the repository and navigate to the MCP server directory:
   ```bash
   cd lago-agent-toolkit/mcp
   ```

2. Build the project:
   ```bash
   cargo build --release
   ```

## Configuration

Set the following environment variables:

```bash
export LAGO_API_KEY="your-lago-api-key"
export LAGO_REGION="us"  # Optional: "us", "eu", or custom URL. Defaults to "us"
```

**Region Configuration Options:**
- `LAGO_REGION="us"` - Use US region (https://api.getlago.com/api/v1)
- `LAGO_REGION="eu"` - Use EU region (https://api.eu.getlago.com/api/v1)  
- `LAGO_REGION="https://custom.lago.com/api/v1"` - Use custom URL
- `LAGO_API_URL="https://custom.lago.com/api/v1"` - Alternative way to specify custom URL

If both `LAGO_REGION` and `LAGO_API_URL` are set, `LAGO_REGION` takes precedence.

## Usage

### Running the Server

```bash
# Run in development mode
cargo run

# Or run the release build
./target/release/lago-mcp-server
```

The server communicates using stdin/stdout following the MCP protocol.

### Available Tools

#### get_invoice

Get a specific invoice by its Lago ID.

**Parameters:**
- `invoice_id` (required): The Lago ID of the invoice to retrieve

**Example Usage:**
```json
{
  "tool": "get_invoice",
  "arguments": {
    "invoice_id": "123e4567-e89b-12d3-a456-426614174000"
  }
}
```

**Response:**
```json
{
  "invoice": {
    "lago_id": "123e4567-e89b-12d3-a456-426614174000",
    "sequential_id": 1,
    "number": "INV-2024-001",
    "issuing_date": "2024-01-15",
    "invoice_type": "subscription",
    "status": "finalized",
    "payment_status": "succeeded",
    "currency": "USD",
    "total_amount_cents": 10000,
    "customer_external_id": "customer-123",
    "customer_name": "Acme Corp"
  }
}
```

#### list_invoices

Lists invoices from your Lago instance with optional filtering.

**Parameters:**
- `customer_id` (optional): Filter by customer Lago ID (Note: currently maps to external_customer_id due to API limitations)
- `customer_external_id` (optional): Filter by customer external ID
- `issuing_date_from` (optional): Filter by issuing date from (ISO 8601 format, e.g., '2023-01-01')
- `issuing_date_to` (optional): Filter by issuing date to (ISO 8601 format, e.g., '2023-12-31')
- `status` (optional): Filter by invoice status (draft, finalized, voided, pending, failed)
- `payment_status` (optional): Filter by payment status (pending, succeeded, failed)
- `invoice_type` (optional): Filter by invoice type (subscription, add_on, credit, one_off, progressive_billing)
- `page` (optional): Page number for pagination (default: 1)
- `per_page` (optional): Number of invoices per page (default: 10, max: 100)

**Example Usage:**
```json
{
  "tool": "list_invoices",
  "arguments": {
    "customer_external_id": "customer-123",
    "status": "finalized",
    "payment_status": "succeeded",
    "issuing_date_from": "2024-01-01",
    "issuing_date_to": "2024-12-31",
    "per_page": 20
  }
}
```

**Response:**
```json
{
  "invoices": [
    {
      "lago_id": "123e4567-e89b-12d3-a456-426614174000",
      "sequential_id": 1,
      "number": "INV-2024-001",
      "issuing_date": "2024-01-15",
      "invoice_type": "subscription",
      "status": "finalized",
      "payment_status": "succeeded",
      "currency": "USD",
      "total_amount_cents": 10000,
      "customer_external_id": "customer-123",
      "customer_name": "Acme Corp"
    }
  ],
  "pagination": {
    "current_page": 1,
    "next_page": null,
    "prev_page": null,
    "total_pages": 1,
    "total_count": 1
  }
}
```

## MCP Client Configuration

To use this server with an MCP client, add the following configuration:

### For Claude Desktop

Add to your `claude_desktop_config.json`:

```json
{
  "mcpServers": {
    "lago": {
      "command": "cargo",
      "args": ["run", "--release"],
      "cwd": "/path/to/lago-agent-toolkit/mcp",
      "env": {
        "LAGO_API_KEY": "your-lago-api-key",
        "LAGO_REGION": "us"
      }
    }
  }
}
```

### For Other MCP Clients

Use the binary directly:

```json
{
  "mcpServers": {
    "lago": {
      "command": "/path/to/lago-agent-toolkit/mcp/target/release/lago-mcp-server",
      "env": {
        "LAGO_API_KEY": "your-lago-api-key",
        "LAGO_REGION": "us"
      }
    }
  }
}
```

## Development

### Project Structure

```
mcp/
├── src/
│   ├── main.rs          # Main entry point
│   ├── tools/           # Tool implementations
│   │   ├── mod.rs
│   │   └── invoices.rs  # Invoice-related tools
│   └── types/           # Type definitions
│       ├── mod.rs
│       └── invoice.rs   # Invoice types
├── Cargo.toml          # Dependencies and metadata
└── README.md           # This file
```

### Adding New Tools

1. Create a new tool in the `src/tools/` directory
2. Implement the tool using the `#[tool]` macro from the rmcp crate
3. Add the tool to your service using `#[tool_router]`
4. Register the service in `src/main.rs`

### Testing

Run tests with:
```bash
cargo test
```

## Architecture

This MCP server is built using:

- **rmcp**: The official Rust SDK for the Model Context Protocol
- **lago-client**: Custom Rust client for the Lago API
- **lago-types**: Type definitions for Lago API structures
- **serde**: Serialization/deserialization
- **tokio**: Async runtime

The server follows the MCP specification and provides a clean interface between MCP clients and the Lago API.

## License

This project is licensed under the MIT License - see the LICENSE file for details.
