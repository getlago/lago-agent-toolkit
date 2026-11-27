use anyhow::Result;
use axum::{
    extract::Request,
    http::{StatusCode, header::WWW_AUTHENTICATE},
    middleware::{self, Next},
    response::{IntoResponse, Response},
};
use clap::{Parser, Subcommand};
use rmcp::{
    ServiceExt,
    transport::{
        stdio,
        streamable_http_server::{
            StreamableHttpServerConfig, StreamableHttpService, session::local::LocalSessionManager,
        },
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

/// Builds the Streamable HTTP config. When `MCP_ALLOWED_HOSTS` (comma separated)
/// is set, only those `Host` headers are accepted and browser `Origin` headers
/// must be `https://<allowed host>`. When unset, Host and Origin are not checked.
fn http_server_config(allowed_hosts: Option<String>) -> StreamableHttpServerConfig {
    let config = StreamableHttpServerConfig::default().with_stateful_mode(false);
    let hosts: Vec<String> = allowed_hosts
        .iter()
        .flat_map(|value| value.split(','))
        .map(str::trim)
        .filter(|host| !host.is_empty())
        .map(str::to_string)
        .collect();

    if hosts.is_empty() {
        tracing::warn!("MCP_ALLOWED_HOSTS is not set: Host and Origin validation are disabled");
        return config.disable_allowed_hosts();
    }

    tracing::info!(allowed_hosts = ?hosts, "Host and Origin validation enabled");
    let origins: Vec<String> = hosts.iter().map(|host| format!("https://{host}")).collect();
    config
        .with_allowed_hosts(hosts)
        .with_allowed_origins(origins)
}

/// Rejects HTTP requests that carry no Lago API key, so the server never falls
/// back to the `LAGO_API_KEY` environment variable in HTTP mode.
async fn require_api_key(request: Request, next: Next) -> Response {
    if tools::api_key_from_headers(request.headers()).is_none() {
        return (
            StatusCode::UNAUTHORIZED,
            [(WWW_AUTHENTICATE, "Bearer")],
            "Missing Lago API key: send it in the X-LAGO-API-KEY, X-API-KEY or Authorization: Bearer header",
        )
            .into_response();
    }
    next.run(request).await
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
            tracing::info!(
                "Starting Lago MCP Server with sse transport on {}:{}",
                host,
                port,
            );

            let config = http_server_config(std::env::var("MCP_ALLOWED_HOSTS").ok());

            let service = StreamableHttpService::new(
                || Ok(LagoMcpServer::new()),
                LocalSessionManager::default().into(),
                config,
            );

            // `route_layer` only wraps routes registered before it, so `/health` stays open.
            let router = axum::Router::new()
                .nest_service("/mcp", service)
                .route_layer(middleware::from_fn(require_api_key))
                .route("/health", axum::routing::get(|| async {}));
            let address = format!("{host}:{port}");
            let tcp_listener = tokio::net::TcpListener::bind(address).await?;
            let _ = axum::serve(tcp_listener, router)
                .with_graceful_shutdown(async { tokio::signal::ctrl_c().await.unwrap() })
                .await;
        }
    }

    Ok(())
}
