use axum::{
    routing::{get, post},
    Router,
};
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};

use super::handlers::{
    end_call, get_session, health_check, process_chat, start_call, AppState,
};

pub fn create_router(state: Arc<AppState>) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        .route("/api/health", get(health_check))
        .route("/api/call/start", post(start_call))
        .route("/api/call/{id}/end", post(end_call))
        .route("/api/call/{id}/chat", post(process_chat))
        .route("/api/call/{id}", get(get_session))
        .layer(cors)
        .with_state(state)
}
