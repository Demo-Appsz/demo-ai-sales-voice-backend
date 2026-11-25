use std::time::Duration;
use async_trait::async_trait;
use serde::Deserialize;

use crate::domain::ports::{AgentSessionToken, VoiceAgentGateway};

pub struct ElevenLabsAgentGateway {
    client: reqwest::Client,
    api_key: String,
    agent_id: String,
}

#[derive(Deserialize)]
struct TokenApiResponse {
    token: String,
    conversation_id: Option<String>,
}

#[derive(Deserialize)]
struct SignedUrlApiResponse {
    signed_url: String,
}

impl ElevenLabsAgentGateway {
    pub fn new(api_key: String, agent_id: String) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(8))
            .pool_max_idle_per_host(32)
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());

        Self {
            client,
            api_key,
            agent_id,
        }
    }
}

#[async_trait]
impl VoiceAgentGateway for ElevenLabsAgentGateway {
    async fn create_session_token(&self) -> Result<AgentSessionToken, String> {
        if self.api_key.is_empty() || self.agent_id.is_empty() {
            return Err("ELEVENLABS_API_KEY or ELEVENLABS_AGENT_ID is not configured".to_string());
        }

        let token_url = format!(
            "https://api.elevenlabs.io/v1/convai/conversation/token?agent_id={}",
            self.agent_id
        );

        let token_res = self
            .client
            .get(&token_url)
            .header("xi-api-key", &self.api_key)
            .send()
            .await
            .map_err(|e| format!("Failed to reach ElevenLabs token API: {}", e))?;

        if token_res.status().is_success() {
            let parsed = token_res
                .json::<TokenApiResponse>()
                .await
                .map_err(|e| format!("Failed to parse token response: {}", e))?;

            return Ok(AgentSessionToken {
                connection_type: "webrtc".to_string(),
                conversation_token: Some(parsed.token),
                conversation_id: parsed.conversation_id,
                signed_url: None,
            });
        }

        let signed_url_endpoint = format!(
            "https://api.elevenlabs.io/v1/convai/conversation/get-signed-url?agent_id={}",
            self.agent_id
        );

        let signed_res = self
            .client
            .get(&signed_url_endpoint)
            .header("xi-api-key", &self.api_key)
            .send()
            .await
            .map_err(|e| format!("Failed to reach ElevenLabs signed URL API: {}", e))?;

        if !signed_res.status().is_success() {
            let status = signed_res.status();
            let body = signed_res.text().await.unwrap_or_default();
            return Err(format!("ElevenLabs API error ({}): {}", status, body));
        }

        let parsed = signed_res
            .json::<SignedUrlApiResponse>()
            .await
            .map_err(|e| format!("Failed to parse signed URL response: {}", e))?;

        Ok(AgentSessionToken {
            connection_type: "websocket".to_string(),
            conversation_token: None,
            conversation_id: None,
            signed_url: Some(parsed.signed_url),
        })
    }
}
