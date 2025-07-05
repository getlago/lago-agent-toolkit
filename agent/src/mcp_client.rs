use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::process::Stdio;
use tokio::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tool {
    pub name: String,
    pub description: Option<String>,
    pub input_schema: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallToolResult {
    pub content: Vec<Content>,
    pub is_error: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Content {
    #[serde(rename = "text")]
    Text { text: String },
    #[serde(rename = "image")]
    Image { data: String, mime_type: String },
    #[serde(rename = "resource")]
    Resource { uri: String, mime_type: Option<String> },
}

pub struct McpClient {
    child: tokio::process::Child,
}

impl McpClient {
    pub async fn new(server_command: &str) -> Result<Self> {
        // Start the MCP server process
        let child = Command::new("sh")
            .arg("-c")
            .arg(server_command)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        Ok(Self { child })
    }

    pub async fn list_tools(&mut self) -> Result<Vec<Tool>> {
        // For now, return hardcoded tools that we know are available
        // In a real implementation, this would communicate with the MCP server
        Ok(vec![
            Tool {
                name: "get_invoice".to_string(),
                description: Some("Get a specific invoice by its Lago ID".to_string()),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "lago_id": {
                            "type": "string",
                            "description": "The Lago ID of the invoice to retrieve"
                        }
                    },
                    "required": ["lago_id"]
                }),
            },
            Tool {
                name: "list_invoices".to_string(),
                description: Some("List invoices from Lago with optional filtering".to_string()),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "customer_external_id": {
                            "type": "string",
                            "description": "Filter by customer external ID"
                        },
                        "status": {
                            "type": "string",
                            "description": "Filter by invoice status"
                        },
                        "page": {
                            "type": "integer",
                            "description": "Page number for pagination"
                        },
                        "per_page": {
                            "type": "integer",
                            "description": "Number of items per page"
                        }
                    }
                }),
            },
        ])
    }

    pub async fn call_tool(&mut self, name: &str, arguments: Value) -> Result<CallToolResult> {
        // For now, simulate calling the tools directly
        // In a real implementation, this would communicate with the MCP server via JSON-RPC
        
        match name {
            "get_invoice" => {
                let result = format!("Would call get_invoice with args: {}", arguments);
                Ok(CallToolResult {
                    content: vec![Content::Text { text: result }],
                    is_error: Some(false),
                })
            }
            "list_invoices" => {
                let result = format!("Would call list_invoices with args: {}", arguments);
                Ok(CallToolResult {
                    content: vec![Content::Text { text: result }],
                    is_error: Some(false),
                })
            }
            _ => Err(anyhow!("Unknown tool: {}", name)),
        }
    }
}

impl Drop for McpClient {
    fn drop(&mut self) {
        // Kill the child process when the client is dropped
        let _ = self.child.kill();
    }
}
