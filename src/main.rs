use std::net::SocketAddr;
use std::sync::Arc;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use backend::application::use_cases::CallService;
use backend::infrastructure::adapters::{
    elevenlabs_gateway::ElevenLabsAgentGateway,
    in_memory_repository::InMemorySessionRepository,
};
use backend::infrastructure::http::handlers::AppState;
use backend::infrastructure::http::routes::create_router;

#[tokio::main]
async fn main() {
    let _ = dotenvy::from_filename(".env.local");
    let _ = dotenvy::from_filename("../.env.local");

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "backend=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let api_key = std::env::var("ELEVENLABS_API_KEY").unwrap_or_default();
    let agent_id = std::env::var("ELEVENLABS_AGENT_ID").unwrap_or_default();

    let repo = Arc::new(InMemorySessionRepository::new());
    let voice_gateway = Arc::new(ElevenLabsAgentGateway::new(api_key, agent_id));
    let call_service = Arc::new(CallService::new(repo, voice_gateway));
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
