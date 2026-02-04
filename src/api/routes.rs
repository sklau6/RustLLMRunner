<<<<<<< HEAD
use axum::{
    routing::{get, post, delete},
    Router,
};
use std::sync::Arc;
use tower_http::cors::{CorsLayer, Any};

use crate::api::handlers::{self, AppState};

pub fn create_router(state: Arc<AppState>) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);
    
    Router::new()
        .route("/v1/chat/completions", post(handlers::chat_completions))
        .route("/api/generate", post(handlers::generate))
        .route("/api/tags", get(handlers::list_models))
        .route("/api/pull", post(handlers::pull_model))
        .route("/api/show", post(handlers::show_model))
        .route("/api/delete", delete(handlers::delete_model))
        .route("/health", get(|| async { "OK" }))
        .with_state(state)
        .layer(cors)
}
=======
use axum::{
    routing::{get, post, delete},
    Router,
};
use std::sync::Arc;
use tower_http::cors::{CorsLayer, Any};

use crate::api::handlers::{self, AppState};

pub fn create_router(state: Arc<AppState>) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);
    
    Router::new()
        // Root endpoint - Ollama compatible
        .route("/", get(|| async { "Rust LLM Runner is running" }))
        // OpenAI compatible endpoints
        .route("/v1/chat/completions", post(handlers::chat_completions))
        .route("/v1/models", get(handlers::list_openai_models))
        // Ollama compatible endpoints
        .route("/api/generate", post(handlers::generate))
        .route("/api/chat", post(handlers::ollama_chat))
        .route("/api/tags", get(handlers::list_models))
        .route("/api/pull", post(handlers::pull_model))
        .route("/api/show", post(handlers::show_model))
        .route("/api/delete", delete(handlers::delete_model))
        .route("/api/version", get(handlers::version))
        // Health check
        .route("/health", get(|| async { "OK" }))
        .with_state(state)
        .layer(cors)
}
>>>>>>> bb9577f (20260204_220651)
