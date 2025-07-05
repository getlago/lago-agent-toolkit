use anyhow::{anyhow, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::env;
use futures::Stream;
use tokio_stream::StreamExt;

#[derive(Debug, Clone)]
pub struct MistralClient {
    client: Client,
    api_key: String,
    base_url: String,
}

#[derive(Debug, Serialize)]
struct ChatCompletionRequest {
    model: String,
    messages: Vec<ChatMessage>,
    temperature: f32,
    max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tools: Option<Vec<Tool>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_choice: Option<String>,
    stream: bool,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
    pub tool_calls: Option<Vec<ToolCall>>,
    pub tool_call_id: Option<String>,
}

impl Serialize for ChatMessage {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        
        let mut state = serializer.serialize_struct("ChatMessage", 4)?;
        state.serialize_field("role", &self.role)?;
        state.serialize_field("content", &self.content)?;
        
        // Only include tool_calls if it's an assistant message and tool_calls is Some
        if self.role == "assistant" && self.tool_calls.is_some() {
            state.serialize_field("tool_calls", &self.tool_calls)?;
        }
        
        // Include tool_call_id for tool messages
        if self.role == "tool" && self.tool_call_id.is_some() {
            state.serialize_field("tool_call_id", &self.tool_call_id)?;
        }
        
        state.end()
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ToolCall {
    pub id: String,
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub r#type: Option<String>,
    pub function: FunctionCall,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub index: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FunctionCall {
    pub name: String,
    pub arguments: String,
}

#[derive(Debug, Serialize)]
struct Tool {
    r#type: String,
    function: FunctionDefinition,
}

#[derive(Debug, Serialize)]
struct FunctionDefinition {
    name: String,
    description: String,
    parameters: serde_json::Value,
}

#[derive(Debug, Deserialize)]
struct ChatCompletionResponse {
    choices: Vec<Choice>,
}

#[derive(Debug, Deserialize)]
struct Choice {
    message: ChatMessage,
    finish_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
struct StreamChoice {
    delta: Delta,
    finish_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Delta {
    content: Option<String>,
    tool_calls: Option<Vec<ToolCall>>,
}

#[derive(Debug, Deserialize)]
struct StreamResponse {
    choices: Vec<StreamChoice>,
}

#[derive(Debug)]
pub struct StreamingResponse {
    pub delta: Option<StreamDelta>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StreamDelta {
    pub content: Option<String>,
    pub tool_calls: Option<Vec<ToolCall>>,
}

impl MistralClient {
    pub fn new() -> Result<Self> {
        let api_key = env::var("MISTRAL_API_KEY")
            .map_err(|_| anyhow!("MISTRAL_API_KEY environment variable not set"))?;
        
        let base_url = env::var("MISTRAL_API_URL")
            .unwrap_or_else(|_| "https://api.mistral.ai/v1".to_string());

        Ok(Self {
            client: Client::new(),
            api_key,
            base_url,
        })
    }

    pub async fn chat_completion(
        &self,
        messages: Vec<ChatMessage>,
        tools: Option<Vec<(String, String, serde_json::Value)>>,
    ) -> Result<ChatMessage> {
        let tools = tools.map(|tool_list| {
            tool_list
                .into_iter()
                .map(|(name, description, parameters)| Tool {
                    r#type: "function".to_string(),
                    function: FunctionDefinition {
                        name,
                        description,
                        parameters,
                    },
                })
                .collect()
        });

        let tool_choice = if tools.is_some() {
            Some("auto".to_string())
        } else {
            None
        };

        let request = ChatCompletionRequest {
            model: "mistral-large-latest".to_string(),
            messages,
            temperature: 0.7,
            max_tokens: Some(4096),
            tools,
            tool_choice,
            stream: false,
        };

        let response = self
            .client
            .post(&format!("{}/chat/completions", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow!("Mistral API error: {}", error_text));
        }

        let response_text = response.text().await?;
        
        // Try to parse the response
        let chat_response: ChatCompletionResponse = serde_json::from_str(&response_text)
            .map_err(|e| anyhow!("Failed to parse Mistral API response: {}. Response was: {}", e, response_text))?;
        
        chat_response
            .choices
            .into_iter()
            .next()
            .map(|choice| choice.message)
            .ok_or_else(|| anyhow!("No response from Mistral API"))
    }

    pub async fn chat_completion_stream(
        &self,
        messages: Vec<ChatMessage>,
        tools: Option<Vec<(String, String, serde_json::Value)>>,
    ) -> Result<impl Stream<Item = Result<StreamingResponse>>> {
        let tools = tools.map(|tool_list| {
            tool_list
                .into_iter()
                .map(|(name, description, parameters)| Tool {
                    r#type: "function".to_string(),
                    function: FunctionDefinition {
                        name,
                        description,
                        parameters,
                    },
                })
                .collect()
        });

        let tool_choice = if tools.is_some() {
            Some("auto".to_string())
        } else {
            None
        };

        let request = ChatCompletionRequest {
            model: "mistral-large-latest".to_string(),
            messages,
            temperature: 0.3,
            max_tokens: Some(4096),
            tools,
            tool_choice,
            stream: true,
        };

        let response = self
            .client
            .post(&format!("{}/chat/completions", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow!("Mistral API error: {}", error_text));
        }

        let stream = response.bytes_stream();
        let parsed_stream = stream.map(|chunk| {
            let chunk = chunk.map_err(|e| anyhow!("Stream error: {}", e))?;
            let chunk_str = String::from_utf8_lossy(&chunk);

            let mut delta_content = String::new();
            let mut delta_tool_calls = None;
            let mut found_content = false;
            
            for line in chunk_str.lines() {
                if line.starts_with("data: ") {
                    let data = &line[6..];
                    if data == "[DONE]" {
                        break;
                    }
                    
                    if data.trim().is_empty() {
                        continue;
                    }

                    match serde_json::from_str::<StreamResponse>(data) {
                        Ok(stream_response) => {
                            if let Some(choice) = stream_response.choices.first() {
                                if let Some(content) = &choice.delta.content {
                                    delta_content.push_str(content);
                                    found_content = true;
                                }
                                if choice.delta.tool_calls.is_some() {
                                    delta_tool_calls = choice.delta.tool_calls.clone();
                                }
                            }
                        }
                        Err(_) => {}
                    }
                    
                    // Only try to parse if the data looks like complete JSON
                    // if data.trim().starts_with("{") && data.trim().ends_with("}") {
                    //     match serde_json::from_str::<StreamResponse>(data) {
                    //         Ok(stream_response) => {
                    //             if let Some(choice) = stream_response.choices.first() {
                    //                 if let Some(content) = &choice.delta.content {
                    //                     // Always include content, even if it's empty or whitespace
                    //                     delta_content.push_str(content);
                    //                     found_content = true;
                    //                 }
                    //                 if choice.delta.tool_calls.is_some() {
                    //                     delta_tool_calls = choice.delta.tool_calls.clone();
                    //                 }
                    //             }
                    //         }
                    //         Err(_) => {
                    //             // Skip invalid JSON chunks
                    //         }
                    //     }
                    // }
                }
            }
            
            // Always return a delta if we found any content, even empty
            let delta = if found_content || delta_tool_calls.is_some() {
                Some(StreamDelta {
                    content: if found_content { Some(delta_content) } else { None },
                    tool_calls: delta_tool_calls,
                })
            } else {
                None
            };
            
            Ok(StreamingResponse {
                delta,
            })
        });

        Ok(parsed_stream)
    }
}
