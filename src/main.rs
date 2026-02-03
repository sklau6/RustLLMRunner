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
        #[arg(short, long, default_value = "127.0.0.1")]
        host: String,
        #[arg(short, long, default_value = "11434")]
        port: u16,
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
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "rust_llm_runner=info,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Serve { host, port } => {
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
