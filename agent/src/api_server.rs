use anyhow::Result;
use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Json, Response},
    routing::{get, post},
    Router,
};
use futures::{Stream, StreamExt};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::Mutex;
use tower_http::cors::{Any, CorsLayer};
use uuid::Uuid;

use crate::agent::LagoAgent;

/// OpenAI-compatible chat completion request
#[derive(Debug, Deserialize)]
pub struct ChatCompletionRequest {
    pub model: String,
    pub messages: Vec<ChatMessage>,
    pub stream: Option<bool>,
    pub max_tokens: Option<u32>,
    pub temperature: Option<f64>,
    pub top_p: Option<f64>,
    pub stop: Option<Vec<String>>,
    pub presence_penalty: Option<f64>,
    pub frequency_penalty: Option<f64>,
    pub user: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ChatMessage {
    pub role: String,
    #[serde(deserialize_with = "deserialize_content")]
    pub content: String,
}

// Custom deserializer to handle both string and array content
fn deserialize_content<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::Error;
    
    let value = Value::deserialize(deserializer)?;
    match value {
        Value::String(s) => Ok(s),
        Value::Array(arr) => {
            // Extract text from array format (OpenAI multi-modal)
            let mut text_parts = Vec::new();
            for item in arr {
                if let Value::Object(obj) = item {
                    if let Some(Value::String(text_type)) = obj.get("type") {
                        if text_type == "text" {
                            if let Some(Value::String(text)) = obj.get("text") {
                                text_parts.push(text.clone());
                            }
                        }
                    }
                }
            }
            Ok(text_parts.join(" "))
        }
        _ => Err(Error::custom("Content must be a string or array")),
    }
}

/// OpenAI-compatible chat completion response
#[derive(Debug, Serialize)]
pub struct ChatCompletionResponse {
    pub id: String,
    pub object: String,
    pub created: u64,
    pub model: String,
    pub choices: Vec<Choice>,
    pub usage: Usage,
}

#[derive(Debug, Serialize)]
pub struct Choice {
    pub index: u32,
    pub message: ResponseMessage,
    pub finish_reason: String,
}

#[derive(Debug, Serialize)]
pub struct ResponseMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Serialize)]
pub struct Usage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

/// OpenAI-compatible streaming response chunk
#[derive(Debug, Serialize)]
pub struct ChatCompletionChunk {
    pub id: String,
    pub object: String,
    pub created: u64,
    pub model: String,
    pub choices: Vec<StreamChoice>,
}

