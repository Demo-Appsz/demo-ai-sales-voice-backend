use std::collections::HashMap;
use std::sync::Arc;
use async_trait::async_trait;
use tokio::sync::RwLock;

use crate::domain::entities::CallSession;
use crate::domain::ports::SessionRepository;
use crate::domain::value_objects::{CallStatus, SessionId};

const MAX_IN_MEMORY_SESSIONS: usize = 10_000;

pub struct InMemorySessionRepository {
    storage: Arc<RwLock<HashMap<SessionId, CallSession>>>,
}

impl InMemorySessionRepository {
    pub fn new() -> Self {
        Self {
            storage: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    fn evict_if_over_capacity(store: &mut HashMap<SessionId, CallSession>) {
        if store.len() < MAX_IN_MEMORY_SESSIONS {
            return;
        }
        // Prefer evicting oldest disconnected sessions first, then oldest overall
        if let Some(oldest_disconnected) = store
            .iter()
            .filter(|(_, s)| s.status == CallStatus::Disconnected)
            .min_by_key(|(_, s)| s.created_at)
            .map(|(k, _)| k.clone())
        {
            store.remove(&oldest_disconnected);
            return;
        }
        if let Some(oldest_any) = store
            .iter()
            .min_by_key(|(_, s)| s.created_at)
            .map(|(k, _)| k.clone())
        {
            store.remove(&oldest_any);
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
        Self::evict_if_over_capacity(&mut store);
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
