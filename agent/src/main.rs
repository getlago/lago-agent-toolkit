use anyhow::Result;
use clap::{Parser, Subcommand};
use tracing_subscriber::EnvFilter;

mod agent;
mod mistral;
mod mcp_client;
mod ui;

use agent::LagoAgent;
use ui::ChatApp;

#[derive(Parser)]
#[command(name = "lago-agent")]
#[command(about = "A Rust agent powered by Mistral AI that connects to Lago MCP Server")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start an interactive chat session with fancy UI
    Chat {
        /// The MCP server command to run
        #[arg(short, long, default_value = "../mcp/target/release/lago-mcp-server")]
        mcp_server: String,
    },
    /// Start a terminal UI chat session with streaming responses
    Tui {
        /// The MCP server command to run
        #[arg(short, long, default_value = "../mcp/target/release/lago-mcp-server")]
        mcp_server: String,
    },
    /// Ask a single question
    Ask {
        /// The question to ask
        question: String,
        /// The MCP server command to run
        #[arg(short, long, default_value = "../mcp/target/release/lago-mcp-server")]
        mcp_server: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    // Load environment variables from .env file if it exists
    let _ = dotenvy::dotenv();
    
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive(tracing::Level::INFO.into()))
        .init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Chat { mcp_server } => {
            let mut agent = LagoAgent::new(&mcp_server).await?;
            agent.start_chat().await?;
        }
        Commands::Tui { mcp_server } => {
            let agent = LagoAgent::new(&mcp_server).await?;
            let mut app = ChatApp::new(agent);
            app.run().await?;
        }
        Commands::Ask { question, mcp_server } => {
            let mut agent = LagoAgent::new(&mcp_server).await?;
            let response = agent.ask_question(&question).await?;
            println!("{}", response);
        }
    }

    Ok(())
}
