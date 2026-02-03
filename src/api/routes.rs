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
