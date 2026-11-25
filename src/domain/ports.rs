use super::entities::CallSession;
use super::value_objects::SessionId;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AgentSessionToken {
    #[serde(rename = "connectionType")]
    pub connection_type: String,
    #[serde(rename = "conversationToken", skip_serializing_if = "Option::is_none")]
    pub conversation_token: Option<String>,
    #[serde(rename = "conversationId", skip_serializing_if = "Option::is_none")]
    pub conversation_id: Option<String>,
    #[serde(rename = "signedUrl", skip_serializing_if = "Option::is_none")]
    pub signed_url: Option<String>,
}

#[async_trait]
pub trait SessionRepository: Send + Sync {
    async fn save(&self, session: CallSession) -> Result<(), String>;
    async fn find_by_id(&self, id: &SessionId) -> Result<Option<CallSession>, String>;
    async fn update(&self, session: CallSession) -> Result<(), String>;
}

#[async_trait]
pub trait VoiceAgentGateway: Send + Sync {
    async fn create_session_token(&self) -> Result<AgentSessionToken, String>;
}
