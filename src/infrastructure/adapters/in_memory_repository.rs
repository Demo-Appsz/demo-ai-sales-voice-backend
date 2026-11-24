use std::collections::HashMap;
use std::sync::Arc;
use async_trait::async_trait;
use tokio::sync::RwLock;

use crate::domain::entities::CallSession;
use crate::domain::ports::SessionRepository;
use crate::domain::value_objects::SessionId;

pub struct InMemorySessionRepository {
    storage: Arc<RwLock<HashMap<SessionId, CallSession>>>,
}

impl InMemorySessionRepository {
    pub fn new() -> Self {
        Self {
            storage: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl Default for InMemorySessionRepository {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl SessionRepository for InMemorySessionRepository {
    async fn save(&self, session: CallSession) -> Result<(), String> {
        let mut store = self.storage.write().await;
        store.insert(session.id.clone(), session);
        Ok(())
    }

    async fn find_by_id(&self, id: &SessionId) -> Result<Option<CallSession>, String> {
        let store = self.storage.read().await;
        Ok(store.get(id).cloned())
    }

    async fn update(&self, session: CallSession) -> Result<(), String> {
        let mut store = self.storage.write().await;
        store.insert(session.id.clone(), session);
        Ok(())
    }
}
