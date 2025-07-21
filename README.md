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
        "getlago/lago-mcp-server:latest"
      ]
    }
  }
}
```

### 2. Set Your Lago Credentials

Replace `your_lago_api_key` with your actual Lago API key. You can find this in your Lago dashboard under API settings.

### 3. Start Chatting!

Once configured, you can ask Claude natural language questions about your billing data:

- *"Show me all pending invoices from last month"*
- *"What's the total amount for customer ABC123's invoices?"*
- *"Find all failed payment invoices"*
- *"Get invoice details for lago_invoice_xyz"*

## Available Tools

- **`get_invoice`**: Retrieve a specific invoice by Lago ID
- **`list_invoices`**: Search and filter invoices with advanced criteria

## Development

### Building from Source

```bash
cd mcp
cargo build --release
```

### Running Locally

```bash
cd mcp
LAGO_API_KEY=your_key LAGO_API_URL=https://api.getlago.com/api/v1 cargo run
```

### Docker Build

```bash
cd mcp
docker build -t lago-mcp-server .
```

## Contributing

We welcome contributions! Whether it's adding new tools, improving existing functionality, or enhancing documentation, your help makes this toolkit better for everyone.

## License

MIT License - see [LICENSE](LICENSE) for details.