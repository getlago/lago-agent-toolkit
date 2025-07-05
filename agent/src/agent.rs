use anyhow::{anyhow, Result};
use serde_json::Value;
use std::io::{self, Write};
use tokio::io::{AsyncBufReadExt, BufReader};
use futures::{Stream, StreamExt as FuturesStreamExt};

use crate::mistral::{ChatMessage, MistralClient, ToolCall};
use crate::mcp_client::{McpClient, Content};

pub struct LagoAgent {
    mistral_client: MistralClient,
    mcp_client: McpClient,
    conversation_history: Vec<ChatMessage>,
}

impl LagoAgent {
    pub async fn new(mcp_server_command: &str) -> Result<Self> {
        let mistral_client = MistralClient::new()
            .map_err(|e| anyhow!("Failed to initialize Mistral client: {}. Please check your MISTRAL_API_KEY environment variable.", e))?;

        let mcp_client = McpClient::new(mcp_server_command).await
            .map_err(|e| anyhow!("Failed to initialize MCP client with command '{}': {}", mcp_server_command, e))?;

        Ok(Self {
            mistral_client,
            mcp_client,
            conversation_history: Vec::new(),
        })
    }

    pub async fn start_chat(&mut self) -> Result<()> {
        println!("🤖 Lago Agent powered by Mistral AI");
        println!("Connected to Lago MCP Server. Type 'exit' to quit.\n");

        let stdin = tokio::io::stdin();
        let mut reader = BufReader::new(stdin);
        let mut line = String::new();

        loop {
            print!("👤 You: ");
            io::stdout().flush()?;
            
            line.clear();
            reader.read_line(&mut line).await?;
            
            let input = line.trim();
            if input.is_empty() {
                continue;
            }
            
            if input.eq_ignore_ascii_case("exit") {
                break;
            }

            match self.process_message(input).await {
                Ok(response) => {
                    println!("🤖 Agent: {}", response);
                }
                Err(e) => {
                    println!("❌ Error: {}", e);
                }
            }
            println!();
        }

        println!("Goodbye! 👋");
        Ok(())
    }

    pub async fn ask_question(&mut self, question: &str) -> Result<String> {
        self.process_message(question).await
    }

    async fn process_message(&mut self, message: &str) -> Result<String> {
        self.conversation_history.push(ChatMessage {
            role: "user".to_string(),
            content: message.to_string(),
            tool_calls: None,
            tool_call_id: None,
        });

        let tools = self.mcp_client.list_tools().await?;
        let tool_definitions: Vec<(String, String, Value)> = tools
            .into_iter()
            .map(|tool| {
                (
                    tool.name,
                    tool.description.unwrap_or_else(|| "No description available".to_string()),
                    tool.input_schema,
                )
            })
            .collect();

        let system_message = ChatMessage {
            role: "system".to_string(),
            content: "You are a helpful assistant that can help users manage their Lago invoices. You have access to tools through an MCP server that can get and list invoices from a Lago instance. Use the tools when users ask questions about invoices, and provide helpful, clear responses based on the data you retrieve.".to_string(),
            tool_calls: None,
            tool_call_id: None,
        };

        // Prepare messages for Mistral API
        let mut messages = vec![system_message];
        messages.extend(self.conversation_history.clone());

        // Get response from Mistral
        let response = self.mistral_client
            .chat_completion(messages, Some(tool_definitions))
            .await?;

        // Handle tool calls if present
        if let Some(tool_calls) = &response.tool_calls {
            let mut tool_results = Vec::new();
            
            for tool_call in tool_calls {
                let tool_result = self.execute_tool_call(tool_call).await?;
                tool_results.push((tool_call.id.clone(), tool_result));
            }

            // Add assistant message with tool calls to history
            self.conversation_history.push(response.clone());

            // Add tool results to conversation
            for (tool_call_id, result) in tool_results {
                self.conversation_history.push(ChatMessage {
                    role: "tool".to_string(),
                    content: result,
                    tool_calls: None,
                    tool_call_id: Some(tool_call_id),
                });
            }

            // Get final response from Mistral with tool results
            let mut final_messages = vec![ChatMessage {
                role: "system".to_string(),
                content: "You are a helpful assistant that can help users manage their Lago invoices. Provide a clear, helpful response based on the tool results.".to_string(),
                tool_calls: None,
                tool_call_id: None,
            }];
            final_messages.extend(self.conversation_history.clone());

            let final_response = self.mistral_client
                .chat_completion(final_messages, None)
                .await?;

            self.conversation_history.push(final_response.clone());
            Ok(final_response.content)
        } else {
            // No tool calls, just return the response
            self.conversation_history.push(response.clone());
            Ok(response.content)
        }
    }