#[derive(Debug, Serialize)]
pub struct StreamChoice {
    pub index: u32,
    pub delta: Delta,
    pub finish_reason: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct Delta {
    pub content: Option<String>,
    pub role: Option<String>,
}

/// Models list response
#[derive(Debug, Serialize)]
pub struct ModelsResponse {
    pub object: String,
    pub data: Vec<Model>,
}

#[derive(Debug, Serialize)]
pub struct Model {
    pub id: String,
    pub object: String,
    pub created: u64,
    pub owned_by: String,
}

/// App state containing a shared agent instance
#[derive(Clone)]
pub struct AppState {
    pub mcp_server_command: String,
    pub agent: Arc<Mutex<LagoAgent>>,
}

impl AppState {
    pub async fn new(mcp_server_command: String) -> Result<Self, Box<dyn std::error::Error>> {
        let agent = LagoAgent::new(&mcp_server_command).await?;
        Ok(Self {
            mcp_server_command,
            agent: Arc::new(Mutex::new(agent)),
        })
    }
}

/// Health check endpoint
async fn health() -> Json<Value> {
    Json(json!({
        "status": "healthy",
        "service": "Lago Agent API",
        "version": env!("CARGO_PKG_VERSION")
    }))
}

/// Get available models
async fn get_models() -> Json<ModelsResponse> {
    Json(ModelsResponse {
        object: "list".to_string(),
        data: vec![
            Model {
                id: "lago-agent".to_string(),
                object: "model".to_string(),
                created: 1640995200, // Fixed timestamp
                owned_by: "lago".to_string(),
            },
            Model {
                id: "mistral-large-latest".to_string(),
                object: "model".to_string(),
                created: 1640995200,
                owned_by: "mistral".to_string(),
            },
        ],
    })
}

/// Chat completion endpoint (non-streaming)
async fn chat_completions(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<ChatCompletionRequest>,
) -> Result<Json<ChatCompletionResponse>, StatusCode> {
    tracing::info!("Chat completion request: {:?}", request);
    
    // Validate API key if provided
    if let Some(auth_header) = headers.get("authorization") {
        if let Ok(auth_str) = auth_header.to_str() {
            if !auth_str.starts_with("Bearer ") {
                return Err(StatusCode::UNAUTHORIZED);
            }
            // Here you could validate the API key against your system
        }
    }

    // Lock the agent for this request
    let mut agent = state.agent.lock().await;

    // Convert messages to the last user message for processing
    let user_message = request
        .messages
        .iter()
        .filter(|m| m.role == "user")
        .last()
        .map(|m| m.content.clone())
        .unwrap_or_else(|| "Hello".to_string());

    tracing::info!("Processing message: {}", user_message);

    // Process the message
    let response_content = agent
        .ask_question(&user_message)
        .await
        .map_err(|e| {
            tracing::error!("Failed to process message: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    tracing::info!("Response content: {}", response_content);

    // Build OpenAI-compatible response
    let response = ChatCompletionResponse {
        id: format!("chatcmpl-{}", Uuid::new_v4()),
        object: "chat.completion".to_string(),
        created: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
        model: request.model.clone(),
        choices: vec![Choice {
            index: 0,
            message: ResponseMessage {
                role: "assistant".to_string(),
                content: response_content.clone(),
            },
            finish_reason: "stop".to_string(),
        }],
        usage: Usage {
            prompt_tokens: user_message.len() as u32 / 4, // Rough estimate
            completion_tokens: response_content.len() as u32 / 4, // Rough estimate
            total_tokens: (user_message.len() + response_content.len()) as u32 / 4,
        },
    };

    Ok(Json(response))
}

/// Chat completion endpoint (streaming)
async fn chat_completions_stream(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<ChatCompletionRequest>,
) -> Result<axum::response::Response, StatusCode> {
    tracing::info!("Streaming chat completion request: {:?}", request);
    
    // Validate API key if provided
    if let Some(auth_header) = headers.get("authorization") {
        if let Ok(auth_str) = auth_header.to_str() {
            if !auth_str.starts_with("Bearer ") {
                return Err(StatusCode::UNAUTHORIZED);
            }
        }
    }

    // Convert messages to the last user message for processing
    let user_message = request
        .messages
        .iter()
        .filter(|m| m.role == "user")
        .last()
        .map(|m| m.content.clone())
        .unwrap_or_else(|| "Hello".to_string());

    // Lock the agent only for the duration of creating the stream
    let stream = {
        let mut agent = state.agent.lock().await;
        agent
            .process_message_stream(&user_message)
            .await
            .map_err(|e| {
                tracing::error!("Failed to create stream: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?
    }; // Agent lock is released here

    let chat_id = format!("chatcmpl-{}", Uuid::new_v4());
    let created = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    // Convert to SSE stream
    let sse_stream = stream.map(move |result| {
        match result {
            Ok(content) => {
                tracing::debug!("Stream content: {}", content);
                let chunk = ChatCompletionChunk {
                    id: chat_id.clone(),
                    object: "chat.completion.chunk".to_string(),
                    created,
                    model: request.model.clone(),
                    choices: vec![StreamChoice {
                        index: 0,
                        delta: Delta {
                            content: Some(content),
                            role: None,
                        },
                        finish_reason: None,
                    }],
                };
                
                // Format as SSE
                let json_str = serde_json::to_string(&chunk).unwrap_or_default();
                let sse_data = format!("data: {}\n\n", json_str);
                tracing::debug!("SSE data: {}", sse_data);
                sse_data
            }
            Err(e) => {
                tracing::error!("Stream error: {}", e);
                "data: [DONE]\n\n".to_string()
            }
        }
    }).map(|data| Ok::<String, axum::Error>(data));

    // Create response with SSE headers
    let response = axum::response::Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "text/event-stream")
        .header("Cache-Control", "no-cache")
        .header("Connection", "keep-alive")
        .header("Access-Control-Allow-Origin", "*")
        .header("Access-Control-Allow-Methods", "GET, POST, OPTIONS")
        .header("Access-Control-Allow-Headers", "Content-Type, Authorization")
        .body(axum::body::Body::from_stream(sse_stream))
        .map_err(|e| {
            tracing::error!("Failed to create response: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(response)
}

/// Main router function
pub async fn create_router(mcp_server_command: String) -> Result<Router, Box<dyn std::error::Error>> {
    let state = AppState::new(mcp_server_command).await?;

    let router = Router::new()
        .route("/health", get(health))
        .route("/v1/models", get(get_models))
        .route("/v1/chat/completions", post(chat_completions_handler))
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        )
        .with_state(state);

    Ok(router)
}

async fn chat_completions_handler(
    state: State<AppState>,
    headers: HeaderMap,
    Json(request): Json<ChatCompletionRequest>,
) -> Result<impl IntoResponse, StatusCode> {
    // Check if streaming is requested
    if request.stream.unwrap_or(false) {
        chat_completions_stream(state, headers, Json(request)).await
    } else {
        chat_completions(state, headers, Json(request))
            .await
            .map(|response| response.into_response())
    }
}

/// Start the API server
pub async fn start_server(port: u16, mcp_server_command: String) -> Result<()> {
    let app = create_router(mcp_server_command).await
        .map_err(|e| anyhow::anyhow!("Failed to create router: {}", e))?;
    
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port))
        .await
        .map_err(|e| anyhow::anyhow!("Failed to bind to port {}: {}", port, e))?;
    
    tracing::info!("🚀 Lago Agent API Server starting on http://0.0.0.0:{}", port);
    tracing::info!("📋 Available endpoints:");
    tracing::info!("  • GET  /health           - Health check");
    tracing::info!("  • GET  /v1/models        - List available models");
    tracing::info!("  • POST /v1/chat/completions - Chat completions (OpenAI compatible)");
    
    axum::serve(listener, app)
        .await
        .map_err(|e| anyhow::anyhow!("Server error: {}", e))?;
    
    Ok(())
}
