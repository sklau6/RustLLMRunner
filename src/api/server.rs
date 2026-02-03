use anyhow::Result;
use std::sync::Arc;
use crate::api::handlers::AppState;
use crate::api::routes::create_router;
use crate::config::Config;
use crate::models::manager::ModelManager;

pub async fn start_server(host: &str, port: u16) -> Result<()> {
    let config = Arc::new(Config::load()?);
    let model_manager = Arc::new(ModelManager::new(config.clone())?);
    
    let state = Arc::new(AppState {
        model_manager,
    });
    
    let app = create_router(state);
    
    let addr = format!("{}:{}", host, port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    
    tracing::info!("Server listening on http://{}", addr);
    tracing::info!("OpenAI API compatible endpoint: http://{}/v1/chat/completions", addr);
    tracing::info!("Ollama API compatible endpoint: http://{}/api/generate", addr);
    
    axum::serve(listener, app).await?;
    
    Ok(())
}
