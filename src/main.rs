mod api;
mod config;
mod inference;
mod models;
mod download;
mod hardware;
mod context;
mod cli;

use anyhow::Result;
use clap::{Parser, Subcommand};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Parser)]
#[command(name = "rust-llm-runner")]
#[command(about = "A comprehensive LLM runner compatible with Ollama", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Serve {
        #[arg(short, long)]
        host: Option<String>,
        #[arg(short, long)]
        port: Option<u16>,
    },
    Pull {
        model: String,
    },
    List,
    Run {
        model: String,
        #[arg(short, long)]
        prompt: Option<String>,
    },
    Rm {
        model: String,
    },
    Show {
        model: String,
    },
    Ps,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Load .env file first
    let _ = dotenvy::dotenv();
    
    // Load config to get settings
    let config = config::Config::load()?;
    
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| format!("rust_llm_runner={},tower_http=debug", config.log_level).into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Serve { host, port } => {
            // Use CLI args if provided, otherwise fall back to .env/config
            let host = host.unwrap_or(config.server_host);
            let port = port.unwrap_or(config.server_port);
            tracing::info!("Starting server on {}:{}", host, port);
            api::server::start_server(&host, port).await?;
        }
        Commands::Pull { model } => {
            cli::commands::pull_model(&model).await?;
        }
        Commands::List => {
            cli::commands::list_models().await?;
        }
        Commands::Run { model, prompt } => {
            cli::commands::run_model(&model, prompt).await?;
        }
        Commands::Rm { model } => {
            cli::commands::remove_model(&model).await?;
        }
        Commands::Show { model } => {
            cli::commands::show_model(&model).await?;
        }
        Commands::Ps => {
            cli::commands::list_running().await?;
        }
    }

    Ok(())
}
