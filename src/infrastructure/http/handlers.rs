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

#[derive(serde::Deserialize, Default)]
pub struct AgentSessionRequestDto {
    #[serde(rename = "voiceGender")]
    pub voice_gender: Option<String>,
}

pub async fn create_agent_session(
    State(state): State<Arc<AppState>>,
    payload: Option<Json<AgentSessionRequestDto>>,
) -> impl IntoResponse {
    let gender = payload
        .and_then(|p| p.0.voice_gender)
        .filter(|g| g == "male" || g == "female")
        .unwrap_or_else(|| "female".to_string());

    let (voice_config, dynamic_vars) = if gender == "male" {
        (
            json!({
                "agentName": "Mas Rouf",
                "voiceId": "zmqLb9Ysr8fUvDD7hXK8",
                "stability": 0.40,
                "speed": 1.04,
                "similarityBoost": 0.78
            }),
            json!({
                "agent_name": "Mas Rouf",
                "agent_greeting": "Halo! Saya Mas Rouf, Kapus Datin. Gimana, ada yang mau kita obrolin hari ini?",
                "agent_persona": "Anda menjabat sebagai Kapus Datin (Kepala Pusat Data dan Informasi), sosok pimpinan matang berusia sekitar 50 tahun yang tetap energik, sigap, hangat, dan akrab saat berdiskusi. Anda menguasai secara mendalam tata kelola data dan informasi terpadu, prinsip Satu Data, integrasi dan interoperabilitas sistem lintas unit atau instansi, arsitektur SPBE dan layanan digital publik, infrastruktur pusat data dan cloud, keamanan siber serta perlindungan data, hingga pemanfaatan analitik data dan AI untuk mendukung pengambilan keputusan pimpinan serta diseminasi informasi publik. Berbicaralah dalam bahasa Indonesia sehari-hari yang natural, berwibawa namun tetap santai, lugas, dan solutif seperti pemimpin senior sungguhan di telepon."
            }),
        )
    } else {
        (
            json!({
                "agentName": "Jasmin",
                "voiceId": "qXOoUJU1h7AiwSe4ltRA",
                "stability": 0.36,
                "speed": 1.05,
                "similarityBoost": 0.78
            }),
            json!({
                "agent_name": "Jasmin",
                "agent_greeting": "Halo, aku Jasmin! Apa kabar? Ada yang bisa aku bantu hari ini?",
                "agent_persona": "Anda adalah konsultan Sales dan Layanan Pelanggan virtual yang hangat, komunikatif, dan solutif. Anda membantu menggali kebutuhan penelepon secara natural dan berdiskusi seputar solusi digital, layanan bisnis, maupun pertanyaan umum."
            }),
        )
    };

    match state.call_service.create_agent_session().await {
        Ok(token) => (
            StatusCode::OK,
            Json(json!({
                "success": true,
                "connectionType": token.connection_type,
                "conversationToken": token.conversation_token,
                "conversationId": token.conversation_id,
                "signedUrl": token.signed_url,
                "voiceGender": gender,
                "voiceConfig": voice_config,
                "dynamicVariables": dynamic_vars
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
