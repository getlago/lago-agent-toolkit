use anyhow::Result;
use clap::{Parser, Subcommand};
use rmcp::{
    ServiceExt,
    transport::{
        stdio,
        streamable_http_server::{StreamableHttpService, session::local::LocalSessionManager},
    },
};
use tracing_subscriber::EnvFilter;

mod server;
mod tools;

use server::LagoMcpServer;

#[derive(Parser)]
#[command(name = "lago-mcp-server")]
#[command(about = "Lago MCP Server with support for stdio and SSE transports")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Stdio,
    Sse {
        #[arg(short, long, default_value = "3000")]
        port: u16,
        #[arg(long, default_value = "127.0.0.1")]
        host: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .json()
        .with_env_filter(EnvFilter::from_default_env().add_directive(tracing::Level::INFO.into()))
        .with_writer(std::io::stderr)
        .init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Stdio => {
            tracing::info!("Starting Lago MCP Server with stdio transport");

            let service = LagoMcpServer::new()
                .serve(stdio())
                .await
                .inspect_err(|e| tracing::error!("Failed to start server: {e:?}"))?;

            service.waiting().await?;
        }
        Commands::Sse { port, host } => {
            let service = StreamableHttpService::new(
                || Ok(LagoMcpServer::new()),
                LocalSessionManager::default().into(),
                Default::default(),
            );

            let router = axum::Router::new().nest_service("/mcp", service);
            let address = format!("{}:{}", host, port);
            let tcp_listener = tokio::net::TcpListener::bind(address).await?;
            let _ = axum::serve(tcp_listener, router)
                .with_graceful_shutdown(async { tokio::signal::ctrl_c().await.unwrap() })
                .await;
        }
    }

    Ok(())
}
