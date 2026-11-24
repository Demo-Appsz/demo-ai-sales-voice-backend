mod application;
mod domain;
mod infrastructure;

use std::net::SocketAddr;
use std::sync::Arc;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::application::use_cases::CallService;
use crate::infrastructure::adapters::in_memory_repository::InMemorySessionRepository;
use crate::infrastructure::http::handlers::AppState;
use crate::infrastructure::http::routes::create_router;

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "backend=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let repo = Arc::new(InMemorySessionRepository::new());
    let call_service = Arc::new(CallService::new(repo));
    let state = Arc::new(AppState { call_service });

    let app = create_router(state);

    let port = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse::<u16>().ok())
        .unwrap_or(8080);
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    println!("🚀 AI Sales Backend running at http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("Failed to bind TCP listener");

    axum::serve(listener, app)
        .await
        .expect("Server error");
}
