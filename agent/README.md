# Lago Agent

A Rust-based AI agent powered by Mistral AI that connects to the L- 🔧 **Tool Integration**: Seamless MCP tool calling with streaming responsesgo MCP Server for invoice management.

## Features

- **Mistral AI Integration**: Uses Mistral AI's language model for natural language processing
- **MCP Client**: Connects to your Lago MCP server to access invoice tools
- **Interactive Chat**: Start a conversation with the agent
- **Single Questions**: Ask individual questions and get immediate responses
- **Tool Integration**: Automatically uses available MCP tools when needed
- **Real-time Streaming**: See responses appear word by word as they're generated from Mistral API
- **Instant Message Display**: User messages appear immediately in the UI, no waiting for responses
- **Beautiful Terminal UI**: Modern interface with proper message formatting and real-time updates

## Prerequisites

- Rust (latest stable version)
- A built Lago MCP server (in `../mcp/target/release/lago-mcp-server`)
- Mistral AI API key

## Installation

1. Navigate to the agent directory:
   ```bash
   cd lago-agent-toolkit/agent
   ```

2. Build the project:
   ```bash
   cargo build --release
   ```

## Configuration

Set the following environment variables:

```bash
# Required: Mistral AI API key
export MISTRAL_API_KEY="your-mistral-api-key"

# Optional: Custom Mistral API URL (defaults to https://api.mistral.ai/v1)
export MISTRAL_API_URL="https://api.mistral.ai/v1"

# Required for MCP server: Lago configuration
export LAGO_API_KEY="your-lago-api-key"
export LAGO_API_URL="https://api.getlago.com/api/v1"
```

You can also create a `.env` file in the agent directory:

```env
MISTRAL_API_KEY=your-mistral-api-key
LAGO_API_KEY=your-lago-api-key
LAGO_API_URL=https://api.getlago.com/api/v1
```

## Usage

### Terminal UI Mode (Recommended)

Start the beautiful terminal UI with streaming responses:

```bash
./target/release/lago-agent tui
```

Or specify a custom MCP server path:

```bash
./target/release/lago-agent tui --mcp-server /path/to/your/mcp/server
```

**Features:**
- 🎨 **Beautiful Terminal UI**: Rich interface using ratatui
- ⚡ **Real-time Streaming**: Responses stream in real-time from Mistral API with proper SSE parsing
- 💬 **Interactive Chat**: Full conversation history with timestamps
- 🎯 **Visual Feedback**: Clear indicators for different message types (User, Agent, System)
- ⌨️ **Keyboard Navigation**: Intuitive controls
- 🚀 **Instant Response**: User messages appear immediately when you press Enter
- 🔧 **Tool Integration**: Seamless MCP tool calling with streaming responses
- 📋 **Copy-Paste Support**: Copy individual messages or entire conversations to clipboard

**Controls:**
- Type your message and press **Enter** to send (message appears instantly)
- Press **Esc** to stop editing  
- Press **q** to quit (when not editing)
- Use **↑/↓ arrows** to navigate messages (in normal mode)
- Press **Ctrl+C** to copy selected message to clipboard
- Press **Ctrl+A** to copy entire conversation to clipboard
- Press **Ctrl+V** to paste from clipboard (while editing)
- Watch responses stream in real-time as they're generated
- MCP tools are called seamlessly when needed

### Interactive Chat Mode

Start a simple text-based conversation:

```bash
./target/release/lago-agent chat
```

### Single Question Mode

Ask a single question and get a response:

```bash
./target/release/lago-agent ask "Show me all pending invoices"
```

```bash
./target/release/lago-agent ask "Get invoice details for invoice ID 12345"
```

### Example Conversations

**Terminal UI:**
```
┌─────────────────── 🤖 Lago Agent Chat ───────────────────┐
│ ℹ️ System (14:32:10)                                      │
│                                                           │
│ 🤖 Welcome to Lago Agent! I'm powered by Mistral AI     │
│ and can help you manage your Lago invoices.              │
│                                                           │
│ 👤 You (14:32:15)                                        │
│                                                           │
│ Show me all invoices for customer CUST-001               │
│                                                           │
│ 🤖 Agent (14:32:16)                                      │
│                                                           │
│ I'll retrieve the invoices for customer CUST-001...      │
│ [Streaming response appears in real-time]                │
└───────────────────────────────────────────────────────────┘
┌─────────────────────── Message ───────────────────────────┐
│ How many pending invoices do I have?                     │
└───────────────────────────────────────────────────────────┘
┌─────────────────────── Help ──────────────────────────────┐
│ Press Esc to stop editing, Enter to send message         │
└───────────────────────────────────────────────────────────┘
```

## Architecture

The agent consists of several components:

- **`agent.rs`**: Main agent logic that orchestrates conversations
- **`mistral.rs`**: Mistral AI API client for language processing
- **`mcp_client.rs`**: MCP client for connecting to the Lago MCP server
- **`main.rs`**: CLI interface and application entry point

## How It Works

1. **Initialization**: The agent starts by connecting to both the Mistral AI API and your Lago MCP server
2. **Tool Discovery**: It queries the MCP server for available tools (like `get_invoice` and `list_invoices`)
3. **Conversation Processing**: When you ask a question:
   - The agent sends your message to Mistral AI along with available tool definitions
   - Mistral AI determines if tools need to be called and returns either a response or tool calls
   - If tools are needed, the agent executes them via the MCP server
   - The agent then sends the tool results back to Mistral AI for a final response
4. **Response**: You receive a natural language response based on the actual data from your Lago instance

## Troubleshooting

### Common Issues

1. **"MISTRAL_API_KEY environment variable not set"**
   - Make sure you've set your Mistral AI API key in the environment variables

2. **"Failed to connect to MCP server"**
   - Ensure the Lago MCP server is built and the path is correct
   - Check that your Lago API credentials are properly configured

3. **"Mistral API error"**
   - Verify your Mistral AI API key is valid
   - Check your internet connection
   - Ensure you have sufficient API credits

### Debug Mode

Run with debug logging to see more detailed information:

```bash
RUST_LOG=debug ./target/release/lago-agent chat
```

## Development

To contribute to the agent:

1. Make your changes
2. Run tests: `cargo test`
3. Build: `cargo build`
4. Test your changes with the interactive mode

The agent is designed to be extensible - you can easily add new MCP tools or modify the conversation flow as needed.

## ✨ Production-Ready Streaming

This agent is built with production-grade streaming capabilities:

- **Real Mistral API Streaming**: Uses actual Server-Sent Events (SSE) from Mistral API, no mock responses
- **Robust Error Handling**: Comprehensive error handling for API failures, tool execution issues, and network problems
- **Tool Call Integration**: Seamlessly handles tool calls within streaming responses
- **Conversation History**: Properly maintains conversation context across streaming interactions
- **Timeout Protection**: Built-in timeouts and error recovery for production reliability
- **Resource Management**: Efficient memory usage and proper cleanup of streaming resources