    async fn execute_tool_call(&mut self, tool_call: &ToolCall) -> Result<String> {
        // Parse tool arguments with better error handling
        let arguments: Value = serde_json::from_str(&tool_call.function.arguments)
            .map_err(|e| anyhow!("Failed to parse tool arguments for '{}': {}", tool_call.function.name, e))?;

        // Execute tool with timeout and error handling
        let result = match tokio::time::timeout(
            std::time::Duration::from_secs(30), // 30 second timeout
            self.mcp_client.call_tool(&tool_call.function.name, arguments)
        ).await {
            Ok(result) => result?,
            Err(_) => {
                return Err(anyhow!("Tool '{}' timed out after 30 seconds", tool_call.function.name));
            }
        };

        // Convert the tool result to a string with improved formatting
        let result_str = match result.content.first() {
            Some(content) => {
                match content {
                    Content::Text { text } => {
                        // Validate that the text is not empty
                        if text.trim().is_empty() {
                            format!("Tool '{}' returned empty result", tool_call.function.name)
                        } else {
                            text.clone()
                        }
                    }
                    Content::Image { .. } => {
                        format!("Tool '{}' returned image content (not supported in text mode)", tool_call.function.name)
                    }
                    Content::Resource { .. } => {
                        format!("Tool '{}' returned resource content (not supported in text mode)", tool_call.function.name)
                    }
                }
            }
            None => format!("Tool '{}' returned no content", tool_call.function.name),
        };

        Ok(result_str)
    }

    pub async fn process_message_stream(
        &mut self,
        message: &str,
    ) -> Result<Box<dyn Stream<Item = Result<String>> + Send + Unpin>> {
        // Add user message to conversation history
        self.conversation_history.push(ChatMessage {
            role: "user".to_string(),
            content: message.to_string(),
            tool_calls: None,
            tool_call_id: None,
        });

        // Get available tools from MCP server
        let tools = self.mcp_client.list_tools().await?;
        let tool_definitions: Vec<(String, String, Value)> = tools
            .into_iter()
            .map(|tool| {
                (
                    tool.name,
                    tool.description.unwrap_or_else(|| "No description available".to_string()),
                    tool.input_schema,
                )
            })
            .collect();

        // Create system message
        let system_message = ChatMessage {
            role: "system".to_string(),
            content: "You are a helpful assistant for managing Lago invoices. You have access to tools that can help you retrieve and analyze invoice data. Use the tools when appropriate to provide accurate and detailed responses about invoices.".to_string(),
            tool_calls: None,
            tool_call_id: None,
        };

        // Prepare messages for Mistral API
        let mut messages = vec![system_message];
        messages.extend(self.conversation_history.clone());

        // Try streaming directly first to preserve all content
        let stream = self.mistral_client.chat_completion_stream(messages, Some(tool_definitions)).await?;
        
        // Create a simple content stream that preserves all chunks
        let content_stream = FuturesStreamExt::map(stream, |result| {
            match result {
                Ok(response) => {
                    if let Some(delta) = response.delta {
                        if let Some(content) = delta.content {
                            Ok(content)
                        } else {
                            Ok(String::new())
                        }
                    } else {
                        Ok(String::new())
                    }
                }
                Err(e) => Err(e),
            }
        });

        Ok(Box::new(content_stream))
    }

    // Add a method to update conversation history after streaming
    pub fn add_assistant_message(&mut self, content: String) {
        self.conversation_history.push(ChatMessage {
            role: "assistant".to_string(),
            content,
            tool_calls: None,
            tool_call_id: None,
        });
    }
}
