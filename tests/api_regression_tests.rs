use std::sync::Arc;
use async_trait::async_trait;
use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use http_body_util::BodyExt;
use serde_json::{json, Value};
use tower::ServiceExt;

use backend::application::use_cases::CallService;
use backend::domain::ports::{AgentSessionToken, VoiceAgentGateway};
use backend::infrastructure::adapters::in_memory_repository::InMemorySessionRepository;
use backend::infrastructure::http::handlers::AppState;
use backend::infrastructure::http::routes::create_router;

struct MockGateway {
    should_fail: bool,
}

#[async_trait]
impl VoiceAgentGateway for MockGateway {
    async fn create_session_token(&self) -> Result<AgentSessionToken, String> {
        if self.should_fail {
            return Err("Mock upstream unavailable".to_string());
        }
        Ok(AgentSessionToken {
            connection_type: "webrtc".to_string(),
            conversation_token: Some("mock-webrtc-token".to_string()),
            conversation_id: Some("conv_reg_test".to_string()),
            signed_url: None,
        })
    }
}

fn build_test_app(should_fail: bool) -> axum::Router {
    let repo = Arc::new(InMemorySessionRepository::new());
    let gateway = Arc::new(MockGateway { should_fail });
    let call_service = Arc::new(CallService::new(repo, gateway));
    let state = Arc::new(AppState { call_service });
    create_router(state)
}

async fn read_json(res: axum::response::Response) -> Value {
    let bytes = res.into_body().collect().await.unwrap().to_bytes();
    serde_json::from_slice(&bytes).unwrap()
}

#[tokio::test]
async fn test_health_endpoint_returns_ok() {
    let app = build_test_app(false);
    let res = app
        .oneshot(Request::builder().uri("/api/health").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);
    let body = read_json(res).await;
    assert_eq!(body["status"], "ok");
    assert_eq!(body["service"], "ai-sales-customer-service");
}

#[tokio::test]
async fn test_agent_session_returns_both_jasmin_and_mas_rouf_voices() {
    let app = build_test_app(false);

    // 1. Female -> Jasmin
    let req_female = Request::builder()
        .method("POST")
        .uri("/api/agent/session")
        .header("content-type", "application/json")
        .body(Body::from(json!({ "voiceGender": "female" }).to_string()))
        .unwrap();
    let res_female = app.clone().oneshot(req_female).await.unwrap();
    assert_eq!(res_female.status(), StatusCode::OK);
    let body_female = read_json(res_female).await;
    assert_eq!(body_female["success"], true);
    assert_eq!(body_female["voiceGender"], "female");
    assert_eq!(body_female["voiceConfig"]["agentName"], "Jasmin");
    assert_eq!(body_female["voiceConfig"]["voiceId"], "qXOoUJU1h7AiwSe4ltRA");
    assert_eq!(body_female["dynamicVariables"]["agent_name"], "Jasmin");

    // 2. Male -> Mas Rouf (Kapus Datin)
    let req_male = Request::builder()
        .method("POST")
        .uri("/api/agent/session")
        .header("content-type", "application/json")
        .body(Body::from(json!({ "voiceGender": "male" }).to_string()))
        .unwrap();
    let res_male = app.clone().oneshot(req_male).await.unwrap();
    assert_eq!(res_male.status(), StatusCode::OK);
    let body_male = read_json(res_male).await;
    assert_eq!(body_male["success"], true);
    assert_eq!(body_male["voiceGender"], "male");
    assert_eq!(body_male["voiceConfig"]["agentName"], "Mas Rouf");
    assert_eq!(body_male["voiceConfig"]["voiceId"], "zmqLb9Ysr8fUvDD7hXK8");
    assert_eq!(body_male["dynamicVariables"]["agent_name"], "Mas Rouf");
    assert!(
        body_male["dynamicVariables"]["agent_persona"]
            .as_str()
            .unwrap()
            .contains("Kapus Datin")
    );

    // 3. Invalid voiceGender defaults to female (Jasmin)
    let req_invalid = Request::builder()
        .method("POST")
        .uri("/api/agent/session")
        .header("content-type", "application/json")
        .body(Body::from(json!({ "voiceGender": "unknown" }).to_string()))
        .unwrap();
    let res_invalid = app.oneshot(req_invalid).await.unwrap();
    assert_eq!(res_invalid.status(), StatusCode::OK);
    let body_invalid = read_json(res_invalid).await;
    assert_eq!(body_invalid["voiceGender"], "female");
    assert_eq!(body_invalid["voiceConfig"]["agentName"], "Jasmin");
}

