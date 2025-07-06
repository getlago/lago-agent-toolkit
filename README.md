# Lago Agent Toolkit

A comprehensive toolkit for building AI agents that interact with your Lago instance. This toolkit provides both a Model Context Protocol (MCP) server and an intelligent agent powered by Mistral AI.

## Components

### 🤖 Lago Agent (`agent/`)
A Rust-based AI agent powered by Mistral AI that provides natural language interaction with your Lago instance. The agent can:
- Answer questions about your invoices
- Retrieve specific invoice details
- List and filter invoices based on various criteria
- Provide intelligent responses using the Mistral AI language model

### 🔧 Lago MCP Server (`mcp/`)
A Model Context Protocol server that provides structured access to your Lago instance through standardized tools:
- **get_invoice**: Retrieve specific invoice details by Lago ID
- **list_invoices**: List and filter invoices with comprehensive filtering options

## Quick Start

### Prerequisites
- Rust (latest stable version)
- Lago instance with API access
- Mistral AI API key
- (Optional) LibreChat.ai for web frontend

### 1. Environment Setup

Create a `.env` file in the project root:

```env
# Mistral AI Configuration
MISTRAL_API_KEY=your-mistral-api-key-here

# Lago Configuration
LAGO_API_KEY=your-lago-api-key-here
LAGO_API_URL=https://api.getlago.com/api/v1
```

### 2. Build the Components

```bash
# Build the MCP server
cd mcp
cargo build --release

# Build the agent
cd ../agent
cargo build --release
```

### 3. Start the Agent

```bash
# Start interactive chat
./target/release/lago-agent chat

# Start terminal UI with modern interface
./target/release/lago-agent tui

# Ask a single question
./target/release/lago-agent ask "Show me all pending invoices"

# Start API server for LibreChat integration
./target/release/lago-agent server --port 8080
```

### 4. LibreChat.ai Integration (Optional)

For a modern web interface, integrate with LibreChat.ai:

```bash
# Start the API server
./target/release/lago-agent server --port 8080

# Follow the integration guide
# See LIBRECHAT_INTEGRATION.md for detailed setup instructions
```

## Example Usage

### Interactive Chat Session
```bash
$ ./target/release/lago-agent chat
🤖 Lago Agent powered by Mistral AI
Connected to Lago MCP Server. Type 'exit' to quit.

Available tools: get_invoice, list_invoices

👤 You: Show me all invoices for customer CUST-001
🤖 Agent: I'll retrieve the invoices for customer CUST-001...

[Agent processes the request and provides detailed invoice information]

👤 You: Get details for invoice 12345
🤖 Agent: Let me fetch the details for invoice 12345...

[Agent retrieves and displays invoice details]

👤 You: exit
Goodbye! 👋
```

### Terminal UI Session
```bash
$ ./target/release/lago-agent tui
# Opens a modern terminal interface with:
# - Real-time streaming responses
# - Copy-paste functionality (Ctrl+C, Ctrl+V)
# - Debug panel (press 'd' to toggle)
# - Modern AI-style theming
```

### Single Question Mode
```bash
$ ./target/release/lago-agent ask "How many pending invoices do I have?"
Based on your Lago instance, you currently have 3 pending invoices...
```

### LibreChat.ai Web Interface
```bash
$ ./target/release/lago-agent server --port 8080
# Then access via LibreChat at http://localhost:3080
# Modern web interface with full conversation management
```

## Architecture

The toolkit follows a modular architecture:

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│                 │    │                 │    │                 │
│   Mistral AI    │◄───┤  Lago Agent     │◄───┤     User        │
│                 │    │                 │    │                 │
└─────────────────┘    └─────────────────┘    └─────────────────┘
                                │
                                │ MCP Protocol
                                ▼
                       ┌─────────────────┐    ┌─────────────────┐
                       │                 │    │                 │
                       │ Lago MCP Server │◄───┤  Lago Instance  │
                       │                 │    │                 │
                       └─────────────────┘    └─────────────────┘

### With LibreChat.ai Web Frontend (Optional)

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│                 │    │                 │    │                 │    │                 │
│   Web Browser   │◄───┤   LibreChat.ai  │◄───┤  Lago Agent     │◄───┤   Mistral AI    │
│                 │    │   Frontend      │    │  API Server     │    │                 │
└─────────────────┘    └─────────────────┘    └─────────────────┘    └─────────────────┘
                                │                        │
                                │ HTTP/REST              │ MCP Protocol
                                ▼                        ▼
                       ┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
                       │                 │    │                 │    │                 │
                       │   MongoDB       │    │ Lago MCP Server │◄───┤  Lago Instance  │
                       │   Database      │    │                 │    │                 │
                       └─────────────────┘    └─────────────────┘    └─────────────────┘
```

1. **User** interacts with the **Lago Agent** through natural language
2. **Lago Agent** uses **Mistral AI** to understand and generate responses
3. **Lago Agent** calls the **Lago MCP Server** to retrieve data
4. **Lago MCP Server** communicates with your **Lago Instance**
5. Results flow back through the chain to provide intelligent responses

## Features

### Lago Agent Features
- 🧠 **Natural Language Processing**: Powered by Mistral AI for intelligent conversations
- 🔧 **Tool Integration**: Automatically uses MCP tools when needed
- 💬 **Multiple Interfaces**: Command-line, TUI, and web-based options
- ⚡ **Real-time Streaming**: Live streaming responses with proper chunk handling
- 🎯 **Context Awareness**: Maintains conversation history for better responses
- 📋 **Copy-Paste Support**: Full clipboard integration in TUI mode
- � **Modern UI**: AI-style theming with role icons and responsive design
- 🐛 **Debug Panel**: Integrated logging and debugging with real-time display
- 🌐 **LibreChat Integration**: Modern web frontend with user management
- 🔄 **Production Ready**: Robust error handling and streaming capabilities

### Lago MCP Server Features
- 📊 **Invoice Management**: Comprehensive invoice retrieval and filtering
- 🔍 **Flexible Filtering**: Filter by customer, status, dates, and more
- 📈 **Pagination Support**: Handle large datasets efficiently
- 🛡️ **Error Handling**: Robust error handling and validation
- 🔧 **Extensible**: Easy to add new tools and capabilities

## Development

### Adding New Tools

1. **Add to MCP Server**: Implement new tools in `mcp/src/tools/`
2. **Update Agent**: The agent automatically discovers new tools through the MCP protocol
3. **Test**: Both components include comprehensive testing

### Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests
5. Submit a pull request

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Support

For questions and support:
- Create an issue in the repository
- Check the documentation in each component's README
- Review the example usage above