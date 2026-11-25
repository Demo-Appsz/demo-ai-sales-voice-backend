use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde_json::json;
use std::sync::Arc;

use crate::application::{
    dtos::ChatRequestDto,
    use_cases::CallService,
};

pub struct AppState {
    pub call_service: Arc<CallService>,
}

pub async fn health_check() -> impl IntoResponse {
    Json(json!({
        "status": "ok",
        "service": "ai-sales-customer-service",
        "version": "2.0.0"
    }))
}

pub async fn create_agent_session(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    match state.call_service.create_agent_session().await {
        Ok(token) => (
            StatusCode::OK,
            Json(json!({
                "success": true,
                "connectionType": token.connection_type,
                "conversationToken": token.conversation_token,
                "conversationId": token.conversation_id,
                "signedUrl": token.signed_url
            })),
        )
            .into_response(),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "success": false, "error": err })),
        )
            .into_response(),
    }
}

pub async fn start_call(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    match state.call_service.start_call().await {
        Ok(res) => (StatusCode::OK, Json(json!(res))).into_response(),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "success": false, "error": err })),
        )
            .into_response(),
    }
}

pub async fn end_call(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match state.call_service.end_call(&id).await {
        Ok(res) => (StatusCode::OK, Json(json!(res))).into_response(),
        Err(err) => (
            StatusCode::NOT_FOUND,
            Json(json!({ "success": false, "error": err })),
        )
            .into_response(),
    }
}

pub async fn process_chat(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(payload): Json<ChatRequestDto>,
) -> impl IntoResponse {
    match state.call_service.process_chat(&id, &payload.message).await {
        Ok(res) => (StatusCode::OK, Json(json!(res))).into_response(),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "success": false, "error": err })),
        )
            .into_response(),
    }
}

pub async fn get_session(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match state.call_service.get_session(&id).await {
        Ok(Some(session)) => (StatusCode::OK, Json(json!({ "success": true, "session": session }))).into_response(),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(json!({ "success": false, "error": "Session not found" })),
        )
            .into_response(),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "success": false, "error": err })),
        )
            .into_response(),
    }
}
