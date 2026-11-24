use super::entities::CallSession;
use super::value_objects::SessionId;
use async_trait::async_trait;

#[async_trait]
pub trait SessionRepository: Send + Sync {
    async fn save(&self, session: CallSession) -> Result<(), String>;
    async fn find_by_id(&self, id: &SessionId) -> Result<Option<CallSession>, String>;
    async fn update(&self, session: CallSession) -> Result<(), String>;
}
