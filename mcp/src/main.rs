use anyhow::Result;
use rmcp::{
    ServiceExt,
    transport::stdio,
};
use tracing_subscriber::EnvFilter;

mod tools;
mod types;
mod server;

use server::LagoMcpServer;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive(tracing::Level::INFO.into()))
        .with_writer(std::io::stderr)
        .with_ansi(false)
        .init();

    tracing::info!("Starting Lago MCP Server");

    // Create and serve the Lago MCP server
    let service = LagoMcpServer::new()
        .serve(stdio())
        .await
        .inspect_err(|e| {
            tracing::error!("Serving error: {:?}", e);
        })?;

    service.waiting().await?;
    Ok(())
}