#[tokio::test]
async fn test_agent_session_gateway_failure_returns_500() {
    let app = build_test_app(true);
    let req = Request::builder()
        .method("POST")
        .uri("/api/agent/session")
        .body(Body::empty())
        .unwrap();
    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::INTERNAL_SERVER_ERROR);
    let body = read_json(res).await;
    assert_eq!(body["success"], false);
}

#[tokio::test]
async fn test_call_lifecycle_regression_and_not_found() {
    let app = build_test_app(false);

    // Start call
    let start_res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/call/start")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(start_res.status(), StatusCode::OK);
    let start_json = read_json(start_res).await;
    let call_id = start_json["callDetails"]["callId"].as_str().unwrap().to_string();

    // Chat
    let chat_res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/call/{}/chat", call_id))
                .header("content-type", "application/json")
                .body(Body::from(json!({ "message": "Berapa harga paket?" }).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(chat_res.status(), StatusCode::OK);

    // Get session
    let get_res = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/api/call/{}", call_id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(get_res.status(), StatusCode::OK);
    let get_json = read_json(get_res).await;
    assert_eq!(get_json["session"]["transcripts"].as_array().unwrap().len(), 3);

    // End call
    let end_res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/call/{}/end", call_id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(end_res.status(), StatusCode::OK);

    // Non-existent session returns 404
    let missing_res = app
        .oneshot(
            Request::builder()
                .uri("/api/call/nonexistent-id")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(missing_res.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_high_concurrency_multi_user_sessions() {
    let app = build_test_app(false);
    let mut handles = Vec::with_capacity(50);

    for idx in 0..50 {
        let router = app.clone();
        handles.push(tokio::spawn(async move {
            let gender = if idx % 2 == 0 { "female" } else { "male" };

            let sess_res = router
                .clone()
                .oneshot(
                    Request::builder()
                        .method("POST")
                        .uri("/api/agent/session")
                        .header("content-type", "application/json")
                        .body(Body::from(json!({ "voiceGender": gender }).to_string()))
                        .unwrap(),
                )
                .await
                .unwrap();
            assert_eq!(sess_res.status(), StatusCode::OK);
            let sess_json = read_json(sess_res).await;
            let expected_agent = if idx % 2 == 0 { "Jasmin" } else { "Mas Rouf" };
            assert_eq!(sess_json["voiceConfig"]["agentName"], expected_agent);

            let start_res = router
                .clone()
                .oneshot(
                    Request::builder()
                        .method("POST")
                        .uri("/api/call/start")
                        .body(Body::empty())
                        .unwrap(),
                )
                .await
                .unwrap();
            assert_eq!(start_res.status(), StatusCode::OK);
            let start_json = read_json(start_res).await;
            let call_id = start_json["callDetails"]["callId"]
                .as_str()
                .unwrap()
                .to_string();

            let msg = format!("Pesan unik pengguna {}", idx);
            let chat_res = router
                .clone()
                .oneshot(
                    Request::builder()
                        .method("POST")
                        .uri(format!("/api/call/{}/chat", call_id))
                        .header("content-type", "application/json")
                        .body(Body::from(json!({ "message": msg }).to_string()))
                        .unwrap(),
                )
                .await
                .unwrap();
            assert_eq!(chat_res.status(), StatusCode::OK);
            let chat_json = read_json(chat_res).await;
            assert!(chat_json["reply"].as_str().unwrap().contains(&msg));

            let end_res = router
                .oneshot(
                    Request::builder()
                        .method("POST")
                        .uri(format!("/api/call/{}/end", call_id))
                        .body(Body::empty())
                        .unwrap(),
                )
                .await
                .unwrap();
            assert_eq!(end_res.status(), StatusCode::OK);

            call_id
        }));
    }

    let mut call_ids = std::collections::HashSet::new();
    for handle in handles {
        let id = handle.await.unwrap();
        assert!(call_ids.insert(id), "Duplicate SessionId generated under concurrency");
    }
    assert_eq!(call_ids.len(), 50);
}
